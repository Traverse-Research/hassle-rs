use crate::intellisense::ffi::*;
use crate::wrapper::Dxc;
use com_rs::ComPtr;
use std::ffi::CString;
use winapi::shared::ntdef::LPSTR;
use winapi::shared::winerror::HRESULT;
use winapi::shared::wtypes::BSTR;

#[derive(Debug)]
pub struct DxcIntellisense {
    inner: ComPtr<IDxcIntelliSense>,
}

impl DxcIntellisense {
    fn new(inner: ComPtr<IDxcIntelliSense>) -> Self {
        Self { inner }
    }

    pub fn get_default_editing_tu_options(&self) -> Result<DxcTranslationUnitFlags, HRESULT> {
        let mut options: DxcTranslationUnitFlags = DxcTranslationUnitFlags::None;
        unsafe {
            return_hr!(
                self.inner.get_default_editing_tu_options(&mut options),
                options
            );
        }
    }

    pub fn create_index(&self) -> Result<DxcIndex, HRESULT> {
        let mut index: ComPtr<IDxcIndex> = ComPtr::new();
        unsafe {
            return_hr!(
                self.inner.create_index(index.as_mut_ptr()),
                DxcIndex::new(index)
            );
        }
    }

    pub fn create_unsaved_file(
        &self,
        file_name: &[u8],
        contents: &[u8],
    ) -> Result<DxcUnsavedFile, HRESULT> {
        let c_file_name = match CString::new(file_name) {
            Ok(cs) => cs,
            Err(_) => return Err(-1),
        };

        let c_contents = match CString::new(contents) {
            Ok(cs) => cs,
            Err(_) => return Err(-1),
        };

        let mut file: ComPtr<IDxcUnsavedFile> = ComPtr::new();
        unsafe {
            return_hr!(
                self.inner.create_unsaved_file(
                    c_file_name.as_ptr(),
                    c_contents.as_ptr(),
                    contents.len() as u32,
                    file.as_mut_ptr()
                ),
                DxcUnsavedFile::new(file)
            );
        }
    }
}

#[derive(Debug)]
pub struct DxcIndex {
    inner: ComPtr<IDxcIndex>,
}

impl DxcIndex {
    fn new(inner: ComPtr<IDxcIndex>) -> Self {
        return Self { inner };
    }
}

impl DxcIndex {
    pub fn parse_translation_unit(
        &self,
        source_filename: &[u8],
        args: &[&[u8]],
        unsaved_files: &[&DxcUnsavedFile],
        options: DxcTranslationUnitFlags,
    ) -> Result<DxcTranslationUnit, HRESULT> {
        let c_source_filename = match CString::new(source_filename) {
            Ok(cs) => cs,
            Err(_) => return Err(-1),
        };

        let mut uf = vec![];

        for unsaved_file in unsaved_files {
            uf.push(unsaved_file.inner.as_ptr());
        }

        unsafe {
            let mut c_args: Vec<CString> = vec![];
            let mut cliargs = vec![];

            for arg in args.into_iter() {
                let c_arg = match CString::new(*arg) {
                    Ok(cs) => cs,
                    Err(_) => return Err(-1),
                };

                cliargs.push(c_arg.as_ptr() as *const u8);
                c_args.push(c_arg);
            }

            let mut tu: ComPtr<IDxcTranslationUnit> = ComPtr::new();
            return_hr!(
                self.inner.parse_translation_unit(
                    c_source_filename.as_ptr() as *const u8,
                    cliargs.as_ptr(),
                    cliargs.len() as i32,
                    uf.as_ptr(),
                    uf.len() as u32,
                    options,
                    tu.as_mut_ptr()
                ),
                DxcTranslationUnit::new(tu)
            );
        }
    }
}

#[derive(Debug)]
pub struct DxcUnsavedFile {
    inner: ComPtr<IDxcUnsavedFile>,
}

impl DxcUnsavedFile {
    pub fn get_length(&self) -> Result<u32, HRESULT> {
        let mut length: u32 = 0;
        unsafe {
            return_hr!(self.inner.get_length(&mut length), length);
        }
    }

    fn new(inner: ComPtr<IDxcUnsavedFile>) -> Self {
        DxcUnsavedFile { inner }
    }
}

#[derive(Debug)]
pub struct DxcTranslationUnit {
    inner: ComPtr<IDxcTranslationUnit>,
}

impl DxcTranslationUnit {
    fn new(inner: ComPtr<IDxcTranslationUnit>) -> Self {
        DxcTranslationUnit { inner }
    }

    pub fn get_file(&self, name: &[u8]) -> Result<DxcFile, HRESULT> {
        let mut file: ComPtr<IDxcFile> = ComPtr::new();
        unsafe {
            return_hr!(
                self.inner.get_file(name.as_ptr(), file.as_mut_ptr()),
                DxcFile::new(file)
            );
        }
    }

    pub fn get_cursor(&self) -> Result<DxcCursor, HRESULT> {
        let mut cursor: ComPtr<IDxcCursor> = ComPtr::new();
        unsafe {
            return_hr!(
                self.inner.get_cursor(cursor.as_mut_ptr()),
                DxcCursor::new(cursor)
            );
        }
    }
}

#[derive(Debug)]
pub struct DxcCursor {
    inner: ComPtr<IDxcCursor>,
}

impl DxcCursor {
    fn new(inner: ComPtr<IDxcCursor>) -> Self {
        DxcCursor { inner }
    }

    pub fn get_children(&self, skip: u32, max_count: u32) -> Result<Vec<DxcCursor>, HRESULT> {
        unsafe {
            let mut result: *mut *mut IDxcCursor = std::ptr::null_mut();
            let mut result_length: u32 = 0;

            let mut children = vec![];
            return_hr!(
                self.inner
                    .GetChildren(skip, max_count, &mut result_length, &mut result),
                {
                    for i in 0..result_length {
                        let mut childcursor = ComPtr::<IDxcCursor>::new();

                        let ptr: *mut *mut IDxcCursor = childcursor.as_mut_ptr();

                        *ptr = (*result).offset(i as isize);

                        children.push(DxcCursor::new(childcursor));
                    }
                    children
                }
            );
        }
    }

    pub fn get_all_children(&self) -> Result<Vec<DxcCursor>, HRESULT> {
        let max_children_count = 10;
        let mut current_children_count = 0;
        let mut children = vec![];

        unsafe {
            let mut result: *mut *mut IDxcCursor = std::ptr::null_mut();
            let mut result_length: u32 = 0;

            loop {
                let hr = self.inner.GetChildren(
                    current_children_count,
                    max_children_count,
                    &mut result_length,
                    &mut result,
                );

                if hr != 0 {
                    return Err(hr);
                }

                for i in 0..result_length {
                    let mut childcursor = ComPtr::<IDxcCursor>::new();

                    let ptr: &mut *mut IDxcCursor = childcursor.as_mut_ptr();

                    *ptr = *(result.offset(i as isize));

                    let dxc_cursor = DxcCursor::new(childcursor);

                    children.push(dxc_cursor);
                }

                if result_length < max_children_count {
                    return Ok(children);
                }

                current_children_count += result_length;
            }
        }
    }

    pub fn get_extent(&self) -> Result<DxcSourceRange, HRESULT> {
        unsafe {
            let mut range: ComPtr<IDxcSourceRange> = ComPtr::new();
            return_hr!(
                self.inner.GetExtent(range.as_mut_ptr()),
                DxcSourceRange::new(range)
            );
        }
    }

    pub fn get_location(&self) -> Result<DxcSourceLocation, HRESULT> {
        unsafe {
            let mut location: ComPtr<IDxcSourceLocation> = ComPtr::new();
            return_hr!(
                self.inner.GetLocation(location.as_mut_ptr()),
                DxcSourceLocation::new(location)
            );
        }
    }

    pub fn get_display_name(&self) -> Result<String, HRESULT> {
        unsafe {
            let mut name: BSTR = std::ptr::null_mut();
            return_hr!(
                self.inner.GetDisplayName(&mut name),
                crate::intellisense::utils::from_bstr(name)?
            );
        }
    }

    pub fn get_formatted_name(&self, formatting: DxcCursorFormatting) -> Result<String, HRESULT> {
        unsafe {
            let mut name: BSTR = std::ptr::null_mut();
            return_hr!(
                self.inner.GetFormattedName(formatting, &mut name),
                crate::intellisense::utils::from_bstr(name)?
            );
        }
    }

    pub fn get_qualified_name(&self, include_template_args: bool) -> Result<String, HRESULT> {
        unsafe {
            let mut name: BSTR = std::ptr::null_mut();
            return_hr!(
                self.inner
                    .GetQualifiedName(include_template_args, &mut name),
                crate::intellisense::utils::from_bstr(name)?
            );
        }
    }

    pub fn get_kind(&self) -> Result<DxcCursorKind, HRESULT> {
        unsafe {
            let mut cursor_kind: DxcCursorKind = DxcCursorKind::UnexposedDecl;
            return_hr!(self.inner.GetKind(&mut cursor_kind), cursor_kind);
        }
    }

    pub fn get_kind_flags(&self) -> Result<DxcCursorKindFlags, HRESULT> {
        unsafe {
            let mut cursor_kind_flags: DxcCursorKindFlags = DxcCursorKindFlags::None;
            return_hr!(
                self.inner.GetKindFlags(&mut cursor_kind_flags),
                cursor_kind_flags
            );
        }
    }

    pub fn get_semantic_parent(&self) -> Result<DxcCursor, HRESULT> {
        unsafe {
            let mut inner = ComPtr::<IDxcCursor>::new();
            return_hr!(
                self.inner.GetSemanticParent(inner.as_mut_ptr()),
                DxcCursor::new(inner)
            );
        }
    }

    pub fn get_lexical_parent(&self) -> Result<DxcCursor, HRESULT> {
        unsafe {
            let mut inner = ComPtr::<IDxcCursor>::new();
            return_hr!(
                self.inner.GetLexicalParent(inner.as_mut_ptr()),
                DxcCursor::new(inner)
            );
        }
    }

    pub fn get_cursor_type(&self) -> Result<DxcType, HRESULT> {
        unsafe {
            let mut inner = ComPtr::<IDxcType>::new();
            return_hr!(
                self.inner.GetCursorType(inner.as_mut_ptr()),
                DxcType::new(inner)
            );
        }
    }

    pub fn get_num_arguments(&self) -> Result<i32, HRESULT> {
        unsafe {
            let mut result: i32 = 0;
            return_hr!(self.inner.GetNumArguments(&mut result), result);
        }
    }

    pub fn get_argument_at(&self, index: i32) -> Result<DxcCursor, HRESULT> {
        unsafe {
            let mut inner = ComPtr::<IDxcCursor>::new();
            return_hr!(
                self.inner.GetArgumentAt(index, inner.as_mut_ptr()),
                DxcCursor::new(inner)
            );
        }
    }

    pub fn get_referenced_cursor(&self) -> Result<DxcCursor, HRESULT> {
        unsafe {
            let mut inner = ComPtr::<IDxcCursor>::new();
            return_hr!(
                self.inner.GetReferencedCursor(inner.as_mut_ptr()),
                DxcCursor::new(inner)
            );
        }
    }

    /// <summary>For a cursor that is either a reference to or a declaration of some entity, retrieve a cursor that describes the definition of that entity.</summary>
    /// <remarks>Some entities can be declared multiple times within a translation unit, but only one of those declarations can also be a definition.</remarks>
    /// <returns>A cursor to the definition of this entity; nullptr if there is no definition in this translation unit.</returns>
    pub fn get_definition_cursor(&self) -> Result<DxcCursor, HRESULT> {
        unsafe {
            let mut inner = ComPtr::<IDxcCursor>::new();
            return_hr!(
                self.inner.GetDefinitionCursor(inner.as_mut_ptr()),
                DxcCursor::new(inner)
            );
        }
    }

    pub fn find_references_in_file(
        &self,
        file: &DxcFile,
        skip: u32,
        top: u32,
    ) -> Result<Vec<DxcCursor>, HRESULT> {
        unsafe {
            let mut result: *mut *mut IDxcCursor = std::ptr::null_mut();
            let mut result_length: u32 = 0;

            let mut children = vec![];
            return_hr!(
                self.inner.FindReferencesInFile(
                    file.inner.as_ptr(),
                    skip,
                    top,
                    &mut result_length,
                    &mut result
                ),
                {
                    for i in 0..result_length {
                        let mut childcursor = ComPtr::<IDxcCursor>::new();

                        let ptr: *mut *mut IDxcCursor = childcursor.as_mut_ptr();

                        *ptr = (*result).offset(i as isize);

                        children.push(DxcCursor::new(childcursor));
                    }
                    children
                }
            );
        }
    }

    /// <summary>Gets the name for the entity references by the cursor, e.g. foo for an 'int foo' variable.</summary>
    pub fn get_spelling(&self) -> Result<String, HRESULT> {
        unsafe {
            let mut spelling: LPSTR = std::ptr::null_mut();
            return_hr!(
                self.inner.GetSpelling(&mut spelling),
                crate::intellisense::utils::from_lpstr(spelling)?
            );
        }
    }

    pub fn is_equal_to(&self, other: &DxcCursor) -> Result<bool, HRESULT> {
        unsafe {
            let mut result: bool = false;
            return_hr!(
                self.inner.IsEqualTo(other.inner.as_ptr(), &mut result),
                result
            );
        }
    }

    pub fn is_null(&self) -> Result<bool, HRESULT> {
        unsafe {
            let mut result: bool = false;
            return_hr!(self.inner.IsNull(&mut result), result);
        }
    }

    pub fn is_definition(&self) -> Result<bool, HRESULT> {
        unsafe {
            let mut result: bool = false;
            return_hr!(self.inner.IsDefinition(&mut result), result);
        }
    }

    /// <summary>Gets the cursor following a location within a compound cursor.</summary>
    pub fn get_snapped_child(&self, location: &DxcSourceLocation) -> Result<DxcCursor, HRESULT> {
        unsafe {
            let mut inner = ComPtr::<IDxcCursor>::new();
            return_hr!(
                self.inner
                    .GetSnappedChild(location.inner.as_ptr(), inner.as_mut_ptr()),
                DxcCursor::new(inner)
            );
        }
    }

    pub fn get_source<'a>(&self, source: &'a str) -> Result<&'a str, HRESULT> {
        let range = self.get_extent()?;

        let DxcSourceOffsets {
            start_offset,
            end_offset,
        } = range.get_offsets()?;

        let source_range = (start_offset as usize)..(end_offset as usize);

        return Ok(&source[source_range]);
    }
}

#[derive(Debug)]
pub struct DxcType {
    inner: ComPtr<IDxcType>,
}

impl DxcType {
    fn new(inner: ComPtr<IDxcType>) -> Self {
        DxcType { inner }
    }

    pub fn get_spelling(&self) -> Result<String, HRESULT> {
        unsafe {
            let mut spelling: LPSTR = std::ptr::null_mut();
            return_hr!(
                self.inner.GetSpelling(&mut spelling),
                crate::intellisense::utils::from_lpstr(spelling)?
            );
        }
    }
}

#[derive(Debug)]
pub struct DxcSourceLocation {
    inner: ComPtr<IDxcSourceLocation>,
}

impl DxcSourceLocation {
    fn new(inner: ComPtr<IDxcSourceLocation>) -> Self {
        DxcSourceLocation { inner }
    }
}

#[derive(Debug)]
pub struct DxcSourceOffsets {
    pub start_offset: u32,
    pub end_offset: u32,
}

#[derive(Debug)]
pub struct DxcSourceRange {
    inner: ComPtr<IDxcSourceRange>,
}

impl DxcSourceRange {
    pub fn get_offsets(&self) -> Result<DxcSourceOffsets, HRESULT> {
        unsafe {
            let mut start_offset: u32 = 0;
            let mut end_offset: u32 = 0;
            return_hr!(
                self.inner.GetOffsets(&mut start_offset, &mut end_offset),
                DxcSourceOffsets {
                    start_offset,
                    end_offset
                }
            );
        }
    }
}

impl DxcSourceRange {
    fn new(inner: ComPtr<IDxcSourceRange>) -> Self {
        DxcSourceRange { inner }
    }
}

#[derive(Debug)]
pub struct DxcFile {
    inner: ComPtr<IDxcFile>,
}

impl DxcFile {
    fn new(inner: ComPtr<IDxcFile>) -> Self {
        DxcFile { inner }
    }
}

impl Dxc {
    pub fn create_intellisense(&self) -> Result<DxcIntellisense, HRESULT> {
        let mut intellisense: ComPtr<IDxcIntelliSense> = ComPtr::new();
        return_hr!(
            self.get_dxc_create_instance()(
                &CLSID_DxcIntelliSense,
                &IID_IDxcIntelliSense,
                intellisense.as_mut_ptr(),
            ),
            DxcIntellisense::new(intellisense)
        );
    }
}
