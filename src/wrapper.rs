#![allow(
    clippy::too_many_arguments,
    clippy::new_without_default,
    clippy::type_complexity
)]

use crate::ffi::*;
use crate::os::{HRESULT, LPCWSTR, LPWSTR, WCHAR};
use crate::utils::{from_wide, to_wide, HassleError, Result};
use com::{class, interfaces::IUnknown, production::Class, production::ClassAllocation, Interface};
use libloading::{Library, Symbol};
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::pin::Pin;

pub struct DxcBlob {
    inner: IDxcBlob,
}

impl DxcBlob {
    fn new(inner: IDxcBlob) -> Self {
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

pub struct DxcBlobEncoding {
    inner: IDxcBlobEncoding,
}

impl DxcBlobEncoding {
    fn new(inner: IDxcBlobEncoding) -> Self {
        Self { inner }
    }
}

impl From<DxcBlobEncoding> for DxcBlob {
    fn from(encoded_blob: DxcBlobEncoding) -> Self {
        DxcBlob::new(encoded_blob.inner.query_interface::<IDxcBlob>().unwrap())
    }
}

pub struct DxcOperationResult {
    inner: IDxcOperationResult,
}

impl DxcOperationResult {
    fn new(inner: IDxcOperationResult) -> Self {
        Self { inner }
    }

    pub fn get_status(&self) -> Result<u32> {
        let mut status: u32 = 0;
        unsafe { self.inner.get_status(&mut status) }.result_with_success(status)
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

                unsafe { *include_source = Some(DxcBlob::from(blob).inner) };
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
    ) -> Result<DxcOperationResult, (DxcOperationResult, HRESULT)> {
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
        let result_hr = unsafe {
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
        };

        let result = result.unwrap();

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

        let result_hr = unsafe {
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
        };
        let result = result.unwrap();
        let debug_blob = debug_blob.unwrap();

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

        // Keep alive on the stack
        let include_handler = include_handler.map(|include_handler| unsafe {
            DxcIncludeHandlerWrapper::create_include_handler(&self.library, include_handler)
        });
        let include_handler = include_handler
            .as_ref()
            .map(|i| i.query_interface().unwrap());

        let mut result = None;
        let result_hr = unsafe {
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
        };

        let result = result.unwrap();

        let mut compile_error = 0u32;
        let status_hr = unsafe { result.get_status(&mut compile_error) };

        if !result_hr.is_err() && !status_hr.is_err() && compile_error == 0 {
            Ok(DxcOperationResult::new(result))
        } else {
            Err((DxcOperationResult::new(result), result_hr))
        }
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
        const CP_UTF8: u32 = 65001; // UTF-8 translation

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

    pub fn get_blob_as_string(&self, blob: &DxcBlob) -> Result<String> {
        let mut blob_utf8 = None;

        unsafe { self.inner.get_blob_as_utf8(&blob.inner, &mut blob_utf8) }.result()?;

        let blob_utf8 = blob_utf8.unwrap();

        Ok(String::from_utf8(DxcBlob::new(blob_utf8.query_interface().unwrap()).to_vec()).unwrap())
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
    Path::new("./libdxcompiler.dylib")
}

impl Dxc {
    /// `dxc_path` can point to a library directly or the directory containing the library,
    /// in which case the appended filename depends on the platform.
    pub fn new(lib_path: Option<PathBuf>) -> Result<Self> {
        let lib_path = if let Some(lib_path) = lib_path {
            if lib_path.is_file() {
                lib_path
            } else {
                lib_path.join(dxcompiler_lib_name())
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

    pub(crate) fn get_dxc_create_instance<T>(&self) -> Result<Symbol<DxcCreateInstanceProc<T>>> {
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

    pub fn validate(&self, blob: DxcBlob) -> Result<DxcBlob, (DxcOperationResult, HassleError)> {
        let mut result = None;
        let result_hr = unsafe {
            self.inner
                .validate(&blob.inner, DXC_VALIDATOR_FLAGS_IN_PLACE_EDIT, &mut result)
        };

        let result = result.unwrap();

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

unsafe fn ffi_char_ptr_to_cstring(s: *mut std::ffi::c_char) -> CString {
    if s.is_null() {
        // Empty string seems like a reasonable way to deal with a null pointer here
        CString::default()
    } else {
        CString::from(CStr::from_ptr(s))
    }
}

#[rustfmt::skip]
#[allow(non_camel_case_types, non_snake_case)]
#[derive(Clone, Debug)]
pub struct D3D12SignatureParameterDesc
{
    pub SemanticName: CString,                      // Name of the semantic
    pub SemanticIndex: u32,                         // Index of the semantic
    pub Register: u32,                              // Number of member variables
    pub SystemValueType: D3D_NAME,                  // A predefined system value, or D3D_NAME_UNDEFINED if not applicable
    pub ComponentType: D3D_REGISTER_COMPONENT_TYPE, // Scalar type (e.g. uint, float, etc.)
    pub Mask: u8,                                   // Mask to indicate which components of the register
                                                    // are used (combination of D3D10_COMPONENT_MASK values)
    pub ReadWriteMask: u8,                          // Mask to indicate whether a given component is
                                                    // never written (if this is an output signature) or
                                                    // always read (if this is an input signature).
                                                    // (combination of D3D_MASK_* values)
    pub Stream: u32,                                // Stream index
    pub MinPrecision: D3D_MIN_PRECISION,            // Minimum desired interpolation precision
}

impl D3D12SignatureParameterDesc {
    pub unsafe fn from_ffi(signature_parameter_desc_ffi: &D3D12_SIGNATURE_PARAMETER_DESC) -> Self {
        D3D12SignatureParameterDesc {
            SemanticName: ffi_char_ptr_to_cstring(signature_parameter_desc_ffi.SemanticName),
            SemanticIndex: signature_parameter_desc_ffi.SemanticIndex,
            Register: signature_parameter_desc_ffi.Register,
            SystemValueType: signature_parameter_desc_ffi.SystemValueType,
            ComponentType: signature_parameter_desc_ffi.ComponentType,
            Mask: signature_parameter_desc_ffi.Mask,
            ReadWriteMask: signature_parameter_desc_ffi.ReadWriteMask,
            Stream: signature_parameter_desc_ffi.Stream,
            MinPrecision: signature_parameter_desc_ffi.MinPrecision,
        }
    }
}

#[rustfmt::skip]
#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct D3D12ShaderBufferDesc
{
    pub Name: CString,                         // Name of the constant buffer
    pub Type: D3D_CBUFFER_TYPE,                // Indicates type of buffer content
    pub Variables: u32,                        // Number of member variables
    pub Size: u32,                             // Size of CB (in bytes)
    pub uFlags: u32,                           // Buffer description flags
}

impl D3D12ShaderBufferDesc {
    pub unsafe fn from_ffi(shader_buffer_desc_ffi: &D3D12_SHADER_BUFFER_DESC) -> Self {
        D3D12ShaderBufferDesc {
            Name: ffi_char_ptr_to_cstring(shader_buffer_desc_ffi.Name),
            Type: shader_buffer_desc_ffi.Type,
            Variables: shader_buffer_desc_ffi.Variables,
            Size: shader_buffer_desc_ffi.Size,
            uFlags: shader_buffer_desc_ffi.uFlags,
        }
    }
}

#[rustfmt::skip]
#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct D3D12ShaderVariableDesc
{
    pub Name: CString,                          // Name of the variable
    pub StartOffset: u32,                       // Offset in constant buffer's backing store
    pub Size: u32,                              // Size of variable (in bytes)
    pub uFlags: u32,                            // Variable flags
    pub DefaultValue: *mut std::ffi::c_void,    // Raw pointer to default value
    pub StartTexture: u32,                      // First texture index (or -1 if no textures used)
    pub TextureSize: u32,                       // Number of texture slots possibly used.
    pub StartSampler: u32,                      // First sampler index (or -1 if no textures used)
    pub SamplerSize: u32,                       // Number of sampler slots possibly used.
}

impl D3D12ShaderVariableDesc {
    pub unsafe fn from_ffi(shader_variable_desc_ffi: &D3D12_SHADER_VARIABLE_DESC) -> Self {
        D3D12ShaderVariableDesc {
            Name: ffi_char_ptr_to_cstring(shader_variable_desc_ffi.Name),
            StartOffset: shader_variable_desc_ffi.StartOffset,
            Size: shader_variable_desc_ffi.Size,
            uFlags: shader_variable_desc_ffi.uFlags,
            DefaultValue: shader_variable_desc_ffi.DefaultValue,
            StartTexture: shader_variable_desc_ffi.StartTexture,
            TextureSize: shader_variable_desc_ffi.TextureSize,
            StartSampler: shader_variable_desc_ffi.StartSampler,
            SamplerSize: shader_variable_desc_ffi.SamplerSize,
        }
    }
}

#[rustfmt::skip]
#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct D3D12ShaderTypeDesc
{
    pub Class: D3D_SHADER_VARIABLE_CLASS,       // Variable class (e.g. object, matrix, etc.)
    pub Type: D3D_SHADER_VARIABLE_TYPE,         // Variable type (e.g. float, sampler, etc.)
    pub Rows: u32,                              // Number of rows (for matrices, 1 for other numeric, 0 if not applicable)
    pub Columns: u32,                           // Number of columns (for vectors & matrices, 1 for other numeric, 0 if not applicable)
    pub Elements: u32,                          // Number of elements (0 if not an array)
    pub Members: u32,                           // Number of members (0 if not a structure)
    pub Offset: u32,                            // Offset from the start of structure (0 if not a structure member)
    pub Name: CString,                          // Name of type, can be NULL
}

impl D3D12ShaderTypeDesc {
    pub unsafe fn from_ffi(shader_type_desc_ffi: &D3D12_SHADER_TYPE_DESC) -> Self {
        D3D12ShaderTypeDesc {
            Class: shader_type_desc_ffi.Class,
            Type: shader_type_desc_ffi.Type,
            Rows: shader_type_desc_ffi.Rows,
            Columns: shader_type_desc_ffi.Columns,
            Elements: shader_type_desc_ffi.Elements,
            Members: shader_type_desc_ffi.Members,
            Offset: shader_type_desc_ffi.Offset,
            Name: ffi_char_ptr_to_cstring(shader_type_desc_ffi.Name),
        }
    }
}

#[rustfmt::skip]
#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct D3D12ShaderDesc
{
    pub Version: u32,                                           // Shader version
    pub Creator: CString,                                       // Creator string
    pub Flags: u32,                                             // Shader compilation/parse flags
    pub ConstantBuffers: u32,                                   // Number of constant buffers
    pub BoundResources: u32,                                    // Number of bound resources
    pub InputParameters: u32,                                   // Number of parameters in the input signature
    pub OutputParameters: u32,                                  // Number of parameters in the output signature
    pub InstructionCount: u32,                                  // Number of emitted instructions
    pub TempRegisterCount: u32,                                 // Number of temporary registers used
    pub TempArrayCount: u32,                                    // Number of temporary arrays used
    pub DefCount: u32,                                          // Number of constant defines
    pub DclCount: u32,                                          // Number of declarations (input + output)
    pub TextureNormalInstructions: u32,                         // Number of non-categorized texture instructions
    pub TextureLoadInstructions: u32,                           // Number of texture load instructions
    pub TextureCompInstructions: u32,                           // Number of texture comparison instructions
    pub TextureBiasInstructions: u32,                           // Number of texture bias instructions
    pub TextureGradientInstructions: u32,                       // Number of texture gradient instructions
    pub FloatInstructionCount: u32,                             // Number of floating point arithmetic instructions used
    pub IntInstructionCount: u32,                               // Number of signed integer arithmetic instructions used
    pub UintInstructionCount: u32,                              // Number of unsigned integer arithmetic instructions used
    pub StaticFlowControlCount: u32,                            // Number of static flow control instructions used
    pub DynamicFlowControlCount: u32,                           // Number of dynamic flow control instructions used
    pub MacroInstructionCount: u32,                             // Number of macro instructions used
    pub ArrayInstructionCount: u32,                             // Number of array instructions used
    pub CutInstructionCount: u32,                               // Number of cut instructions used
    pub EmitInstructionCount: u32,                              // Number of emit instructions used
    pub GSOutputTopology: D3D_PRIMITIVE_TOPOLOGY,               // Geometry shader output topology
    pub GSMaxOutputVertexCount: u32,                            // Geometry shader maximum output vertex count
    pub InputPrimitive: D3D_PRIMITIVE_TOPOLOGY,                 // GS/HS input primitive
    pub PatchConstantParameters: u32,                           // Number of parameters in the patch constant signature
    pub cGSInstanceCount: u32,                                  // Number of Geometry shader instances
    pub cControlPoints: u32,                                    // Number of control points in the HS->DS stage
    pub HSOutputPrimitive: D3D_TESSELLATOR_OUTPUT_PRIMITIVE,    // Primitive output by the tessellator
    pub HSPartitioning: D3D_TESSELLATOR_PARTITIONING,           // Partitioning mode of the tessellator
    pub TessellatorDomain: D3D_TESSELLATOR_DOMAIN,              // Domain of the tessellator (quad, tri, isoline)
    pub cBarrierInstructions: u32,                              // Number of barrier instructions in a compute shader
    pub cInterlockedInstructions: u32,                          // Number of interlocked instructions
    pub cTextureStoreInstructions: u32,                         // Number of texture writes
}

impl D3D12ShaderDesc {
    pub unsafe fn from_ffi(shader_desc_ffi: &D3D12_SHADER_DESC) -> Self {
        D3D12ShaderDesc {
            Version: shader_desc_ffi.Version,
            Creator: ffi_char_ptr_to_cstring(shader_desc_ffi.Creator),
            Flags: shader_desc_ffi.Flags,
            ConstantBuffers: shader_desc_ffi.ConstantBuffers,
            BoundResources: shader_desc_ffi.BoundResources,
            InputParameters: shader_desc_ffi.InputParameters,
            OutputParameters: shader_desc_ffi.OutputParameters,
            InstructionCount: shader_desc_ffi.InstructionCount,
            TempRegisterCount: shader_desc_ffi.TempRegisterCount,
            TempArrayCount: shader_desc_ffi.TempArrayCount,
            DefCount: shader_desc_ffi.DefCount,
            DclCount: shader_desc_ffi.DclCount,
            TextureNormalInstructions: shader_desc_ffi.TextureNormalInstructions,
            TextureLoadInstructions: shader_desc_ffi.TextureLoadInstructions,
            TextureCompInstructions: shader_desc_ffi.TextureCompInstructions,
            TextureBiasInstructions: shader_desc_ffi.TextureBiasInstructions,
            TextureGradientInstructions: shader_desc_ffi.TextureGradientInstructions,
            FloatInstructionCount: shader_desc_ffi.FloatInstructionCount,
            IntInstructionCount: shader_desc_ffi.IntInstructionCount,
            UintInstructionCount: shader_desc_ffi.UintInstructionCount,
            StaticFlowControlCount: shader_desc_ffi.StaticFlowControlCount,
            DynamicFlowControlCount: shader_desc_ffi.DynamicFlowControlCount,
            MacroInstructionCount: shader_desc_ffi.MacroInstructionCount,
            ArrayInstructionCount: shader_desc_ffi.ArrayInstructionCount,
            CutInstructionCount: shader_desc_ffi.CutInstructionCount,
            EmitInstructionCount: shader_desc_ffi.EmitInstructionCount,
            GSOutputTopology: shader_desc_ffi.GSOutputTopology,
            GSMaxOutputVertexCount: shader_desc_ffi.GSMaxOutputVertexCount,
            InputPrimitive: shader_desc_ffi.InputPrimitive,
            PatchConstantParameters: shader_desc_ffi.PatchConstantParameters,
            cGSInstanceCount: shader_desc_ffi.cGSInstanceCount,
            cControlPoints: shader_desc_ffi.cControlPoints,
            HSOutputPrimitive: shader_desc_ffi.HSOutputPrimitive,
            HSPartitioning: shader_desc_ffi.HSPartitioning,
            TessellatorDomain: shader_desc_ffi.TessellatorDomain,
            cBarrierInstructions: shader_desc_ffi.cBarrierInstructions,
            cInterlockedInstructions: shader_desc_ffi.cInterlockedInstructions,
            cTextureStoreInstructions: shader_desc_ffi.cTextureStoreInstructions,
        }
    }
}

#[rustfmt::skip]
#[allow(non_camel_case_types, non_snake_case)]
#[repr(C)]
#[derive(Clone, Debug)]
pub struct D3D12ShaderInputBindDesc
{
    pub Name: CString,                          // Name of the resource
    pub Type: D3D_SHADER_INPUT_TYPE,            // Type of resource (e.g. texture, cbuffer, etc.)
    pub BindPoint: u32,                         // Starting bind point
    pub BindCount: u32,                         // Number of contiguous bind points (for arrays)
    pub uFlags: u32,                            // Input binding flags
    pub ReturnType: D3D_RESOURCE_RETURN_TYPE,   // Return type (if texture)
    pub Dimension: D3D_SRV_DIMENSION,           // Dimension (if texture)
    pub NumSamples: u32,                        // Number of samples (0 if not MS texture)
    pub Space: u32,                             // Register space
    pub uID: u32,                               // Range ID in the bytecode
}

impl D3D12ShaderInputBindDesc {
    pub unsafe fn from_ffi(shader_input_bind_desc_ffi: &D3D12_SHADER_INPUT_BIND_DESC) -> Self {
        D3D12ShaderInputBindDesc {
            Name: ffi_char_ptr_to_cstring(shader_input_bind_desc_ffi.Name),
            Type: shader_input_bind_desc_ffi.Type,
            BindPoint: shader_input_bind_desc_ffi.BindPoint,
            BindCount: shader_input_bind_desc_ffi.BindCount,
            uFlags: shader_input_bind_desc_ffi.uFlags,
            ReturnType: shader_input_bind_desc_ffi.ReturnType,
            Dimension: shader_input_bind_desc_ffi.Dimension,
            NumSamples: shader_input_bind_desc_ffi.NumSamples,
            Space: shader_input_bind_desc_ffi.Space,
            uID: shader_input_bind_desc_ffi.uID,
        }
    }
}

pub struct D3D12ShaderReflectionType {
    inner: ID3D12ShaderReflectionType,
    owner: ID3D12ShaderReflection,
}

impl D3D12ShaderReflectionType {
    pub fn get_desc(&self) -> Result<D3D12ShaderTypeDesc> {
        unsafe {
            let mut shader_type_desc_ffi: D3D12_SHADER_TYPE_DESC =
                MaybeUninit::zeroed().assume_init();
            let result = self
                .inner
                .get_desc(&mut shader_type_desc_ffi as *mut D3D12_SHADER_TYPE_DESC);

            if result.is_err() {
                return Err(HassleError::Win32Error(result));
            }

            Ok(D3D12ShaderTypeDesc::from_ffi(&shader_type_desc_ffi))
        }
    }

    pub fn get_member_type_by_index(&self, index: u32) -> D3D12ShaderReflectionType {
        unsafe {
            let ty = self.inner.get_member_type_by_index(index);
            D3D12ShaderReflectionType {
                inner: ty,
                owner: self.owner.clone(),
            }
        }
    }

    pub fn get_member_type_by_name<'a>(
        &self,
        name: impl Into<&'a CStr>,
    ) -> D3D12ShaderReflectionType {
        unsafe {
            let ty = self.inner.get_member_type_by_name(name.into().as_ptr());
            D3D12ShaderReflectionType {
                inner: ty,
                owner: self.owner.clone(),
            }
        }
    }
    pub fn get_member_type_name(&self, index: u32) -> CString {
        unsafe {
            let type_name = self.inner.get_member_type_name(index);
            CString::from(CStr::from_ptr(type_name))
        }
    }

    pub fn is_equal(&self, other: &D3D12ShaderReflectionType) -> Result<bool> {
        unsafe {
            let result = self.inner.is_equal(other.inner);

            if result.is_err() {
                return Err(HassleError::Win32Error(result));
            }

            Ok(result.0 != 0)
        }
    }

    pub fn get_sub_type(&self) -> D3D12ShaderReflectionType {
        unsafe {
            let ty = self.inner.get_sub_type();
            D3D12ShaderReflectionType {
                inner: ty,
                owner: self.owner.clone(),
            }
        }
    }

    pub fn get_base_class(&self) -> D3D12ShaderReflectionType {
        unsafe {
            let ty = self.inner.get_base_class();
            D3D12ShaderReflectionType {
                inner: ty,
                owner: self.owner.clone(),
            }
        }
    }

    pub fn get_num_interfaces(&self) -> u32 {
        unsafe { self.inner.get_num_interfaces() }
    }

    pub fn get_interface_by_index(&self, index: u32) -> D3D12ShaderReflectionType {
        unsafe {
            let ty = self.inner.get_interface_by_index(index);
            D3D12ShaderReflectionType {
                inner: ty,
                owner: self.owner.clone(),
            }
        }
    }

    pub fn is_of_type(&self, other: &D3D12ShaderReflectionType) -> Result<bool> {
        unsafe {
            let result = self.inner.is_of_type(other.inner);

            if result.is_err() {
                return Err(HassleError::Win32Error(result));
            }

            Ok(result.0 != 0)
        }
    }

    pub fn implements_interface(&self, other: &D3D12ShaderReflectionType) -> Result<bool> {
        unsafe {
            let result = self.inner.implements_interface(other.inner);

            if result.is_err() {
                return Err(HassleError::Win32Error(result));
            }

            Ok(result.0 != 0)
        }
    }
}

pub struct D3D12ShaderReflectionVariable {
    inner: ID3D12ShaderReflectionVariable,
    owner: ID3D12ShaderReflection,
}

impl D3D12ShaderReflectionVariable {
    pub fn get_desc(&self) -> Result<D3D12ShaderVariableDesc> {
        unsafe {
            let mut shader_variable_desc_ffi: D3D12_SHADER_VARIABLE_DESC =
                MaybeUninit::zeroed().assume_init();
            let result = self
                .inner
                .get_desc(&mut shader_variable_desc_ffi as *mut D3D12_SHADER_VARIABLE_DESC);

            if result.is_err() {
                return Err(HassleError::Win32Error(result));
            }

            Ok(D3D12ShaderVariableDesc::from_ffi(&shader_variable_desc_ffi))
        }
    }

    pub fn get_type(&self) -> D3D12ShaderReflectionType {
        unsafe {
            D3D12ShaderReflectionType {
                inner: self.inner.get_type(),
                owner: self.owner.clone(),
            }
        }
    }

    pub fn get_buffer(&self) -> D3D12ShaderReflectionConstantBuffer {
        unsafe {
            D3D12ShaderReflectionConstantBuffer {
                inner: self.inner.get_buffer(),
                owner: self.owner.clone(),
            }
        }
    }

    pub fn get_interface_slot(&self, array_index: u32) -> u32 {
        unsafe { self.inner.get_interface_slot(array_index) }
    }
}

pub struct D3D12ShaderReflectionConstantBuffer {
    inner: ID3D12ShaderReflectionConstantBuffer,
    owner: ID3D12ShaderReflection,
}

impl D3D12ShaderReflectionConstantBuffer {
    pub fn get_desc(&self) -> Result<D3D12ShaderBufferDesc> {
        unsafe {
            let mut shader_buffer_desc_ffi: D3D12_SHADER_BUFFER_DESC =
                MaybeUninit::zeroed().assume_init();
            let result = self
                .inner
                .get_desc(&mut shader_buffer_desc_ffi as *mut D3D12_SHADER_BUFFER_DESC);

            if result.is_err() {
                return Err(HassleError::Win32Error(result));
            }

            Ok(D3D12ShaderBufferDesc::from_ffi(&shader_buffer_desc_ffi))
        }
    }

    pub fn get_variable_by_index(&self, index: u32) -> Result<D3D12ShaderReflectionVariable> {
        unsafe {
            let variable_ffi = self.inner.get_variable_by_index(index);

            Ok(D3D12ShaderReflectionVariable {
                inner: variable_ffi,
                owner: self.owner.clone(),
            })
        }
    }

    pub fn get_variable_by_name<'a>(
        &self,
        name: impl Into<&'a CStr>,
    ) -> Result<D3D12ShaderReflectionVariable> {
        unsafe {
            let variable_ffi = self.inner.get_variable_by_name(name.into().as_ptr());

            Ok(D3D12ShaderReflectionVariable {
                inner: variable_ffi,
                owner: self.owner.clone(),
            })
        }
    }
}

pub struct Reflection {
    inner: ID3D12ShaderReflection,
}
impl Reflection {
    fn new(inner: ID3D12ShaderReflection) -> Self {
        Self { inner }
    }

    pub fn get_desc(&self) -> Result<D3D12ShaderDesc> {
        unsafe {
            let mut shader_desc_ffi: D3D12_SHADER_DESC = MaybeUninit::zeroed().assume_init();
            let result = self
                .inner
                .get_desc(&mut shader_desc_ffi as *mut D3D12_SHADER_DESC);

            if result.is_err() {
                return Err(HassleError::Win32Error(result));
            }

            Ok(D3D12ShaderDesc::from_ffi(&shader_desc_ffi))
        }
    }

    pub fn get_constant_buffer_by_index(&self, index: u32) -> D3D12ShaderReflectionConstantBuffer {
        unsafe {
            let constant_buffer = self.inner.get_constant_buffer_by_index(index);
            D3D12ShaderReflectionConstantBuffer {
                inner: constant_buffer,
                owner: self.inner.clone(),
            }
        }
    }

    pub fn get_constant_buffer_by_name<'a>(
        &self,
        name: impl Into<&'a CStr>,
    ) -> D3D12ShaderReflectionConstantBuffer {
        unsafe {
            let constant_buffer = self.inner.get_constant_buffer_by_name(name.into().as_ptr());
            D3D12ShaderReflectionConstantBuffer {
                inner: constant_buffer,
                owner: self.inner.clone(),
            }
        }
    }

    pub fn get_resource_binding_desc(
        &self,
        resource_index: u32,
    ) -> Result<D3D12ShaderInputBindDesc> {
        unsafe {
            let mut shader_input_bind_desc_ffi: D3D12_SHADER_INPUT_BIND_DESC =
                MaybeUninit::zeroed().assume_init();
            let result = self.inner.get_resource_binding_desc(
                resource_index,
                &mut shader_input_bind_desc_ffi as *mut D3D12_SHADER_INPUT_BIND_DESC,
            );

            if result.is_err() {
                return Err(HassleError::Win32Error(result));
            }

            Ok(D3D12ShaderInputBindDesc::from_ffi(
                &shader_input_bind_desc_ffi,
            ))
        }
    }

    pub fn get_input_parameter_desc(
        &self,
        parameter_index: u32,
    ) -> Result<D3D12SignatureParameterDesc> {
        unsafe {
            let mut signature_parameter_desc_ffi: D3D12_SIGNATURE_PARAMETER_DESC =
                MaybeUninit::zeroed().assume_init();
            let result = self.inner.get_input_parameter_desc(
                parameter_index,
                &mut signature_parameter_desc_ffi as *mut D3D12_SIGNATURE_PARAMETER_DESC,
            );

            if result.is_err() {
                return Err(HassleError::Win32Error(result));
            }

            Ok(D3D12SignatureParameterDesc::from_ffi(
                &signature_parameter_desc_ffi,
            ))
        }
    }

    pub fn get_output_parameter_desc(
        &self,
        parameter_index: u32,
    ) -> Result<D3D12SignatureParameterDesc> {
        unsafe {
            let mut signature_parameter_desc_ffi: D3D12_SIGNATURE_PARAMETER_DESC =
                MaybeUninit::zeroed().assume_init();
            let result = self.inner.get_output_parameter_desc(
                parameter_index,
                &mut signature_parameter_desc_ffi as *mut D3D12_SIGNATURE_PARAMETER_DESC,
            );

            if result.is_err() {
                return Err(HassleError::Win32Error(result));
            }

            Ok(D3D12SignatureParameterDesc::from_ffi(
                &signature_parameter_desc_ffi,
            ))
        }
    }

    pub fn get_patch_constant_parameter_desc(
        &self,
        parameter_index: u32,
    ) -> Result<D3D12SignatureParameterDesc> {
        unsafe {
            let mut signature_parameter_desc_ffi: D3D12_SIGNATURE_PARAMETER_DESC =
                MaybeUninit::zeroed().assume_init();
            let result = self.inner.get_patch_constant_parameter_desc(
                parameter_index,
                &mut signature_parameter_desc_ffi as *mut D3D12_SIGNATURE_PARAMETER_DESC,
            );

            if result.is_err() {
                return Err(HassleError::Win32Error(result));
            }

            Ok(D3D12SignatureParameterDesc::from_ffi(
                &signature_parameter_desc_ffi,
            ))
        }
    }

    pub fn get_variable_by_name<'a>(
        &self,
        name: impl Into<&'a CStr>,
    ) -> D3D12ShaderReflectionVariable {
        unsafe {
            let variable = self.inner.get_variable_by_name(name.into().as_ptr());
            D3D12ShaderReflectionVariable {
                inner: variable,
                owner: self.inner.clone(),
            }
        }
    }

    pub fn get_resource_binding_desc_by_name<'a>(
        &self,
        name: impl Into<&'a CStr>,
    ) -> Result<D3D12SignatureParameterDesc> {
        unsafe {
            let mut signature_parameter_desc_ffi: D3D12_SIGNATURE_PARAMETER_DESC =
                MaybeUninit::zeroed().assume_init();
            let result = self.inner.get_resource_binding_desc_by_name(
                name.into().as_ptr(),
                &mut signature_parameter_desc_ffi as *mut D3D12_SIGNATURE_PARAMETER_DESC,
            );

            if result.is_err() {
                return Err(HassleError::Win32Error(result));
            }

            Ok(D3D12SignatureParameterDesc::from_ffi(
                &signature_parameter_desc_ffi,
            ))
        }
    }
    pub(crate) fn get_mov_instruction_count(&self) -> u32 {
        unsafe { self.inner.get_mov_instruction_count() }
    }
    pub(crate) fn get_movc_instruction_count(&self) -> u32 {
        unsafe { self.inner.get_movc_instruction_count() }
    }
    pub(crate) fn get_conversion_instruction_count(&self) -> u32 {
        unsafe { self.inner.get_conversion_instruction_count() }
    }
    pub(crate) fn get_bitwise_instruction_count(&self) -> u32 {
        unsafe { self.inner.get_bitwise_instruction_count() }
    }
    pub(crate) fn get_gs_input_primitive(&self) -> D3D_PRIMITIVE {
        unsafe { self.inner.get_gs_input_primitive() }
    }
    pub(crate) fn is_sample_frequency_shader(&self) -> bool {
        unsafe { self.inner.is_sample_frequency_shader() }
    }
    pub(crate) fn get_num_interface_slots(&self) -> u32 {
        unsafe { self.inner.get_num_interface_slots() }
    }

    pub fn thread_group_size(&self) -> [u32; 3] {
        let (mut size_x, mut size_y, mut size_z) = (0u32, 0u32, 0u32);
        unsafe {
            self.inner
                .get_thread_group_size(&mut size_x, &mut size_y, &mut size_z)
        };
        [size_x, size_y, size_z]
    }

    pub(crate) fn get_requires_flags(&self) -> u64 {
        unsafe { self.inner.get_requires_flags() }
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

    fn get_dxc_create_instance<T>(&self) -> Result<Symbol<DxcCreateInstanceProc<T>>> {
        Ok(unsafe { self.dxil_lib.get(b"DxcCreateInstance\0")? })
    }

    pub fn create_validator(&self) -> Result<DxcValidator> {
        let mut validator = None;
        self.get_dxc_create_instance()?(&CLSID_DxcValidator, &IDxcValidator::IID, &mut validator)
            .result()?;
        Ok(DxcValidator::new(validator.unwrap()))
    }
}
