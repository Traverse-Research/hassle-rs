use crate::utils::HassleError;

/// Helper function to validate a DXIL binary independant from the compilation process,
/// this function expects `dxcompiler.dll` and `dxil.dll` to be available in the current
/// execution environment.
/// `dxil.dll` is currently not available on Linux.
pub fn validate_dxil(data: &[u8]) -> Result<Vec<u8>, HassleError> {
    let dxc = crate::Dxc::new()?;

    let validator = dxc.create_validator()?;
    let library = dxc.create_library()?;

    let blob_encoding = library
        .create_blob_with_encoding(&data)
        .map_err(HassleError::Win32Error)?;

    match validator.validate(blob_encoding.into()) {
        Ok(blob) => Ok(blob.to_vec()),
        Err(result) => {
            let error_blob = result
                .0
                .get_error_buffer()
                .map_err(HassleError::Win32Error)?;
            Err(HassleError::ValidationError(
                library.get_blob_as_string(&error_blob),
            ))
        }
    }
}
