#![allow(clippy::transmute_ptr_to_ptr)]
#![allow(clippy::too_many_arguments)]

use crate::os::{HRESULT, LPCWSTR, LPWSTR};
use com::{interfaces, interfaces::IUnknown, IID};
use std::ffi::c_void;

pub use crate::ffi_enums::*;

pub fn shader_version_program_type(version: u32) -> D3D12_SHADER_VERSION_TYPE {
    // see https://learn.microsoft.com/en-us/windows/win32/api/d3d12shader/ns-d3d12shader-d3d12_shader_desc
    D3D12_SHADER_VERSION_TYPE(((version & 0xFFFF0000u32) >> 16) as i32)
}

pub fn shader_version_major_version(version: u32) -> u32 {
    // see https://learn.microsoft.com/en-us/windows/win32/api/d3d12shader/ns-d3d12shader-d3d12_shader_desc
    (version & 0x000000F0) >> 4
}

pub fn shader_version_minor_version(version: u32) -> u32 {
    // see https://learn.microsoft.com/en-us/windows/win32/api/d3d12shader/ns-d3d12shader-d3d12_shader_desc
    version & 0x0000000F
}

#[rustfmt::skip]
#[allow(non_camel_case_types, non_snake_case)]
#[repr(C)]
pub struct D3D12_SIGNATURE_PARAMETER_DESC
{
    pub SemanticName: *mut std::ffi::c_char,        // Name of the semantic
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

#[rustfmt::skip]
#[allow(non_camel_case_types, non_snake_case)]
#[repr(C)]
pub struct D3D12_SHADER_BUFFER_DESC
{
    pub Name: *mut std::ffi::c_char,            // Name of the constant buffer
    pub Type: D3D_CBUFFER_TYPE,                 // Indicates type of buffer content
    pub Variables: u32,                         // Number of member variables
    pub Size: u32,                              // Size of CB (in bytes)
    pub uFlags: u32,                            // Buffer description flags
}

#[rustfmt::skip]
#[allow(non_camel_case_types, non_snake_case)]
#[repr(C)]
pub struct D3D12_SHADER_VARIABLE_DESC
{
    pub Name: *mut std::ffi::c_char,            // Name of the variable
    pub StartOffset: u32,                       // Offset in constant buffer's backing store
    pub Size: u32,                              // Size of variable (in bytes)
    pub uFlags: u32,                            // Variable flags
    pub DefaultValue: *mut std::ffi::c_void,    // Raw pointer to default value
    pub StartTexture: u32,                      // First texture index (or -1 if no textures used)
    pub TextureSize: u32,                       // Number of texture slots possibly used.
    pub StartSampler: u32,                      // First sampler index (or -1 if no textures used)
    pub SamplerSize: u32,                       // Number of sampler slots possibly used.
}

#[rustfmt::skip]
#[allow(non_camel_case_types, non_snake_case)]
#[repr(C)]
pub struct D3D12_SHADER_TYPE_DESC
{
    pub Class: D3D_SHADER_VARIABLE_CLASS,       // Variable class (e.g. object, matrix, etc.)
    pub Type: D3D_SHADER_VARIABLE_TYPE,         // Variable type (e.g. float, sampler, etc.)
    pub Rows: u32,                              // Number of rows (for matrices, 1 for other numeric, 0 if not applicable)
    pub Columns: u32,                           // Number of columns (for vectors & matrices, 1 for other numeric, 0 if not applicable)
    pub Elements: u32,                          // Number of elements (0 if not an array)
    pub Members: u32,                           // Number of members (0 if not a structure)
    pub Offset: u32,                            // Offset from the start of structure (0 if not a structure member)
    pub Name: *mut std::ffi::c_char,            // Name of type, can be NULL
}

#[rustfmt::skip]
#[allow(non_camel_case_types, non_snake_case)]
#[repr(C)]
pub struct D3D12_SHADER_DESC
{
    pub Version: u32,                                           // Shader version
    pub Creator: *mut std::ffi::c_char,                         // Creator string
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

#[rustfmt::skip]
#[allow(non_camel_case_types, non_snake_case)]
#[repr(C)]
pub struct D3D12_SHADER_INPUT_BIND_DESC
{
    pub Name: *mut std::ffi::c_char,            // Name of the resource
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

//
// The following structs are psuedo-com objects that have no QueryInterface, AddRef, or RemoveRef
// calls. So we have to manually specify them. They are not reference counted and are only valid
// as long as the underlying "owning" reflection object remains available. The safe wrappers hold
// onto both one of these structs and a ref-counted pointer to the owning COM object.
//
#[rustfmt::skip]
#[allow(non_snake_case)]
#[repr(C)]
pub(crate) struct ID3D12ShaderReflectionTypeVTable {
    pub GetDesc: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionTypeVPtr>, *mut D3D12_SHADER_TYPE_DESC) -> HRESULT,
    pub GetMemberTypeByIndex: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionTypeVPtr>, index: u32) -> ID3D12ShaderReflectionType,
    pub GetMemberTypeByName: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionTypeVPtr>, name: *const std::ffi::c_char) -> ID3D12ShaderReflectionType,
    pub GetMemberTypeName: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionTypeVPtr>, index: u32) -> *const std::ffi::c_char,
    pub IsEqual: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionTypeVPtr>, ty: ID3D12ShaderReflectionType) -> HRESULT,
    pub GetSubType: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionTypeVPtr>) -> ID3D12ShaderReflectionType,
    pub GetBaseClass: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionTypeVPtr>) -> ID3D12ShaderReflectionType,
    pub GetNumInterfaces: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionTypeVPtr>) -> u32,
    pub GetInterfaceByIndex: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionTypeVPtr>, index: u32) -> ID3D12ShaderReflectionType,
    pub IsOfType: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionTypeVPtr>, other: ID3D12ShaderReflectionType) -> HRESULT,
    pub ImplementsInterface: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionTypeVPtr>, other: ID3D12ShaderReflectionType) -> HRESULT,
}

pub(crate) type ID3D12ShaderReflectionTypeVPtr =
    ::core::ptr::NonNull<ID3D12ShaderReflectionTypeVTable>;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ID3D12ShaderReflectionType {
    inner: ::core::ptr::NonNull<ID3D12ShaderReflectionTypeVPtr>,
}

impl ID3D12ShaderReflectionType {
    pub(crate) unsafe fn get_desc(&self, desc: impl Into<*mut D3D12_SHADER_TYPE_DESC>) -> HRESULT {
        (self.inner.as_ref().as_ref().GetDesc)(self.inner, desc.into())
    }

    pub(crate) unsafe fn get_member_type_by_index(&self, index: u32) -> ID3D12ShaderReflectionType {
        (self.inner.as_ref().as_ref().GetMemberTypeByIndex)(self.inner, index)
    }

    pub(crate) unsafe fn get_member_type_by_name(
        &self,
        name: *const std::ffi::c_char,
    ) -> ID3D12ShaderReflectionType {
        (self.inner.as_ref().as_ref().GetMemberTypeByName)(self.inner, name)
    }

    pub(crate) unsafe fn get_member_type_name(&self, index: u32) -> *const std::ffi::c_char {
        (self.inner.as_ref().as_ref().GetMemberTypeName)(self.inner, index)
    }

    pub(crate) unsafe fn is_equal(&self, ty: ID3D12ShaderReflectionType) -> HRESULT {
        (self.inner.as_ref().as_ref().IsEqual)(self.inner, ty)
    }

    pub(crate) unsafe fn get_sub_type(&self) -> ID3D12ShaderReflectionType {
        (self.inner.as_ref().as_ref().GetSubType)(self.inner)
    }

    pub(crate) unsafe fn get_base_class(&self) -> ID3D12ShaderReflectionType {
        (self.inner.as_ref().as_ref().GetBaseClass)(self.inner)
    }

    pub(crate) unsafe fn get_num_interfaces(&self) -> u32 {
        (self.inner.as_ref().as_ref().GetNumInterfaces)(self.inner)
    }

    pub(crate) unsafe fn get_interface_by_index(&self, index: u32) -> ID3D12ShaderReflectionType {
        (self.inner.as_ref().as_ref().GetInterfaceByIndex)(self.inner, index)
    }

    pub(crate) unsafe fn is_of_type(&self, ty: ID3D12ShaderReflectionType) -> HRESULT {
        (self.inner.as_ref().as_ref().IsOfType)(self.inner, ty)
    }

    pub(crate) unsafe fn implements_interface(&self, ty: ID3D12ShaderReflectionType) -> HRESULT {
        (self.inner.as_ref().as_ref().ImplementsInterface)(self.inner, ty)
    }
}

#[rustfmt::skip]
#[allow(non_snake_case)]
#[repr(C)]
pub(crate) struct ID3D12ShaderReflectionVariableVTable {
    pub GetDesc: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionVariableVPtr>, *mut D3D12_SHADER_VARIABLE_DESC) -> HRESULT,
    pub GetType: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionVariableVPtr> ) -> ID3D12ShaderReflectionType,
    pub GetBuffer: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionVariableVPtr>) -> ID3D12ShaderReflectionConstantBuffer,
    pub GetInterfaceSlot: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionVariableVPtr>, u32) -> u32,
}

pub(crate) type ID3D12ShaderReflectionVariableVPtr =
    ::core::ptr::NonNull<ID3D12ShaderReflectionVariableVTable>;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ID3D12ShaderReflectionVariable {
    inner: ::core::ptr::NonNull<ID3D12ShaderReflectionVariableVPtr>,
}
impl ID3D12ShaderReflectionVariable {
    pub(crate) unsafe fn get_desc(
        &self,
        p_desc: impl Into<*mut D3D12_SHADER_VARIABLE_DESC>,
    ) -> HRESULT {
        (self.inner.as_ref().as_ref().GetDesc)(self.inner, p_desc.into())
    }

    pub(crate) unsafe fn get_type(&self) -> ID3D12ShaderReflectionType {
        (self.inner.as_ref().as_ref().GetType)(self.inner)
    }

    pub(crate) unsafe fn get_buffer(&self) -> ID3D12ShaderReflectionConstantBuffer {
        (self.inner.as_ref().as_ref().GetBuffer)(self.inner)
    }

    pub(crate) unsafe fn get_interface_slot(&self, array_index: u32) -> u32 {
        (self.inner.as_ref().as_ref().GetInterfaceSlot)(self.inner, array_index)
    }
}

#[rustfmt::skip]
#[allow(non_snake_case)]
#[repr(C)]
pub(crate) struct ID3D12ShaderReflectionConstantBufferVTable {
    pub GetDesc: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionConstantBufferVPtr>, *mut D3D12_SHADER_BUFFER_DESC) -> HRESULT,
    pub GetVariableByIndex: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionConstantBufferVPtr>, u32) -> ID3D12ShaderReflectionVariable,
    pub GetVariableByName: unsafe extern "system" fn(::core::ptr::NonNull<ID3D12ShaderReflectionConstantBufferVPtr>, *const std::ffi::c_char) -> ID3D12ShaderReflectionVariable,
}

pub(crate) type ID3D12ShaderReflectionConstantBufferVPtr =
    ::core::ptr::NonNull<ID3D12ShaderReflectionConstantBufferVTable>;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ID3D12ShaderReflectionConstantBuffer {
    inner: ::core::ptr::NonNull<ID3D12ShaderReflectionConstantBufferVPtr>,
}

impl ID3D12ShaderReflectionConstantBuffer {
    pub(crate) unsafe fn get_desc(
        &self,
        p_desc: impl Into<*mut D3D12_SHADER_BUFFER_DESC>,
    ) -> HRESULT {
        (self.inner.as_ref().as_ref().GetDesc)(self.inner, p_desc.into())
    }

    pub(crate) unsafe fn get_variable_by_index(
        &self,
        index: u32,
    ) -> ID3D12ShaderReflectionVariable {
        (self.inner.as_ref().as_ref().GetVariableByIndex)(self.inner, index)
    }

    pub(crate) unsafe fn get_variable_by_name(
        &self,
        name: impl Into<*const std::ffi::c_char>,
    ) -> ID3D12ShaderReflectionVariable {
        (self.inner.as_ref().as_ref().GetVariableByName)(self.inner, name.into())
    }
}

pub type DxcCreateInstanceProc<T> =
    extern "system" fn(rclsid: &IID, riid: &IID, ppv: *mut Option<T>) -> HRESULT;

pub type DxcCreateInstanceProc2 = extern "system" fn(
    malloc: /* IMalloc */ *const c_void,
    rclsid: &IID,
    riid: &IID,
    ppv: *mut *mut c_void,
) -> HRESULT;

pub const DFCC_DXIL: u32 = u32::from_le_bytes([b'D', b'X', b'I', b'L']);

interfaces! {
    #[uuid("8ba5fb08-5195-40e2-ac58-0d989c3a0102")]
    pub(crate) unsafe interface IDxcBlob: IUnknown {
        pub(crate) fn get_buffer_pointer(&self) -> *mut c_void;
        pub(crate) fn get_buffer_size(&self) -> usize;
    }

    #[uuid("7241d424-2646-4191-97c0-98e96e42fc68")]
    pub(crate) unsafe interface IDxcBlobEncoding: IDxcBlob {
        pub(crate) fn get_encoding(&self, known: *mut u32, code_page: *mut u32) -> HRESULT;
    }

    #[uuid("e5204dc7-d18c-4c3c-bdfb-851673980fe7")]
    pub(crate) unsafe interface IDxcLibrary: IUnknown {
        pub(crate) fn set_malloc(&self, malloc: *const c_void) -> HRESULT;
        pub(crate) fn create_blob_from_blob(
            &self,
            blob: IDxcBlob,
            offset: u32,
            length: u32,
            result_blob: *mut Option<IDxcBlob>,
        ) -> HRESULT;
        pub(crate) fn create_blob_from_file(
            &self,
            filename: LPCWSTR,
            code_page: *const u32,
            blob_encoding: *mut Option<IDxcBlobEncoding>,
        ) -> HRESULT;
        pub(crate) fn create_blob_with_encoding_from_pinned(
            &self,
            text: *const c_void,
            size: u32,
            code_page: u32,
            blob_encoding: *mut Option<IDxcBlobEncoding>,
        ) -> HRESULT;
        pub(crate) fn create_blob_with_encoding_on_heap_copy(
            &self,
            text: *const c_void,
            size: u32,
            code_page: u32,
            blob_encoding: *mut Option<IDxcBlobEncoding>,
        ) -> HRESULT;
        pub(crate) fn create_blob_with_encoding_on_malloc(
            &self,
            text: *const c_void,
            malloc: *const /* IMalloc */ c_void,
            size: u32,
            code_page: u32,
            blob_encoding: *mut Option<IDxcBlobEncoding>,
        ) -> HRESULT;
        pub(crate) fn create_include_handler(
            &self,
            include_handler: *mut Option<IDxcIncludeHandler>,
        ) -> HRESULT;
        pub(crate) fn create_stream_from_blob_read_only(
            &self,
            blob: IDxcBlob,
            stream: *mut *mut /* IStream */ c_void,
        ) -> HRESULT;
        pub(crate) fn get_blob_as_utf8(
            &self,
            blob: IDxcBlob,
            blob_encoding: *mut Option<IDxcBlobEncoding>,
        ) -> HRESULT;
        pub(crate) fn get_blob_as_utf16(
            &self,
            blob: IDxcBlob,
            blob_encoding: *mut Option<IDxcBlobEncoding>,
        ) -> HRESULT;
    }

    #[uuid("cedb484a-d4e9-445a-b991-ca21ca157dc2")]
    pub(crate) unsafe interface IDxcOperationResult: IUnknown {
        pub(crate) fn get_status(&self, status: *mut u32) -> HRESULT;
        pub(crate) fn get_result(&self, result: *mut Option<IDxcBlob>) -> HRESULT;
        pub(crate) fn get_error_buffer(&self, errors: *mut Option<IDxcBlobEncoding>) -> HRESULT;
    }

    #[uuid("7f61fc7d-950d-467f-b3e3-3c02fb49187c")]
    pub(crate) unsafe interface IDxcIncludeHandler: IUnknown {
        pub(crate) fn load_source(
            &self,
            filename: LPCWSTR,
            include_source: *mut Option<IDxcBlob>,
        ) -> HRESULT;
    }
}

#[repr(C)]
pub struct DxcDefine {
    pub name: LPCWSTR,
    pub value: LPCWSTR,
}

interfaces! {
    #[uuid("8c210bf3-011f-4422-8d70-6f9acb8db617")]
    pub(crate) unsafe interface IDxcCompiler: IUnknown {
        pub(crate) fn compile(
            &self,
            blob: IDxcBlob,
            source_name: LPCWSTR,
            entry_point: LPCWSTR,
            target_profile: LPCWSTR,
            arguments: *const LPCWSTR,
            arg_count: u32,
            defines: *const DxcDefine,
            def_count: u32,
            include_handler: Option<IDxcIncludeHandler>,
            result: *mut Option<IDxcOperationResult>,
        ) -> HRESULT;

        pub(crate) fn preprocess(
            &self,
            blob: IDxcBlob,
            source_name: LPCWSTR,
            arguments: *const LPCWSTR,
            arg_count: u32,
            defines: *const DxcDefine,
            def_count: u32,
            include_handler: Option<IDxcIncludeHandler>,
            result: *mut Option<IDxcOperationResult>,
        ) -> HRESULT;

        pub(crate) fn disassemble(
            &self,
            blob: IDxcBlob,
            disassembly: *mut Option<IDxcBlobEncoding>,
        ) -> HRESULT;
    }

    #[uuid("a005a9d9-b8bb-4594-b5c9-0e633bec4d37")]
    pub(crate) unsafe interface IDxcCompiler2: IDxcCompiler {
        pub(crate) fn compile_with_debug(
            &self,
            blob: IDxcBlob,
            source_name: LPCWSTR,
            entry_point: LPCWSTR,
            target_profile: LPCWSTR,
            arguments: *const LPCWSTR,
            arg_count: u32,
            defines: *const DxcDefine,
            def_count: u32,
            include_handler: Option<IDxcIncludeHandler>,
            result: *mut Option<IDxcOperationResult>,
            debug_blob_name: *mut LPWSTR,
            debug_blob: *mut Option<IDxcBlob>,
        ) -> HRESULT;
    }

    #[uuid("f1b5be2a-62dd-4327-a1c2-42ac1e1e78e6")]
    pub(crate) unsafe interface IDxcLinker: IUnknown {
        pub(crate) fn register_library(&self, lib_name: LPCWSTR, lib: IDxcBlob) -> HRESULT;

        pub(crate) fn link(
            &self,
            entry_name: LPCWSTR,
            target_profile: LPCWSTR,
            lib_names: *const LPCWSTR,
            lib_count: u32,
            arguments: *const LPCWSTR,
            arg_count: u32,
            result: *mut Option<IDxcOperationResult>,
        ) -> HRESULT;
    }
}

// From dxcapi.h for use with IDxcValidator
pub const DXC_VALIDATOR_FLAGS_DEFAULT: u32 = 0;
pub const DXC_VALIDATOR_FLAGS_IN_PLACE_EDIT: u32 = 1; // Validator is allowed to update shader blob in-place.
pub const DXC_VALIDATOR_FLAGS_ROOT_SIGNATURE_ONLY: u32 = 2;
pub const DXC_VALIDATOR_FLAGS_MODULE_ONLY: u32 = 4;
pub const DXC_VALIDATOR_FLAGS_VALID_MASK: u32 = 0x7;

interfaces! {
    #[uuid("a6e82bd2-1fd7-4826-9811-2857e797f49a")]
    pub(crate) unsafe interface IDxcValidator: IUnknown {
        pub(crate) fn validate(
            &self,
            shader: IDxcBlob,
            flags: u32,
            result: *mut Option<IDxcOperationResult>,
        ) -> HRESULT;
    }

    #[uuid("334b1f50-2292-4b35-99a1-25588d8c17fe")]
    pub(crate) unsafe interface IDxcContainerBuilder: IUnknown {
        pub(crate) fn load(&self, dxil_container_header: IDxcBlob) -> HRESULT;
        pub(crate) fn add_part(&self, four_cc: u32, source: IDxcBlob) -> HRESULT;
        pub(crate) fn remove_part(&self, four_cc: u32) -> HRESULT;
        pub(crate) fn seralize_container(
            &self,
            result: *mut Option<IDxcOperationResult>,
        ) -> HRESULT;
    }

    #[uuid("091f7a26-1c1f-4948-904b-e6e3a8a771d5")]
    pub(crate) unsafe interface IDxcAssembler: IUnknown {
        pub(crate) fn assemble_to_container(
            &self,
            shader: IDxcBlob,
            result: *mut Option<IDxcOperationResult>,
        ) -> HRESULT;
    }

    #[uuid("d2c21b26-8350-4bdc-976a-331ce6f4c54c")]
    pub(crate) unsafe interface IDxcContainerReflection: IUnknown {
        pub(crate) fn load(&self, container: IDxcBlob) -> HRESULT;
        pub(crate) fn get_part_count(&self, result: *mut u32) -> HRESULT;
        pub(crate) fn get_part_kind(&self, idx: u32, result: *mut u32) -> HRESULT;
        pub(crate) fn get_part_content(&self, idx: u32, result: *mut Option<IDxcBlob>) -> HRESULT;
        pub(crate) fn find_first_part_kind(&self, kind: u32, result: *mut u32) -> HRESULT;
        pub(crate) fn get_part_reflection(
            &self,
            idx: u32,
            iid: *const IID,
            object: *mut Option<IUnknown>,
        ) -> HRESULT;
    }

    //NOTE: Unlike the other interfaces which are coming from dxcapi.h, this comes from
    // d3d12shader.h. Some of the returned values are COM-like objects that have a vtable similar
    // to COM with QueryInterface, AddRef and RemoveRef, but don't have IUnknown as a base. The
    // COM helper library doesn't deal well with this, so we have to manually implement interfaces
    // for those types.
    #[uuid("5a58797d-a72c-478d-8ba2-efc6b0efe88e")]
    pub(crate) unsafe interface ID3D12ShaderReflection: IUnknown {
        pub(crate) fn get_desc(&self, p_desc: *mut D3D12_SHADER_DESC) -> HRESULT;
        pub(crate) fn get_constant_buffer_by_index(&self, index: u32) -> ID3D12ShaderReflectionConstantBuffer;
        pub(crate) fn get_constant_buffer_by_name(&self, name: *const std::ffi::c_char) -> ID3D12ShaderReflectionConstantBuffer;
        pub(crate) fn get_resource_binding_desc(
            &self,
            resource_index: u32,
            p_desc: *mut D3D12_SHADER_INPUT_BIND_DESC,
        ) -> HRESULT;
        pub(crate) fn get_input_parameter_desc(
            &self,
            parameter_index: u32,
            p_desc: *mut D3D12_SIGNATURE_PARAMETER_DESC,
        ) -> HRESULT;
        pub(crate) fn get_output_parameter_desc(
            &self,
            parameter_index: u32,
            p_desc: *mut D3D12_SIGNATURE_PARAMETER_DESC,
        ) -> HRESULT;
        pub(crate) fn get_patch_constant_parameter_desc(
            &self,
            parameter_index: u32,
            p_desc: *mut D3D12_SIGNATURE_PARAMETER_DESC,
        ) -> HRESULT;
        pub(crate) fn get_variable_by_name(&self, name: *const std::ffi::c_char) -> ID3D12ShaderReflectionVariable;
        pub(crate) fn get_resource_binding_desc_by_name(
            &self,
            name: *const std::ffi::c_char,
            p_desc: *mut D3D12_SIGNATURE_PARAMETER_DESC,
        ) -> HRESULT;
        pub(crate) fn get_mov_instruction_count(&self) -> u32;
        pub(crate) fn get_movc_instruction_count(&self) -> u32;
        pub(crate) fn get_conversion_instruction_count(&self) -> u32;
        pub(crate) fn get_bitwise_instruction_count(&self) -> u32;
        pub(crate) fn get_gs_input_primitive(&self) -> D3D_PRIMITIVE;
        pub(crate) fn is_sample_frequency_shader(&self) -> bool;
        pub(crate) fn get_num_interface_slots(&self) -> u32;
        pub(crate) fn get_min_feature_level(&self, p_level: *mut D3D_FEATURE_LEVEL) -> HRESULT;
        pub(crate) fn get_thread_group_size(
            &self,
            size_x: *mut u32,
            size_y: *mut u32,
            size_z: *mut u32,
        ) -> u32;
        pub(crate) fn get_requires_flags(&self) -> u64;
    }

    #[uuid("ae2cd79f-cc22-453f-9b6b-b124e7a5204c")]
    pub(crate) unsafe interface IDxcOptimizerPass: IUnknown {
        pub(crate) fn get_option_name(&self, result: *mut LPWSTR) -> HRESULT;
        pub(crate) fn get_description(&self, result: *mut LPWSTR) -> HRESULT;
        pub(crate) fn get_option_arg_count(&self, count: *mut u32) -> HRESULT;
        pub(crate) fn get_option_arg_name(&self, arg_idx: u32, result: *mut LPWSTR) -> HRESULT;
        pub(crate) fn get_option_arg_description(
            &self,
            arg_idx: u32,
            result: *mut LPWSTR,
        ) -> HRESULT;
    }

    #[uuid("25740e2e-9cba-401b-9119-4fb42f39f270")]
    pub(crate) unsafe interface IDxcOptimizer: IUnknown {
        pub(crate) fn get_available_pass_count(&self, count: *mut u32) -> HRESULT;
        pub(crate) fn get_available_pass(
            &self,
            index: u32,
            result: *mut Option<IDxcOptimizerPass>,
        ) -> HRESULT;
        pub(crate) fn run_optimizer(
            &self,
            blob: IDxcBlob,
            options: *const LPCWSTR,
            option_count: u32,
            output_module: *mut Option<IDxcBlob>,
            output_text: *mut Option<IDxcBlobEncoding>,
        ) -> HRESULT;
    }
}

// From dxcapi.h for use with IDxcVersionInfo
pub const DXC_VERSION_INFO_FLAGS_NONE: u32 = 0;
pub const DXC_VERSION_INFO_FLAGS_DEBUG: u32 = 1; // Matches VS_FF_DEBUG
pub const DXC_VERSION_INFO_FLAGS_INTERNAL: u32 = 2; // Internal Validator (non-signing)

interfaces! {
    #[uuid("b04f5b50-2059-4f12-a8ff-a1e0cde1cc7e")]
    pub(crate) unsafe interface IDxcVersionInfo: IUnknown {
        pub(crate) fn get_version(&self, major: *mut u32, minor: *mut u32) -> HRESULT;
        pub(crate) fn get_flags(&self, flags: *mut u32) -> HRESULT;
    }

    #[uuid("fb6904c4-42f0-4b62-9c46-983af7da7c83")]
    pub(crate) unsafe interface IDxcVersionInfo2: IUnknown {
        pub(crate) fn get_commit_info(
            &self,
            commit_count: *mut u32,
            commit_hash: *mut *mut u8,
        ) -> HRESULT;
    }
}

pub const CLSID_DxcCompiler: IID = IID {
    data1: 0x73e22d93,
    data2: 0xe6ce,
    data3: 0x47f3,
    data4: [0xb5, 0xbf, 0xf0, 0x66, 0x4f, 0x39, 0xc1, 0xb0],
};
pub const CLSID_DxcLinker: IID = IID {
    data1: 0xef6a8087,
    data2: 0xb0ea,
    data3: 0x4d56,
    data4: [0x9e, 0x45, 0xd0, 0x7e, 0x1a, 0x8b, 0x78, 0x6],
};
pub const CLSID_DxcDiaDataSource: IID = IID {
    data1: 0xcd1f6b73,
    data2: 0x2ab0,
    data3: 0x484d,
    data4: [0x8e, 0xdc, 0xeb, 0xe7, 0xa4, 0x3c, 0xa0, 0x9f],
};
pub const CLSID_DxcLibrary: IID = IID {
    data1: 0x6245d6af,
    data2: 0x66e0,
    data3: 0x48fd,
    data4: [0x80, 0xb4, 0x4d, 0x27, 0x17, 0x96, 0x74, 0x8c],
};
pub const CLSID_DxcValidator: IID = IID {
    data1: 0x8ca3e215,
    data2: 0xf728,
    data3: 0x4cf3,
    data4: [0x8c, 0xdd, 0x88, 0xaf, 0x91, 0x75, 0x87, 0xa1],
};
pub const CLSID_DxcAssembler: IID = IID {
    data1: 0xd728db68,
    data2: 0xf903,
    data3: 0x4f80,
    data4: [0x94, 0xcd, 0xdc, 0xcf, 0x76, 0xec, 0x71, 0x51],
};
pub const CLSID_DxcContainerReflection: IID = IID {
    data1: 0xb9f54489,
    data2: 0x55b8,
    data3: 0x400c,
    data4: [0xba, 0x3a, 0x16, 0x75, 0xe4, 0x72, 0x8b, 0x91],
};
pub const CLSID_DxcOptimizer: IID = IID {
    data1: 0xae2cd79f,
    data2: 0xcc22,
    data3: 0x453f,
    data4: [0x9b, 0x6b, 0xb1, 0x24, 0xe7, 0xa5, 0x20, 0x4c],
};
pub const CLSID_DxcContainerBuilder: IID = IID {
    data1: 0x94134294,
    data2: 0x411f,
    data3: 0x4574,
    data4: [0xb4, 0xd0, 0x87, 0x41, 0xe2, 0x52, 0x40, 0xd2],
};
