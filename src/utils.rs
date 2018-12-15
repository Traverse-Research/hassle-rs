use crate::ffi::*;
use crate::wrapper::*;
use com_rs::ComPtr;
use std::ffi::c_void;

pub(crate) fn to_wide(msg: &str) -> Vec<u16> {
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
    let dxc = Dxc::new();

    let compiler = dxc.create_compiler().unwrap();
    let library = dxc.create_library().unwrap();

    let blob = library
        .create_blob_with_encoding_from_str(shader_text)
        .unwrap();

    let result = compiler.compile(
        &blob,
        source_name,
        entry_point,
        target_profile,
        args,
        defines,
    );

    match result {
        Err(result) => {
            let error_blob = result.0.get_error_buffer().unwrap();
            Err(library.get_blob_as_string(error_blob))
        }
        Ok(result) => {
            let result_blob = result.get_result();
            Ok(result_blob.unwrap().to_vec())
        }
    }
}
