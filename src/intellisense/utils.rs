use crate::intellisense::DxcCursor;
use crate::intellisense::wrapper::DxcSourceOffsets;
use crate::intellisense::ffi::DxcCursorFormatting;
use winapi::shared::winerror::HRESULT;
use winapi::shared::wtypes::BSTR;
use winapi::shared::ntdef::LPSTR;
use winapi::shared::ntdef::LPCSTR;
use winapi::um::oleauto::{SysFreeString, SysStringLen, SysStringByteLen};


pub(crate) fn from_bstr(string: BSTR) -> Result<String, HRESULT> {
    unsafe {        
        let len = SysStringLen(string);

        let slice: &[u16] = ::std::slice::from_raw_parts(string, len as usize);
            
        let result = match String::from_utf16(slice) {
            Ok(s) => Ok(s),
            Err(_) => Err(-1)
        };

        SysFreeString(string);

        return result;
    }
}

pub(crate) fn from_lpstr(string: LPSTR) -> Result<String, HRESULT> {
    unsafe { 
        let len = (0..).take_while(|&i| *string.offset(i) != 0).count();

        let slice: &[u8] = ::std::slice::from_raw_parts(string as *const u8, len);

        match std::str::from_utf8(slice) {
            Ok(s) => Ok(s.to_owned()),
            Err(_) => Err(-1)
        }
    }
}

pub fn print_cursor_tree(cursor: &DxcCursor, source: &str) -> Result<(), HRESULT> {
    return print_indented_cursor_tree(cursor, source, 0);
}

pub fn print_cursor(cursor: &DxcCursor, source: &str)  -> Result<(), HRESULT> {
    return print_indented_cursor(cursor, source, 0);
}

fn print_indented_cursor_tree(cursor: &DxcCursor, source: &str, indent: usize) -> Result<(), HRESULT> {

    print_indented_cursor(cursor, source, indent)?;

    let child_cursors = cursor.get_all_children()?;

    for child_cursor in &child_cursors {
        print_indented_cursor_tree(child_cursor, source,  indent + 1)?;
    }

    return Ok(());
}

pub fn print_indented_cursor(cursor: &DxcCursor, source: &str, indent: usize)  -> Result<(), HRESULT> {
    let range = cursor.get_extent()?;
    let location = cursor.get_location()?;
    let kind = cursor.get_kind()?;
    let kind_flags = cursor.get_kind_flags()?;
    let cursor_type = cursor.get_cursor_type()?.get_spelling()?;
    let spelling = cursor.get_spelling()?;

    let num_args = cursor.get_num_arguments()?;

    let display_name = cursor.get_display_name()?;
    let format_name = cursor.get_formatted_name(DxcCursorFormatting::_Default)?;
    let qualified_name = cursor.get_qualified_name(true)?;

    let DxcSourceOffsets {
        start_offset,
        end_offset,
    } = range.get_offsets()?;

    let source_range = (start_offset as usize)..(end_offset as usize);

    let source_text = &source[source_range];

    let child_count = cursor.get_all_children()?.len();

    println!("{: <indent$} - [{}:{}] {:?} kind_flags: {:?} cursor type: {:?} spelling: {:?}",
        "",
        start_offset,
        end_offset,
        display_name,
        kind_flags,
        cursor_type,
        spelling,
        // num_args,
        indent = indent,
    );

    println!("{: <indent$} > formatted name {:?}", "", format_name, indent = indent + 1);
    println!("{: <indent$} > qualified name {:?}", "", qualified_name, indent = indent + 1);
    println!("{: <indent$} > source {:?}", "", source_text, indent = indent + 1);

    return Ok(());
}