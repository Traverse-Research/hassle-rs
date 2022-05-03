#![allow(
    clippy::too_many_arguments,
    clippy::new_without_default,
    clippy::type_complexity
)]

use crate::ffi::*;
use crate::os::{HRESULT, LPCWSTR, LPWSTR, WCHAR};
use crate::utils::{from_wide, to_wide, HassleError, Result};
use com_rs::ComPtr;
use libloading::{Library, Symbol};
use std::path::{Path, PathBuf};
use std::pin::Pin;

#[derive(Debug)]
pub struct DxcBlob {
    inner: ComPtr<IDxcBlob>,
}

impl DxcBlob {
    fn new(inner: ComPtr<IDxcBlob>) -> Self {
        Self { inner }
    }

    pub fn as_slice<T>(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(
                self.inner.get_buffer_pointer().cast(),
                self.inner.get_buffer_size() / std::mem::size_of::<T>(),
            )
        }
    }

    pub fn as_mut_slice<T>(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.inner.get_buffer_pointer().cast(),
                self.inner.get_buffer_size() / std::mem::size_of::<T>(),
            )
        }
    }

    pub fn to_vec<T>(&self) -> Vec<T>
    where
        T: Clone,
    {
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

#[derive(Debug)]
pub struct DxcBlobEncoding {
    inner: ComPtr<IDxcBlobEncoding>,
}

impl DxcBlobEncoding {
    fn new(inner: ComPtr<IDxcBlobEncoding>) -> Self {
        Self { inner }
    }
}

impl From<DxcBlobEncoding> for DxcBlob {
    fn from(encoded_blob: DxcBlobEncoding) -> Self {
        DxcBlob::new((&encoded_blob.inner).into())
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

    pub fn get_status(&self) -> Result<u32> {
        let mut status: u32 = 0;
        unsafe { self.inner.get_status(&mut status) }.result_with_success(status)
    }

    pub fn get_result(&self) -> Result<DxcBlob> {
        let mut blob: ComPtr<IDxcBlob> = ComPtr::new();
        unsafe { self.inner.get_result(blob.as_mut_ptr()) }.result()?;
        Ok(DxcBlob::new(blob))
    }

    pub fn get_error_buffer(&self) -> Result<DxcBlobEncoding> {
        let mut blob: ComPtr<IDxcBlobEncoding> = ComPtr::new();

        unsafe { self.inner.get_error_buffer(blob.as_mut_ptr()) }.result()?;
        Ok(DxcBlobEncoding::new(blob))
    }
}

pub trait DxcIncludeHandler {
    fn load_source(&mut self, filename: String) -> Option<String>;
}

#[repr(C)]
struct DxcIncludeHandlerWrapperVtbl {
    query_interface: extern "system" fn(
        *const com_rs::IUnknown,
        &com_rs::IID,
        *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    add_ref: extern "system" fn(*const com_rs::IUnknown) -> HRESULT,
    release: extern "system" fn(*const com_rs::IUnknown) -> HRESULT,
    #[cfg(not(windows))]
    complete_object_destructor: extern "system" fn(*const com_rs::IUnknown) -> HRESULT,
    #[cfg(not(windows))]
    deleting_destructor: extern "system" fn(*const com_rs::IUnknown) -> HRESULT,
    load_source: extern "system" fn(*mut com_rs::IUnknown, LPCWSTR, *mut *mut IDxcBlob) -> HRESULT,
}

#[repr(C)]
struct DxcIncludeHandlerWrapper<'a, 'i> {
    vtable: Box<DxcIncludeHandlerWrapperVtbl>,
    handler: &'i mut dyn DxcIncludeHandler,
    pinned: Vec<Pin<String>>,
    library: &'a DxcLibrary,
}

impl<'a, 'i> DxcIncludeHandlerWrapper<'a, 'i> {
    extern "system" fn query_interface(
        _me: *const com_rs::IUnknown,
        _rrid: &com_rs::IID,
        _ppv_obj: *mut *mut core::ffi::c_void,
    ) -> HRESULT {
        HRESULT(0) // dummy impl
    }

    extern "system" fn dummy(_me: *const com_rs::IUnknown) -> HRESULT {
        HRESULT(0) // dummy impl
    }

    extern "system" fn load_source(
        me: *mut com_rs::IUnknown,
        filename: LPCWSTR,
        include_source: *mut *mut IDxcBlob,
    ) -> HRESULT {
        let me = me.cast::<DxcIncludeHandlerWrapper>();

        let filename = crate::utils::from_wide(filename);

        let source = unsafe { (*me).handler.load_source(filename) };

        if let Some(source) = source {
            let source = Pin::new(source);
            let mut blob = unsafe {
                (*me)
                    .library
                    .create_blob_with_encoding_from_str(&source)
                    .unwrap()
            };

            unsafe {
                blob.inner.add_ref();
                *include_source = *blob.inner.as_mut_ptr();
                (*me).pinned.push(source);
            }

            0
        } else {
            -2_147_024_894 // ERROR_FILE_NOT_FOUND / 0x80070002
        }
        .into()
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
        // Temporary false-positive in Rust 1.60, Vec::push() below isn't available on slices.
        #[allow(clippy::ptr_arg)] dxc_defines: &mut Vec<DxcDefine>,
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

    fn prep_include_handler<'a, 'i>(
        library: &'a DxcLibrary,
        include_handler: Option<&'i mut dyn DxcIncludeHandler>,
    ) -> Option<Box<DxcIncludeHandlerWrapper<'a, 'i>>> {
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
        include_handler: Option<&mut dyn DxcIncludeHandler>,
        defines: &[(&str, Option<&str>)],
    ) -> Result<DxcOperationResult, (DxcOperationResult, HRESULT)> {
        let mut wide_args = vec![];
        let mut dxc_args = vec![];
        Self::prep_args(args, &mut wide_args, &mut dxc_args);

        let mut wide_defines = vec![];
        let mut dxc_defines = vec![];
        Self::prep_defines(defines, &mut wide_defines, &mut dxc_defines);

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
        let status_hr = unsafe { result.get_status(&mut compile_error) };

        if !result_hr.is_err() && !status_hr.is_err() && compile_error == 0 {
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
        include_handler: Option<&mut dyn DxcIncludeHandler>,
        defines: &[(&str, Option<&str>)],
    ) -> Result<(DxcOperationResult, String, DxcBlob), (DxcOperationResult, HRESULT)> {
        let mut wide_args = vec![];
        let mut dxc_args = vec![];
        Self::prep_args(args, &mut wide_args, &mut dxc_args);

        let mut wide_defines = vec![];
        let mut dxc_defines = vec![];
        Self::prep_defines(defines, &mut wide_defines, &mut dxc_defines);

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
        let status_hr = unsafe { result.get_status(&mut compile_error) };

        if !result_hr.is_err() && !status_hr.is_err() && compile_error == 0 {
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
        include_handler: Option<&mut dyn DxcIncludeHandler>,
        defines: &[(&str, Option<&str>)],
    ) -> Result<DxcOperationResult, (DxcOperationResult, HRESULT)> {
        let mut wide_args = vec![];
        let mut dxc_args = vec![];
        Self::prep_args(args, &mut wide_args, &mut dxc_args);

        let mut wide_defines = vec![];
        let mut dxc_defines = vec![];
        Self::prep_defines(defines, &mut wide_defines, &mut dxc_defines);

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
        let status_hr = unsafe { result.get_status(&mut compile_error) };

        if !result_hr.is_err() && !status_hr.is_err() && compile_error == 0 {
            Ok(DxcOperationResult::new(result))
        } else {
            Err((DxcOperationResult::new(result), result_hr))
        }
    }

    pub fn disassemble(&self, blob: &DxcBlob) -> Result<DxcBlobEncoding> {
        let mut result_blob: ComPtr<IDxcBlobEncoding> = ComPtr::new();
        unsafe {
            self.inner
                .disassemble(blob.inner.as_ptr(), result_blob.as_mut_ptr())
        }
        .result()?;
        Ok(DxcBlobEncoding::new(result_blob))
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

    pub fn create_blob_with_encoding(&self, data: &[u8]) -> Result<DxcBlobEncoding> {
        let mut blob: ComPtr<IDxcBlobEncoding> = ComPtr::new();

        unsafe {
            self.inner.create_blob_with_encoding_from_pinned(
                data.as_ptr().cast(),
                data.len() as u32,
                0, // Binary; no code page
                blob.as_mut_ptr(),
            )
        }
        .result()?;
        Ok(DxcBlobEncoding::new(blob))
    }

    pub fn create_blob_with_encoding_from_str(&self, text: &str) -> Result<DxcBlobEncoding> {
        let mut blob: ComPtr<IDxcBlobEncoding> = ComPtr::new();
        const CP_UTF8: u32 = 65001; // UTF-8 translation

        unsafe {
            self.inner.create_blob_with_encoding_from_pinned(
                text.as_ptr().cast(),
                text.len() as u32,
                CP_UTF8,
                blob.as_mut_ptr(),
            )
        }
        .result()?;
        Ok(DxcBlobEncoding::new(blob))
    }

    pub fn get_blob_as_string(&self, blob: &DxcBlob) -> Result<String> {
        let mut blob_utf8: ComPtr<IDxcBlobEncoding> = ComPtr::new();

        unsafe {
            self.inner
                .get_blob_as_utf8(blob.inner.as_ptr(), blob_utf8.as_mut_ptr())
        }
        .result()?;

        Ok(String::from_utf8(DxcBlob::new((&blob_utf8).into()).to_vec()).unwrap())
    }
}

#[derive(Debug)]
pub struct Dxc {
    dxc_lib: Library,
}

#[cfg(target_os = "windows")]
fn dxcompiler_lib_name() -> &'static Path {
    Path::new("dxcompiler.dll")
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn dxcompiler_lib_name() -> &'static Path {
    Path::new("./libdxcompiler.so")
}

#[cfg(target_os = "macos")]
fn dxcompiler_lib_name() -> &'static Path {
    Path::new("./libdxcompiler.dynlib")
}

impl Dxc {
    /// `dxc_path` can point to a library directly or the directory containing the library,
    /// in which case the appended filename depends on the platform.
    pub fn new(lib_path: Option<PathBuf>) -> Result<Self> {
        let lib_path = if let Some(lib_path) = lib_path {
            if lib_path.is_file() {
                lib_path
            } else {
                lib_path.join(&dxcompiler_lib_name())
            }
        } else {
            dxcompiler_lib_name().to_owned()
        };
        let dxc_lib =
            unsafe { Library::new(&lib_path) }.map_err(|e| HassleError::LoadLibraryError {
                filename: lib_path,
                inner: e,
            })?;

        Ok(Self { dxc_lib })
    }

    pub(crate) fn get_dxc_create_instance(&self) -> Result<Symbol<DxcCreateInstanceProc>> {
        Ok(unsafe { self.dxc_lib.get(b"DxcCreateInstance\0")? })
    }

    pub fn create_compiler(&self) -> Result<DxcCompiler> {
        let mut compiler: ComPtr<IDxcCompiler2> = ComPtr::new();

        self.get_dxc_create_instance()?(
            &CLSID_DxcCompiler,
            &IID_IDxcCompiler2,
            compiler.as_mut_ptr(),
        )
        .result()?;
        Ok(DxcCompiler::new(compiler, self.create_library()?))
    }

    pub fn create_library(&self) -> Result<DxcLibrary> {
        let mut library: ComPtr<IDxcLibrary> = ComPtr::new();

        self.get_dxc_create_instance()?(&CLSID_DxcLibrary, &IID_IDxcLibrary, library.as_mut_ptr())
            .result()?;
        Ok(DxcLibrary::new(library))
    }

    pub fn create_reflector(&self) -> Result<DxcReflector> {
        let mut reflector: ComPtr<IDxcContainerReflection> = ComPtr::new();

        self.get_dxc_create_instance()?(
            &CLSID_DxcContainerReflection,
            &IID_IDxcContainerReflection,
            reflector.as_mut_ptr(),
        )
        .result()?;
        Ok(DxcReflector::new(reflector))
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

    pub fn version(&self) -> Result<DxcValidatorVersion> {
        let mut version: ComPtr<IDxcVersionInfo> = ComPtr::new();

        HRESULT::from(unsafe {
            self.inner
                .query_interface(&IID_IDxcVersionInfo, version.as_mut_ptr())
        })
        .result()?;

        let mut major = 0;
        let mut minor = 0;

        unsafe { version.get_version(&mut major, &mut minor) }.result_with_success((major, minor))
    }

    pub fn validate(&self, blob: DxcBlob) -> Result<DxcBlob, (DxcOperationResult, HassleError)> {
        let mut result: ComPtr<IDxcOperationResult> = ComPtr::new();
        let result_hr = unsafe {
            self.inner.validate(
                blob.inner.as_ptr(),
                DXC_VALIDATOR_FLAGS_IN_PLACE_EDIT,
                result.as_mut_ptr(),
            )
        };

        let mut validate_status = 0u32;
        let status_hr = unsafe { result.get_status(&mut validate_status) };

        if !result_hr.is_err() && !status_hr.is_err() && validate_status == 0 {
            Ok(blob)
        } else {
            Err((
                DxcOperationResult::new(result),
                HassleError::Win32Error(result_hr),
            ))
        }
    }
}

pub struct Reflection {
    inner: ComPtr<ID3D12ShaderReflection>,
}
impl Reflection {
    fn new(inner: ComPtr<ID3D12ShaderReflection>) -> Self {
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
    inner: ComPtr<IDxcContainerReflection>,
}
impl DxcReflector {
    fn new(inner: ComPtr<IDxcContainerReflection>) -> Self {
        Self { inner }
    }

    pub fn reflect(&self, blob: DxcBlob) -> Result<Reflection> {
        let result_hr = unsafe { self.inner.load(blob.inner.as_ptr()) };
        if result_hr.is_err() {
            return Err(HassleError::Win32Error(result_hr));
        }

        let mut shader_idx = 0;
        let result_hr = unsafe { self.inner.find_first_part_kind(DFCC_DXIL, &mut shader_idx) };
        if result_hr.is_err() {
            return Err(HassleError::Win32Error(result_hr));
        }

        let mut reflection: ComPtr<ID3D12ShaderReflection> = ComPtr::new();
        let result_hr = unsafe {
            self.inner.get_part_reflection(
                shader_idx,
                &IID_ID3D12ShaderReflection,
                reflection.as_mut_ptr(),
            )
        };
        if result_hr.is_err() {
            return Err(HassleError::Win32Error(result_hr));
        }

        Ok(Reflection::new(reflection))
    }
}

#[derive(Debug)]
pub struct Dxil {
    dxil_lib: Library,
}

impl Dxil {
    #[cfg(not(windows))]
    pub fn new(_: Option<PathBuf>) -> Result<Self> {
        Err(HassleError::WindowsOnly(
            "DXIL Signing is only supported on Windows".to_string(),
        ))
    }

    /// `dxil_path` can point to a library directly or the directory containing the library,
    /// in which case `dxil.dll` is appended.
    #[cfg(windows)]
    pub fn new(lib_path: Option<PathBuf>) -> Result<Self> {
        let lib_path = if let Some(lib_path) = lib_path {
            if lib_path.is_file() {
                lib_path
            } else {
                lib_path.join("dxil.dll")
            }
        } else {
            PathBuf::from("dxil.dll")
        };

        let dxil_lib =
            unsafe { Library::new(&lib_path) }.map_err(|e| HassleError::LoadLibraryError {
                filename: lib_path.to_owned(),
                inner: e,
            })?;

        Ok(Self { dxil_lib })
    }

    fn get_dxc_create_instance(&self) -> Result<Symbol<DxcCreateInstanceProc>> {
        Ok(unsafe { self.dxil_lib.get(b"DxcCreateInstance\0")? })
    }

    pub fn create_validator(&self) -> Result<DxcValidator> {
        let mut validator: ComPtr<IDxcValidator> = ComPtr::new();

        self.get_dxc_create_instance()?(
            &CLSID_DxcValidator,
            &IID_IDxcValidator,
            validator.as_mut_ptr(),
        )
        .result()?;
        Ok(DxcValidator::new(validator))
    }
}
