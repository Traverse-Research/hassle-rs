#![allow(
    clippy::too_many_arguments,
    clippy::new_without_default,
    clippy::type_complexity
)]

use crate::dxil::ffi::*;
use crate::ffi::*;
use crate::utils::HassleError;
use com_rs::ComPtr;
use libloading::{Library, Symbol};

#[derive(Debug)]
pub struct Dxil {
    dxil_lib: Library,
}

impl Dxil {
    pub fn new() -> Result<Self, HassleError> {
        let dxil_lib = Library::new("dxil.dll").map_err(|e| HassleError::LoadLibraryError {
            filename: "dxil".to_string(),
            inner: e,
        })?;

        Ok(Self { dxil_lib })
    }

    fn get_dxc_create_instance(&self) -> Result<Symbol<DxcCreateInstanceProc>, HassleError> {
        Ok(unsafe { self.dxil_lib.get(b"DxcCreateInstance\0")? })
    }

    pub fn create_container_reflection(&self) -> Result<DxilContainerReflection, HassleError> {
        let mut reflection: ComPtr<IDxcContainerReflection> = ComPtr::new();
        return_hr_wrapped!(
            self.get_dxc_create_instance()?(
                &CLSID_DxcContainerReflection,
                &IID_IDxcContainerReflection,
                reflection.as_mut_ptr(),
            ),
            DxilContainerReflection::new(reflection)
        );
    }
}

#[derive(Debug)]
pub struct DxilContainerReflection {
    inner: ComPtr<IDxcContainerReflection>,
}

impl DxilContainerReflection {
    fn new(inner: ComPtr<IDxcContainerReflection>) -> Self {
        Self { inner }
    }

    pub fn load(&self, p_dxc_blob: &IDxcBlob) {
        unsafe {
            self.inner.load(p_dxc_blob);
        }
    }

    pub fn find_first_part_kind(&self) -> Result<u32, HassleError> {
        let mut shader_idx = 0u32;
        return_hr_wrapped!(
            unsafe {
                self.inner
                    .find_first_part_kind(Self::fourcc(['D', 'X', 'I', 'L']), &mut shader_idx)
            },
            shader_idx
        );
    }

    fn fourcc(chars: [char; 4]) -> u32 {
        (chars[0] as u32)
            | (chars[1] as u32) << 8
            | (chars[2] as u32) << 16
            | (chars[3] as u32) << 24
    }

    pub fn get_part_reflection(
        &self,
        idx: u32,
    ) -> Result<ComPtr<ID3D12ShaderReflection>, HassleError> {
        let mut p_reflection: ComPtr<ID3D12ShaderReflection> = ComPtr::new();
        return_hr_wrapped!(
            unsafe {
                self.inner.get_part_reflection(
                    idx,
                    &IID_ID3D12ShaderReflection,
                    p_reflection.as_mut_ptr(),
                )
            },
            p_reflection
        );
    }
}
