use crate::ffi::*;
use com_rs::{ComPtr, IID};
use std::ffi::c_void;

fn to_wide(msg: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;

    let wide: Vec<u16> = OsStr::new(msg).encode_wide().chain(once(0)).collect();
    wide
}

/// Helper function to directly compile a HLSL shader to an intermediate language,
/// this function expects `dxcompiler.dll` to be available in the current
/// executable environment.
///
/// Specify -spirv as one of the `args` to compile to SPIR-V
pub fn compile_hlsl(
    source_name: &str,
    shader_text: &str,
    entry_point: &str,
    target_profile: &str,
    args: &Vec<&str>,
    defines: &Vec<(&str, Option<&str>)>,
) -> Result<Vec<u32>, String> {
    use libloading::*;

    let dxc_lib = Library::new("dxcompiler.dll").expect("Failed to load dxcompiler.dll");
    const CP_UTF8: u32 = 65001; // UTF-8 translation

    // typedef HRESULT (__stdcall *DxcCreateInstanceProc)(_In_ REFCLSID rclsid, _In_ REFIID riid, _Out_ LPVOID* ppv);
    let dxc_create_instance: Symbol<
        extern "system" fn(rclsid: &IID, riid: &IID, ppv: *mut *mut c_void) -> u32,
    > = unsafe { dxc_lib.get(b"DxcCreateInstance\0").unwrap() };

    let mut library: ComPtr<IDxcLibrary> = ComPtr::new();
    let _hr_lib = dxc_create_instance(&CLSID_DxcLibrary, &IID_IDxcLibrary, library.as_mut_ptr());

    let mut compiler: ComPtr<IDxcCompiler> = ComPtr::new();
    let _hr_comp =
        dxc_create_instance(&CLSDI_DxcCompiler, &IID_IDxcCompiler, compiler.as_mut_ptr());

    let mut blob: ComPtr<IDxcBlobEncoding> = ComPtr::new();

    let _blob_hr = unsafe {
        library.create_blob_with_encoding_from_pinned(
            shader_text.as_ptr() as *const c_void,
            shader_text.len() as u32,
            CP_UTF8,
            blob.as_mut_ptr(),
        )
    };

    let mut result: ComPtr<IDxcOperationResult> = ComPtr::new();

    let mut wide_args = vec![];
    for a in args {
        wide_args.push(to_wide(a));
    }

    let mut dxc_args = vec![];
    for ref a in &wide_args {
        dxc_args.push(a.as_ptr());
    }

    // move names to `wide` vector so they stay in scope
    let mut wide_defines = vec![];
    for (name, value) in defines {
        if value.is_none() {
            wide_defines.push((to_wide(name), to_wide("1")));
        } else {
            wide_defines.push((to_wide(name), to_wide(value.unwrap())));
        }
    }

    let mut dxc_defines = vec![];
    for (ref name, ref value) in &wide_defines {
        dxc_defines.push(DxcDefine {
            name: name.as_ptr(),
            value: value.as_ptr(),
        });
    }

    let _res_hr = unsafe {
        compiler.compile(
            blob.as_ptr(),
            to_wide(source_name).as_ptr(),
            to_wide(entry_point).as_ptr(),
            to_wide(target_profile).as_ptr(),
            dxc_args.as_ptr(),
            dxc_args.len() as u32,
            dxc_defines.as_ptr(),
            dxc_defines.len() as u32,
            std::ptr::null(),
            result.as_mut_ptr(),
        )
    };

    let mut status = 0;
    unsafe { result.get_status(&mut status) };

    if status != 0 {
        let mut error_blob: ComPtr<IDxcBlobEncoding> = ComPtr::new();
        let mut error_blob_utf8: ComPtr<IDxcBlobEncoding> = ComPtr::new();
        unsafe { result.get_error_buffer(error_blob.as_mut_ptr()) };

        unsafe { library.get_blob_as_utf8(error_blob.as_ptr(), error_blob_utf8.as_mut_ptr()) };

        let error_slice = unsafe {
            std::slice::from_raw_parts(
                error_blob_utf8.get_buffer_pointer() as *const u8,
                error_blob_utf8.get_buffer_size(),
            )
        };

        return Err(String::from_utf8(error_slice.to_vec()).unwrap());
    } else {
        let mut result_blob: ComPtr<IDxcBlob> = ComPtr::new();
        unsafe { result.get_result(result_blob.as_mut_ptr()) };

        let result_slice = unsafe {
            std::slice::from_raw_parts(
                result_blob.get_buffer_pointer() as *const u32,
                result_blob.get_buffer_size() / 4,
            )
        };

        return Ok(result_slice.to_vec());
    }
}
