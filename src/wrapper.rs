#![allow(
    clippy::too_many_arguments,
    clippy::new_without_default,
    clippy::type_complexity
)]

use crate::dxil::*;
use crate::ffi::*;
use crate::os::{HRESULT, LPCWSTR, LPWSTR, WCHAR};
use crate::utils::{from_wide, to_wide, HassleError};
use com_rs::ComPtr;
use libloading::{Library, Symbol};
use std::convert::Into;
use std::ffi::c_void;
use std::rc::Rc;

#[macro_export]
macro_rules! check_hr {
    ($hr:expr, $v: expr) => {{
        let hr = $hr;
        if hr == 0 {
            Ok($v)
        } else {
            Err(hr)
        }
    }};
}

macro_rules! check_hr_wrapped {
    ($hr:expr, $v: expr) => {{
        let hr = $hr;
        if hr == 0 {
            Ok($v)
        } else {
            Err(HassleError::Win32Error(hr))
        }
    }};
}

#[derive(Debug, Clone)]
pub struct DxcBlob {
    pub(crate) inner: ComPtr<IDxcBlob>,
}

impl DxcBlob {
    pub(crate) fn new(inner: ComPtr<IDxcBlob>) -> Self {
        unsafe {
            inner.add_ref();
        }
        Self { inner }
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, HassleError> {
        let mut blob = ComPtr::<IDxcBlob>::new();

        // This weird pointer cast from IDxcBlob to ID3DBlob is safe:
        // See https://github.com/microsoft/DirectXShaderCompiler/blob/285490211f2cbb44444a23d5f1c329881b96a43b/docs/HLSLChanges.rst
        // `The HLSL compiler avoids pulling in DirectX headers and defines an
        // IDxcBlob interface that has the same layout and interface identifier (IID).`
        let blob_ptr: *mut *mut IDxcBlob = blob.as_mut_ptr::<IDxcBlob>();
        check_hr_wrapped!(
            unsafe { winapi::um::d3dcompiler::D3DCreateBlob(slice.len(), blob_ptr as *mut *mut _) },
            unsafe {
                std::ptr::copy_nonoverlapping(
                    slice.as_ptr(),
                    blob.get_buffer_pointer() as *mut u8,
                    slice.len(),
                );
                Self { inner: blob }
            }
        )
    }

    pub fn to_vec<T>(&self) -> Vec<T>
    where
        T: Clone,
    {
        let slice = unsafe {
            std::slice::from_raw_parts(
                self.inner.get_buffer_pointer() as *const T,
                self.inner.get_buffer_size() / std::mem::size_of::<T>(),
            )
        };

        slice.to_vec()
    }
}

#[derive(Debug)]
pub struct DxcBlobEncoding {
    inner: ComPtr<IDxcBlobEncoding>,
}

impl DxcBlobEncoding {
    fn new(inner: ComPtr<IDxcBlobEncoding>) -> Self {
        Self { inner }
    }
}

impl Into<DxcBlob> for DxcBlobEncoding {
    fn into(self) -> DxcBlob {
        DxcBlob::new(ComPtr::from(&self.inner))
    }
}

#[derive(Debug)]
pub struct DxcOperationResult {
    inner: ComPtr<IDxcOperationResult>,
}

impl DxcOperationResult {
    fn new(inner: ComPtr<IDxcOperationResult>) -> Self {
        Self { inner }
    }

    pub fn get_status(&self) -> Result<u32, HRESULT> {
        let mut status: u32 = 0;
        check_hr!(unsafe { self.inner.get_status(&mut status) }, status)
    }

    pub fn get_result(&self) -> Result<DxcBlob, HRESULT> {
        let mut blob: ComPtr<IDxcBlob> = ComPtr::new();
        check_hr!(
            unsafe { self.inner.get_result(blob.as_mut_ptr()) },
            DxcBlob::new(blob)
        )
    }

    pub fn get_error_buffer(&self) -> Result<DxcBlobEncoding, HRESULT> {
        let mut blob: ComPtr<IDxcBlobEncoding> = ComPtr::new();
        check_hr!(
            unsafe { self.inner.get_error_buffer(blob.as_mut_ptr()) },
            DxcBlobEncoding::new(blob)
        )
    }
}

pub trait DxcIncludeHandler {
    fn load_source(&self, filename: String) -> Option<String>;
}

#[repr(C)]
struct DxcIncludeHandlerWrapperVtbl {
    query_interface: extern "stdcall" fn(
        *const com_rs::IUnknown,
        &com_rs::IID,
        *mut *mut core::ffi::c_void,
    ) -> com_rs::HResult,
    add_ref: extern "stdcall" fn(*const com_rs::IUnknown) -> HRESULT,
    release: extern "stdcall" fn(*const com_rs::IUnknown) -> HRESULT,
    #[cfg(not(windows))]
    complete_object_destructor: extern "stdcall" fn(*const com_rs::IUnknown) -> HRESULT,
    #[cfg(not(windows))]
    deleting_destructor: extern "stdcall" fn(*const com_rs::IUnknown) -> HRESULT,
    load_source:
        extern "stdcall" fn(*mut com_rs::IUnknown, LPCWSTR, *mut *mut IDxcBlob) -> com_rs::HResult,
}

#[repr(C)]
struct DxcIncludeHandlerWrapper<'a> {
    vtable: Box<DxcIncludeHandlerWrapperVtbl>,
    handler: Box<dyn DxcIncludeHandler>,
    pinned: Vec<Rc<String>>,
    library: &'a DxcLibrary,
}

impl<'a> DxcIncludeHandlerWrapper<'a> {
    extern "stdcall" fn query_interface(
        _me: *const com_rs::IUnknown,
        _rrid: &com_rs::IID,
        _ppv_obj: *mut *mut core::ffi::c_void,
    ) -> com_rs::HResult {
        0 // dummy impl
    }

    extern "stdcall" fn dummy(_me: *const com_rs::IUnknown) -> HRESULT {
        0 // dummy impl
    }

    extern "stdcall" fn load_source(
        me: *mut com_rs::IUnknown,
        filename: LPCWSTR,
        include_source: *mut *mut IDxcBlob,
    ) -> com_rs::HResult {
        let me = me as *mut DxcIncludeHandlerWrapper;

        let filename = crate::utils::from_wide(filename as *mut _);

        let source = unsafe { (*me).handler.load_source(filename) };

        if let Some(source) = source {
            let pinned_source = Rc::new(source);

            let mut blob = unsafe {
                (*me)
                    .library
                    .create_blob_with_encoding_from_str(pinned_source.as_str())
                    .unwrap()
            };

            unsafe {
                blob.inner.add_ref();
                *include_source = *blob.inner.as_mut_ptr();
                (*me).pinned.push(pinned_source);
            }

            0
        } else {
            -2_147_024_894 // ERROR_FILE_NOT_FOUND / 0x80070002
        }
    }
}

#[derive(Debug)]
pub struct DxcCompiler {
    inner: ComPtr<IDxcCompiler2>,
    library: DxcLibrary,
}

impl DxcCompiler {
    fn new(inner: ComPtr<IDxcCompiler2>, library: DxcLibrary) -> Self {
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

    fn prep_include_handler(
        library: &DxcLibrary,
        include_handler: Option<Box<dyn DxcIncludeHandler>>,
    ) -> Option<Box<DxcIncludeHandlerWrapper>> {
        if let Some(include_handler) = include_handler {
            let vtable = DxcIncludeHandlerWrapperVtbl {
                query_interface: DxcIncludeHandlerWrapper::query_interface,
                add_ref: DxcIncludeHandlerWrapper::dummy,
                release: DxcIncludeHandlerWrapper::dummy,
                #[cfg(not(windows))]
                complete_object_destructor: DxcIncludeHandlerWrapper::dummy,
                #[cfg(not(windows))]
                deleting_destructor: DxcIncludeHandlerWrapper::dummy,
                load_source: DxcIncludeHandlerWrapper::load_source,
            };

            Some(Box::new(DxcIncludeHandlerWrapper {
                vtable: Box::new(vtable),
                handler: include_handler,
                library,
                pinned: vec![],
            }))
        } else {
            None
        }
    }

    pub fn compile(
        &self,
        blob: &DxcBlobEncoding,
        source_name: &str,
        entry_point: &str,
        target_profile: &str,
        args: &[&str],
        include_handler: Option<Box<dyn DxcIncludeHandler>>,
        defines: &[(&str, Option<&str>)],
    ) -> Result<DxcOperationResult, (DxcOperationResult, HRESULT)> {
        let mut wide_args = vec![];
        let mut dxc_args = vec![];
        Self::prep_args(&args, &mut wide_args, &mut dxc_args);

        let mut wide_defines = vec![];
        let mut dxc_defines = vec![];
        Self::prep_defines(&defines, &mut wide_defines, &mut dxc_defines);

        let handler_wrapper = Self::prep_include_handler(&self.library, include_handler);

        let mut result: ComPtr<IDxcOperationResult> = ComPtr::new();
        let result_hr = unsafe {
            self.inner.compile(
                blob.inner.as_ptr(),
                to_wide(source_name).as_ptr(),
                to_wide(entry_point).as_ptr(),
                to_wide(target_profile).as_ptr(),
                dxc_args.as_ptr(),
                dxc_args.len() as u32,
                dxc_defines.as_ptr(),
                dxc_defines.len() as u32,
                handler_wrapper
                    .as_ref()
                    .map_or(std::ptr::null(), |v| &**v as *const _ as *const _),
                result.as_mut_ptr(),
            )
        };

        let mut compile_error = 0u32;
        unsafe {
            result.get_status(&mut compile_error);
        }

        if result_hr == 0 && compile_error == 0 {
            Ok(DxcOperationResult::new(result))
        } else {
            Err((DxcOperationResult::new(result), result_hr))
        }
    }

    pub fn compile_with_debug(
        &self,
        blob: &DxcBlobEncoding,
        source_name: &str,
        entry_point: &str,
        target_profile: &str,
        args: &[&str],
        include_handler: Option<Box<dyn DxcIncludeHandler>>,
        defines: &[(&str, Option<&str>)],
    ) -> Result<(DxcOperationResult, String, DxcBlob), (DxcOperationResult, HRESULT)> {
        let mut wide_args = vec![];
        let mut dxc_args = vec![];
        Self::prep_args(&args, &mut wide_args, &mut dxc_args);

        let mut wide_defines = vec![];
        let mut dxc_defines = vec![];
        Self::prep_defines(&defines, &mut wide_defines, &mut dxc_defines);

        let handler_wrapper = Self::prep_include_handler(&self.library, include_handler);

        let mut result: ComPtr<IDxcOperationResult> = ComPtr::new();
        let mut debug_blob: ComPtr<IDxcBlob> = ComPtr::new();
        let mut debug_filename: LPWSTR = std::ptr::null_mut();

        let result_hr = unsafe {
            self.inner.compile_with_debug(
                blob.inner.as_ptr(),
                to_wide(source_name).as_ptr(),
                to_wide(entry_point).as_ptr(),
                to_wide(target_profile).as_ptr(),
                dxc_args.as_ptr(),
                dxc_args.len() as u32,
                dxc_defines.as_ptr(),
                dxc_defines.len() as u32,
                handler_wrapper
                    .as_ref()
                    .map_or(std::ptr::null(), |v| &**v as *const _ as *const _),
                result.as_mut_ptr(),
                &mut debug_filename,
                debug_blob.as_mut_ptr(),
            )
        };

        let mut compile_error = 0u32;
        unsafe {
            result.get_status(&mut compile_error);
        }

        if result_hr == 0 && compile_error == 0 {
            Ok((
                DxcOperationResult::new(result),
                from_wide(debug_filename),
                DxcBlob::new(debug_blob),
            ))
        } else {
            Err((DxcOperationResult::new(result), result_hr))
        }
    }

    pub fn preprocess(
        &self,
        blob: &DxcBlobEncoding,
        source_name: &str,
        args: &[&str],
        include_handler: Option<Box<dyn DxcIncludeHandler>>,
        defines: &[(&str, Option<&str>)],
    ) -> Result<DxcOperationResult, (DxcOperationResult, HRESULT)> {
        let mut wide_args = vec![];
        let mut dxc_args = vec![];
        Self::prep_args(&args, &mut wide_args, &mut dxc_args);

        let mut wide_defines = vec![];
        let mut dxc_defines = vec![];
        Self::prep_defines(&defines, &mut wide_defines, &mut dxc_defines);

        let handler_wrapper = Self::prep_include_handler(&self.library, include_handler);

        let mut result: ComPtr<IDxcOperationResult> = ComPtr::new();
        let result_hr = unsafe {
            self.inner.preprocess(
                blob.inner.as_ptr(),
                to_wide(source_name).as_ptr(),
                dxc_args.as_ptr(),
                dxc_args.len() as u32,
                dxc_defines.as_ptr(),
                dxc_defines.len() as u32,
                handler_wrapper
                    .as_ref()
                    .map_or(std::ptr::null(), |v| &**v as *const _ as *const _),
                result.as_mut_ptr(),
            )
        };

        let mut compile_error = 0u32;
        unsafe {
            result.get_status(&mut compile_error);
        }

        if result_hr == 0 && compile_error == 0 {
            Ok(DxcOperationResult::new(result))
        } else {
            Err((DxcOperationResult::new(result), result_hr))
        }
    }

    pub fn disassemble(&self, blob: &DxcBlob) -> Result<DxcBlobEncoding, HRESULT> {
        let mut result_blob: ComPtr<IDxcBlobEncoding> = ComPtr::new();
        check_hr!(
            unsafe {
                self.inner
                    .disassemble(blob.inner.as_ptr(), result_blob.as_mut_ptr())
            },
            DxcBlobEncoding::new(result_blob)
        )
    }
}

#[derive(Debug)]
pub struct DxcLibrary {
    inner: ComPtr<IDxcLibrary>,
}

impl DxcLibrary {
    fn new(inner: ComPtr<IDxcLibrary>) -> Self {
        Self { inner }
    }

    pub fn create_blob_with_encoding(&self, data: &[u8]) -> Result<DxcBlobEncoding, HRESULT> {
        let mut blob: ComPtr<IDxcBlobEncoding> = ComPtr::new();
        check_hr!(
            unsafe {
                self.inner.create_blob_with_encoding_from_pinned(
                    data.as_ptr() as *const c_void,
                    data.len() as u32,
                    0, // Binary; no code page
                    blob.as_mut_ptr(),
                )
            },
            DxcBlobEncoding::new(blob)
        )
    }

    pub fn create_blob_with_encoding_from_str(
        &self,
        text: &str,
    ) -> Result<DxcBlobEncoding, HRESULT> {
        let mut blob: ComPtr<IDxcBlobEncoding> = ComPtr::new();
        const CP_UTF8: u32 = 65001; // UTF-8 translation

        check_hr!(
            unsafe {
                self.inner.create_blob_with_encoding_from_pinned(
                    text.as_ptr() as *const c_void,
                    text.len() as u32,
                    CP_UTF8,
                    blob.as_mut_ptr(),
                )
            },
            DxcBlobEncoding::new(blob)
        )
    }

    pub fn get_blob_as_string(&self, blob: &DxcBlobEncoding) -> String {
        let mut blob_utf8: ComPtr<IDxcBlobEncoding> = ComPtr::new();

        unsafe {
            self.inner
                .get_blob_as_utf8(blob.inner.as_ptr(), blob_utf8.as_mut_ptr())
        };

        let slice = unsafe {
            std::slice::from_raw_parts(
                blob_utf8.get_buffer_pointer() as *const u8,
                blob_utf8.get_buffer_size(),
            )
        };

        String::from_utf8(slice.to_vec()).unwrap()
    }
}

#[derive(Debug)]
pub struct Dxc {
    dxc_lib: Library,
}

#[cfg(target_os = "windows")]
fn dxcompiler_lib_name() -> &'static str {
    "dxcompiler.dll"
}

#[cfg(target_os = "linux")]
fn dxcompiler_lib_name() -> &'static str {
    "./libdxcompiler.so"
}

#[cfg(target_os = "macos")]
fn dxcompiler_lib_name() -> &'static str {
    "./libdxcompiler.dynlib"
}

impl Dxc {
    pub fn new() -> Result<Self, HassleError> {
        let lib_name = dxcompiler_lib_name();
        let dxc_lib = Library::new(lib_name).map_err(|e| HassleError::LoadLibraryError {
            filename: lib_name.to_string(),
            inner: e,
        })?;

        Ok(Self { dxc_lib })
    }

    pub(crate) fn get_dxc_create_instance(
        &self,
    ) -> Result<Symbol<DxcCreateInstanceProc>, HassleError> {
        Ok(unsafe { self.dxc_lib.get(b"DxcCreateInstance\0")? })
    }

    pub fn create_compiler(&self) -> Result<DxcCompiler, HassleError> {
        let mut compiler: ComPtr<IDxcCompiler2> = ComPtr::new();
        check_hr_wrapped!(
            self.get_dxc_create_instance()?(
                &CLSID_DxcCompiler,
                &IID_IDxcCompiler2,
                compiler.as_mut_ptr(),
            ),
            DxcCompiler::new(compiler, self.create_library()?)
        )
    }

    pub fn create_library(&self) -> Result<DxcLibrary, HassleError> {
        let mut library: ComPtr<IDxcLibrary> = ComPtr::new();
        check_hr_wrapped!(
            self.get_dxc_create_instance()?(
                &CLSID_DxcLibrary,
                &IID_IDxcLibrary,
                library.as_mut_ptr(),
            ),
            DxcLibrary::new(library)
        )
    }

    pub fn create_container_reflection(&self) -> Result<DxcContainerReflection, HassleError> {
        let mut reflection: ComPtr<IDxcContainerReflection> = ComPtr::new();
        check_hr_wrapped!(
            self.get_dxc_create_instance()?(
                &CLSID_DxcContainerReflection,
                &IID_IDxcContainerReflection,
                reflection.as_mut_ptr(),
            ),
            DxcContainerReflection::new(reflection)
        )
    }

    pub fn create_validator(&self) -> Result<DxcValidator, HassleError> {
        let mut validator: ComPtr<IDxcValidator> = ComPtr::new();
        check_hr_wrapped!(
            self.get_dxc_create_instance()?(
                &CLSID_DxcValidator,
                &IID_IDxcValidator,
                validator.as_mut_ptr(),
            ),
            DxcValidator::new(validator)
        )
    }
}

#[derive(Debug)]
pub struct DxcValidator {
    inner: ComPtr<IDxcValidator>,
}

pub type DxcValidatorVersion = (u32, u32);

impl DxcValidator {
    pub(crate) fn new(inner: ComPtr<IDxcValidator>) -> Self {
        Self { inner }
    }

    pub fn version(&self) -> Result<DxcValidatorVersion, HRESULT> {
        let mut version: ComPtr<IDxcVersionInfo> = ComPtr::new();

        let result_hr = unsafe {
            self.inner
                .query_interface(&IID_IDxcVersionInfo, version.as_mut_ptr())
        };

        if result_hr != 0 {
            return Err(result_hr);
        }

        let mut major = 0;
        let mut minor = 0;

        check_hr! {
            unsafe { version.get_version(&mut major, &mut minor) },
            (major, minor)
        }
    }

    pub fn validate(&self, blob: DxcBlob) -> Result<DxcBlob, (DxcOperationResult, HRESULT)> {
        let mut result: ComPtr<IDxcOperationResult> = ComPtr::new();
        let result_hr = unsafe {
            self.inner.validate(
                blob.inner.as_ptr(),
                DXC_VALIDATOR_FLAGS_IN_PLACE_EDIT,
                result.as_mut_ptr(),
            )
        };

        let mut validate_status = 0u32;
        unsafe { result.get_status(&mut validate_status) };

        if result_hr == 0 && validate_status == 0 {
            Ok(blob)
        } else {
            Err((DxcOperationResult::new(result), result_hr))
        }
    }
}
