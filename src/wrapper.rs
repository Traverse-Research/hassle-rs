use crate::ffi::*;
use crate::os::{HRESULT, LPCSTR, LPSTR};
use crate::utils::{from_wide, to_wide};
use com_rs::ComPtr;
use libloading::{Library, Symbol};
use std::convert::Into;
use std::ffi::c_void;
use std::rc::Rc;

#[macro_export]
macro_rules! return_hr {
    ($hr:expr, $v: expr) => {
        let hr = $hr;
        if hr == 0 {
            return Ok($v);
        } else {
            return Err(hr);
        }
    };
}

#[derive(Debug)]
pub struct DxcBlob {
    inner: ComPtr<IDxcBlob>,
}

impl DxcBlob {
    fn new(inner: ComPtr<IDxcBlob>) -> Self {
        Self { inner }
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
        return_hr!(unsafe { self.inner.get_status(&mut status) }, status);
    }

    pub fn get_result(&self) -> Result<DxcBlob, HRESULT> {
        let mut blob: ComPtr<IDxcBlob> = ComPtr::new();
        return_hr!(
            unsafe { self.inner.get_result(blob.as_mut_ptr()) },
            DxcBlob::new(blob)
        );
    }

    pub fn get_error_buffer(&self) -> Result<DxcBlobEncoding, HRESULT> {
        let mut blob: ComPtr<IDxcBlobEncoding> = ComPtr::new();
        return_hr!(
            unsafe { self.inner.get_error_buffer(blob.as_mut_ptr()) },
            DxcBlobEncoding::new(blob)
        );
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
    add_ref: extern "stdcall" fn(*const com_rs::IUnknown) -> u32,
    release: extern "stdcall" fn(*const com_rs::IUnknown) -> u32,
    load_source:
        extern "stdcall" fn(*mut com_rs::IUnknown, LPCWSTR, *mut *mut IDxcBlob) -> com_rs::HResult,
}

#[repr(C)]
struct DxcIncludeHandlerWrapper<'a> {
    vtable: Box<DxcIncludeHandlerWrapperVtbl>,
    handler: Box<dyn DxcIncludeHandler>,
    blobs: Vec<DxcBlobEncoding>,
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

    extern "stdcall" fn add_ref(_me: *const com_rs::IUnknown) -> u32 {
        0 // dummy impl
    }

    extern "stdcall" fn release(_me: *const com_rs::IUnknown) -> u32 {
        0 // dummy impl
    }

    extern "stdcall" fn load_source(
        me: *mut com_rs::IUnknown,
        filename: LPCWSTR,
        include_source: *mut *mut IDxcBlob,
    ) -> com_rs::HResult {
        let me = me as *mut DxcIncludeHandlerWrapper;

        let filename = crate::utils::from_wide(filename as *mut _);

        let source = unsafe { &(*me).handler.load_source(filename) };

        if let Some(source) = source {
            let pinned_source = Rc::new(source.clone());

            let mut blob = unsafe {
                (*me)
                    .library
                    .create_blob_with_encoding_from_str(Rc::clone(&pinned_source))
                    .unwrap()
            };

            unsafe {
                *include_source = *blob.inner.as_mut_ptr();
                (*me).blobs.push(blob);
                (*me).pinned.push(Rc::clone(&pinned_source));
            }

            0
        } else {
            -2147024894i32 // ERROR_FILE_NOT_FOUND / 0x80070002
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
        wide_defines: &mut Vec<(Vec<u16>, Vec<u16>)>,
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

    fn prep_args(args: &[&str], wide_args: &mut Vec<Vec<u16>>, dxc_args: &mut Vec<LPCWSTR>) {
        for a in args {
            wide_args.push(to_wide(a));
        }

        for ref a in wide_args {
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
                add_ref: DxcIncludeHandlerWrapper::add_ref,
                release: DxcIncludeHandlerWrapper::release,
                load_source: DxcIncludeHandlerWrapper::load_source,
            };

            Some(Box::new(DxcIncludeHandlerWrapper {
                vtable: Box::new(vtable),
                handler: include_handler,
                blobs: vec![],
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
                handler_wrapper.map_or(std::ptr::null(), |v| Box::into_raw(v) as _),
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
                handler_wrapper.map_or(std::ptr::null(), |v| Box::into_raw(v) as _),
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
                handler_wrapper.map_or(std::ptr::null(), |v| Box::into_raw(v) as _),
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
        return_hr!(
            unsafe {
                self.inner
                    .disassemble(blob.inner.as_ptr(), result_blob.as_mut_ptr())
            },
            DxcBlobEncoding::new(result_blob)
        );
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
        return_hr!(
            unsafe {
                self.inner.create_blob_with_encoding_from_pinned(
                    data.as_ptr() as *const c_void,
                    data.len() as u32,
                    0, // Binary; no code page
                    blob.as_mut_ptr(),
                )
            },
            DxcBlobEncoding::new(blob)
        );
    }

    pub fn create_blob_with_encoding_from_str(
        &self,
        text: Rc<String>,
    ) -> Result<DxcBlobEncoding, HRESULT> {
        let mut blob: ComPtr<IDxcBlobEncoding> = ComPtr::new();
        const CP_UTF8: u32 = 65001; // UTF-8 translation

        return_hr!(
            unsafe {
                self.inner.create_blob_with_encoding_from_pinned(
                    text.as_ptr() as *const c_void,
                    text.len() as u32,
                    CP_UTF8,
                    blob.as_mut_ptr(),
                )
            },
            DxcBlobEncoding::new(blob)
        );
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

impl Dxc {
    pub fn new() -> Self {
        let dxc_lib = Library::new("dxcompiler.dll").expect("Failed to load dxcompiler.dll");

        Self { dxc_lib }
    }

    pub(crate) fn get_dxc_create_instance(&self) -> Symbol<DxcCreateInstanceProc> {
        unsafe { self.dxc_lib.get(b"DxcCreateInstance\0").unwrap() }
    }

    pub fn create_compiler(&self) -> Result<DxcCompiler, HRESULT> {
        let mut compiler: ComPtr<IDxcCompiler2> = ComPtr::new();
        return_hr!(
            self.get_dxc_create_instance()(
                &CLSID_DxcCompiler,
                &IID_IDxcCompiler2,
                compiler.as_mut_ptr(),
            ),
            DxcCompiler::new(compiler, self.create_library().unwrap())
        );
    }

    pub fn create_library(&self) -> Result<DxcLibrary, HRESULT> {
        let mut library: ComPtr<IDxcLibrary> = ComPtr::new();
        return_hr!(
            self.get_dxc_create_instance()(
                &CLSID_DxcLibrary,
                &IID_IDxcLibrary,
                library.as_mut_ptr(),
            ),
            DxcLibrary::new(library)
        );
    }
}

#[derive(Debug)]
pub struct DxcValidator {
    inner: ComPtr<IDxcValidator>,
}

pub type DxcValidatorVersion = (u32, u32);

impl DxcValidator {
    fn new(inner: ComPtr<IDxcValidator>) -> Self {
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

        return_hr! {
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

#[derive(Debug)]
pub struct Dxil {
    dxil_lib: Library,
}

impl Dxil {
    pub fn new() -> Self {
        let dxil_lib = Library::new("dxil.dll").expect("Failed to load dxil.dll");
        Self { dxil_lib }
    }

    fn get_dxc_create_instance(&self) -> Symbol<DxcCreateInstanceProc> {
        unsafe { self.dxil_lib.get(b"DxcCreateInstance\0").unwrap() }
    }

    pub fn create_validator(&self) -> Result<DxcValidator, HRESULT> {
        let mut validator: ComPtr<IDxcValidator> = ComPtr::new();
        return_hr!(
            self.get_dxc_create_instance()(
                &CLSID_DxcValidator,
                &IID_IDxcValidator,
                validator.as_mut_ptr(),
            ),
            DxcValidator::new(validator)
        );
    }
}
