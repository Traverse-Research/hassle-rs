#![allow(
    clippy::too_many_arguments,
    clippy::new_without_default,
    clippy::type_complexity
)]

use crate::ffi::*;
use crate::os::{HRESULT, LPCWSTR, LPWSTR, WCHAR};
use crate::utils::{from_wide, to_wide, HassleError, Result};
use com::{class, interfaces::IUnknown, production::Class, production::ClassAllocation, Interface};
use libloading::{library_filename, Library, Symbol};
use std::cell::RefCell;
use std::convert::TryFrom;
use std::fmt;
use std::ops::Deref;
use std::path::PathBuf;
use std::pin::Pin;

#[repr(transparent)]
pub struct DxcBlob {
    inner: IDxcBlob,
}

impl fmt::Debug for DxcBlob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DxcBlob")
            .field("inner", &self.inner)
            .field("buffer_pointer()", &unsafe {
                self.inner.get_buffer_pointer()
            })
            .field("buffer_size()", &unsafe { self.inner.get_buffer_size() })
            .finish()
    }
}

impl DxcBlob {
    fn new(inner: IDxcBlob) -> Self {
        Self { inner }
    }

    pub fn as_slice<T>(&self) -> &[T] {
        let bytes = unsafe { self.inner.get_buffer_size() };
        if bytes == 0 {
            &[]
        } else {
            unsafe {
                std::slice::from_raw_parts(
                    self.inner.get_buffer_pointer().cast(),
                    bytes / size_of::<T>(),
                )
            }
        }
    }

    pub fn as_mut_slice<T>(&mut self) -> &mut [T] {
        let bytes = unsafe { self.inner.get_buffer_size() };
        if bytes == 0 {
            &mut []
        } else {
            unsafe {
                std::slice::from_raw_parts_mut(
                    self.inner.get_buffer_pointer().cast(),
                    bytes / size_of::<T>(),
                )
            }
        }
    }

    pub fn to_vec<T: Clone>(&self) -> Vec<T> {
        self.as_slice().to_vec()
    }
}

impl AsRef<[u8]> for DxcBlob {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl AsMut<[u8]> for DxcBlob {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DxcEncoding {
    Unknown,
    Utf8,
}

#[repr(transparent)]
pub struct DxcBlobEncoding {
    inner: IDxcBlobEncoding,
}

impl fmt::Debug for DxcBlobEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DxcBlobEncoding")
            .field("blob", self.deref())
            .field("encoding", &self.encoding().unwrap())
            .finish()
    }
}

impl DxcBlobEncoding {
    fn new(inner: IDxcBlobEncoding) -> Self {
        Self { inner }
    }

    fn encoding(&self) -> Result<DxcEncoding> {
        let mut known = 0;
        let mut code_page = 0;
        unsafe { self.inner.get_encoding(&mut known, &mut code_page) }.result()?;
        Ok(if known == 0 {
            assert_eq!(code_page, CP_ACP);
            DxcEncoding::Unknown
        } else if known == 1 {
            match code_page {
                CP_UTF8 => DxcEncoding::Utf8,
                x => todo!("Unknown codepage {x:#}"),
            }
        } else {
            panic!("Unexpected boolean value {:#}", known)
        })
    }

    /// Returns [`None`] if [`Self::encoding()`] is not [`DxcEncoding::Utf8`].  Call
    /// [`DxcLibrary::get_blob_as_utf8()`] to convert it, and call this function on the resulting
    /// blob again.
    pub fn as_str(&self) -> Option<&str> {
        match self.encoding().expect("Known encoding") {
            DxcEncoding::Unknown => None,
            DxcEncoding::Utf8 => Some(
                std::str::from_utf8(self.as_slice())
                    .expect("DXC claimed the buffer should contain UTF-8"),
            ),
        }
    }
}

impl From<DxcBlobEncoding> for DxcBlob {
    fn from(blob: DxcBlobEncoding) -> Self {
        // Implemented by transmute because IDxcBlob is a supertrait of IDxcBlobEncoding
        unsafe { std::mem::transmute(blob) }
    }
}

impl TryFrom<DxcBlob> for DxcBlobEncoding {
    type Error = ();

    fn try_from(blob: DxcBlob) -> Result<Self, Self::Error> {
        blob.inner
            .query_interface::<IDxcBlobEncoding>()
            .map(DxcBlobEncoding::new)
            .ok_or(())
    }
}

impl Deref for DxcBlobEncoding {
    type Target = DxcBlob;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

pub struct DxcOperationResult {
    inner: IDxcOperationResult,
}

impl DxcOperationResult {
    fn new(inner: IDxcOperationResult) -> Self {
        Self { inner }
    }

    pub fn get_status(&self) -> Result<HRESULT> {
        let mut status = 0;
        unsafe { self.inner.get_status(&mut status) }.result_with_success(HRESULT(status))
    }

    pub fn get_result(&self) -> Result<DxcBlob> {
        let mut blob = None;
        unsafe { self.inner.get_result(&mut blob) }.result()?;
        Ok(DxcBlob::new(blob.unwrap()))
    }

    pub fn get_error_buffer(&self) -> Result<DxcBlobEncoding> {
        let mut blob = None;

        unsafe { self.inner.get_error_buffer(&mut blob) }.result()?;
        Ok(DxcBlobEncoding::new(blob.unwrap()))
    }
}

pub trait DxcIncludeHandler {
    fn load_source(&mut self, filename: String) -> Option<String>;
}

class! {
    #[no_class_factory]
    class DxcIncludeHandlerWrapper: IDxcIncludeHandler {
        // Com-rs intentionally does not support lifetimes in its class structs
        // since they live on the heap and their lifetime can be prolonged for
        // as long as someone keeps a reference through `add_ref()`.
        // The only way for us to access the library and handler implementation,
        // which are now intentionally behind a borrow to signify our promise
        // regarding lifetime, is by transmuting them away and "ensuring" the
        // class object is discarded at the end of our function call.

        library: &'static DxcLibrary,
        handler: RefCell<&'static mut dyn DxcIncludeHandler>,

        pinned: RefCell<Vec<Pin<String>>>,
    }

    impl IDxcIncludeHandler for DxcIncludeHandlerWrapper {
        fn load_source(&self, filename: LPCWSTR, include_source: *mut Option<IDxcBlob>) -> HRESULT {
            let filename = crate::utils::from_wide(filename);

            let mut handler = self.handler.borrow_mut();
            let source = handler.load_source(filename);

            if let Some(source) = source {
                let source = Pin::new(source);
                let blob = self.library
                    .create_blob_with_encoding_from_str(&source)
                    .unwrap();

                // unsafe { *include_source = Some(DxcBlob::from(blob).inner) };
                // TODO: We shouldn't need a clone here, if we could freely upcast by moving.
                unsafe { *include_source = Some(blob.deref().inner.clone()) };
                self.pinned.borrow_mut().push(source);

                // NOERROR
                0
            } else {
                -2_147_024_894 // ERROR_FILE_NOT_FOUND / 0x80070002
            }
            .into()
        }
    }
}

/// Represents a reference to a COM object that should only live as long as itself
///
/// In other words, on [`drop()`] we assert that the refcount is decremented to zero,
/// rather than allowing it to be referenced externally (i.e. [`Class::dec_ref_count()`]
/// returning `> 0`).
/// This object functions a lot like [`ClassAllocation`]: see its similar [`drop()`]
/// implementation for details.
///
/// Note that COM objects live on the heap by design, because of this refcount system.
struct LocalClassAllocation<T: Class>(core::pin::Pin<Box<T>>);

impl<T: Class> LocalClassAllocation<T> {
    fn new(allocation: ClassAllocation<T>) -> Self {
        // TODO: There is no way to take the internal, owned box out of com-rs's
        // allocation wrapper.
        // https://github.com/microsoft/com-rs/issues/236 covers this issue as a whole,
        // including lifetime support and this `LocalClassAllocation` upstream.
        let inner: core::mem::ManuallyDrop<core::pin::Pin<Box<T>>> =
            unsafe { std::mem::transmute(allocation) };

        Self(core::mem::ManuallyDrop::into_inner(inner))
    }

    // TODO: Return a borrow of this interface?
    // query_interface() is not behind one of the traits
    // fn query_interface<T>(&self) -> Option<T> {
    //     self.0.query_interface::<T>().unwrap()
    // }
}

impl<T: Class> Deref for LocalClassAllocation<T> {
    type Target = core::pin::Pin<Box<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Class> Drop for LocalClassAllocation<T> {
    fn drop(&mut self) {
        // Check if we are the only remaining reference to this object
        assert_eq!(
            unsafe { self.0.dec_ref_count() },
            0,
            "COM object is still referenced"
        );
        // Now that we're the last one to give up our refcount, it is safe
        // for the internal object to get dropped.
    }
}

impl DxcIncludeHandlerWrapper {
    /// SAFETY: Make sure the returned object does _not_ outlive the lifetime
    /// of either `library` nor `include_handler`
    unsafe fn create_include_handler(
        library: &'_ DxcLibrary,
        include_handler: &'_ mut dyn DxcIncludeHandler,
    ) -> LocalClassAllocation<DxcIncludeHandlerWrapper> {
        #[allow(clippy::missing_transmute_annotations)]
        LocalClassAllocation::new(Self::allocate(
            std::mem::transmute(library),
            RefCell::new(std::mem::transmute(include_handler)),
            RefCell::new(vec![]),
        ))
    }
}

pub struct DxcCompiler {
    inner: IDxcCompiler2,
    library: DxcLibrary,
}

impl DxcCompiler {
    fn new(inner: IDxcCompiler2, library: DxcLibrary) -> Self {
        Self { inner, library }
    }

    fn prep_defines(
        defines: &[(&str, Option<&str>)],
        wide_defines: &mut Vec<(Vec<WCHAR>, Vec<WCHAR>)>,
        dxc_defines: &mut Vec<DxcDefine>,
    ) {
        for (name, value) in defines {
            if value.is_none() {
                wide_defines.push((to_wide(name), to_wide("1")));
            } else {
                wide_defines.push((to_wide(name), to_wide(value.unwrap())));
            }
        }

        for (ref name, ref value) in wide_defines {
            dxc_defines.push(DxcDefine {
                name: name.as_ptr(),
                value: value.as_ptr(),
            });
        }
    }

    fn prep_args(args: &[&str], wide_args: &mut Vec<Vec<WCHAR>>, dxc_args: &mut Vec<LPCWSTR>) {
        for a in args {
            wide_args.push(to_wide(a));
        }

        for a in wide_args {
            dxc_args.push(a.as_ptr());
        }
    }

    pub fn compile(
        &self,
        blob: &DxcBlobEncoding,
        source_name: &str,
        entry_point: &str,
        target_profile: &str,
        args: &[&str],
        include_handler: Option<&mut dyn DxcIncludeHandler>,
        defines: &[(&str, Option<&str>)],
        // ) -> Result<DxcBlob, (HRESULT, DxcBlob)> {
    ) -> Result<DxcOperationResult, HassleError> {
        let mut wide_args = vec![];
        let mut dxc_args = vec![];
        Self::prep_args(args, &mut wide_args, &mut dxc_args);

        let mut wide_defines = vec![];
        let mut dxc_defines = vec![];
        Self::prep_defines(defines, &mut wide_defines, &mut dxc_defines);

        // Keep alive on the stack
        let include_handler = include_handler.map(|include_handler| unsafe {
            DxcIncludeHandlerWrapper::create_include_handler(&self.library, include_handler)
        });
        // TODO: query_interface() should have a borrow on LocalClassAllocation to prevent things going kaboom
        let include_handler = include_handler
            .as_ref()
            .map(|i| i.query_interface().unwrap());

        let mut result = None;
        unsafe {
            self.inner.compile(
                &blob.inner,
                to_wide(source_name).as_ptr(),
                to_wide(entry_point).as_ptr(),
                to_wide(target_profile).as_ptr(),
                dxc_args.as_ptr(),
                dxc_args.len() as u32,
                dxc_defines.as_ptr(),
                dxc_defines.len() as u32,
                &include_handler,
                &mut result,
            )
        }
        .result()?;

        let result = result.expect("Non-null IDxcOperationResult");
        Ok(DxcOperationResult::new(result))
    }

    pub fn compile_with_debug(
        &self,
        blob: &DxcBlobEncoding,
        source_name: &str,
        entry_point: &str,
        target_profile: &str,
        args: &[&str],
        include_handler: Option<&mut dyn DxcIncludeHandler>,
        defines: &[(&str, Option<&str>)],
    ) -> Result<(DxcOperationResult, (String, DxcBlob)), HassleError> {
        let mut wide_args = vec![];
        let mut dxc_args = vec![];
        Self::prep_args(args, &mut wide_args, &mut dxc_args);

        let mut wide_defines = vec![];
        let mut dxc_defines = vec![];
        Self::prep_defines(defines, &mut wide_defines, &mut dxc_defines);

        // Keep alive on the stack
        let include_handler = include_handler.map(|include_handler| unsafe {
            DxcIncludeHandlerWrapper::create_include_handler(&self.library, include_handler)
        });
        let include_handler = include_handler
            .as_ref()
            .map(|i| i.query_interface().unwrap());

        let mut result = None;
        let mut debug_blob = None;
        let mut debug_filename: LPWSTR = std::ptr::null_mut();

        unsafe {
            self.inner.compile_with_debug(
                &blob.inner,
                to_wide(source_name).as_ptr(),
                to_wide(entry_point).as_ptr(),
                to_wide(target_profile).as_ptr(),
                dxc_args.as_ptr(),
                dxc_args.len() as u32,
                dxc_defines.as_ptr(),
                dxc_defines.len() as u32,
                include_handler,
                &mut result,
                &mut debug_filename,
                &mut debug_blob,
            )
        }
        .result()?;

        let result = result.expect("Non-null IDxcOperationResult");
        let debug_blob = debug_blob.expect("Non-null debug blob");
        Ok((
            DxcOperationResult::new(result),
            (from_wide(debug_filename), DxcBlob::new(debug_blob)),
        ))
    }

    pub fn preprocess(
        &self,
        blob: &DxcBlobEncoding,
        source_name: &str,
        args: &[&str],
        include_handler: Option<&mut dyn DxcIncludeHandler>,
        defines: &[(&str, Option<&str>)],
    ) -> Result<DxcOperationResult, HassleError> {
        let mut wide_args = vec![];
        let mut dxc_args = vec![];
        Self::prep_args(args, &mut wide_args, &mut dxc_args);

        let mut wide_defines = vec![];
        let mut dxc_defines = vec![];
        Self::prep_defines(defines, &mut wide_defines, &mut dxc_defines);

        // Keep alive on the stack
        let include_handler = include_handler.map(|include_handler| unsafe {
            DxcIncludeHandlerWrapper::create_include_handler(&self.library, include_handler)
        });
        let include_handler = include_handler
            .as_ref()
            .map(|i| i.query_interface().unwrap());

        let mut result = None;
        unsafe {
            self.inner.preprocess(
                &blob.inner,
                to_wide(source_name).as_ptr(),
                dxc_args.as_ptr(),
                dxc_args.len() as u32,
                dxc_defines.as_ptr(),
                dxc_defines.len() as u32,
                include_handler,
                &mut result,
            )
        }
        .result()?;

        let result = result.expect("Non-null IDxcOperationResult");
        Ok(DxcOperationResult::new(result))
    }

    pub fn disassemble(&self, blob: &DxcBlob) -> Result<DxcBlobEncoding> {
        let mut result_blob = None;
        unsafe { self.inner.disassemble(&blob.inner, &mut result_blob) }.result()?;
        Ok(DxcBlobEncoding::new(result_blob.unwrap()))
    }
}

#[derive(Clone)]
pub struct DxcLibrary {
    inner: IDxcLibrary,
}

impl DxcLibrary {
    fn new(inner: IDxcLibrary) -> Self {
        Self { inner }
    }

    pub fn create_blob_with_encoding(&self, data: &[u8]) -> Result<DxcBlobEncoding> {
        let mut blob = None;

        unsafe {
            self.inner.create_blob_with_encoding_from_pinned(
                data.as_ptr().cast(),
                data.len() as u32,
                0, // Binary; no code page
                &mut blob,
            )
        }
        .result()?;
        Ok(DxcBlobEncoding::new(blob.unwrap()))
    }

    pub fn create_blob_with_encoding_from_str(&self, text: &str) -> Result<DxcBlobEncoding> {
        let mut blob = None;

        unsafe {
            self.inner.create_blob_with_encoding_from_pinned(
                text.as_ptr().cast(),
                text.len() as u32,
                CP_UTF8,
                &mut blob,
            )
        }
        .result()?;
        Ok(DxcBlobEncoding::new(blob.unwrap()))
    }

    /// Convert or return matching encoded text blob as UTF-8.
    pub fn get_blob_as_utf8(&self, blob: &DxcBlob) -> Result<DxcBlobEncoding> {
        let mut blob_utf8 = None;

        unsafe { self.inner.get_blob_as_utf8(&blob.inner, &mut blob_utf8) }.result()?;

        let blob_utf8 = DxcBlobEncoding::new(blob_utf8.unwrap());
        assert_eq!(blob_utf8.encoding()?, DxcEncoding::Utf8);

        Ok(blob_utf8)
    }
}

#[derive(Debug)]
pub struct Dxc {
    dxc_lib: Library,
}

impl Dxc {
    /// `lib_path` is an optional path to the library.  Otherwise
    /// [`libloading::library_filename("dxcompiler")`] is used.
    pub fn new(lib_path: Option<PathBuf>) -> Result<Self> {
        let lib_path = lib_path.unwrap_or_else(|| PathBuf::from(library_filename("dxcompiler")));

        let dxc_lib =
            unsafe { Library::new(&lib_path) }.map_err(|e| HassleError::LoadLibraryError {
                filename: lib_path,
                inner: e,
            })?;

        Ok(Self { dxc_lib })
    }

    pub(crate) fn get_dxc_create_instance<T>(
        &self,
    ) -> Result<Symbol<'_, DxcCreateInstanceProc<T>>> {
        Ok(unsafe { self.dxc_lib.get(b"DxcCreateInstance\0")? })
    }

    pub fn create_compiler(&self) -> Result<DxcCompiler> {
        let mut compiler = None;

        self.get_dxc_create_instance()?(&CLSID_DxcCompiler, &IDxcCompiler2::IID, &mut compiler)
            .result()?;
        Ok(DxcCompiler::new(
            compiler.unwrap(),
            self.create_library().unwrap(),
        ))
    }

    pub fn create_library(&self) -> Result<DxcLibrary> {
        let mut library = None;
        self.get_dxc_create_instance()?(&CLSID_DxcLibrary, &IDxcLibrary::IID, &mut library)
            .result()?;
        Ok(DxcLibrary::new(library.unwrap()))
    }

    pub fn create_reflector(&self) -> Result<DxcReflector> {
        let mut reflector = None;

        self.get_dxc_create_instance()?(
            &CLSID_DxcContainerReflection,
            &IDxcContainerReflection::IID,
            &mut reflector,
        )
        .result()?;
        Ok(DxcReflector::new(reflector.unwrap()))
    }
}

pub struct DxcValidator {
    inner: IDxcValidator,
}

pub type DxcValidatorVersion = (u32, u32);

impl DxcValidator {
    fn new(inner: IDxcValidator) -> Self {
        Self { inner }
    }

    pub fn version(&self) -> Result<DxcValidatorVersion> {
        let version = self
            .inner
            .query_interface::<IDxcVersionInfo>()
            .ok_or(HassleError::Win32Error(HRESULT(com::sys::E_NOINTERFACE)))?;

        let mut major = 0;
        let mut minor = 0;

        unsafe { version.get_version(&mut major, &mut minor) }.result_with_success((major, minor))
    }

    pub fn validate(&self, blob: &DxcBlob) -> Result<DxcOperationResult> {
        let mut result = None;
        unsafe {
            self.inner
                .validate(&blob.inner, DXC_VALIDATOR_FLAGS_IN_PLACE_EDIT, &mut result)
        }
        .result()?;

        let result = result.expect("Non-null IDxcOperationResult");
        Ok(DxcOperationResult::new(result))
    }
}

pub struct Reflection {
    inner: ID3D12ShaderReflection,
}
impl Reflection {
    fn new(inner: ID3D12ShaderReflection) -> Self {
        Self { inner }
    }

    pub fn thread_group_size(&self) -> [u32; 3] {
        let (mut size_x, mut size_y, mut size_z) = (0u32, 0u32, 0u32);
        unsafe {
            self.inner
                .get_thread_group_size(&mut size_x, &mut size_y, &mut size_z)
        };
        [size_x, size_y, size_z]
    }
}

pub struct DxcReflector {
    inner: IDxcContainerReflection,
}
impl DxcReflector {
    fn new(inner: IDxcContainerReflection) -> Self {
        Self { inner }
    }

    pub fn reflect(&self, blob: DxcBlob) -> Result<Reflection> {
        let result_hr = unsafe { self.inner.load(blob.inner) };
        if result_hr.is_err() {
            return Err(HassleError::Win32Error(result_hr));
        }

        let mut shader_idx = 0;
        let result_hr = unsafe { self.inner.find_first_part_kind(DFCC_DXIL, &mut shader_idx) };
        if result_hr.is_err() {
            return Err(HassleError::Win32Error(result_hr));
        }

        let mut reflection = None::<IUnknown>;
        let result_hr = unsafe {
            self.inner.get_part_reflection(
                shader_idx,
                &ID3D12ShaderReflection::IID,
                &mut reflection,
            )
        };
        if result_hr.is_err() {
            return Err(HassleError::Win32Error(result_hr));
        }

        Ok(Reflection::new(
            reflection.unwrap().query_interface().unwrap(),
        ))
    }
}

#[derive(Debug)]
pub struct Dxil {
    dxil_lib: Library,
}

impl Dxil {
    /// `lib_path` is an optional path to the library.  Otherwise
    /// [`libloading::library_filename("dxil")`] is used.
    pub fn new(lib_path: Option<PathBuf>) -> Result<Self> {
        let lib_path = lib_path.unwrap_or_else(|| PathBuf::from(library_filename("dxil")));

        let dxil_lib =
            unsafe { Library::new(&lib_path) }.map_err(|e| HassleError::LoadLibraryError {
                filename: lib_path.to_owned(),
                inner: e,
            })?;

        Ok(Self { dxil_lib })
    }

    fn get_dxc_create_instance<T>(&self) -> Result<Symbol<'_, DxcCreateInstanceProc<T>>> {
        Ok(unsafe { self.dxil_lib.get(b"DxcCreateInstance\0")? })
    }

    pub fn create_validator(&self) -> Result<DxcValidator> {
        let mut validator = None;
        self.get_dxc_create_instance()?(&CLSID_DxcValidator, &IDxcValidator::IID, &mut validator)
            .result()?;
        Ok(DxcValidator::new(validator.unwrap()))
    }
}
