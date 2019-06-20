use com_rs::{com_interface, iid, IUnknown};
use winapi::shared::ntdef::{LPCSTR, LPSTR};
use winapi::shared::winerror::HRESULT;
use winapi::shared::wtypes::BSTR;

bitflags! {
    pub struct DxcGlobalOptions : u32
    {
        const NONE = 0x0;
        const ThreadBackgroundPriorityForIndexing = 0x1;
        const ThreadBackgroundPriorityForEditing = 0x2;
        const ThreadBackgroundPriorityForAll
            = DxcGlobalOptions::ThreadBackgroundPriorityForIndexing.bits
            | DxcGlobalOptions::ThreadBackgroundPriorityForEditing.bits;
    }
}

// Describes the severity of a particular diagnostic.
bitflags! {
    pub struct  DxcDiagnosticSeverity : u32
    {
        // A diagnostic that has been suppressed, e.g., by a command-line option.
        const Ignored = 0;

        // This diagnostic is a note that should be attached to the previous (non-note) diagnostic.
        const Note = 1;

        // This diagnostic indicates suspicious code that may not be wrong.
        const Warning = 2;

        // This diagnostic indicates that the code is ill-formed.
        const Error = 3;

        // This diagnostic indicates that the code is ill-formed such that future
        // parser rec unlikely to produce useful results.
        const Fatal = 4;
    }
}

bitflags! {
    pub struct  DxcTokenKind: u32
    {
        const Punctuation = 0; // A token that contains some kind of punctuation.
        const Keyword = 1;     // A language keyword.
        const Identifier = 2;  // An identifier (that is not a keyword).
        const Literal = 3;     // A numeric, string, or character literal.
        const Comment = 4;     // A comment.
        const Unknown = 5;     // An unknown token (possibly known to a future version).
        const BuiltInType = 6; // A built-in type like int, void or float3.
    }
}

bitflags! {
    pub struct   DxcCursorFormatting : u32
    {
        const _Default = 0x0;             // Default rules, language-insensitive formatting.
        const UseLanguageOptions = 0x1;  // Language-sensitive formatting.
        const SuppressSpecifiers = 0x2;  // Supresses type specifiers.
        const SuppressTagKeyword = 0x4;  // Suppressed tag keyword (eg, 'class').
        const IncludeNamespaceKeyword = 0x8;  // Include namespace keyword.
    }
}

bitflags! {
    pub struct  DxcTranslationUnitFlags : u32
    {
        // Used to indicate that no special translation-unit options are needed.
     const   None = 0x0;

        // Used to indicate that the parser should construct a "detailed"
        // preprocessing record, including all macro definitions and instantiations.
      const    DetailedPreprocessingRecord = 0x01;

        // Used to indicate that the translation unit is incomplete.
     const     Incomplete = 0x02;

        // Used to indicate that the translation unit should be built with an
        // implicit precompiled header for the preamble.
     const     PrecompiledPreamble = 0x04;

        // Used to indicate that the translation unit should cache some
        // code-completion results with each reparse of the source file.
     const     CacheCompletionResults = 0x08;

        // Used to indicate that the translation unit will be serialized with
        // SaveTranslationUnit.
      const    ForSerialization = 0x10;

        // DEPRECATED
      const    CXXChainedPCH = 0x20;

        // Used to indicate that function/method bodies should be skipped while parsing.
      const    SkipFunctionBodies = 0x40;

        // Used to indicate that brief documentation comments should be
        // included into the set of code completions returned from this translation
        // unit.
      const    IncludeBriefCommentsInCodeCompletion = 0x80;

        // Used to indicate that compilation should occur on the caller's thread.
       const   UseCallerThread = 0x800;
    }
}

// Options to control the display of diagnostics.
bitflags! {
    pub struct  DxcDiagnosticDisplayOptions : u32
    {
        // Display the source-location information where the diagnostic was located.
       const DisplaySourceLocation = 0x01;

        // If displaying the source-location information of the diagnostic,
        // also include the column number.
      const   DisplayColumn = 0x02;

        // If displaying the source-location information of the diagnostic,
        // also include information about source ranges in a machine-parsable format.
      const   DisplaySourceRanges = 0x04;

        // Display the option name associated with this diagnostic, if any.
       const  DisplayOption = 0x08;

        // Display the category number associated with this diagnostic, if any.
       const  DisplayCategoryId = 0x10;

        // Display the category name associated with this diagnostic, if any.
       const  DisplayCategoryName = 0x20;

        // Display the severity of the diagnostic message.
       const  DisplaySeverity = 0x200;
    }
}

bitflags! {
    pub struct DxcCursorKindFlags: u32
    {
       const None = 0;
       const Declaration = 0x1;
       const Reference = 0x2;
       const Expression = 0x4;
       const Statement = 0x8;
       const Attribute = 0x10;
       const Invalid = 0x20;
       const TranslationUnit = 0x40;
       const Preprocessing = 0x80;
       const Unexposed = 0x100;
    }
}

bitflags! {
    pub struct DxcCursorKind: u32 {
        /* Declarations */
        const UnexposedDecl = 1; // A declaration whose specific kind is not exposed via this interface.
        const StructDecl = 2; // A C or C++ struct.
        const UnionDecl = 3; // A C or C++ union.
        const ClassDecl = 4; // A C++ class.
        const EnumDecl = 5; // An enumeration.
        const FieldDecl = 6; // A field (in C) or non-static data member (in C++) in a struct; union, or C++ class.
        const EnumConstantDecl = 7; // An enumerator constant.
        const FunctionDecl = 8; // A function.
        const VarDecl = 9; // A variable.
        const ParmDecl = 10; // A function or method parameter.
        const ObjCInterfaceDecl = 11; // An Objective-C interface.
        const ObjCCategoryDecl = 12; // An Objective-C interface for a category.
        const ObjCProtocolDecl = 13; // An Objective-C protocol declaration.
        const ObjCPropertyDecl = 14; // An Objective-C property declaration.
        const ObjCIvarDecl = 15; // An Objective-C instance variable.
        const ObjCInstanceMethodDecl = 16; // An Objective-C instance method.
        const ObjCClassMethodDecl = 17; // An Objective-C class method.
        const ObjCImplementationDecl = 18; // An Objective-C \@implementation.
        const ObjCCategoryImplDecl = 19; // An Objective-C \@implementation for a category.
        const TypedefDecl = 20; // A typedef
        const CXXMethod = 21; // A C++ class method.
        const Namespace = 22; // A C++ namespace.
        const LinkageSpec = 23; // A linkage specification, e.g. 'extern "C"'.
        const Constructor = 24; // A C++ constructor.
        const Destructor = 25; // A C++ destructor.
        const ConversionFunction = 26; // A C++ conversion function.
        const TemplateTypeParameter = 27; // A C++ template type parameter.
        const NonTypeTemplateParameter = 28; // A C++ non-type template parameter.
        const TemplateTemplateParameter = 29; // A C++ template template parameter.
        const FunctionTemplate = 30; // A C++ function template.
        const ClassTemplate = 31; // A C++ class template.
        const ClassTemplatePartialSpecialization = 32; // A C++ class template partial specialization.
        const NamespaceAlias = 33; // A C++ namespace alias declaration.
        const UsingDirective = 34; // A C++ using directive.
        const UsingDeclaration = 35; // A C++ using declaration.
        const TypeAliasDecl = 36; // A C++ alias declaration
        const ObjCSynthesizeDecl = 37; // An Objective-C \@synthesize definition.
        const ObjCDynamicDecl = 38; // An Objective-C \@dynamic definition.
        const CXXAccessSpecifier = 39; // An access specifier.

        const FirstDecl = DxcCursorKind::UnexposedDecl.bits;
        const LastDecl = DxcCursorKind::CXXAccessSpecifier.bits;

        /* References */
        const FirstRef = 40; /* Decl references */
        const ObjCSuperClassRef = 40;
        const ObjCProtocolRef = 41;
        const ObjCClassRef = 42;
        /**
         * \brief A reference to a type declaration.
        *
        * A type reference occurs anywhere where a type is named but not
        * declared. For example, given:
        *
        * \code
        * typedef unsigned size_type;
        * size_type size;
        * \endcode
        *
        * The typedef is a declaration of size_type (TypedefDecl),
        * while the type of the variable "size" is referenced. The cursor
        * referenced by the type of size is the typedef for size_type.
        */
        const TypeRef = 43; // A reference to a type declaration.
        const CXXBaseSpecifier = 44;
        const TemplateRef = 45; // A reference to a class template, function template, template template parameter, or class template partial specialization.
        const NamespaceRef = 46; // A reference to a namespace or namespace alias.
        const MemberRef = 47; // A reference to a member of a struct, union, or class that occurs in some non-expression context, e.g., a designated initializer.
        /**
         * \brief A reference to a labeled statement.
        *
        * This cursor kind is used to describe the jump to "start_over" in the
        * goto statement in the following example:
        *
        * \code
        *   start_over:
        *     ++counter;
        *
        *     goto start_over;
        * \endcode
        *
        * A label reference cursor refers to a label statement.
        */
        const LabelRef = 48; // A reference to a labeled statement.

        // A reference to a set of overloaded functions or function templates
        // that has not yet been resolved to a specific function or function template.
        //
        // An overloaded declaration reference cursor occurs in C++ templates where
        // a dependent name refers to a function.
        const OverloadedDeclRef = 49;
        const VariableRef = 50; // A reference to a variable that occurs in some non-expression context, e.g., a C++ lambda capture list.

        const LastRef = DxcCursorKind::VariableRef.bits;

        /* Error conditions */
        const FirstInvalid = 70;
        const InvalidFile = 70;
        const NoDeclFound = 71;
        const NotImplemented = 72;
        const InvalidCode = 73;
        const LastInvalid = DxcCursorKind::InvalidCode.bits;

        /* Expressions */
        const FirstExpr = 100;

        /**
         * \brief An expression whose specific kind is not exposed via this
        * interface.
        *
        * Unexposed expressions have the same operations as any other kind
        * of expression; one can extract their location information,
        * spelling, children, etc. However, the specific kind of the
        * expression is not reported.
        */
        const UnexposedExpr = 100; // An expression whose specific kind is not exposed via this interface.
        const DeclRefExpr = 101; // An expression that refers to some value declaration, such as a function, varible, or enumerator.
        const MemberRefExpr = 102; // An expression that refers to a member of a struct, union, class, Objective-C class, etc.
        const CallExpr = 103; // An expression that calls a function.
        const ObjCMessageExpr = 104; // An expression that sends a message to an Objective-C object or class.
        const BlockExpr = 105; // An expression that represents a block literal.
        const IntegerLiteral = 106; // An integer literal.
        const FloatingLiteral = 107; // A floating point number literal.
        const ImaginaryLiteral = 108; // An imaginary number literal.
        const StringLiteral = 109; // A string literal.
        const CharacterLiteral = 110; // A character literal.
        const ParenExpr = 111; // A parenthesized expression, e.g. "(1)". This AST node is only formed if full location information is requested.
        const UnaryOperator = 112; // This represents the unary-expression's (except sizeof and alignof).
        const ArraySubscriptExpr = 113; // [C99 6.5.2.1] Array Subscripting.
        const BinaryOperator = 114; // A builtin binary operation expression such as "x + y" or "x <= y".
        const CompoundAssignOperator = 115; // Compound assignment such as "+=".
        const ConditionalOperator = 116; // The ?: ternary operator.
        const CStyleCastExpr = 117; // An explicit cast in C (C99 6.5.4) or a C-style cast in C++ (C++ [expr.cast]), which uses the syntax (Type)expr, eg: (int)f.
        const CompoundLiteralExpr = 118; // [C99 6.5.2.5]
        const InitListExpr = 119; // Describes an C or C++ initializer list.
        const AddrLabelExpr = 120; // The GNU address of label extension, representing &&label.
        const StmtExpr = 121; // This is the GNU Statement Expression extension: ({int X=4; X;})
        const GenericSelectionExpr = 122; // Represents a C11 generic selection.

        /** \brief Implements the GNU __null extension, which is a name for a null
         * pointer constant that has integral type (e.g., int or long) and is the same
        * size and alignment as a pointer.
        *
        * The __null extension is typically only used by system headers, which define
        * NULL as __null in C++ rather than using 0 (which is an integer that may not
        * match the size of a pointer).
        */
        const GNUNullExpr = 123;
        const CXXStaticCastExpr = 124; // C++'s static_cast<> expression.
        const CXXDynamicCastExpr = 125; // C++'s dynamic_cast<> expression.
        const CXXReinterpretCastExpr = 126; // C++'s reinterpret_cast<> expression.
        const CXXConstCastExpr = 127; // C++'s const_cast<> expression.

        /** \brief Represents an explicit C++ type conversion that uses "functional"
         * notion (C++ [expr.type.conv]).
        *
        * Example:
        * \code
        *   x = int(0.5);
        * \endcode
        */
        const CXXFunctionalCastExpr = 128;
        const CXXTypeidExpr = 129; // A C++ typeid expression (C++ [expr.typeid]).
        const CXXBoolLiteralExpr = 130; // [C++ 2.13.5] C++ Boolean Literal.
        const CXXNullPtrLiteralExpr = 131; // [C++0x 2.14.7] C++ Pointer Literal.
        const CXXThisExpr = 132; // Represents the "this" expression in C++
        const CXXThrowExpr = 133; // [C++ 15] C++ Throw Expression, both 'throw' and 'throw' assignment-expression.
        const CXXNewExpr = 134; // A new expression for memory allocation and constructor calls, e.g: "new CXXNewExpr(foo)".
        const CXXDeleteExpr = 135; // A delete expression for memory deallocation and destructor calls, e.g. "delete[] pArray".
        const UnaryExpr = 136; // A unary expression.
        const ObjCStringLiteral = 137; // An Objective-C string literal i.e. @"foo".
        const ObjCEncodeExpr = 138; // An Objective-C \@encode expression.
        const ObjCSelectorExpr = 139; // An Objective-C \@selector expression.
        const ObjCProtocolExpr = 140; // An Objective-C \@protocol expression.

        /** \brief An Objective-C "bridged" cast expression, which casts between
         * Objective-C pointers and C pointers, transferring ownership in the process.
        *
        * \code
        *   NSString *str = (__bridge_transfer NSString *)CFCreateString();
        * \endcode
        */
        const ObjCBridgedCastExpr = 141;

        /** \brief Represents a C++0x pack expansion that produces a sequence of
         * expressions.
        *
        * A pack expansion expression contains a pattern (which itself is an
        * expression) followed by an ellipsis. For example:
        *
        * \code
        * template<typename F, typename ...Types>
        * void forward(F f, Types &&...args) {
        *  f(static_cast<Types&&>(args)...);
        * }
        * \endcode
        */
        const PackExpansionExpr = 142;

        /** \brief Represents an expression that computes the length of a parameter
         * pack.
        *
        * \code
        * template<typename ...Types>
        * struct count {
        *   static const unsigned value = sizeof...(Types);
        * };
        * \endcode
        */
        const SizeOfPackExpr = 143;

        /* \brief Represents a C++ lambda expression that produces a local function
        * object.
        *
        * \code
        * void abssort(float *x, unsigned N) {
        *   std::sort(x, x + N,
        *             [](float a, float b) {
        *               return std::abs(a) < std::abs(b);
        *             });
        * }
        * \endcode
        */
        const LambdaExpr = 144;
        const ObjCBoolLiteralExpr = 145; // Objective-c Boolean Literal.
        const ObjCSelfExpr = 146; // Represents the "self" expression in a ObjC method.
        const LastExpr = DxcCursorKind::ObjCSelfExpr.bits;

        /* Statements */
        const FirstStmt = 200;
        /**
         * \brief A statement whose specific kind is not exposed via this
        * interface.
        *
        * Unexposed statements have the same operations as any other kind of
        * statement; one can extract their location information, spelling,
        * children, etc. However, the specific kind of the statement is not
        * reported.
        */
        const UnexposedStmt = 200;

        /** \brief A labelled statement in a function.
         *
        * This cursor kind is used to describe the "start_over:" label statement in
        * the following example:
        *
        * \code
        *   start_over:
        *     ++counter;
        * \endcode
        *
        */
        const LabelStmt = 201;
        const CompoundStmt = 202; // A group of statements like { stmt stmt }. This cursor kind is used to describe compound statements, e.g. function bodies.
        const CaseStmt = 203; // A case statement.
        const DefaultStmt = 204; // A default statement.
        const IfStmt = 205; // An if statement
        const SwitchStmt = 206; // A switch statement.
        const WhileStmt = 207; // A while statement.
        const DoStmt = 208; // A do statement.
        const ForStmt = 209; // A for statement.
        const GotoStmt = 210; // A goto statement.
        const IndirectGotoStmt = 211; // An indirect goto statement.
        const ContinueStmt = 212; // A continue statement.
        const BreakStmt = 213; // A break statement.
        const ReturnStmt = 214; // A return statement.
        const GCCAsmStmt = 215; // A GCC inline assembly statement extension.
        const AsmStmt = DxcCursorKind::GCCAsmStmt.bits;

        const ObjCAtTryStmt = 216; // Objective-C's overall \@try-\@catch-\@finally statement.
        const ObjCAtCatchStmt = 217; // Objective-C's \@catch statement.
        const ObjCAtFinallyStmt = 218; // Objective-C's \@finally statement.
        const ObjCAtThrowStmt = 219; // Objective-C's \@throw statement.
        const ObjCAtSynchronizedStmt = 220; // Objective-C's \@synchronized statement.
        const ObjCAutoreleasePoolStmt = 221; // Objective-C's autorelease pool statement.
        const ObjCForCollectionStmt = 222; // Objective-C's collection statement.

        const CXXCatchStmt = 223; // C++'s catch statement.
        const CXXTryStmt = 224; // C++'s try statement.
        const CXXForRangeStmt = 225; // C++'s for (* : *) statement.

        const SEHTryStmt = 226; // Windows Structured Exception Handling's try statement.
        const SEHExceptStmt = 227; // Windows Structured Exception Handling's except statement.
        const SEHFinallyStmt = 228; // Windows Structured Exception Handling's finally statement.

        const MSAsmStmt = 229; // A MS inline assembly statement extension.
        const NullStmt = 230; // The null satement ";": C99 6.8.3p3.
        const DeclStmt = 231; // Adaptor class for mixing declarations with statements and expressions.
        const OMPParallelDirective = 232; // OpenMP parallel directive.
        const OMPSimdDirective = 233;  // OpenMP SIMD directive.
        const OMPForDirective = 234;  // OpenMP for directive.
        const OMPSectionsDirective = 235;  // OpenMP sections directive.
        const OMPSectionDirective = 236;  // OpenMP section directive.
        const OMPSingleDirective = 237;  // OpenMP single directive.
        const OMPParallelForDirective = 238;  // OpenMP parallel for directive.
        const OMPParallelSectionsDirective = 239;  // OpenMP parallel sections directive.
        const OMPTaskDirective = 240;  // OpenMP task directive.
        const OMPMasterDirective = 241;  // OpenMP master directive.
        const OMPCriticalDirective = 242;  // OpenMP critical directive.
        const OMPTaskyieldDirective = 243;  // OpenMP taskyield directive.
        const OMPBarrierDirective = 244;  // OpenMP barrier directive.
        const OMPTaskwaitDirective = 245;  // OpenMP taskwait directive.
        const OMPFlushDirective = 246;  // OpenMP flush directive.
        const SEHLeaveStmt = 247;  // Windows Structured Exception Handling's leave statement.
        const OMPOrderedDirective = 248;  // OpenMP ordered directive.
        const OMPAtomicDirective = 249;  // OpenMP atomic directive.
        const OMPForSimdDirective = 250;  // OpenMP for SIMD directive.
        const OMPParallelForSimdDirective = 251;  // OpenMP parallel for SIMD directive.
        const OMPTargetDirective = 252;  // OpenMP target directive.
        const OMPTeamsDirective = 253;  // OpenMP teams directive.
        const OMPTaskgroupDirective = 254;  // OpenMP taskgroup directive.
        const OMPCancellationPointDirective = 255;  // OpenMP cancellation point directive.
        const OMPCancelDirective = 256;  // OpenMP cancel directive.
        const LastStmt = DxcCursorKind::OMPCancelDirective.bits;

        const TranslationUnit = 300; // Cursor that represents the translation unit itself.

        /* Attributes */
        const FirstAttr = 400;
        /**
         * \brief An attribute whose specific kind is not exposed via this
        * interface.
        */
        const UnexposedAttr = 400;

        const IBActionAttr = 401;
        const IBOutletAttr = 402;
        const IBOutletCollectionAttr = 403;
        const CXXFinalAttr = 404;
        const CXXOverrideAttr = 405;
        const AnnotateAttr = 406;
        const AsmLabelAttr = 407;
        const PackedAttr = 408;
        const PureAttr = 409;
        const ConstAttr = 410;
        const NoDuplicateAttr = 411;
        const CUDAConstantAttr = 412;
        const CUDADeviceAttr = 413;
        const CUDAGlobalAttr = 414;
        const CUDAHostAttr = 415;
        const CUDASharedAttr = 416;
        const LastAttr = DxcCursorKind::CUDASharedAttr.bits;

        /* Preprocessing */
        const PreprocessingDirective = 500;
        const MacroDefinition = 501;
        const MacroExpansion = 502;
        const MacroInstantiation = DxcCursorKind::MacroExpansion.bits;
        const InclusionDirective = 503;
        const FirstPreprocessing = DxcCursorKind::PreprocessingDirective.bits;
        const LastPreprocessing = DxcCursorKind::InclusionDirective.bits;

        /* Extra Declarations */
        /**
         * \brief A module import declaration.
        */
        const ModuleImportDecl = 600;
        const FirstExtraDecl = DxcCursorKind::ModuleImportDecl.bits;
        const LastExtraDecl = DxcCursorKind::ModuleImportDecl.bits;
    }
}

iid!(pub IID_IDxcDiagnostic = 0x4f76b234, 0x3659, 0x4d33, 0x99, 0xb0, 0x3b, 0x0d, 0xb9, 0x94, 0xb5, 0x64);
com_interface! {
    interface IDxcDiagnostic: IUnknown{
        iid: IID_IDxcDiagnostic,
        vtable: IDxcDiagnosticVtbl,

        fn FormatDiagnostic(options: DxcDiagnosticDisplayOptions, result: *mut LPSTR) -> HRESULT;
        fn GetSeverity(result: *mut DxcDiagnosticSeverity) -> HRESULT;
        fn GetLocation(result: *mut *mut IDxcSourceLocation) -> HRESULT;
        fn GetSpelling(result: *mut LPSTR) -> HRESULT;
        fn GetCategoryText(result: *mut LPSTR) -> HRESULT;
        fn GetNumRanges(result: *mut u32) -> HRESULT;
        fn GetRangeAt(index: u32, result: *mut *mut IDxcSourceRange) -> HRESULT;
        fn GetNumFixIts(result: *mut u32) -> HRESULT;
        fn GetFixItAt(index: u32, replacementRange: *mut *mut IDxcSourceRange, text: *mut LPSTR) -> HRESULT;
    }
}

iid!(pub IID_IDxcInclusion = 0x0c364d65, 0xdf44, 0x4412, 0x88, 0x8e, 0x4e, 0x55, 0x2f, 0xc5, 0xe3, 0xd6);
com_interface! {
    interface IDxcInclusion: IUnknown{
        iid: IID_IDxcInclusion,
        vtable: IDxcInclusionVtbl,

        fn GetIncludedFile(result: *mut *mut IDxcFile) -> HRESULT;
        fn GetStackLength(result: *mut u32) -> HRESULT;
        fn GetStackItem(index: u32, result: *mut *mut IDxcSourceLocation) -> HRESULT;
    }
}

iid!(pub IID_IDxcToken = 0x7f90b9ff, 0xa275, 0x4932, 0x97, 0xd8, 0x3c, 0xfd, 0x23, 0x44, 0x82, 0xa2);
com_interface! {
    interface IDxcToken: IUnknown{
        iid: IID_IDxcToken,
        vtable: IDxcTokenVtbl,

        fn GetKind(value: *mut DxcTokenKind) -> HRESULT;

        fn GetLocation(value: *mut *mut IDxcSourceLocation) -> HRESULT;

        fn GetExtent(value: *mut *mut IDxcSourceRange) -> HRESULT;

        fn GetSpelling(value: *mut LPSTR) -> HRESULT;
    }
}

iid!(pub IID_IDxcType = 0x2ec912fd, 0xb144, 0x4a15, 0xad, 0x0d, 0x1c, 0x54, 0x39, 0xc8, 0x1e, 0x46);
com_interface! {
    interface IDxcType: IUnknown{
        iid: IID_IDxcType,
        vtable: IDxcTypeVtbl,
        fn GetSpelling(result: *mut LPSTR) -> HRESULT;
        fn IsEqualTo(other: *const IDxcType, result: *mut bool) -> HRESULT;
        fn GetKind(result: *mut IDxcType) -> HRESULT;
    }
}

iid!(pub IID_IDxcSourceLocation = 0x8e7ddf1c, 0xd7d3, 0x4d69, 0xb2, 0x86, 0x85, 0xfc, 0xcb, 0xa1, 0xe0, 0xcf);
com_interface! {
    interface IDxcSourceLocation: IUnknown{
        iid: IID_IDxcSourceLocation,
        vtable: IDxcSourceLocationVtbl,

        fn IsEqualTo(other: *const IDxcSourceLocation, result: *mut bool) ->HRESULT;
        fn GetSpellingLocation(file: *mut *mut IDxcFile, line: *mut u32, col: *mut u32, offset: *mut u32) ->HRESULT;
        fn IsNull(result: *mut bool) ->HRESULT;
    }
}

iid!(pub IID_IDxcSourceRange = 0xf1359b36, 0xa53f, 0x4e81, 0xb5, 0x14, 0xb6, 0xb8, 0x41, 0x22, 0xa1, 0x3f);
com_interface! {
    interface IDxcSourceRange: IUnknown{
        iid: IID_IDxcSourceRange,
        vtable: IDxcSourceRangeVtbl,

        fn IsNull(value: *mut bool) -> HRESULT;
        fn GetStart(value: *mut *mut IDxcSourceLocation) -> HRESULT;
        fn GetEnd(value: *mut *mut IDxcSourceLocation) -> HRESULT;
        fn GetOffsets(startOffset: *mut u32, endOffset: *mut u32) -> HRESULT;
    }
}

iid!(pub IID_IDxcCursor = 0x1467b985, 0x288d, 0x4d2a, 0x80, 0xc1, 0xef, 0x89, 0xc4, 0x2c, 0x40, 0xbc);
com_interface! {
    interface IDxcCursor: IUnknown{
        iid: IID_IDxcCursor,
        vtable: IDxcCursorVtbl,

        fn GetExtent(range: *mut *mut IDxcSourceRange) -> HRESULT;

        fn GetLocation(result: *mut *mut IDxcSourceLocation) -> HRESULT;

        fn GetKind(result: *mut DxcCursorKind) -> HRESULT;

        fn GetKindFlags(result: *mut DxcCursorKindFlags) -> HRESULT;

        fn GetSemanticParent(result: *mut*mut IDxcCursor) -> HRESULT;

        fn GetLexicalParent(result:*mut*mut IDxcCursor) -> HRESULT;

        fn GetCursorType(result:*mut*mut IDxcType) -> HRESULT;

        fn GetNumArguments(result:*mut i32) -> HRESULT;

        fn GetArgumentAt(index: i32, result: *mut *mut IDxcCursor) -> HRESULT;

        fn GetReferencedCursor(result:*mut *mut IDxcCursor) -> HRESULT;

        /// <summary>For a cursor that is either a reference to or a declaration of some entity, retrieve a cursor that describes the definition of that entity.</summary>
        /// <remarks>Some entities can be declared multiple times within a translation unit, but only one of those declarations can also be a definition.</remarks>
        /// <returns>A cursor to the definition of this entity; nullptr if there is no definition in this translation unit.</returns>
        fn GetDefinitionCursor(result:*mut *mut IDxcCursor) -> HRESULT;

        fn FindReferencesInFile(file: *const IDxcFile, skip: u32, top:u32, resultLength: *mut u32, result: *mut *mut *mut IDxcCursor) -> HRESULT;

        /// <summary>Gets the name for the entity references by the cursor, e.g. foo for an 'int foo' variable.</summary>
        fn GetSpelling(result: *mut LPSTR) -> HRESULT;

        fn IsEqualTo(other: *const IDxcCursor, result:*mut bool) -> HRESULT;

        fn IsNull(result:*mut bool) -> HRESULT;

        fn IsDefinition(result:*mut bool) -> HRESULT;

        /// <summary>Gets the display name for the cursor, including e.g. parameter types for a function.</summary>
        fn GetDisplayName(result:*mut BSTR) -> HRESULT;

        /// <summary>Gets the qualified name for the symbol the cursor refers to.</summary>
        fn GetQualifiedName(includeTemplateArgs:bool, result:*mut BSTR) -> HRESULT;

        /// <summary>Gets a name for the cursor, applying the specified formatting flags.</summary>
        fn GetFormattedName(formatting: DxcCursorFormatting, result:*mut BSTR) -> HRESULT;

        /// <summary>Gets children in result up to top elements.</summary>
        fn GetChildren(skip: u32, top: u32,resultLength:*mut u32, result:*mut*mut*mut IDxcCursor) -> HRESULT;

        /// <summary>Gets the cursor following a location within a compound cursor.</summary>
        fn GetSnappedChild(location:  *const IDxcSourceLocation, result:*mut*mut IDxcCursor) -> HRESULT;
    }
}

iid!(pub IID_IDxcUnsavedFile = 0x8ec00f98, 0x07d0, 0x4e60, 0x9d, 0x7c, 0x5a, 0x50, 0xb5, 0xb0, 0x01, 0x7f);
com_interface! {
    interface IDxcUnsavedFile: IUnknown{
        iid: IID_IDxcUnsavedFile,
        vtable: IDxcUnsavedFileVtbl,

        fn get_file_name(fileName: *mut LPSTR) -> HRESULT;
        fn get_contents(contents: *mut LPSTR) -> HRESULT;
        fn get_length(lenth : *mut u32) -> HRESULT;
    }
}

iid!(pub IID_IDxcFile = 0xbb2fca9e, 0x1478, 0x47ba, 0xb0, 0x8c, 0x2c, 0x50, 0x2a, 0xda, 0x48, 0x95);
com_interface! {
    interface IDxcFile: IUnknown{
        iid: IID_IDxcFile,
        vtable: IDxcFileVtbl,

        /// <summary>Gets the file name for this file.</summary>
        fn GetName(result: *mut LPSTR) -> HRESULT;

        /// <summary>Checks whether this file is equal to the other specified file.</summary>
        fn IsEqualTo(other : *const IDxcFile, result: *mut bool) -> HRESULT;
    }
}

iid!(pub IID_IDxcTranslationUnit = 0x9677dee0, 0xc0e5, 0x46a1, 0x8b, 0x40, 0x3d, 0xb3, 0x16, 0x8b, 0xe6, 0x3d);
com_interface! {
    interface IDxcTranslationUnit: IUnknown{
        iid: IID_IDxcTranslationUnit,
        vtable: IDxcTranslationUnitVtbl,

        fn get_cursor(cursor: *mut *mut IDxcCursor) -> HRESULT;

        fn tokenize(range: *const IDxcSourceRange, tokens: *mut *mut *mut IDxcToken, tokenCount: *mut u32) -> HRESULT;

        fn get_location( file: *mut IDxcFile, line: u32, column: u32, result: *mut *mut IDxcSourceLocation) -> HRESULT;

        fn get_num_diagnostics(value : *mut u32) -> HRESULT;

        fn get_diagnostic(index: u32, value: *mut *mut IDxcDiagnostic) -> HRESULT;

        fn get_file(name : *const u8, result : *mut *mut IDxcFile) -> HRESULT;

        fn get_file_name(result : *mut LPSTR) -> HRESULT;

        fn reparse(unsaved_files : *mut *mut IDxcUnsavedFile, num_unsaved_files: u32) -> HRESULT;

        fn get_cursor_for_location(location: *const IDxcSourceLocation, result : *mut *mut IDxcCursor) -> HRESULT;

        fn get_location_for_offset(file : *const IDxcFile, offset: u32, result: *mut *mut IDxcSourceLocation) -> HRESULT;

        fn get_skipped_ranges(file: *const IDxcFile, resultCount: *mut u32, result: *mut *mut *mut IDxcSourceRange) -> HRESULT;

        fn get_diagnostic_details(
            index: u32,  options: DxcDiagnosticDisplayOptions, errorCode: *mut u32, errorLine: *mut u32, errorColumn: *mut u32,
            errorFile: *mut BSTR, errorOffset: *mut u32, errorLength: *mut u32, errorMessage: *mut BSTR) -> HRESULT;

        fn get_inclusion_list(resultCount: *mut u32, result: *mut *mut *mut IDxcInclusion) -> HRESULT;
    }
}

iid!(pub IID_IDxcIndex = 0x937824a0, 0x7f5a, 0x4815, 0x9b, 0xa, 0x7c, 0xc0, 0x42, 0x4f, 0x41, 0x73);
com_interface! {
    interface IDxcIndex: IUnknown{
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
    interface IDxcIntelliSense: IUnknown{
        iid: IID_IDxcIntelliSense,
        vtable: IDxcIntelliSenseVtbl,

        fn create_index(index: *mut *mut IDxcIndex) -> HRESULT;
        fn get_null_location(location: *mut *mut  IDxcSourceLocation)  -> HRESULT;
        fn get_null_range(location: *mut *mut  IDxcSourceRange)  -> HRESULT;
        fn get_range( start: *const IDxcSourceLocation, end: *const IDxcSourceLocation, location: *mut *mut IDxcSourceRange)  -> HRESULT;
        fn get_default_diagnostic_display_options(value: *mut DxcDiagnosticDisplayOptions)  -> HRESULT;
        fn get_default_editing_tu_options(value: *mut DxcTranslationUnitFlags)  -> HRESULT;
        fn create_unsaved_file(fileName: LPCSTR, contents: LPCSTR, contentLength: u32 , result: *mut *mut IDxcUnsavedFile)  -> HRESULT;
    }
}

iid!(pub CLSID_DxcIntelliSense = 0x3047833c, 0xd1c0, 0x4b8e, 0x9d, 0x40, 0x10, 0x28, 0x78, 0x60, 0x59, 0x85);
