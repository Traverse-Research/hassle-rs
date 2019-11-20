#[cfg(windows)]
mod os_defs {
    pub use winapi::shared::{
        ntdef::{LPSTR, LPWSTR},
        wtypes::BSTR,
    };
}

#[cfg(not(windows))]
mod os_defs {
    pub type CHAR = i8;
    pub type WCHAR = u32;
    pub type LPSTR = *mut CHAR;
    pub type LPWSTR = *mut WCHAR;
    pub type LPCSTR = *const CHAR;
    pub type LPCWSTR = *const WCHAR;
    pub type HRESULT = LONG;
    pub type LONG = i64;
}

pub use os_defs::*;
