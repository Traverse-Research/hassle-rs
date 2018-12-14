use com_rs::{com_interface, iid, IUnknown};
use std::ffi::c_void;
use winapi::shared::ntdef::LPCWSTR;

#[repr(C)]
#[derive(Debug)]
pub struct DxcDefine {
    pub name: LPCWSTR,
    pub value: LPCWSTR,
}

iid!(pub IID_IDxcBlob = 0x8BA5FB08, 0x5195, 0x40e2, 0xAC, 0x58, 0x0D, 0x98, 0x9C, 0x3A, 0x01, 0x02);
com_interface! {
    interface IDxcBlob: IUnknown{
        iid: IID_IDxcBlob,
        vtable: IDxcBlobVtbl,
        fn get_buffer_pointer() -> *const c_void;
        fn get_buffer_size() -> usize;
    }
}

iid!(pub IID_IDxcBlobEncoding = 0x7241d424, 0x2646, 0x4191, 0x97, 0xc0, 0x98, 0xe9, 0x6e, 0x42, 0xfc, 0x68);
com_interface! {
    interface IDxcBlobEncoding: IDxcBlob, IUnknown{
        iid: IID_IDxcBlobEncoding,
        vtable: IDxcBlobEncodingVtbl,
        fn get_encoding(known: *mut u32, code_page: *mut u32) -> u32;
    }
}

iid!(pub IID_IDxcOperationResult = 0xCEDB484A, 0xD4E9, 0x445A, 0xB9, 0x91, 0xCA, 0x21, 0xCA, 0x15, 0x7D, 0xC2);
com_interface! {
    interface IDxcOperationResult: IUnknown{
        iid: IID_IDxcOperationResult,
        vtable: IDxcOperationResultVtbl,
        fn get_status(status: *mut u32) -> u32;
        fn get_result(result: *mut *mut IDxcBlob) -> u32;
        fn get_error_buffer(errors: *mut *mut IDxcBlobEncoding) -> u32;
    }
}

iid!(pub IID_IDxcCompiler = 0x8c210bf3, 0x011f, 0x4422, 0x8d, 0x70, 0x6f, 0x9a, 0xcb, 0x8d, 0xb6, 0x17);
com_interface! {
    interface IDxcCompiler: IUnknown{
        iid: IID_IDxcCompiler,
        vtable: IDxcCompilerVtbl,
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
            result: *mut *mut IDxcOperationResult) -> u32;

        fn preprocess(
            blob: *const IDxcBlob,
            source_name: LPCWSTR,
            arguments: *const LPCWSTR,
            arg_count: u32,
            defines: *const DxcDefine,
            def_count: u32,
            include_handler: *const c_void,
            result: *mut *mut IDxcOperationResult) -> u32;

        fn disassemble(
            blob: *const IDxcBlob,
            disassembly: *mut *mut IDxcBlobEncoding) -> u32;
    }
}

iid!(pub IID_IDxcLibrary = 0xe5204dc7, 0xd18c, 0x4c3c, 0xbd, 0xfb, 0x85, 0x16, 0x73, 0x98, 0x0f, 0xe7);
com_interface! {
    interface IDxcLibrary: IUnknown{
        iid: IID_IDxcLibrary,
        vtable: IDxcLibraryVtbl,
        fn set_malloc(malloc: *const c_void) -> u32;
        fn create_blob_from_blob(blob: *const IDxcBlob, offset: u32, length: u32, result_blob: *mut *mut IDxcBlob) -> u32;
        fn create_blob_from_file(filename: LPCWSTR, code_page: *const u32, blob_encoding: *mut *mut IDxcBlobEncoding) -> u32;
        fn create_blob_with_encoding_from_pinned(text: *const c_void, size: u32, code_page: u32, blob_encoding: *mut *mut IDxcBlobEncoding) -> u32;
        fn create_blob_with_encoding_on_heap_copy(text: *const c_void, size: u32, code_page: u32, blob_encoding: *mut *mut IDxcBlobEncoding) -> u32;
        fn create_blob_with_encoding_on_malloc(text: *const c_void, malloc: *const c_void, size: u32, code_page: u32, blob_encoding: *mut *mut IDxcBlobEncoding) -> u32;
        fn create_include_handler(include_handler: *const c_void) -> u32;
        fn create_stream_from_blob_read_only(blob: *const IDxcBlob, stream: *mut *mut c_void) -> u32;
        fn get_blob_as_utf8(blob: *const IDxcBlob, blob_encoding: *mut *mut IDxcBlobEncoding) -> u32;
        fn get_blob_as_utf16(blob: *const IDxcBlob, blob_encoding: *mut *mut IDxcBlobEncoding) -> u32;
    }
}

iid!(pub CLSID_DxcLibrary = 0x6245d6af, 0x66e0, 0x48fd, 0x80, 0xb4, 0x4d, 0x27, 0x17, 0x96, 0x74, 0x8c);
iid!(pub CLSDI_DxcCompiler = 0x73e22d93, 0xe6ce, 0x47f3, 0xb5, 0xbf, 0xf0, 0x66, 0x4f, 0x39, 0xc1, 0xb0);
