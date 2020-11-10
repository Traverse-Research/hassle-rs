#![allow(clippy::transmute_ptr_to_ptr)]
#![allow(clippy::too_many_arguments)]

use crate::os::{HRESULT, LPCSTR};
use com_rs::{com_interface, iid, IUnknown};
use winapi::um::{d3d12shader, d3dcommon};

iid!(pub IID_ID3D12ShaderReflectionVariable = 0x8337_a8a6, 0xa216, 0x444a, 0xb2, 0xf4, 0x31, 0x47, 0x33, 0xa7, 0x3a, 0xea);
com_interface! {
    interface ID3D12ShaderReflectionVariable: IUnknown {
        iid: IID_ID3D12ShaderReflectionVariable,
        vtable: ID3D12ShaderReflectionVariableVtbl,

        fn get_desc(p_desc: *mut d3d12shader::D3D12_SHADER_VARIABLE_DESC) -> HRESULT;
        fn get_type() -> *mut ID3D12ShaderReflectionType;
        fn get_buffer() -> *mut ID3D12ShaderReflectionConstantBuffer;
        fn get_interface_slot(array_index: u32) -> u32;
    }
}

iid!(pub IID_ID3D12ShaderReflectionConstantBuffer = 0xc595_98b4, 0x48b3, 0x4869, 0xb9, 0xb1, 0xb1, 0x61, 0x8b, 0x14, 0xa8, 0xb7);
com_interface! {
    interface ID3D12ShaderReflectionConstantBuffer: IUnknown {
        iid: IID_ID3D12ShaderReflectionConstantBuffer,
        vtable: ID3D12ShaderReflectionConstantBufferVtbl,

        fn get_desc(p_desc: *mut d3d12shader::D3D12_SHADER_BUFFER_DESC) -> HRESULT;
        fn get_variable_by_index(index: u32) -> *mut ID3D12ShaderReflectionVariable;
        fn get_variable_by_name(name: LPCSTR) -> *mut ID3D12ShaderReflectionVariable;
    }
}

iid!(pub IID_ID3D12LibraryReflection = 0x8e34_9d19, 0x54db, 0x4a56, 0x9d, 0xc9, 0x11, 0x9d, 0x87, 0xbd, 0xb8, 0x04);
com_interface! {
    interface ID3D12LibraryReflection: IUnknown {
        iid: IID_ID3D12LibraryReflection,
        vtable: ID3D12LibraryReflectionVtbl,

        fn get_desc(p_desc: *mut d3d12shader::D3D12_LIBRARY_DESC) -> HRESULT;
        fn get_function_by_index(function_index: i32) -> *mut ID3D12FunctionReflection;
    }
}

iid!(pub IID_ID3D12FunctionReflection = 0x1108_795c, 0x2772, 0x4ba9, 0xb2, 0xa8, 0xd4, 0x64, 0xdc, 0x7e, 0x27, 0x99);
com_interface! {
    interface ID3D12FunctionReflection: IUnknown {
        iid: IID_ID3D12FunctionReflection,
        vtable: ID3D12FunctionReflectionVtbl,

        fn get_desc(p_desc: *mut d3d12shader::D3D12_FUNCTION_DESC) -> HRESULT;
        fn get_constant_buffer_by_index(buffer_index: u32) -> *mut ID3D12ShaderReflectionConstantBuffer;
        fn get_constant_buffer_by_name(name: LPCSTR) -> *mut ID3D12ShaderReflectionConstantBuffer;
        fn get_resource_binding_desc(resource_index: u32, p_desc: *mut d3d12shader::D3D12_SHADER_INPUT_BIND_DESC) -> HRESULT;
        fn get_variable_by_name(name: LPCSTR) -> *mut ID3D12ShaderReflectionVariable;
        fn get_resource_binding_desc_by_name(name: LPCSTR, p_desc: *mut d3d12shader::D3D12_SHADER_INPUT_BIND_DESC) -> HRESULT;
        fn get_function_parameter(parameter_index: i32) -> *mut ID3D12FunctionParameterReflection;
    }
}

iid!(pub IID_ID3D12FunctionParameterReflection = 0xec25_f42d, 0x7006, 0x4f2b, 0xb3, 0x3e, 0x02, 0xcc, 0x33, 0x75, 0x73, 0x3f);
com_interface! {
    interface ID3D12FunctionParameterReflection: IUnknown {
        iid: IID_ID3D12FunctionParameterReflection,
        vtable: ID3D12FunctionParameterReflectionVtbl,

        fn get_desc(p_desc: *mut d3d12shader::D3D12_PARAMETER_DESC) -> HRESULT;
    }
}

iid!(pub IID_ID3D12ShaderReflectionType = 0xe913_c351, 0x783d, 0x48ca, 0xa1, 0xd1, 0x4f, 0x30, 0x62, 0x84, 0xad, 0x56);
com_interface! {
    interface ID3D12ShaderReflectionType: IUnknown {
        iid: IID_ID3D12ShaderReflectionType,
        vtable: ID3D12ShaderReflectionTypeVtbl,

        fn get_desc(p_desc: *mut d3d12shader::D3D12_SHADER_TYPE_DESC) -> HRESULT;
        fn get_member_type_by_index(index: u32) -> *mut ID3D12ShaderReflectionType;
        fn get_member_type_by_name(name: LPCSTR) -> *mut ID3D12ShaderReflectionType;
        fn get_member_type_name(index: u32) -> LPCSTR;
        fn is_equal(p_desc: *mut ID3D12ShaderReflectionType) -> HRESULT;
        fn get_sub_type() -> *mut ID3D12ShaderReflectionType;
        fn get_base_class() -> *mut ID3D12ShaderReflectionType;
        fn get_num_interfaces() -> u32;
        fn get_interface_by_index(index: u32) -> *mut ID3D12ShaderReflectionType;
        fn is_of_type(p_desc: *mut ID3D12ShaderReflectionType) -> HRESULT;
        fn implements_interface(p_base: *mut ID3D12ShaderReflectionType) -> HRESULT;
    }
}

iid!(pub IID_ID3D12ShaderReflection = 0x5a58_797d, 0xa72c, 0x478d, 0x8b, 0xa2, 0xef, 0xc6, 0xb0, 0xef, 0xe8, 0x8e);
com_interface! {
    interface ID3D12ShaderReflection: IUnknown {
        iid: IID_ID3D12ShaderReflection,
        vtable: ID3D12ShaderReflectionVtbl,
        fn get_desc(p_desc: *mut d3d12shader::D3D12_SHADER_DESC) -> HRESULT;
        fn get_constant_buffer_by_index(index: u32) -> *mut ID3D12ShaderReflectionConstantBuffer;
        fn get_constant_buffer_by_name(name: LPCSTR) -> *mut ID3D12ShaderReflectionConstantBuffer;
        fn get_resource_binding_desc(resource_index: u32, p_desc: *mut d3d12shader::D3D12_SHADER_INPUT_BIND_DESC) -> HRESULT;
        fn get_input_parameter_desc(parameter_index: u32, p_desc: *mut d3d12shader::D3D12_SIGNATURE_PARAMETER_DESC) -> HRESULT;
        fn get_output_parameter_desc(parameter_index: u32, p_desc: *mut d3d12shader::D3D12_SIGNATURE_PARAMETER_DESC) -> HRESULT;
        fn get_patch_constant_parameter_desc(parameter_index: u32, p_desc: *mut d3d12shader::D3D12_SIGNATURE_PARAMETER_DESC) -> HRESULT;
        fn get_variable_by_name(name: LPCSTR) -> *mut ID3D12ShaderReflectionVariable;
        fn get_resource_binding_desc_by_name(name: LPCSTR, p_desc: *mut d3d12shader::D3D12_SHADER_INPUT_BIND_DESC) -> HRESULT;
        fn get_mov_instruction_count() -> u32;
        fn get_movc_instruction_count() -> u32;
        fn get_conversion_instruction_count() -> u32;
        fn get_bitwise_instruction_count() -> u32;
        fn get_gs_input_primitive() -> d3dcommon::D3D_PRIMITIVE;
        fn is_sample_frequency_shader() -> bool;
        fn get_num_interface_slots() -> u32;
        fn get_min_feature_level(p_level: *mut d3dcommon::D3D_FEATURE_LEVEL) -> HRESULT;
        fn get_thread_group_size(size_x: *mut u32, size_y: *mut u32, size_z: *mut u32) -> u32;
        fn get_requires_flags() -> u64;
    }
}
