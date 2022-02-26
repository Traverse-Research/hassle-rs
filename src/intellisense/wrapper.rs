use crate::intellisense::ffi::*;
use crate::os::{CoTaskMemFree, BSTR, LPSTR};
use crate::utils::Result;
use crate::wrapper::Dxc;
use std::ffi::CString;
use tinycom::ComPtr;

#[derive(Debug)]
pub struct DxcIntellisense {
    inner: ComPtr<IDxcIntelliSense>,
}

impl DxcIntellisense {
    fn new(inner: ComPtr<IDxcIntelliSense>) -> Self {
        Self { inner }
    }

    pub fn get_default_editing_tu_options(&self) -> Result<DxcTranslationUnitFlags> {
        let mut options: DxcTranslationUnitFlags = DxcTranslationUnitFlags::NONE;
        unsafe { self.inner.get_default_editing_tu_options(&mut options) }
            .result_with_success(options)
    }

    pub fn create_index(&self) -> Result<DxcIndex> {
        let mut index: ComPtr<IDxcIndex> = ComPtr::new();
        unsafe { self.inner.create_index(index.as_mut_ptr()) }.result()?;
        Ok(DxcIndex::new(index))
    }

    pub fn create_unsaved_file(&self, file_name: &str, contents: &str) -> Result<DxcUnsavedFile> {
        let c_file_name = CString::new(file_name).expect("Failed to convert `file_name`");
        let c_contents = CString::new(contents).expect("Failed to convert `contents`");

        let mut file: ComPtr<IDxcUnsavedFile> = ComPtr::new();
        unsafe {
            self.inner.create_unsaved_file(
                c_file_name.as_ptr(),
                c_contents.as_ptr(),
                contents.len() as u32,
                file.as_mut_ptr(),
            )
        }
        .result()?;
        Ok(DxcUnsavedFile::new(file))
    }
}

#[derive(Debug)]
pub struct DxcIndex {
    inner: ComPtr<IDxcIndex>,
}

impl DxcIndex {
    fn new(inner: ComPtr<IDxcIndex>) -> Self {
        Self { inner }
    }
}

impl DxcIndex {
    pub fn parse_translation_unit(
        &self,
        source_filename: &str,
        args: &[&str],
        unsaved_files: &[&DxcUnsavedFile],
        options: DxcTranslationUnitFlags,
    ) -> Result<DxcTranslationUnit> {
        let c_source_filename =
            CString::new(source_filename).expect("Failed to convert `source_filename`");

        let mut uf = vec![];

        for unsaved_file in unsaved_files {
            uf.push(unsaved_file.inner.as_ptr());
        }

        let mut c_args: Vec<CString> = vec![];
        let mut cliargs = vec![];

        for arg in args.iter() {
            let c_arg = CString::new(*arg).expect("Failed to convert `arg`");
            cliargs.push(c_arg.as_ptr().cast());
            c_args.push(c_arg);
        }

        let mut tu: ComPtr<IDxcTranslationUnit> = ComPtr::new();

        unsafe {
            self.inner.parse_translation_unit(
                c_source_filename.as_ptr().cast(),
                cliargs.as_ptr(),
                cliargs.len() as i32,
                uf.as_ptr(),
                uf.len() as u32,
                options,
                tu.as_mut_ptr(),
            )
        }
        .result()?;
        Ok(DxcTranslationUnit::new(tu))
    }
}

#[derive(Debug)]
pub struct DxcUnsavedFile {
    inner: ComPtr<IDxcUnsavedFile>,
}

impl DxcUnsavedFile {
    pub fn get_length(&self) -> Result<u32> {
        let mut length: u32 = 0;
        unsafe { self.inner.get_length(&mut length) }.result_with_success(length)
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

    pub fn get_file(&self, name: &[u8]) -> Result<DxcFile> {
        let mut file: ComPtr<IDxcFile> = ComPtr::new();
        unsafe { self.inner.get_file(name.as_ptr(), file.as_mut_ptr()) }.result()?;
        Ok(DxcFile::new(file))
    }

    pub fn get_cursor(&self) -> Result<DxcCursor> {
        let mut cursor: ComPtr<IDxcCursor> = ComPtr::new();
        unsafe { self.inner.get_cursor(cursor.as_mut_ptr()) }.result()?;
        Ok(DxcCursor::new(cursor))
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

    pub fn get_children(&self, skip: u32, max_count: u32) -> Result<Vec<DxcCursor>> {
        let mut result: *mut *mut IDxcCursor = std::ptr::null_mut();
        let mut result_length: u32 = 0;

        unsafe {
            self.inner
                .get_children(skip, max_count, &mut result_length, &mut result)
        }
        .result()?;

        // get_children allocates a buffer to pass the result in.
        let child_cursors = unsafe { std::slice::from_raw_parts(result, result_length as usize) }
            .iter()
            .map(|&ptr| {
                let mut childcursor = ComPtr::<IDxcCursor>::new();
                *childcursor.as_mut_ptr() = ptr;
                DxcCursor::new(childcursor)
            })
            .collect::<Vec<_>>();
        unsafe { CoTaskMemFree(result.cast()) };
        Ok(child_cursors)
    }

    pub fn get_all_children(&self) -> Result<Vec<DxcCursor>> {
        const MAX_CHILDREN_PER_CHUNK: u32 = 10;
        let mut children = vec![];

        loop {
            let res = self.get_children(children.len() as u32, MAX_CHILDREN_PER_CHUNK)?;
            let res_len = res.len();
            children.extend(res);
            if res_len < MAX_CHILDREN_PER_CHUNK as usize {
                break Ok(children);
            }
        }
    }

    pub fn get_extent(&self) -> Result<DxcSourceRange> {
        let mut range: ComPtr<IDxcSourceRange> = ComPtr::new();
        unsafe { self.inner.get_extent(range.as_mut_ptr()) }.result()?;
        Ok(DxcSourceRange::new(range))
    }

    pub fn get_location(&self) -> Result<DxcSourceLocation> {
        let mut location: ComPtr<IDxcSourceLocation> = ComPtr::new();
        unsafe { self.inner.get_location(location.as_mut_ptr()) }.result()?;
        Ok(DxcSourceLocation::new(location))
    }

    pub fn get_display_name(&self) -> Result<String> {
        let mut name: BSTR = std::ptr::null_mut();
        unsafe { self.inner.get_display_name(&mut name) }.result()?;
        Ok(crate::utils::from_bstr(name))
    }

    pub fn get_formatted_name(&self, formatting: DxcCursorFormatting) -> Result<String> {
        let mut name: BSTR = std::ptr::null_mut();
        unsafe { self.inner.get_formatted_name(formatting, &mut name) }.result()?;
        Ok(crate::utils::from_bstr(name))
    }

    pub fn get_qualified_name(&self, include_template_args: bool) -> Result<String> {
        let mut name: BSTR = std::ptr::null_mut();
        unsafe {
            self.inner
                .get_qualified_name(include_template_args, &mut name)
        }
        .result()?;
        Ok(crate::utils::from_bstr(name))
    }

    pub fn get_kind(&self) -> Result<DxcCursorKind> {
        let mut cursor_kind: DxcCursorKind = DxcCursorKind::UNEXPOSED_DECL;
        unsafe { self.inner.get_kind(&mut cursor_kind) }.result_with_success(cursor_kind)
    }

    pub fn get_kind_flags(&self) -> Result<DxcCursorKindFlags> {
        let mut cursor_kind_flags: DxcCursorKindFlags = DxcCursorKindFlags::NONE;
        unsafe { self.inner.get_kind_flags(&mut cursor_kind_flags) }
            .result_with_success(cursor_kind_flags)
    }

    pub fn get_semantic_parent(&self) -> Result<DxcCursor> {
        let mut inner = ComPtr::<IDxcCursor>::new();
        unsafe { self.inner.get_semantic_parent(inner.as_mut_ptr()) }.result()?;
        Ok(DxcCursor::new(inner))
    }

    pub fn get_lexical_parent(&self) -> Result<DxcCursor> {
        let mut inner = ComPtr::<IDxcCursor>::new();
        unsafe { self.inner.get_lexical_parent(inner.as_mut_ptr()) }.result()?;
        Ok(DxcCursor::new(inner))
    }

    pub fn get_cursor_type(&self) -> Result<DxcType> {
        let mut inner = ComPtr::<IDxcType>::new();
        unsafe { self.inner.get_cursor_type(inner.as_mut_ptr()) }.result()?;
        Ok(DxcType::new(inner))
    }

    pub fn get_num_arguments(&self) -> Result<i32> {
        let mut result: i32 = 0;

        unsafe { self.inner.get_num_arguments(&mut result) }.result_with_success(result)
    }

    pub fn get_argument_at(&self, index: i32) -> Result<DxcCursor> {
        let mut inner = ComPtr::<IDxcCursor>::new();
        unsafe { self.inner.get_argument_at(index, inner.as_mut_ptr()) }.result()?;
        Ok(DxcCursor::new(inner))
    }

    pub fn get_referenced_cursor(&self) -> Result<DxcCursor> {
        let mut inner = ComPtr::<IDxcCursor>::new();
        unsafe { self.inner.get_referenced_cursor(inner.as_mut_ptr()) }.result()?;
        Ok(DxcCursor::new(inner))
    }

    pub fn get_definition_cursor(&self) -> Result<DxcCursor> {
        let mut inner = ComPtr::<IDxcCursor>::new();
        unsafe { self.inner.get_definition_cursor(inner.as_mut_ptr()) }.result()?;
        Ok(DxcCursor::new(inner))
    }

    pub fn find_references_in_file(
        &self,
        file: &DxcFile,
        skip: u32,
        top: u32,
    ) -> Result<Vec<DxcCursor>> {
        let mut result: *mut *mut IDxcCursor = std::ptr::null_mut();
        let mut result_length: u32 = 0;

        unsafe {
            self.inner.find_references_in_file(
                file.inner.as_ptr(),
                skip,
                top,
                &mut result_length,
                &mut result,
            )
        }
        .result()?;

        // find_references_in_file allocates a buffer to pass the result in.
        let child_cursors = unsafe { std::slice::from_raw_parts(result, result_length as usize) }
            .iter()
            .map(|&ptr| {
                let mut childcursor = ComPtr::<IDxcCursor>::new();
                *childcursor.as_mut_ptr() = ptr;
                DxcCursor::new(childcursor)
            })
            .collect::<Vec<_>>();
        unsafe { CoTaskMemFree(result.cast()) };
        Ok(child_cursors)
    }

    pub fn get_spelling(&self) -> Result<String> {
        let mut spelling: LPSTR = std::ptr::null_mut();
        unsafe { self.inner.get_spelling(&mut spelling) }.result()?;
        Ok(crate::utils::from_lpstr(spelling))
    }

    pub fn is_equal_to(&self, other: &DxcCursor) -> Result<bool> {
        let mut result: bool = false;
        unsafe { self.inner.is_equal_to(other.inner.as_ptr(), &mut result) }
            .result_with_success(result)
    }

    pub fn is_null(&mut self) -> Result<bool> {
        let mut result: bool = false;
        unsafe { IDxcCursor::is_null(&self.inner, &mut result) }.result_with_success(result)
    }

    pub fn is_definition(&self) -> Result<bool> {
        let mut result: bool = false;
        unsafe { self.inner.is_definition(&mut result) }.result_with_success(result)
    }

    pub fn get_snapped_child(&self, location: &DxcSourceLocation) -> Result<DxcCursor> {
        let mut inner = ComPtr::<IDxcCursor>::new();
        unsafe {
            self.inner
                .get_snapped_child(location.inner.as_ptr(), inner.as_mut_ptr())
        }
        .result()?;
        Ok(DxcCursor::new(inner))
    }

    pub fn get_source<'a>(&self, source: &'a str) -> Result<&'a str> {
        let range = self.get_extent()?;

        let DxcSourceOffsets {
            start_offset,
            end_offset,
        } = range.get_offsets()?;

        let source_range = (start_offset as usize)..(end_offset as usize);

        Ok(&source[source_range])
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

    pub fn get_spelling(&self) -> Result<String> {
        let mut spelling: LPSTR = std::ptr::null_mut();
        unsafe { self.inner.get_spelling(&mut spelling) }
            .result_with_success(crate::utils::from_lpstr(spelling))
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
    pub fn get_offsets(&self) -> Result<DxcSourceOffsets> {
        let mut start_offset: u32 = 0;
        let mut end_offset: u32 = 0;
        unsafe { self.inner.get_offsets(&mut start_offset, &mut end_offset) }.result_with_success(
            DxcSourceOffsets {
                start_offset,
                end_offset,
            },
        )
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
    pub fn create_intellisense(&self) -> Result<DxcIntellisense> {
        let mut intellisense: ComPtr<IDxcIntelliSense> = ComPtr::new();

        self.get_dxc_create_instance()?(
            &CLSID_DxcIntelliSense,
            &IID_IDxcIntelliSense,
            intellisense.as_mut_ptr(),
        )
        .result()?;
        Ok(DxcIntellisense::new(intellisense))
    }
}
