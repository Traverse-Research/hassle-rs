use crate::os::{BSTR, HRESULT, LPCSTR, LPSTR};
pub(crate) use crate::unknown::IDxcUnknownShim;
use bitflags::bitflags;
use tinycom::{com_interface, iid, IUnknown};

bitflags! {
    pub struct DxcGlobalOptions : u32 {
        const NONE = 0x0;
        const THREAD_BACKGROUND_PRIORITY_FOR_INDEXING = 0x1;
        const THREAD_BACKGROUND_PRIORITY_FOR_EDITING = 0x2;
        const THREAD_BACKGROUND_PRIORITY_FOR_ALL
            = DxcGlobalOptions::THREAD_BACKGROUND_PRIORITY_FOR_INDEXING.bits
            | DxcGlobalOptions::THREAD_BACKGROUND_PRIORITY_FOR_EDITING.bits;
    }
}

bitflags! {
    pub struct DxcDiagnosticSeverity : u32 {
        const IGNORED = 0;
        const NOTE = 1;
        const WARNING = 2;
        const ERROR = 3;
        const FATAL = 4;
    }
}

bitflags! {
    pub struct DxcTokenKind : u32 {
        const PUNCTUATION = 0;
        const KEYWORD = 1;
        const IDENTIFIER = 2;
        const LITERAL = 3;
        const COMMENT = 4;
        const UNKNOWN = 5;
        const BUILT_IN_TYPE = 6;
    }
}

bitflags! {
    pub struct DxcTypeKind : u32 {
        const Invalid = 0; // Reprents an invalid type (e.g., where no type is available).
        const Unexposed = 1; // A type whose specific kind is not exposed via this interface.
        // Builtin types
        const Void = 2;
        const Bool = 3;
        const Char_U = 4;
        const UChar = 5;
        const Char16 = 6;
        const Char32 = 7;
        const UShort = 8;
        const UInt = 9;
        const ULong = 10;
        const ULongLong = 11;
        const UInt128 = 12;
        const Char_S = 13;
        const SChar = 14;
        const WChar = 15;
        const Short = 16;
        const Int = 17;
        const Long = 18;
        const LongLong = 19;
        const Int128 = 20;
        const Float = 21;
        const Double = 22;
        const LongDouble = 23;
        const NullPtr = 24;
        const Overload = 25;
        const Dependent = 26;
        const ObjCId = 27;
        const ObjCClass = 28;
        const ObjCSel = 29;
        const FirstBuiltin = DxcTypeKind::Void.bits;
        const LastBuiltin = DxcTypeKind::ObjCSel.bits;

        const Complex = 100;
        const Pointer = 101;
        const BlockPointer = 102;
        const LValueReference = 103;
        const RValueReference = 104;
        const Record = 105;
        const Enum = 106;
        const Typedef = 107;
        const ObjCInterface = 108;
        const ObjCObjectPointer = 109;
        const FunctionNoProto = 110;
        const FunctionProto = 111;
        const ConstantArray = 112;
        const Vector = 113;
        const IncompleteArray = 114;
        const VariableArray = 115;
        const DependentSizedArray = 116;
        const MemberPointer = 117;
    }
}

bitflags! {
    pub struct DxcCursorFormatting : u32 {
        const DEFAULT = 0x0;
        const USE_LANGUAGE_OPTIONS = 0x1;
        const SUPPRESS_SPECIFIERS = 0x2;
        const SUPPRESS_TAG_KEYWORD = 0x4;
        const INCLUDE_NAMESPACE_KEYWORD = 0x8;
    }
}

bitflags! {
    pub struct DxcTranslationUnitFlags : u32 {
        const NONE = 0x0;
        const DETAILED_PREPROCESSING_RECORD = 0x01;
        const INCOMPLETE = 0x02;
        const PRECOMPILED_PREAMBLE = 0x04;
        const CACHE_COMPLETION_RESULTS = 0x08;
        const FOR_SERIALIZATION = 0x10;
        const CXX_CHAINED_PCH = 0x20;
        const SKIP_FUNCTION_BODIES = 0x40;
        const INCLUDE_BRIEF_COMMENTS_IN_CODE_COMPLETION = 0x80;
        const USE_CALLER_THREAD = 0x800;
    }
}

bitflags! {
    pub struct DxcDiagnosticDisplayOptions : u32 {
        const DISPLAY_SOURCE_LOCATION = 0x01;
        const DISPLAY_COLUMN = 0x02;
        const DISPLAY_SOURCE_RANGES = 0x04;
        const DISPLAY_OPTION = 0x08;
        const DISPLAY_CATEGORY_ID = 0x10;
        const DISPLAY_CATEGORY_NAME = 0x20;
        const DISPLAY_SEVERITY = 0x200;
    }
}

bitflags! {
    pub struct DxcCursorKindFlags : u32 {
       const NONE = 0;
       const DECLARATION = 0x1;
       const REFERENCE = 0x2;
       const EXPRESSION = 0x4;
       const STATEMENT = 0x8;
       const ATTRIBUTE = 0x10;
       const INVALID = 0x20;
       const TRANSLATION_UNIT = 0x40;
       const PREPROCESSING = 0x80;
       const UNEXPOSED = 0x100;
    }
}

bitflags! {
    pub struct DxcCursorKind : u32 {
        const UNEXPOSED_DECL = 1;
        const STRUCT_DECL = 2;
        const UNION_DECL = 3;
        const CLASS_DECL = 4;
        const ENUM_DECL = 5;
        const FIELD_DECL = 6;
        const ENUM_CONSTANT_DECL = 7;
        const FUNCTION_DECL = 8;
        const VAR_DECL = 9;
        const PARM_DECL = 10;
        const OBJ_C_INTERFACE_DECL = 11;
        const OBJ_C_CATEGORY_DECL = 12;
        const OBJ_C_PROTOCOL_DECL = 13;
        const OBJ_C_PROPERTY_DECL = 14;
        const OBJ_C_IVAR_DECL = 15;
        const OBJ_C_INSTANCE_METHOD_DECL = 16;
        const OBJ_C_CLASS_METHOD_DECL = 17;
        const OBJ_C_IMPLEMENTATION_DECL = 18;
        const OBJ_C_CATEGORY_IMPL_DECL = 19;
        const TYPEDEF_DECL = 20;
        const CXX_METHOD = 21;
        const NAMESPACE = 22;
        const LINKAGE_SPEC = 23;
        const CONSTRUCTOR = 24;
        const DESTRUCTOR = 25;
        const CONVERSION_FUNCTION = 26;
        const TEMPLATE_TYPE_PARAMETER = 27;
        const NON_TYPE_TEMPLATE_PARAMETER = 28;
        const TEMPLATE_TEMPLATE_PARAMETER = 29;
        const FUNCTION_TEMPLATE = 30;
        const CLASS_TEMPLATE = 31;
        const CLASS_TEMPLATE_PARTIAL_SPECIALIZATION = 32;
        const NAMESPACE_ALIAS = 33;
        const USING_DIRECTIVE = 34;
        const USING_DECLARATION = 35;
        const TYPE_ALIAS_DECL = 36;
        const OBJ_C_SYNTHESIZE_DECL = 37;
        const OBJ_C_DYNAMIC_DECL = 38;
        const CXX_ACCESS_SPECIFIER = 39;

        const FIRST_DECL = DxcCursorKind::UNEXPOSED_DECL.bits;
        const LAST_DECL = DxcCursorKind::CXX_ACCESS_SPECIFIER.bits;

        const FIRST_REF = 40;
        const OBJ_C_SUPER_CLASS_REF = 40;
        const OBJ_C_PROTOCOL_REF = 41;
        const OBJ_C_CLASS_REF = 42;
        const TYPE_REF = 43;
        const CXX_BASE_SPECIFIER = 44;
        const TEMPLATE_REF = 45;
        const NAMESPACE_REF = 46;
        const MEMBER_REF = 47;
        const LABEL_REF = 48;
        const OVERLOADED_DECL_REF = 49;
        const VARIABLE_REF = 50;
        const LAST_REF = DxcCursorKind::VARIABLE_REF.bits;
        const FIRST_INVALID = 70;
        const INVALID_FILE = 70;
        const NO_DECL_FOUND = 71;
        const NOT_IMPLEMENTED = 72;
        const INVALID_CODE = 73;
        const LAST_INVALID = DxcCursorKind::INVALID_CODE.bits;
        const FIRST_EXPR = 100;
        const UNEXPOSED_EXPR = 100;
        const DECL_REF_EXPR = 101;
        const MEMBER_REF_EXPR = 102;
        const CALL_EXPR = 103;
        const OBJ_C_MESSAGE_EXPR = 104;
        const BLOCK_EXPR = 105;
        const INTEGER_LITERAL = 106;
        const FLOATING_LITERAL = 107;
        const IMAGINARY_LITERAL = 108;
        const STRING_LITERAL = 109;
        const CHARACTER_LITERAL = 110;
        const PAREN_EXPR = 111;
        const UNARY_OPERATOR = 112;
        const ARRAY_SUBSCRIPT_EXPR = 113;
        const BINARY_OPERATOR = 114;
        const COMPOUND_ASSIGN_OPERATOR = 115;
        const CONDITIONAL_OPERATOR = 116;
        const C_STYLE_CAST_EXPR = 117;
        const COMPOUND_LITERAL_EXPR = 118;
        const INIT_LIST_EXPR = 119;
        const ADDR_LABEL_EXPR = 120;
        const STMT_EXPR = 121;
        const GENERIC_SELECTION_EXPR = 122;
        const GNU_NULL_EXPR = 123;
        const CXX_STATIC_CAST_EXPR = 124;
        const CXX_DYNAMIC_CAST_EXPR = 125;
        const CXX_REINTERPRET_CAST_EXPR = 126;
        const CXX_CONST_CAST_EXPR = 127;
        const CXX_FUNCTIONAL_CAST_EXPR = 128;
        const CXX_TYPEID_EXPR = 129;
        const CXX_BOOL_LITERAL_EXPR = 130;
        const CXX_NULL_PTR_LITERAL_EXPR = 131;
        const CXX_THIS_EXPR = 132;
        const CXX_THROW_EXPR = 133;
        const CXX_NEW_EXPR = 134;
        const CXX_DELETE_EXPR = 135;
        const UNARY_EXPR = 136;
        const OBJ_C_STRING_LITERAL = 137;
        const OBJ_C_ENCODE_EXPR = 138;
        const OBJ_C_SELECTOR_EXPR = 139;
        const OBJ_C_PROTOCOL_EXPR = 140;
        const OBJ_C_BRIDGED_CAST_EXPR = 141;
        const PACK_EXPANSION_EXPR = 142;
        const SIZE_OF_PACK_EXPR = 143;
        const LAMBDA_EXPR = 144;
        const OBJ_C_BOOL_LITERAL_EXPR = 145;
        const OBJ_C_SELF_EXPR = 146;
        const LAST_EXPR = DxcCursorKind::OBJ_C_SELF_EXPR.bits;
        const FIRST_STMT = 200;
        const UNEXPOSED_STMT = 200;
        const LABEL_STMT = 201;
        const COMPOUND_STMT = 202;
        const CASE_STMT = 203;
        const DEFAULT_STMT = 204;
        const IF_STMT = 205;
        const SWITCH_STMT = 206;
        const WHILE_STMT = 207;
        const DO_STMT = 208;
        const FOR_STMT = 209;
        const GOTO_STMT = 210;
        const INDIRECT_GOTO_STMT = 211;
        const CONTINUE_STMT = 212;
        const BREAK_STMT = 213;
        const RETURN_STMT = 214;
        const GCC_ASM_STMT = 215;
        const ASM_STMT = DxcCursorKind::GCC_ASM_STMT.bits;

        const OBJ_C_AT_TRY_STMT = 216;
        const OBJ_C_AT_CATCH_STMT = 217;
        const OBJ_C_AT_FINALLY_STMT = 218;
        const OBJ_C_AT_THROW_STMT = 219;
        const OBJ_C_AT_SYNCHRONIZED_STMT = 220;
        const OBJ_C_AUTORELEASE_POOL_STMT = 221;
        const OBJ_C_FOR_COLLECTION_STMT = 222;
        const CXX_CATCH_STMT = 223;
        const CXX_TRY_STMT = 224;
        const CXX_FOR_RANGE_STMT = 225;
        const SEH_TRY_STMT = 226;
        const SEH_EXCEPT_STMT = 227;
        const SEH_FINALLY_STMT = 228;
        const MS_ASM_STMT = 229;
        const NULL_STMT = 230;
        const DECL_STMT = 231;
        const OMP_PARALLEL_DIRECTIVE = 232;
        const OMP_SIMD_DIRECTIVE = 233;
        const OMP_FOR_DIRECTIVE = 234;
        const OMP_SECTIONS_DIRECTIVE = 235;
        const OMP_SECTION_DIRECTIVE = 236;
        const OMP_SINGLE_DIRECTIVE = 237;
        const OMP_PARALLEL_FOR_DIRECTIVE = 238;
        const OMP_PARALLEL_SECTIONS_DIRECTIVE = 239;
        const OMP_TASK_DIRECTIVE = 240;
        const OMP_MASTER_DIRECTIVE = 241;
        const OMP_CRITICAL_DIRECTIVE = 242;
        const OMP_TASKYIELD_DIRECTIVE = 243;
        const OMP_BARRIER_DIRECTIVE = 244;
        const OMP_TASKWAIT_DIRECTIVE = 245;
        const OMP_FLUSH_DIRECTIVE = 246;
        const SEH_LEAVE_STMT = 247;
        const OMP_ORDERED_DIRECTIVE = 248;
        const OMP_ATOMIC_DIRECTIVE = 249;
        const OMP_FOR_SIMD_DIRECTIVE = 250;
        const OMP_PARALLEL_FOR_SIMD_DIRECTIVE = 251;
        const OMP_TARGET_DIRECTIVE = 252;
        const OMP_TEAMS_DIRECTIVE = 253;
        const OMP_TASKGROUP_DIRECTIVE = 254;
        const OMP_CANCELLATION_POINT_DIRECTIVE = 255;
        const OMP_CANCEL_DIRECTIVE = 256;
        const LAST_STMT = DxcCursorKind::OMP_CANCEL_DIRECTIVE.bits;

        const TRANSLATION_UNIT = 300;

        const FIRST_ATTR = 400;
        const UNEXPOSED_ATTR = 400;

        const IB_ACTION_ATTR = 401;
        const IB_OUTLET_ATTR = 402;
        const IB_OUTLET_COLLECTION_ATTR = 403;
        const CXX_FINAL_ATTR = 404;
        const CXX_OVERRIDE_ATTR = 405;
        const ANNOTATE_ATTR = 406;
        const ASM_LABEL_ATTR = 407;
        const PACKED_ATTR = 408;
        const PURE_ATTR = 409;
        const CONST_ATTR = 410;
        const NO_DUPLICATE_ATTR = 411;
        const CUDA_CONSTANT_ATTR = 412;
        const CUDA_DEVICE_ATTR = 413;
        const CUDA_GLOBAL_ATTR = 414;
        const CUDA_HOST_ATTR = 415;
        const CUDA_SHARED_ATTR = 416;
        const LAST_ATTR = DxcCursorKind::CUDA_SHARED_ATTR.bits;

        const PREPROCESSING_DIRECTIVE = 500;
        const MACRO_DEFINITION = 501;
        const MACRO_EXPANSION = 502;
        const MACRO_INSTANTIATION = DxcCursorKind::MACRO_EXPANSION.bits;
        const INCLUSION_DIRECTIVE = 503;
        const FIRST_PREPROCESSING = DxcCursorKind::PREPROCESSING_DIRECTIVE.bits;
        const LAST_PREPROCESSING = DxcCursorKind::INCLUSION_DIRECTIVE.bits;

        const MODULE_IMPORT_DECL = 600;
        const FIRST_EXTRA_DECL = DxcCursorKind::MODULE_IMPORT_DECL.bits;
        const LAST_EXTRA_DECL = DxcCursorKind::MODULE_IMPORT_DECL.bits;
    }
}

iid!(pub IID_IDxcDiagnostic = 0x4f76b234, 0x3659, 0x4d33, 0x99, 0xb0, 0x3b, 0x0d, 0xb9, 0x94, 0xb5, 0x64);
com_interface! {
    interface IDxcDiagnostic: IDxcUnknownShim, IUnknown {
        iid: IID_IDxcDiagnostic,
        vtable: IDxcDiagnosticVtbl,
        fn format_diagnostic(options: DxcDiagnosticDisplayOptions, result: *mut LPSTR) -> HRESULT;
        fn get_severity(result: *mut DxcDiagnosticSeverity) -> HRESULT;
        fn get_location(result: *mut *mut IDxcSourceLocation) -> HRESULT;
        fn get_spelling(result: *mut LPSTR) -> HRESULT;
        fn get_category_text(result: *mut LPSTR) -> HRESULT;
        fn get_num_ranges(result: *mut u32) -> HRESULT;
        fn get_range_at(index: u32, result: *mut *mut IDxcSourceRange) -> HRESULT;
        fn get_num_fix_its(result: *mut u32) -> HRESULT;
        fn get_fix_it_at(index: u32, replacement_range: *mut *mut IDxcSourceRange, text: *mut LPSTR) -> HRESULT;
    }
}

iid!(pub IID_IDxcInclusion = 0x0c364d65, 0xdf44, 0x4412, 0x88, 0x8e, 0x4e, 0x55, 0x2f, 0xc5, 0xe3, 0xd6);
com_interface! {
    interface IDxcInclusion: IDxcUnknownShim, IUnknown {
        iid: IID_IDxcInclusion,
        vtable: IDxcInclusionVtbl,
        fn get_included_file(result: *mut *mut IDxcFile) -> HRESULT;
        fn get_stack_length(result: *mut u32) -> HRESULT;
        fn get_stack_item(index: u32, result: *mut *mut IDxcSourceLocation) -> HRESULT;
    }
}

iid!(pub IID_IDxcToken = 0x7f90b9ff, 0xa275, 0x4932, 0x97, 0xd8, 0x3c, 0xfd, 0x23, 0x44, 0x82, 0xa2);
com_interface! {
    interface IDxcToken: IDxcUnknownShim, IUnknown {
        iid: IID_IDxcToken,
        vtable: IDxcTokenVtbl,
        fn get_kind(value: *mut DxcTokenKind) -> HRESULT;
        fn get_location(value: *mut *mut IDxcSourceLocation) -> HRESULT;
        fn get_extent(value: *mut *mut IDxcSourceRange) -> HRESULT;
        fn get_spelling(value: *mut LPSTR) -> HRESULT;
    }
}

iid!(pub IID_IDxcType = 0x2ec912fd, 0xb144, 0x4a15, 0xad, 0x0d, 0x1c, 0x54, 0x39, 0xc8, 0x1e, 0x46);
com_interface! {
    interface IDxcType: IDxcUnknownShim, IUnknown {
        iid: IID_IDxcType,
        vtable: IDxcTypeVtbl,
        fn get_spelling(result: *mut LPSTR) -> HRESULT;
        fn is_equal_to(other: *const IDxcType, result: *mut bool) -> HRESULT;
        fn get_kind(result: *mut DxcTypeKind) -> HRESULT;
    }
}

iid!(pub IID_IDxcSourceLocation = 0x8e7ddf1c, 0xd7d3, 0x4d69, 0xb2, 0x86, 0x85, 0xfc, 0xcb, 0xa1, 0xe0, 0xcf);
com_interface! {
    interface IDxcSourceLocation: IDxcUnknownShim, IUnknown {
        iid: IID_IDxcSourceLocation,
        vtable: IDxcSourceLocationVtbl,
        fn is_equal_to(other: *const IDxcSourceLocation, result: *mut bool) -> HRESULT;
        fn get_spelling_location(file: *mut *mut IDxcFile, line: *mut u32, col: *mut u32, offset: *mut u32) -> HRESULT;
        fn is_null(result: *mut bool) -> HRESULT;
    }
}

iid!(pub IID_IDxcSourceRange = 0xf1359b36, 0xa53f, 0x4e81, 0xb5, 0x14, 0xb6, 0xb8, 0x41, 0x22, 0xa1, 0x3f);
com_interface! {
    interface IDxcSourceRange: IDxcUnknownShim, IUnknown {
        iid: IID_IDxcSourceRange,
        vtable: IDxcSourceRangeVtbl,
        fn is_null(value: *mut bool) -> HRESULT;
        fn get_start(value: *mut *mut IDxcSourceLocation) -> HRESULT;
        fn get_end(value: *mut *mut IDxcSourceLocation) -> HRESULT;
        fn get_offsets(start_offset: *mut u32, end_offset: *mut u32) -> HRESULT;
    }
}

iid!(pub IID_IDxcCursor = 0x1467b985, 0x288d, 0x4d2a, 0x80, 0xc1, 0xef, 0x89, 0xc4, 0x2c, 0x40, 0xbc);
com_interface! {
    interface IDxcCursor: IDxcUnknownShim, IUnknown {
        iid: IID_IDxcCursor,
        vtable: IDxcCursorVtbl,
        fn get_extent(range: *mut *mut IDxcSourceRange) -> HRESULT;
        fn get_location(result: *mut *mut IDxcSourceLocation) -> HRESULT;
        fn get_kind(result: *mut DxcCursorKind) -> HRESULT;
        fn get_kind_flags(result: *mut DxcCursorKindFlags) -> HRESULT;
        fn get_semantic_parent(result: *mut*mut IDxcCursor) -> HRESULT;
        fn get_lexical_parent(result:*mut*mut IDxcCursor) -> HRESULT;
        fn get_cursor_type(result:*mut*mut IDxcType) -> HRESULT;
        fn get_num_arguments(result:*mut i32) -> HRESULT;
        fn get_argument_at(index: i32, result: *mut *mut IDxcCursor) -> HRESULT;
        fn get_referenced_cursor(result:*mut *mut IDxcCursor) -> HRESULT;
        fn get_definition_cursor(result:*mut *mut IDxcCursor) -> HRESULT;
        fn find_references_in_file(file: *const IDxcFile, skip: u32, top:u32, result_length: *mut u32, result: *mut *mut *mut IDxcCursor) -> HRESULT;
        fn get_spelling(result: *mut LPSTR) -> HRESULT;
        fn is_equal_to(other: *const IDxcCursor, result:*mut bool) -> HRESULT;
        fn is_null(result:*mut bool) -> HRESULT;
        fn is_definition(result:*mut bool) -> HRESULT;
        fn get_display_name(result:*mut BSTR) -> HRESULT;
        fn get_qualified_name(include_template_args:bool, result:*mut BSTR) -> HRESULT;
        fn get_formatted_name(formatting: DxcCursorFormatting, result:*mut BSTR) -> HRESULT;
        fn get_children(skip: u32, top: u32, result_length:*mut u32, result:*mut*mut*mut IDxcCursor) -> HRESULT;
        fn get_snapped_child(location:  *const IDxcSourceLocation, result:*mut*mut IDxcCursor) -> HRESULT;
    }
}

iid!(pub IID_IDxcUnsavedFile = 0x8ec00f98, 0x07d0, 0x4e60, 0x9d, 0x7c, 0x5a, 0x50, 0xb5, 0xb0, 0x01, 0x7f);
com_interface! {
    interface IDxcUnsavedFile: IDxcUnknownShim, IUnknown {
        iid: IID_IDxcUnsavedFile,
        vtable: IDxcUnsavedFileVtbl,
        fn get_file_name(file_name: *mut LPSTR) -> HRESULT;
        fn get_contents(contents: *mut LPSTR) -> HRESULT;
        fn get_length(lenth : *mut u32) -> HRESULT;
    }
}

iid!(pub IID_IDxcFile = 0xbb2fca9e, 0x1478, 0x47ba, 0xb0, 0x8c, 0x2c, 0x50, 0x2a, 0xda, 0x48, 0x95);
com_interface! {
    interface IDxcFile: IDxcUnknownShim, IUnknown {
        iid: IID_IDxcFile,
        vtable: IDxcFileVtbl,
        fn get_name(result: *mut LPSTR) -> HRESULT;
        fn is_equal_to(other : *const IDxcFile, result: *mut bool) -> HRESULT;
    }
}

iid!(pub IID_IDxcTranslationUnit = 0x9677dee0, 0xc0e5, 0x46a1, 0x8b, 0x40, 0x3d, 0xb3, 0x16, 0x8b, 0xe6, 0x3d);
com_interface! {
    interface IDxcTranslationUnit: IDxcUnknownShim, IUnknown {
        iid: IID_IDxcTranslationUnit,
        vtable: IDxcTranslationUnitVtbl,
        fn get_cursor(cursor: *mut *mut IDxcCursor) -> HRESULT;
        fn tokenize(range: *const IDxcSourceRange, tokens: *mut *mut *mut IDxcToken, token_count: *mut u32) -> HRESULT;
        fn get_location( file: *mut IDxcFile, line: u32, column: u32, result: *mut *mut IDxcSourceLocation) -> HRESULT;
        fn get_num_diagnostics(value : *mut u32) -> HRESULT;
        fn get_diagnostic(index: u32, value: *mut *mut IDxcDiagnostic) -> HRESULT;
        fn get_file(name : *const u8, result : *mut *mut IDxcFile) -> HRESULT;
        fn get_file_name(result : *mut LPSTR) -> HRESULT;
        fn reparse(unsaved_files : *mut *mut IDxcUnsavedFile, num_unsaved_files: u32) -> HRESULT;
        fn get_cursor_for_location(location: *const IDxcSourceLocation, result : *mut *mut IDxcCursor) -> HRESULT;
        fn get_location_for_offset(file : *const IDxcFile, offset: u32, result: *mut *mut IDxcSourceLocation) -> HRESULT;
        fn get_skipped_ranges(file: *const IDxcFile, result_count: *mut u32, result: *mut *mut *mut IDxcSourceRange) -> HRESULT;
        fn get_diagnostic_details(
            index: u32,  options: DxcDiagnosticDisplayOptions, error_code: *mut u32, error_line: *mut u32, error_column: *mut u32,
            error_file: *mut BSTR, error_offset: *mut u32, error_length: *mut u32, error_message: *mut BSTR) -> HRESULT;
        fn get_inclusion_list(result_count: *mut u32, result: *mut *mut *mut IDxcInclusion) -> HRESULT;
    }
}

iid!(pub IID_IDxcIndex = 0x937824a0, 0x7f5a, 0x4815, 0x9b, 0xa, 0x7c, 0xc0, 0x42, 0x4f, 0x41, 0x73);
com_interface! {
    interface IDxcIndex: IDxcUnknownShim, IUnknown {
        iid: IID_IDxcIndex,
        vtable: IDxcIndexVtbl,
        fn set_global_options(options: DxcGlobalOptions) -> HRESULT;
        fn get_global_options(options: *mut DxcGlobalOptions) -> HRESULT;
        fn parse_translation_unit(
            source_filename: *const u8,
            command_line_args: *const *const u8,
            num_command_line_args: i32,
            unsaved_files: *const *const IDxcUnsavedFile,
            num_unsaved_files: u32,
            options: DxcTranslationUnitFlags,
            translation_unit: *mut *mut IDxcTranslationUnit) -> HRESULT;
    }
}

iid!(pub IID_IDxcIntelliSense = 0xb1f99513, 0x46d6, 0x4112, 0x81, 0x69, 0xdd, 0x0d, 0x60, 0x53, 0xf1, 0x7d);
com_interface! {
    interface IDxcIntelliSense: IDxcUnknownShim, IUnknown {
        iid: IID_IDxcIntelliSense,
        vtable: IDxcIntelliSenseVtbl,
        fn create_index(index: *mut *mut IDxcIndex) -> HRESULT;
        fn get_null_location(location: *mut *mut  IDxcSourceLocation)  -> HRESULT;
        fn get_null_range(location: *mut *mut  IDxcSourceRange)  -> HRESULT;
        fn get_range( start: *const IDxcSourceLocation, end: *const IDxcSourceLocation, location: *mut *mut IDxcSourceRange)  -> HRESULT;
        fn get_default_diagnostic_display_options(value: *mut DxcDiagnosticDisplayOptions)  -> HRESULT;
        fn get_default_editing_tu_options(value: *mut DxcTranslationUnitFlags)  -> HRESULT;
        fn create_unsaved_file(file_name: LPCSTR, contents: LPCSTR, content_length: u32 , result: *mut *mut IDxcUnsavedFile)  -> HRESULT;
    }
}

iid!(pub CLSID_DxcIntelliSense = 0x3047833c, 0xd1c0, 0x4b8e, 0x9d, 0x40, 0x10, 0x28, 0x78, 0x60, 0x59, 0x85);
