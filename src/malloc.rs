//! Letting DXC allocate through an allocator of your own instead of the process heap.
//!
//! DXC routes every `operator new`/`delete` through the `IMalloc` the called object was created
//! with ([`Dxc::create_compiler_with_malloc()`](crate::Dxc::create_compiler_with_malloc)).
//! The default is the COM task allocator, i.e. the process heap, whose lock serialises every
//! compile running concurrently in one process.
//!
//! A plain `IMalloc` over another allocator is not enough, because DXC does not keep the two
//! apart:
//!
//! - Some function-local statics are built lazily *inside* a compile (SPIRV-Tools'
//!   `mesh_vuid_map`, for one), so they live in your allocator. Their destructors run when
//!   `dxcompiler.dll` detaches, after DXC has switched back to its default allocator, which then
//!   frees your block: `STATUS_HEAP_CORRUPTION`.
//! - Memory DXC allocated with its default allocator (static initialisation at load) can be
//!   freed or resized during a compile, through your allocator.
//!
//! [`dxc_malloc()`] therefore wraps a [`DxcAllocator`] in two objects that route by owner: an
//! `IMalloc` for DXC that forwards blocks the allocator does not own to the task allocator, and
//! an `IMallocSpy` on the task allocator that frees blocks it does own with the allocator.
//!
//! Windows only: the task allocator and its spy are COM's.

use std::{
    any::{Any, TypeId},
    ffi::c_void,
    ptr::null_mut,
    sync::OnceLock,
};

use com::{interfaces, interfaces::IUnknown, Interface};

use crate::{os::HRESULT, utils::HassleError, Result};

/// An allocator DXC can allocate from, see [`dxc_malloc()`].
///
/// # Safety
///
/// [`Self::owns()`] must be exact: `true` for every live block returned by [`Self::alloc()`] or
/// [`Self::realloc()`], `false` for every other pointer, including blocks of the COM task
/// allocator. The routing relies on it to never hand a block to the wrong allocator.
pub unsafe trait DxcAllocator: Any + Send + Sync {
    /// A block of at least `size` bytes, aligned for any type (16 bytes on x86-64), or null.
    fn alloc(&self, size: usize) -> *mut c_void;
    /// Resize `p` to `size` bytes, moving it if needed.
    ///
    /// # Safety
    ///
    /// `p` is a live block this allocator owns and `size` is non-zero.
    unsafe fn realloc(&self, p: *mut c_void, size: usize) -> *mut c_void;
    /// Free `p`.
    ///
    /// # Safety
    ///
    /// `p` is a live block this allocator owns.
    unsafe fn free(&self, p: *mut c_void);
    /// The usable size of `p`.
    ///
    /// # Safety
    ///
    /// `p` is a live block this allocator owns.
    unsafe fn size(&self, p: *const c_void) -> usize;
    /// Whether `p` is a live block of this allocator.
    fn owns(&self, p: *const c_void) -> bool;
}

interfaces! {
    #[uuid("00000002-0000-0000-C000-000000000046")]
    pub unsafe interface IMalloc: IUnknown {
        pub fn alloc(&self, size: usize) -> *mut c_void;
        pub fn realloc(&self, p: *mut c_void, size: usize) -> *mut c_void;
        pub fn free(&self, p: *mut c_void);
        pub fn get_size(&self, p: *mut c_void) -> usize;
        pub fn did_alloc(&self, p: *mut c_void) -> i32;
        pub fn heap_minimize(&self);
    }

    #[uuid("0000001d-0000-0000-C000-000000000046")]
    pub unsafe interface IMallocSpy: IUnknown {
        pub fn pre_alloc(&self, size: usize) -> usize;
        pub fn post_alloc(&self, actual: *mut c_void) -> *mut c_void;
        pub fn pre_free(&self, request: *mut c_void, spyed: i32) -> *mut c_void;
        pub fn post_free(&self, spyed: i32);
        pub fn pre_realloc(
            &self,
            request: *mut c_void,
            size: usize,
            new_request: *mut *mut c_void,
            spyed: i32,
        ) -> usize;
        pub fn post_realloc(&self, actual: *mut c_void, spyed: i32) -> *mut c_void;
        pub fn pre_get_size(&self, request: *mut c_void, spyed: i32) -> *mut c_void;
        pub fn post_get_size(&self, actual: usize, spyed: i32) -> usize;
        pub fn pre_did_alloc(&self, request: *mut c_void, spyed: i32) -> *mut c_void;
        pub fn post_did_alloc(&self, request: *mut c_void, spyed: i32, actual: i32) -> i32;
        pub fn pre_heap_minimize(&self);
        pub fn post_heap_minimize(&self);
    }
}

// `com::class!` brings in `com`'s registry helpers (`RegCreateKeyExA` &c.), which the `com`
// crate does not link itself.
#[link(name = "advapi32")]
extern "system" {}

#[link(name = "ole32")]
extern "system" {
    fn CoGetMalloc(context: u32, malloc: *mut Option<IMalloc>) -> HRESULT;
    fn CoRegisterMallocSpy(spy: *mut c_void) -> HRESULT;
}

/// The COM task allocator: what DXC allocates from by default.
fn task_malloc() -> &'static IMalloc {
    static TASK: OnceLock<Shared<IMalloc>> = OnceLock::new();
    &TASK
        .get_or_init(|| {
            let mut malloc = None;
            // SAFETY: plain out-parameter call; the context must be 1 (`MEMCTX_TASK`).
            let hr = unsafe { CoGetMalloc(1, &mut malloc) };
            assert!(!hr.is_err(), "CoGetMalloc failed: {}", hr);
            Shared(malloc.expect("CoGetMalloc succeeded without an allocator"))
        })
        .0
}

com::class! {
    class RoutingMalloc: IMalloc {
        // `Option` because `com::class!` fields must be `Default`; always set by `dxc_malloc()`.
        allocator: Option<&'static dyn DxcAllocator>,
    }

    impl IMalloc for RoutingMalloc {
        fn alloc(&self, size: usize) -> *mut c_void {
            self.allocator().alloc(size)
        }

        fn realloc(&self, p: *mut c_void, size: usize) -> *mut c_void {
            if p.is_null() {
                self.allocator().alloc(size)
            } else if self.allocator().owns(p) {
                // `IMalloc::Realloc` to zero frees, which `realloc` is not asked to do.
                if size == 0 {
                    unsafe { self.allocator().free(p) };
                    null_mut()
                } else {
                    unsafe { self.allocator().realloc(p, size) }
                }
            } else {
                // Allocated by DXC's default allocator: let it keep the block.
                unsafe { task_malloc().realloc(p, size) }
            }
        }

        fn free(&self, p: *mut c_void) {
            if self.allocator().owns(p) {
                unsafe { self.allocator().free(p) }
            } else if !p.is_null() {
                unsafe { task_malloc().free(p) }
            }
        }

        fn get_size(&self, p: *mut c_void) -> usize {
            if p.is_null() {
                usize::MAX // what `IMalloc::GetSize(NULL)` is documented to return
            } else if self.allocator().owns(p) {
                unsafe { self.allocator().size(p) }
            } else {
                unsafe { task_malloc().get_size(p) }
            }
        }

        fn did_alloc(&self, p: *mut c_void) -> i32 {
            i32::from(self.allocator().owns(p))
        }

        fn heap_minimize(&self) {}
    }
}

com::class! {
    class RoutingSpy: IMallocSpy {
        // `Option` because `com::class!` fields must be `Default`; always set by `dxc_malloc()`.
        allocator: Option<&'static dyn DxcAllocator>,
    }

    impl IMallocSpy for RoutingSpy {
        fn pre_alloc(&self, size: usize) -> usize {
            size
        }

        fn post_alloc(&self, actual: *mut c_void) -> *mut c_void {
            actual
        }

        /// The unload case: one of our blocks reaching the task allocator is freed by its owner,
        /// and the task allocator is handed null, a no-op.
        fn pre_free(&self, request: *mut c_void, _spyed: i32) -> *mut c_void {
            if self.allocator().owns(request) {
                unsafe { self.allocator().free(request) };
                null_mut()
            } else {
                request
            }
        }

        fn post_free(&self, _spyed: i32) {}

        /// Not seen in practice, and a spy cannot redirect a resize: fail loudly rather than
        /// corrupt the heap.
        fn pre_realloc(
            &self,
            request: *mut c_void,
            size: usize,
            new_request: *mut *mut c_void,
            _spyed: i32,
        ) -> usize {
            assert!(
                !self.allocator().owns(request),
                "COM task allocator asked to resize a block DXC got from a DxcAllocator"
            );
            unsafe { *new_request = request };
            size
        }

        fn post_realloc(&self, actual: *mut c_void, _spyed: i32) -> *mut c_void {
            actual
        }

        fn pre_get_size(&self, request: *mut c_void, _spyed: i32) -> *mut c_void {
            assert!(
                !self.allocator().owns(request),
                "COM task allocator asked the size of a block DXC got from a DxcAllocator"
            );
            request
        }

        fn post_get_size(&self, actual: usize, _spyed: i32) -> usize {
            actual
        }

        fn pre_did_alloc(&self, request: *mut c_void, _spyed: i32) -> *mut c_void {
            request
        }

        fn post_did_alloc(&self, _request: *mut c_void, _spyed: i32, actual: i32) -> i32 {
            actual
        }

        fn pre_heap_minimize(&self) {}

        fn post_heap_minimize(&self) {}
    }
}

impl RoutingMalloc {
    fn allocator(&self) -> &'static dyn DxcAllocator {
        self.allocator.expect("allocated by dxc_malloc()")
    }
}

impl RoutingSpy {
    fn allocator(&self) -> &'static dyn DxcAllocator {
        self.allocator.expect("allocated by dxc_malloc()")
    }
}

/// A COM object that lives until the process exits and whose methods are thread-safe.
struct Shared<I>(I);
// SAFETY: only ever holds the task allocator or the stateless routing objects above, all of
// which may be called from any thread.
unsafe impl<I> Send for Shared<I> {}
unsafe impl<I> Sync for Shared<I> {}

/// The `IMalloc` to hand DXC so that it allocates from `allocator`, valid until the process
/// exits; pass it to [`Dxc::create_compiler_with_malloc()`](crate::Dxc::create_compiler_with_malloc)
/// or use [`Dxc::create_compiler_with_allocator()`](crate::Dxc::create_compiler_with_allocator).
///
/// The first call registers the task-allocator spy, which COM allows once per process and which
/// is never revoked (blocks can reach the task allocator until the process exits). Every later
/// call must pass the same allocator, or gets [`HassleError::AllocatorMismatch`]. Fails if another
/// spy already owns the process, in which case DXC has to keep its default allocator.
pub fn dxc_malloc(allocator: &'static dyn DxcAllocator) -> Result<&'static IMalloc> {
    static INSTALLED: OnceLock<(Identity, std::result::Result<Shared<IMalloc>, HRESULT>)> =
        OnceLock::new();
    let (owner, malloc) = INSTALLED.get_or_init(|| {
        task_malloc();
        let spy = RoutingSpy::allocate(Some(allocator))
            .query_interface::<IMallocSpy>()
            .expect("RoutingSpy implements IMallocSpy");
        // SAFETY: the spy is a valid `IMallocSpy`; COM holds its own reference from here on.
        let hr = unsafe { CoRegisterMallocSpy(spy.as_raw().as_ptr().cast()) };
        let malloc = if !hr.is_err() {
            Ok(Shared(
                RoutingMalloc::allocate(Some(allocator))
                    .query_interface::<IMalloc>()
                    .expect("RoutingMalloc implements IMalloc"),
            ))
        } else {
            Err(hr)
        };
        (identity(allocator), malloc)
    });
    match malloc {
        Err(hr) => Err(HassleError::Win32Error(*hr)),
        Ok(_) if *owner != identity(allocator) => Err(HassleError::AllocatorMismatch),
        Ok(malloc) => Ok(&malloc.0),
    }
}

/// The address alone does not tell zero-sized allocators apart.
type Identity = (usize, TypeId);

fn identity(allocator: &'static dyn DxcAllocator) -> Identity {
    (
        (allocator as *const dyn DxcAllocator).cast::<()>() as usize,
        (*allocator).type_id(),
    )
}

#[cfg(test)]
mod tests {
    use std::{
        alloc::{alloc, dealloc, realloc, Layout},
        collections::HashMap,
        sync::Mutex,
    };

    use super::*;

    #[link(name = "ole32")]
    extern "system" {
        fn CoTaskMemAlloc(size: usize) -> *mut c_void;
        fn CoTaskMemFree(p: *mut c_void);
    }

    /// `std::alloc` with every live block and its size in a map, so `owns` is exact.
    struct Tracked(Mutex<Option<HashMap<usize, usize>>>);

    fn layout(size: usize) -> Layout {
        Layout::from_size_align(size, 16).unwrap()
    }

    impl Tracked {
        fn live(&self) -> std::sync::MutexGuard<'_, Option<HashMap<usize, usize>>> {
            let mut live = self.0.lock().unwrap();
            live.get_or_insert_with(HashMap::new);
            live
        }
    }

    unsafe impl DxcAllocator for Tracked {
        fn alloc(&self, size: usize) -> *mut c_void {
            let p = unsafe { alloc(layout(size.max(1))) };
            self.live()
                .as_mut()
                .unwrap()
                .insert(p as usize, size.max(1));
            p.cast()
        }

        unsafe fn realloc(&self, p: *mut c_void, size: usize) -> *mut c_void {
            let mut live = self.live();
            let live = live.as_mut().unwrap();
            let old = live.remove(&(p as usize)).unwrap();
            let q = unsafe { realloc(p.cast(), layout(old), size) };
            live.insert(q as usize, size);
            q.cast()
        }

        unsafe fn free(&self, p: *mut c_void) {
            let size = self.live().as_mut().unwrap().remove(&(p as usize)).unwrap();
            unsafe { dealloc(p.cast(), layout(size)) }
        }

        unsafe fn size(&self, p: *const c_void) -> usize {
            self.live().as_ref().unwrap()[&(p as usize)]
        }

        fn owns(&self, p: *const c_void) -> bool {
            self.live().as_ref().unwrap().contains_key(&(p as usize))
        }
    }

    static TRACKED: Tracked = Tracked(Mutex::new(None));
    static OTHER: Tracked = Tracked(Mutex::new(None));

    #[test]
    fn mismatched_frees_are_routed_to_the_owner() {
        let malloc = dxc_malloc(&TRACKED).expect("spy registers");
        unsafe {
            // One of ours freed by the task allocator: the dll-unload case.
            let p = malloc.alloc(40);
            assert!(TRACKED.owns(p));
            CoTaskMemFree(p);

            // A task-allocator block sized, resized and freed through ours.
            let q = CoTaskMemAlloc(24);
            assert!(!q.is_null() && !TRACKED.owns(q));
            assert!(malloc.get_size(q) >= 24);
            let q = malloc.realloc(q, 4096);
            assert!(!TRACKED.owns(q));
            malloc.free(q);

            // Ordinary use.
            let r = malloc.realloc(null_mut(), 8).cast::<u8>();
            r.write(7);
            let r = malloc.realloc(r.cast(), 100_000).cast::<u8>();
            assert_eq!(r.read(), 7);
            assert!(malloc.realloc(r.cast(), 0).is_null());
        }
    }

    #[test]
    fn another_allocator_is_an_error() {
        dxc_malloc(&TRACKED).expect("spy registers");
        assert!(matches!(
            dxc_malloc(&OTHER),
            Err(HassleError::AllocatorMismatch)
        ));
    }

    struct Zst<const N: usize>;

    unsafe impl<const N: usize> DxcAllocator for Zst<N> {
        fn alloc(&self, _size: usize) -> *mut c_void {
            unreachable!()
        }

        unsafe fn realloc(&self, _p: *mut c_void, _size: usize) -> *mut c_void {
            unreachable!()
        }

        unsafe fn free(&self, _p: *mut c_void) {
            unreachable!()
        }

        unsafe fn size(&self, _p: *const c_void) -> usize {
            unreachable!()
        }

        fn owns(&self, _p: *const c_void) -> bool {
            false
        }
    }

    #[test]
    fn zero_sized_allocators_are_told_apart() {
        static A: Zst<0> = Zst;
        static B: Zst<1> = Zst;
        assert_eq!(identity(&A), identity(&A));
        assert_ne!(identity(&A), identity(&B));
    }
}
