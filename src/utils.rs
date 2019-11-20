#[cfg(windows)]
use crate::os::{BSTR, LPSTR, LPWSTR};
use crate::os::{LPWSTR, WCHAR};
use crate::wrapper::*;
use std::rc::Rc;

#[cfg(windows)]
use winapi::um::oleauto::{SysFreeString, SysStringLen};

pub(crate) fn to_wide(msg: &str) -> Vec<WCHAR> {
    widestring::WideCString::from_str(msg).unwrap().into_vec()
}

pub(crate) fn from_wide(wide: LPWSTR) -> String {
    unsafe {
        widestring::WideCStr::from_ptr_str(wide)
            .to_string()
            .expect("utf16 decode failed")
    }
}

#[cfg(windows)]
pub(crate) fn from_bstr(string: BSTR) -> String {
    unsafe {
        let len = SysStringLen(string);
        let slice: &[WCHAR] = ::std::slice::from_raw_parts(string, len as usize);
        let result = String::from_utf16(slice).unwrap();
        SysFreeString(string);

        return result;
    }
}

#[cfg(windows)]
pub(crate) fn from_lpstr(string: LPSTR) -> String {
    unsafe {
        let len = (0..).take_while(|&i| *string.offset(i) != 0).count();

        let slice: &[u8] = std::slice::from_raw_parts(string as *const u8, len);
        std::str::from_utf8(slice).map(|s| s.to_owned()).unwrap()
    }
}

struct DefaultIncludeHandler {}

impl DxcIncludeHandler for DefaultIncludeHandler {
    fn load_source(&self, filename: String) -> Option<String> {
        use std::io::Read;
        match std::fs::File::open(filename) {
            Ok(mut f) => {
                let mut content = String::new();
                f.read_to_string(&mut content).unwrap();
                Some(content)
            }
            Err(_) => None,
        }
    }
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
    args: &[&str],
    defines: &[(&str, Option<&str>)],
) -> Result<Vec<u8>, String> {
    let dxc = Dxc::new();

    let compiler = dxc.create_compiler().unwrap();
    let library = dxc.create_library().unwrap();

    //let source = Rc::new(String::from(shader_text));

    let blob = library
        .create_blob_with_encoding_from_str(shader_text)
        .unwrap();

    let result = compiler.compile(
        &blob,
        source_name,
        entry_point,
        target_profile,
        args,
        Some(Box::new(DefaultIncludeHandler {})),
        defines,
    );

    match result {
        Err(result) => {
            let error_blob = result.0.get_error_buffer().unwrap();
            Err(library.get_blob_as_string(&error_blob))
        }
        Ok(result) => {
            let result_blob = result.get_result().unwrap();

            Ok(result_blob.to_vec())
        }
    }
}

/// Helper function to validate a DXIL binary independant from the compilation process,
/// this function expected `dxcompiler.dll` and `dxil.dll` to be available in the current
/// execution environment.
pub fn validate_dxil(data: &[u8]) -> Result<Vec<u8>, String> {
    let dxc = Dxc::new();
    let dxil = Dxil::new();

    let validator = dxil.create_validator().unwrap();
    let library = dxc.create_library().unwrap();

    let blob_encoding = library.create_blob_with_encoding(&data).unwrap();

    match validator.validate(blob_encoding.into()) {
        Ok(blob) => Ok(blob.to_vec()),
        Err(result) => {
            let error_blob = result.0.get_error_buffer().unwrap();
            Err(library.get_blob_as_string(&error_blob))
        }
    }
}
