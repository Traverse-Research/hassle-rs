#![allow(clippy::transmute_ptr_to_ptr)]
#![allow(clippy::too_many_arguments)]

use crate::os::{HRESULT, LPCWSTR, LPWSTR};
use com_rs::{com_interface, iid, IUnknown, IID};
use std::ffi::c_void;

pub type DxcCreateInstanceProc =
    extern "system" fn(rclsid: &IID, riid: &IID, ppv: *mut *mut c_void) -> HRESULT;

pub type DxcCreateInstanceProc2 = extern "system" fn(
    malloc: *const c_void,
    rclsid: &IID,
    riid: &IID,
    ppv: *mut *mut c_void,
) -> HRESULT;

iid!(pub IID_IDxcBlob = 0x8BA5_FB08, 0x5195, 0x40e2, 0xAC, 0x58, 0x0D, 0x98, 0x9C, 0x3A, 0x01, 0x02);
com_interface! {
    interface IDxcBlob: IUnknown{
        iid: IID_IDxcBlob,
        vtable: IDxcBlobVtbl,
        fn dummy0() -> HRESULT;
        fn dummy1() -> HRESULT;
        fn get_buffer_pointer() -> *const c_void;
        fn get_buffer_size() -> usize;
    }
}

iid!(pub IID_IDxcBlobEncoding = 0x7241_d424, 0x2646, 0x4191, 0x97, 0xc0, 0x98, 0xe9, 0x6e, 0x42, 0xfc, 0x68);
com_interface! {
    interface IDxcBlobEncoding: IDxcBlob, IUnknown{
        iid: IID_IDxcBlobEncoding,
        vtable: IDxcBlobEncodingVtbl,
        fn dummy0() -> HRESULT;
        fn dummy1() -> HRESULT;
        fn get_encoding(known: *mut u32, code_page: *mut u32) -> HRESULT;
    }
}

iid!(pub IID_IDxcLibrary = 0xe520_4dc7, 0xd18c, 0x4c3c, 0xbd, 0xfb, 0x85, 0x16, 0x73, 0x98, 0x0f, 0xe7);
com_interface! {
    interface IDxcLibrary: IUnknown{
        iid: IID_IDxcLibrary,
        vtable: IDxcLibraryVtbl,
        fn dummy0() -> HRESULT;
        fn dummy1() -> HRESULT;
        fn set_malloc(malloc: *const c_void) -> HRESULT;
        fn create_blob_from_blob(blob: *const IDxcBlob, offset: u32, length: u32, result_blob: *mut *mut IDxcBlob) -> HRESULT;
        fn create_blob_from_file(filename: LPCWSTR, code_page: *const u32, blob_encoding: *mut *mut IDxcBlobEncoding) -> HRESULT;
        fn create_blob_with_encoding_from_pinned(text: *const c_void, size: u32, code_page: u32, blob_encoding: *mut *mut IDxcBlobEncoding) -> HRESULT;
        fn create_blob_with_encoding_on_heap_copy(text: *const c_void, size: u32, code_page: u32, blob_encoding: *mut *mut IDxcBlobEncoding) -> HRESULT;
        fn create_blob_with_encoding_on_malloc(text: *const c_void, malloc: *const c_void, size: u32, code_page: u32, blob_encoding: *mut *mut IDxcBlobEncoding) -> HRESULT;
        fn create_include_handler(include_handler: *mut *mut c_void) -> HRESULT;
        fn create_stream_from_blob_read_only(blob: *const IDxcBlob, stream: *mut *mut c_void) -> HRESULT;
        fn get_blob_as_utf8(blob: *const IDxcBlob, blob_encoding: *mut *mut IDxcBlobEncoding) -> HRESULT;
        fn get_blob_as_utf16(blob: *const IDxcBlob, blob_encoding: *mut *mut IDxcBlobEncoding) -> HRESULT;
    }
}

iid!(pub IID_IDxcOperationResult = 0xCEDB_484A, 0xD4E9, 0x445A, 0xB9, 0x91, 0xCA, 0x21, 0xCA, 0x15, 0x7D, 0xC2);
com_interface! {
    interface IDxcOperationResult: IUnknown{
        iid: IID_IDxcOperationResult,
        vtable: IDxcOperationResultVtbl,
        fn dummy0() -> HRESULT;
        fn dummy1() -> HRESULT;
        fn get_status(status: *mut u32) -> HRESULT;
        fn get_result(result: *mut *mut IDxcBlob) -> HRESULT;
        fn get_error_buffer(errors: *mut *mut IDxcBlobEncoding) -> HRESULT;
    }
}

iid!(pub IID_IDxcIncludeHandler = 0x7f61_fc7d, 0x950d, 0x467f, 0xb3, 0xe3, 0x3c, 0x02, 0xfb, 0x49, 0x18, 0x7c);
com_interface! {
    interface IDxcIncludeHandler: IUnknown{
        iid: IID_IDxcIncludeHandler,
        vtable: IDxcIncludeHandlerVtbl,
        fn dummy0() -> HRESULT;
        fn dummy1() -> HRESULT;
        fn load_source(filename: LPCWSTR, include_source: *mut *mut IDxcBlob) -> HRESULT;
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct DxcDefine {
    pub name: LPCWSTR,
    pub value: LPCWSTR,
}

iid!(pub IID_IDxcCompiler = 0x8c21_0bf3, 0x011f, 0x4422, 0x8d, 0x70, 0x6f, 0x9a, 0xcb, 0x8d, 0xb6, 0x17);
com_interface! {
    interface IDxcCompiler: IUnknown{
        iid: IID_IDxcCompiler,
        vtable: IDxcCompilerVtbl,
        fn dummy0() -> HRESULT;
        fn dummy1() -> HRESULT;
        fn compile(
            blob: *const IDxcBlob,
            source_name: LPCWSTR,
            entry_point: LPCWSTR,
            target_profile: LPCWSTR,
            arguments: *const LPCWSTR,
            arg_count: u32,
            defines: *const DxcDefine,
            def_count: u32,
            include_handler: *const c_void,
            result: *mut *mut IDxcOperationResult) -> HRESULT;

        fn preprocess(
            blob: *const IDxcBlob,
            source_name: LPCWSTR,
            arguments: *const LPCWSTR,
            arg_count: u32,
            defines: *const DxcDefine,
            def_count: u32,
            include_handler: *const c_void,
            result: *mut *mut IDxcOperationResult) -> HRESULT;

        fn disassemble(
            blob: *const IDxcBlob,
            disassembly: *mut *mut IDxcBlobEncoding) -> HRESULT;
    }
}

iid!(pub IID_IDxcCompiler2 = 0xA005_A9D9, 0xB8BB, 0x4594, 0xB5, 0xC9, 0x0E, 0x63, 0x3B, 0xEC, 0x4D, 0x37);
com_interface! {
    interface IDxcCompiler2: IDxcCompiler, IUnknown{
        iid: IID_IDxcCompiler2,
        vtable: IDxcCompiler2Vtbl,
        fn dummy0() -> HRESULT;
        fn dummy1() -> HRESULT;

        fn compile_with_debug(
            blob: *const IDxcBlob,
            source_name: LPCWSTR,
            entry_point: LPCWSTR,
            target_profile: LPCWSTR,
            arguments: *const LPCWSTR,
            arg_count: u32,
            defines: *const DxcDefine,
            def_count: u32,
            include_handler: *const c_void,
            result: *mut *mut IDxcOperationResult,
            debug_blob_name: *mut LPWSTR,
            debug_blob: *mut *mut IDxcBlob) -> HRESULT;
    }
}

iid!(pub IID_IDxcLinker = 0xF1B5_BE2A, 0x62DD, 0x4327, 0xA1, 0xC2, 0x42, 0xAC, 0x1E, 0x1E, 0x78, 0xE6);
com_interface! {
    interface IDxcLinker: IUnknown{
        iid: IID_IDxcLinker,
        vtable: IDxcLinkerVtbl,
        fn dummy0() -> HRESULT;
        fn dummy1() -> HRESULT;

        fn register_library(lib_name: LPCWSTR, lib: *const IDxcBlob) -> HRESULT;

        fn link(
            entry_name: LPCWSTR,
            target_profile: LPCWSTR,
            lib_names: *const LPCWSTR,
            lib_count: u32,
            arguments: *const LPCWSTR,
            arg_count: u32,
            result: *mut *mut IDxcOperationResult) -> HRESULT;
    }
}

pub const DXC_VALIDATOR_FLAGS_DEFAULT: u32 = 0;
pub const DXC_VALIDATOR_FLAGS_IN_PLACE_EDIT: u32 = 1; // Validator is allowed to update shader blob in-place.
pub const DXC_VALIDATOR_FLAGS_ROOT_SIGNATURE_ONLY: u32 = 2;
pub const DXC_VALIDATOR_FLAGS_MODULE_ONLY: u32 = 4;
pub const DXC_VALIDATOR_FLAGS_VALID_MASK: u32 = 0x7;

iid!(pub IID_IDxcValidator = 0xA6E8_2BD2, 0x1FD7, 0x4826, 0x98, 0x11, 0x28, 0x57, 0xE7, 0x97, 0xF4, 0x9A);
com_interface! {
    interface IDxcValidator: IUnknown{
        iid: IID_IDxcValidator,
        vtable: IDxcValidatorVtbl,
        fn dummy0() -> HRESULT;
        fn dummy1() -> HRESULT;

        fn validate(shader: *const IDxcBlob, flags: u32, result: *mut *mut IDxcOperationResult) -> HRESULT;
    }
}

iid!(pub IID_IDxcContainerBuilder = 0x334b_1f50, 0x2292, 0x4b35, 0x99, 0xa1, 0x25, 0x58, 0x8d, 0x8c, 0x17, 0xfe);
com_interface! {
    interface IDxcContainerBuilder: IUnknown{
        iid: IID_IDxcContainerBuilder,
        vtable: IDxcContainerBuilderVtbl,
        fn dummy0() -> HRESULT;
        fn dummy1() -> HRESULT;

        fn load(dxil_container_header: *const IDxcBlob) -> HRESULT;
        fn add_part(four_cc: u32, source: *const IDxcBlob) -> HRESULT;
        fn remove_part(four_cc: u32) -> HRESULT;
        fn seralize_container(result: *mut *mut IDxcOperationResult) -> HRESULT;
    }
}

iid!(pub IID_IDxcAssembler = 0x091f_7a26, 0x1c1f, 0x4948, 0x90, 0x4b, 0xe6, 0xe3, 0xa8, 0xa7, 0x71, 0xd5);
com_interface! {
    interface IDxcAssembler: IUnknown{
        iid: IID_IDxcAssembler,
        vtable: IDxcAssemblerVtbl,
        fn dummy0() -> HRESULT;
        fn dummy1() -> HRESULT;

        fn assemble_to_container(shader: *const IDxcBlob, result: *mut *mut IDxcOperationResult) -> HRESULT;
    }
}

iid!(pub IID_IDxcContainerReflection = 0xd2c2_1b26, 0x8350, 0x4bdc, 0x97, 0x6a, 0x33, 0x1c, 0xe6, 0xf4, 0xc5, 0x4c);
com_interface! {
    interface IDxcContainerReflection: IUnknown{
        iid: IID_IDxcContainerReflection,
        vtable: IDxcContainerReflectionVtbl,
        fn dummy0() -> HRESULT;
        fn dummy1() -> HRESULT;

        fn load(container: *const IDxcBlob) -> HRESULT;
        fn get_part_count(result: *mut u32) -> HRESULT;
        fn get_part_kind(idx: u32, result: *mut u32) -> HRESULT;
        fn get_part_content(idx: u32, result: *mut *mut IDxcBlob) -> HRESULT;
        fn find_first_part_kind(kind: u32, result: *mut u32) -> HRESULT;
        fn get_part_reflection(idx: u32, iid: &IID, object: *mut *mut c_void) -> HRESULT;
    }
}

iid!(pub IID_IDxcOptimizerPass = 0xAE2C_D79F, 0xCC22, 0x453F, 0x9B, 0x6B, 0xB1, 0x24, 0xE7, 0xA5, 0x20, 0x4C);
com_interface! {
    interface IDxcOptimizerPass: IUnknown{
        iid: IID_IDxcOptimizerPass,
        vtable: IDxcOptimizerPassVtbl,
        fn dummy0() -> HRESULT;
        fn dummy1() -> HRESULT;

        fn get_option_name(result: *mut LPWSTR) -> HRESULT;
        fn get_description(result: *mut LPWSTR) -> HRESULT;
        fn get_option_arg_count(count: *mut u32) -> HRESULT;
        fn get_option_arg_name(arg_idx: u32, result: *mut LPWSTR) -> HRESULT;
        fn get_option_arg_description(arg_idx: u32, result: *mut LPWSTR) -> HRESULT;
    }
}

iid!(pub IID_IDxcOptimizer = 0x2574_0E2E, 0x9CBA, 0x401B, 0x91, 0x19, 0x4F, 0xB4, 0x2F, 0x39, 0xF2, 0x70);
com_interface! {
    interface IDxcOptimizer: IUnknown{
        iid: IID_IDxcOptimizer,
        vtable: IDxcOptimizerVtbl,
        fn dummy0() -> HRESULT;
        fn dummy1() -> HRESULT;

        fn get_available_pass_count(count: *mut u32) -> HRESULT;
        fn get_available_pass(index: u32, result: *mut *mut IDxcOptimizerPass) -> HRESULT;
        fn run_optimizer(
            blob: *const IDxcBlob,
            options: *const LPCWSTR,
            option_count: u32,
            output_module: *mut *mut IDxcBlob,
            output_text: *mut *mut IDxcBlobEncoding) -> HRESULT;
    }
}

pub const DXC_VERSION_INFO_FLAGS_NONE: u32 = 0;
pub const DXC_VERSION_INFO_FLAGS_DEBUG: u32 = 1; // Matches VS_FF_DEBUG
pub const DXC_VERSION_INFO_FLAGS_INTERNAL: u32 = 2; // Internal Validator (non-signing)

iid!(pub IID_IDxcVersionInfo = 0xb04f_5b50, 0x2059, 0x4f12, 0xa8, 0xff, 0xa1, 0xe0, 0xcd, 0xe1, 0xcc, 0x7e);
com_interface! {
    interface IDxcVersionInfo: IUnknown{
        iid: IID_IDxcVersionInfo,
        vtable: IDxcVersionInfoVtbl,
        fn dummy0() -> HRESULT;
        fn dummy1() -> HRESULT;

        fn get_version(major: *mut u32, minor: *mut u32) -> HRESULT;
        fn get_flags(flags: *mut u32) -> HRESULT;
    }
}

iid!(pub IID_IDxcVersionInfo2 = 0xfb69_04c4, 0x42f0, 0x4b62, 0x9c, 0x46, 0x98, 0x3a, 0xf7, 0xda, 0x7c, 0x83);
com_interface! {
    interface IDxcVersionInfo2: IUnknown{
        iid: IID_IDxcVersionInfo2,
        vtable: IDxcVersionInfo2Vtbl,
        fn dummy0() -> HRESULT;
        fn dummy1() -> HRESULT;

        fn get_commit_info(commit_count: *mut u32, commit_hash: *mut *mut u8) -> HRESULT;
    }
}

iid!(pub CLSID_DxcCompiler = 0x73e22d93, 0xe6ce, 0x47f3, 0xb5, 0xbf, 0xf0, 0x66, 0x4f, 0x39, 0xc1, 0xb0);
iid!(pub CLSID_DxcLinker = 0xef6a8087, 0xb0ea, 0x4d56, 0x9e, 0x45, 0xd0, 0x7e, 0x1a, 0x8b, 0x78, 0x6);
iid!(pub CLSID_DxcDiaDataSource = 0xcd1f6b73, 0x2ab0, 0x484d, 0x8e, 0xdc, 0xeb, 0xe7, 0xa4, 0x3c, 0xa0, 0x9f );
iid!(pub CLSID_DxcLibrary = 0x6245d6af, 0x66e0, 0x48fd, 0x80, 0xb4, 0x4d, 0x27, 0x17, 0x96, 0x74, 0x8c);
iid!(pub CLSID_DxcValidator = 0x8ca3e215, 0xf728, 0x4cf3, 0x8c, 0xdd, 0x88, 0xaf, 0x91, 0x75, 0x87, 0xa1 );
iid!(pub CLSID_DxcAssembler = 0xd728db68, 0xf903, 0x4f80, 0x94, 0xcd, 0xdc, 0xcf, 0x76, 0xec, 0x71, 0x51);
iid!(pub CLSID_DxcContainerReflection = 0xb9f54489, 0x55b8, 0x400c, 0xba, 0x3a, 0x16, 0x75, 0xe4, 0x72, 0x8b, 0x91);
iid!(pub CLSID_DxcOptimizer = 0xae2cd79f, 0xcc22, 0x453f, 0x9b, 0x6b, 0xb1, 0x24, 0xe7, 0xa5, 0x20, 0x4c);
iid!(pub CLSID_DxcContainerBuilder = 0x94134294, 0x411f, 0x4574, 0xb4, 0xd0, 0x87, 0x41, 0xe2, 0x52, 0x40, 0xd2 );
