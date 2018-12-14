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

pub fn compiler(
    shader_text: &str,
    entry_point: &str,
    target_profile: &str,
) -> Result<Vec<u32>, String> {
    use libloading::*;
    let dxc_lib = Library::new("dxcompiler.dll").expect("Failed to load dxcompiler.dll");

    unsafe {
        /*
            typedef HRESULT (__stdcall *DxcCreateInstanceProc)(
            _In_ REFCLSID   rclsid,
            _In_ REFIID     riid,
            _Out_ LPVOID*   ppv
        );*/

        let dxc_create_instance: Symbol<
            extern "system" fn(rclsid: &IID, riid: &IID, ppv: *mut *mut c_void) -> u32,
        > = dxc_lib.get(b"DxcCreateInstance\0").unwrap();

        let mut library: ComPtr<IDxcLibrary> = ComPtr::new();
        let _hr_lib =
            dxc_create_instance(&CLSID_DxcLibrary, &IID_IDxcLibrary, library.as_mut_ptr());

        let mut compiler: ComPtr<IDxcCompiler> = ComPtr::new();
        let _hr_comp =
            dxc_create_instance(&CLSDI_DxcCompiler, &IID_IDxcCompiler, compiler.as_mut_ptr());

        const CP_UTF8: u32 = 65001; // UTF-8 translation
        let mut blob: ComPtr<IDxcBlobEncoding> = ComPtr::new();

        let _blob_hr = library.create_blob_with_encoding_from_pinned(
            shader_text.as_ptr() as *const c_void,
            shader_text.len() as u32,
            CP_UTF8,
            blob.as_mut_ptr(),
        );

        let mut result: ComPtr<IDxcOperationResult> = ComPtr::new();

        let args = vec![to_wide("-spirv").as_ptr()];

        let _res_hr = compiler.compile(
            blob.as_ptr(),
            to_wide("shader").as_ptr(),
            to_wide(entry_point).as_ptr(),
            to_wide(target_profile).as_ptr(),
            args.as_ptr(),
            args.len() as u32,
            std::ptr::null(),
            0,
            std::ptr::null(),
            result.as_mut_ptr(),
        );

        let mut status = 0;
        result.get_status(&mut status);

        if status != 0 {
            let mut error_blob: ComPtr<IDxcBlobEncoding> = ComPtr::new();
            let mut error_blob_utf8: ComPtr<IDxcBlobEncoding> = ComPtr::new();
            result.get_error_buffer(error_blob.as_mut_ptr());

            library.get_blob_as_utf8(error_blob.as_ptr(), error_blob_utf8.as_mut_ptr());

            let error_slice = std::slice::from_raw_parts(
                error_blob_utf8.get_buffer_pointer() as *const u8,
                error_blob_utf8.get_buffer_size(),
            );

            return Err(String::from_utf8(error_slice.to_vec()).unwrap());
        } else {
            let mut result_blob: ComPtr<IDxcBlob> = ComPtr::new();
            result.get_result(result_blob.as_mut_ptr());

            let result_slice = std::slice::from_raw_parts(
                result_blob.get_buffer_pointer() as *const u32,
                result_blob.get_buffer_size() / 4,
            );

            return Ok(result_slice.to_vec());
        }
    }
}
