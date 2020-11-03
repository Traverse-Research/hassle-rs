#![allow(
    clippy::too_many_arguments,
    clippy::new_without_default,
    clippy::type_complexity
)]

use crate::dxil::ffi::*;
use crate::ffi::*;
use crate::os::HRESULT;
use crate::utils::HassleError;
use com_rs::ComPtr;
use libloading::{Library, Symbol};
use std::ffi::CStr;
use winapi::um::{d3d12shader, d3dcommon};

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

    pub fn create_container_reflection(&self) -> Result<DxcContainerReflection, HassleError> {
        let mut reflection: ComPtr<IDxcContainerReflection> = ComPtr::new();
        return_hr_wrapped!(
            self.get_dxc_create_instance()?(
                &CLSID_DxcContainerReflection,
                &IID_IDxcContainerReflection,
                reflection.as_mut_ptr(),
            ),
            DxcContainerReflection::new(reflection)
        );
    }
}

#[derive(Debug)]
pub struct DxcContainerReflection {
    inner: ComPtr<IDxcContainerReflection>,
}

impl DxcContainerReflection {
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

    pub fn get_part_reflection(&self, idx: u32) -> Result<D3D12ShaderReflection, HassleError> {
        let mut p_reflection: ComPtr<ID3D12ShaderReflection> = ComPtr::new();
        return_hr_wrapped!(
            unsafe {
                self.inner.get_part_reflection(
                    idx,
                    &IID_ID3D12ShaderReflection,
                    p_reflection.as_mut_ptr(),
                )
            },
            D3D12ShaderReflection::new(p_reflection)
        );
    }
}

#[derive(Debug)]
pub struct D3D12LibraryReflection {
    inner: ComPtr<ID3D12LibraryReflection>,
}

impl D3D12LibraryReflection {
    fn new(inner: ComPtr<ID3D12LibraryReflection>) -> Self {
        Self { inner }
    }

    pub fn get_desc(&self) -> Result<d3d12shader::D3D12_LIBRARY_DESC, HassleError> {
        let mut desc = d3d12shader::D3D12_LIBRARY_DESC::default();
        return_hr_wrapped!(unsafe { self.inner.get_desc(&mut desc as *mut _) }, desc);
    }

    pub fn get_function_by_index(&self, function_index: i32) -> D3D12FunctionReflection {
        let mut ptr: ComPtr<ID3D12FunctionReflection> =
            unsafe { std::mem::transmute(self.inner.get_function_by_index(function_index)) };
        D3D12FunctionReflection::new(ptr)
    }
}

#[derive(Debug)]
pub struct D3D12FunctionParameterReflection {
    inner: ComPtr<ID3D12FunctionParameterReflection>,
}

impl D3D12FunctionParameterReflection {
    fn new(inner: ComPtr<ID3D12FunctionParameterReflection>) -> Self {
        Self { inner }
    }

    pub fn get_desc(&self) -> Result<d3d12shader::D3D12_PARAMETER_DESC, HassleError> {
        let mut desc = d3d12shader::D3D12_PARAMETER_DESC::default();
        return_hr_wrapped!(unsafe { self.inner.get_desc(&mut desc as *mut _) }, desc);
    }
}

#[derive(Debug)]
pub struct D3D12FunctionReflection {
    inner: ComPtr<ID3D12FunctionReflection>,
}

impl D3D12FunctionReflection {
    fn new(inner: ComPtr<ID3D12FunctionReflection>) -> Self {
        Self { inner }
    }

    pub fn get_constant_buffer_by_index(
        &self,
        buffer_index: u32,
    ) -> D3D12ShaderReflectionConstantBuffer {
        let mut ptr: ComPtr<ID3D12ShaderReflectionConstantBuffer> =
            unsafe { std::mem::transmute(self.inner.get_constant_buffer_by_index(buffer_index)) };
        D3D12ShaderReflectionConstantBuffer::new(ptr)
    }

    pub fn get_constant_buffer_by_name(&self, name: &CStr) -> D3D12ShaderReflectionConstantBuffer {
        let mut ptr: ComPtr<ID3D12ShaderReflectionConstantBuffer> = unsafe {
            std::mem::transmute(
                self.inner
                    .get_constant_buffer_by_name(name.as_ptr() as *const i8),
            )
        };
        D3D12ShaderReflectionConstantBuffer::new(ptr)
    }

    pub fn get_desc(&self) -> Result<d3d12shader::D3D12_FUNCTION_DESC, HassleError> {
        let mut desc = d3d12shader::D3D12_FUNCTION_DESC::default();
        return_hr_wrapped!(unsafe { self.inner.get_desc(&mut desc as *mut _) }, desc);
    }

    pub fn get_function_parameter(&self, parameter_index: i32) -> D3D12FunctionParameterReflection {
        let mut ptr: ComPtr<ID3D12FunctionParameterReflection> =
            unsafe { std::mem::transmute(self.inner.get_function_parameter(parameter_index)) };

        D3D12FunctionParameterReflection::new(ptr)
    }

    pub fn get_resource_binding_desc(
        &self,
        resource_index: u32,
    ) -> Result<d3d12shader::D3D12_SHADER_INPUT_BIND_DESC, HassleError> {
        let mut desc = d3d12shader::D3D12_SHADER_INPUT_BIND_DESC::default();
        return_hr_wrapped!(
            unsafe {
                self.inner
                    .get_resource_binding_desc(resource_index, &mut desc as *mut _)
            },
            desc
        );
    }

    pub fn get_resource_binding_desc_by_name(
        &self,
        name: &CStr,
    ) -> Result<d3d12shader::D3D12_SHADER_INPUT_BIND_DESC, HassleError> {
        let mut desc = d3d12shader::D3D12_SHADER_INPUT_BIND_DESC::default();
        return_hr_wrapped!(
            unsafe {
                self.inner.get_resource_binding_desc_by_name(
                    name.as_ptr() as *const i8,
                    &mut desc as *mut _,
                )
            },
            desc
        );
    }

    pub fn get_variable_by_name(&self, name: &CStr) -> D3D12ShaderReflectionVariable {
        let mut ptr: ComPtr<ID3D12ShaderReflectionVariable> = unsafe {
            std::mem::transmute(self.inner.get_variable_by_name(name.as_ptr() as *const i8))
        };
        D3D12ShaderReflectionVariable::new(ptr)
    }
}

#[derive(Debug)]
pub struct D3D12ShaderReflectionType {
    inner: ComPtr<ID3D12ShaderReflectionType>,
}

impl D3D12ShaderReflectionType {
    fn new(inner: ComPtr<ID3D12ShaderReflectionType>) -> Self {
        Self { inner }
    }

    pub fn get_base_class(&self) -> D3D12ShaderReflectionType {
        let mut ptr: ComPtr<ID3D12ShaderReflectionType> =
            unsafe { std::mem::transmute(self.inner.get_base_class()) };
        D3D12ShaderReflectionType::new(ptr)
    }

    pub fn get_desc(&self) -> Result<d3d12shader::D3D12_SHADER_TYPE_DESC, HassleError> {
        let mut desc = d3d12shader::D3D12_SHADER_TYPE_DESC::default();
        return_hr_wrapped!(unsafe { self.inner.get_desc(&mut desc as *mut _) }, desc);
    }

    pub fn get_inferface_by_index(&self, index: u32) -> D3D12ShaderReflectionType {
        let mut ptr: ComPtr<ID3D12ShaderReflectionType> =
            unsafe { std::mem::transmute(self.inner.get_interface_by_index(index)) };
        D3D12ShaderReflectionType::new(ptr)
    }

    pub fn get_member_type_by_index(&self, index: u32) -> D3D12ShaderReflectionType {
        unsafe { D3D12ShaderReflectionType::new(self.inner.get_member_type_by_index(index)) }
    }

    pub fn get_member_type_by_name(&self, name: &CStr) -> D3D12ShaderReflectionType {
        unsafe {
            D3D12ShaderReflectionType::new(
                self.inner
                    .get_member_type_by_name(name.as_ptr() as *const i8),
            )
        }
    }

    pub fn get_member_type_name(&self, index: u32) -> Result<&str, HassleError> {
        unsafe {
            CStr::from_ptr(self.inner.get_member_type_name(index))
                .to_str()
                .map_err(HassleError::Utf8Error)
        }
    }

    pub fn get_num_interfaces(&self) -> u32 {
        unsafe { self.inner.get_num_interfaces() }
    }

    pub fn get_sub_type(&self) -> D3D12ShaderReflectionType {
        unsafe { D3D12ShaderReflectionType::new(self.inner.get_sub_type()) }
    }

    pub fn implements_interface(&self, base: &D3D12ShaderReflectionType) -> HRESULT {
        unsafe { self.inner.implements_interface(base.inner.clone()) }
    }

    pub fn is_equal(&self, desc: &D3D12ShaderReflectionType) -> HRESULT {
        unsafe { self.inner.is_equal(desc.inner.clone()) }
    }

    pub fn is_of_type(&self, desc: &D3D12ShaderReflectionType) -> HRESULT {
        unsafe { self.inner.is_of_type(desc.inner.clone()) }
    }
}

#[derive(Debug)]
pub struct D3D12ShaderReflectionVariable {
    inner: ComPtr<ID3D12ShaderReflectionVariable>,
}

impl D3D12ShaderReflectionVariable {
    fn new(inner: ComPtr<ID3D12ShaderReflectionVariable>) -> Self {
        Self { inner }
    }

    pub fn get_buffer(&self) -> D3D12ShaderReflectionConstantBuffer {
        D3D12ShaderReflectionConstantBuffer::new(unsafe { self.inner.get_buffer() })
    }

    pub fn get_desc(&self) -> Result<d3d12shader::D3D12_SHADER_VARIABLE_DESC, HassleError> {
        let mut desc = d3d12shader::D3D12_SHADER_VARIABLE_DESC::default();
        return_hr_wrapped!(unsafe { self.inner.get_desc(&mut desc as *mut _) }, desc);
    }

    pub fn get_interface_slot(&self, array_index: u32) -> u32 {
        unsafe { self.inner.get_interface_slot(array_index) }
    }

    pub fn get_type(&self) -> D3D12ShaderReflectionType {
        unsafe { D3D12ShaderReflectionType::new(self.inner.get_type()) }
    }
}

#[derive(Debug)]
pub struct D3D12ShaderReflectionConstantBuffer {
    inner: ComPtr<ID3D12ShaderReflectionConstantBuffer>,
}

impl D3D12ShaderReflectionConstantBuffer {
    fn new(buffer: ComPtr<ID3D12ShaderReflectionConstantBuffer>) -> Self {
        Self { inner: buffer }
    }
    pub fn get_desc(&self) -> Result<d3d12shader::D3D12_SHADER_BUFFER_DESC, HassleError> {
        let mut p_desc = d3d12shader::D3D12_SHADER_BUFFER_DESC::default();
        return_hr_wrapped!(
            unsafe { self.inner.get_desc(&mut p_desc as *mut _) },
            p_desc
        );
    }

    pub fn get_variable_by_index(&self, index: u32) -> D3D12ShaderReflectionVariable {
        unsafe { D3D12ShaderReflectionVariable::new(self.inner.get_variable_by_index(index)) }
    }

    pub fn get_variable_by_name(&self, name: &CStr) -> D3D12ShaderReflectionVariable {
        unsafe {
            D3D12ShaderReflectionVariable::new(
                self.inner.get_variable_by_name(name.as_ptr() as *const i8),
            )
        }
    }
}

#[derive(Debug)]
pub struct D3D12ShaderReflection {
    inner: ComPtr<ID3D12ShaderReflection>,
}

impl D3D12ShaderReflection {
    fn new(inner: ComPtr<ID3D12ShaderReflection>) -> Self {
        Self { inner }
    }

    pub fn get_bitwise_count(&self) -> u32 {
        unsafe { self.inner.get_bitwise_instruction_count() }
    }

    pub fn get_constant_buffer_by_index(&self, index: u32) -> D3D12ShaderReflectionConstantBuffer {
        unsafe {
            D3D12ShaderReflectionConstantBuffer::new(self.inner.get_constant_buffer_by_index(index))
        }
    }

    pub fn get_constant_buffer_by_name(&self, name: &CStr) -> D3D12ShaderReflectionConstantBuffer {
        unsafe {
            D3D12ShaderReflectionConstantBuffer::new(
                self.inner
                    .get_constant_buffer_by_name(name.as_ptr() as *const _),
            )
        }
    }

    pub fn get_conversion_instruction_count(&self) -> u32 {
        unsafe { self.inner.get_conversion_instruction_count() }
    }

    pub fn get_desc(&self) -> Result<d3d12shader::D3D12_SHADER_DESC, HassleError> {
        let mut desc = d3d12shader::D3D12_SHADER_DESC::default();
        return_hr_wrapped!(unsafe { self.inner.get_desc(&mut desc as *mut _) }, desc);
    }

    pub fn get_gs_input_primitive(&self) -> d3dcommon::D3D_PRIMITIVE {
        unsafe { self.inner.get_gs_input_primitive() }
    }

    pub fn get_min_feature_level(&self) -> Result<d3dcommon::D3D_FEATURE_LEVEL, HassleError> {
        let mut feature_level = d3dcommon::D3D_FEATURE_LEVEL::default();
        return_hr_wrapped!(
            unsafe {
                self.inner
                    .get_min_feature_level(&mut feature_level as *mut _)
            },
            feature_level
        );
    }

    pub fn get_movc_instruction_count(&self) -> u32 {
        unsafe { self.inner.get_movc_instruction_count() }
    }

    pub fn get_mov_instruction_count(&self) -> u32 {
        unsafe { self.inner.get_mov_instruction_count() }
    }

    pub fn get_num_interface_slots(&self) -> u32 {
        unsafe { self.inner.get_num_interface_slots() }
    }

    pub fn get_output_parameter_desc(
        &self,
        parameter_index: u32,
    ) -> Result<d3d12shader::D3D12_SIGNATURE_PARAMETER_DESC, HassleError> {
        let mut desc = d3d12shader::D3D12_SIGNATURE_PARAMETER_DESC::default();
        return_hr_wrapped!(
            unsafe {
                self.inner
                    .get_output_parameter_desc(parameter_index, &mut desc as *mut _)
            },
            desc
        );
    }

    pub fn get_patch_constant_parameter_desc(
        &self,
        parameter_index: u32,
    ) -> Result<d3d12shader::D3D12_SIGNATURE_PARAMETER_DESC, HassleError> {
        let mut desc = d3d12shader::D3D12_SIGNATURE_PARAMETER_DESC::default();
        return_hr_wrapped!(
            unsafe {
                self.inner
                    .get_patch_constant_parameter_desc(parameter_index, &mut desc as *mut _)
            },
            desc
        );
    }

    pub fn get_requires_flags(&self) -> u64 {
        unsafe { self.inner.get_requires_flags() }
    }

    pub fn get_resource_binding_desc(
        &self,
        resource_index: u32,
    ) -> Result<d3d12shader::D3D12_SHADER_INPUT_BIND_DESC, HassleError> {
        let mut desc = d3d12shader::D3D12_SHADER_INPUT_BIND_DESC::default();
        return_hr_wrapped!(
            unsafe {
                self.inner
                    .get_resource_binding_desc(resource_index, &mut desc as *mut _)
            },
            desc
        );
    }

    pub fn get_resource_binding_desc_by_name(
        &self,
        name: &CStr,
    ) -> Result<d3d12shader::D3D12_SHADER_INPUT_BIND_DESC, HassleError> {
        let mut desc = d3d12shader::D3D12_SHADER_INPUT_BIND_DESC::default();
        return_hr_wrapped!(
            unsafe {
                self.inner.get_resource_binding_desc_by_name(
                    name.as_ptr() as *const i8,
                    &mut desc as *mut _,
                )
            },
            desc
        );
    }

    pub fn get_thread_group_size(&self) -> (u32, u32, u32, u32) {
        let mut x = 0u32;
        let mut y = 0u32;
        let mut z = 0u32;
        let total_size = unsafe { self.inner.get_thread_group_size(&mut x, &mut y, &mut z) };
        (x, y, z, total_size)
    }

    pub fn get_variable_by_name(&self, name: &CStr) -> D3D12ShaderReflectionVariable {
        unsafe {
            D3D12ShaderReflectionVariable::new(
                self.inner.get_variable_by_name(name.as_ptr() as *const i8),
            )
        }
    }
}
