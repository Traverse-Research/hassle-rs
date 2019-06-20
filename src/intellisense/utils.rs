use crate::intellisense::DxcCursor;
use crate::intellisense::wrapper::DxcSourceOffsets;
use crate::intellisense::ffi::DxcCursorFormatting;
use winapi::shared::winerror::HRESULT;
use winapi::shared::wtypes::BSTR;
use winapi::shared::ntdef::LPSTR;
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

pub fn print_cursor_tree(cursor: &DxcCursor) -> Result<(), HRESULT> {

    return print_indented_cursor_tree(cursor, 0);
}

pub fn print_cursor(cursor: &DxcCursor)  -> Result<(), HRESULT> {
    return print_indented_cursor(cursor, 0);
}

fn print_indented_cursor_tree(cursor: &DxcCursor, indent: usize) -> Result<(), HRESULT> {

    print_indented_cursor(cursor, indent)?;

    let child_cursors = cursor.get_all_children()?;

    for child_cursor in &child_cursors {
        print_indented_cursor_tree(child_cursor, indent + 1)?;
    }

    return Ok(());
}

pub fn print_indented_cursor(cursor: &DxcCursor, indent: usize)  -> Result<(), HRESULT> {
    let range = cursor.get_extent()?;
    let location = cursor.get_location()?;
    let name = cursor.get_display_name()?;
    let kind = cursor.get_kind()?;
    let kind_flags = cursor.get_kind_flags()?;

    let DxcSourceOffsets {
        start_offset,
        end_offset,
    } = range.get_offsets()?;

    let child_count = cursor.get_all_children()?.len();    
    
    println!("{}", name);

    // println!("{: <indent$} - [{}:{}] '{}' type: {:?} kind_flags: '{:?}' {} children(s)",
    //     "",
    //     start_offset,
    //     end_offset,
    //     name,
    //     kind,
    //     kind_flags,
    //     child_count,
    //     indent = indent,
    // );

    return Ok(());
}
