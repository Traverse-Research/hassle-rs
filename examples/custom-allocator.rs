//! Compiles a shader with DXC allocating through a caller-supplied `IMalloc`, and reports how
//! many allocations that took. Handing DXC a per-thread allocator (mimalloc, jemalloc, ...) this
//! way keeps concurrent compilations in one process from serialising on the default allocator's
//! lock.

use std::alloc::{alloc, dealloc, realloc, Layout};
use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use hassle_rs::*;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

/// `IMalloc`'s vtable, in order, after the three `IUnknown` methods.
#[repr(C)]
struct IMallocVtbl {
    query_interface: extern "system" fn(*mut c_void, *const c_void, *mut *mut c_void) -> i32,
    add_ref: extern "system" fn(*mut c_void) -> u32,
    release: extern "system" fn(*mut c_void) -> u32,
    alloc: extern "system" fn(*mut c_void, usize) -> *mut c_void,
    realloc: extern "system" fn(*mut c_void, *mut c_void, usize) -> *mut c_void,
    free: extern "system" fn(*mut c_void, *mut c_void),
    get_size: extern "system" fn(*mut c_void, *mut c_void) -> usize,
    did_alloc: extern "system" fn(*mut c_void, *mut c_void) -> i32,
    heap_minimize: extern "system" fn(*mut c_void),
}

/// A `'static` COM object, so there is nothing to keep alive and `AddRef`/`Release` are no-ops.
#[repr(C)]
struct CountingMalloc {
    vtbl: *const IMallocVtbl,
}

// SAFETY: a pointer to an immutable static vtable.
unsafe impl Sync for CountingMalloc {}

// The size is stored in a header so `free`/`realloc`/`get_size` can rebuild the `Layout`.
const HEADER: usize = std::mem::size_of::<usize>();
const ALIGN: usize = 16;

fn layout(size: usize) -> Layout {
    Layout::from_size_align(HEADER + size, ALIGN).unwrap()
}

extern "system" fn query_interface(
    this: *mut c_void,
    _iid: *const c_void,
    out: *mut *mut c_void,
) -> i32 {
    // Only ever asked for IUnknown or IMalloc, both of which are `this`.
    unsafe { *out = this };
    0
}
extern "system" fn add_ref(_: *mut c_void) -> u32 {
    1
}
extern "system" fn release(_: *mut c_void) -> u32 {
    1
}
extern "system" fn malloc(_: *mut c_void, size: usize) -> *mut c_void {
    ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    unsafe {
        let p = alloc(layout(size));
        if p.is_null() {
            return std::ptr::null_mut();
        }
        p.cast::<usize>().write(size);
        p.add(HEADER).cast()
    }
}
extern "system" fn free(_: *mut c_void, p: *mut c_void) {
    if p.is_null() {
        return;
    }
    unsafe {
        let base = p.cast::<u8>().sub(HEADER);
        dealloc(base, layout(base.cast::<usize>().read()));
    }
}
extern "system" fn grow(_: *mut c_void, p: *mut c_void, size: usize) -> *mut c_void {
    if p.is_null() {
        return malloc(std::ptr::null_mut(), size);
    }
    unsafe {
        let base = p.cast::<u8>().sub(HEADER);
        let old = base.cast::<usize>().read();
        let new = realloc(base, layout(old), HEADER + size);
        if new.is_null() {
            return std::ptr::null_mut();
        }
        new.cast::<usize>().write(size);
        new.add(HEADER).cast()
    }
}
extern "system" fn get_size(_: *mut c_void, p: *mut c_void) -> usize {
    if p.is_null() {
        return 0;
    }
    unsafe { p.cast::<u8>().sub(HEADER).cast::<usize>().read() }
}
extern "system" fn did_alloc(_: *mut c_void, _: *mut c_void) -> i32 {
    // "Don't know", which is a valid answer.
    -1
}
extern "system" fn heap_minimize(_: *mut c_void) {}

static VTBL: IMallocVtbl = IMallocVtbl {
    query_interface,
    add_ref,
    release,
    alloc: malloc,
    realloc: grow,
    free,
    get_size,
    did_alloc,
    heap_minimize,
};
static MALLOC: CountingMalloc = CountingMalloc { vtbl: &VTBL };

fn main() {
    let dxc = Dxc::new(None).expect("Failed to load dxcompiler");

    // SAFETY: `MALLOC` is a valid `IMalloc` and is `'static`, so it outlives everything the
    // compiler allocates.
    let compiler =
        unsafe { dxc.create_compiler_with_malloc(std::ptr::from_ref(&MALLOC).cast()) }.unwrap();
    let library =
        unsafe { dxc.create_library_with_malloc(std::ptr::from_ref(&MALLOC).cast()) }.unwrap();

    let source = include_str!("copy.hlsl");
    let blob = library
        .create_blob_with_encoding_from_str(source)
        .expect("Failed to create blob");

    let result = compiler
        .compile(&blob, "copy.hlsl", "copyCs", "cs_6_0", &[], None, &[])
        .expect("Failed to compile");

    let dxil = result.get_result().unwrap();
    println!(
        "Compiled {} bytes of DXIL with {} allocations",
        dxil.as_slice().len(),
        ALLOCATIONS.load(Ordering::Relaxed)
    );
}
