#[cfg(windows)]
mod os_defs {
    pub use winapi::shared::{
        ntdef::{HRESULT, LPCSTR, LPCWSTR, LPSTR, LPWSTR, WCHAR},
        wtypes::BSTR,
    };
}

#[cfg(not(windows))]
mod os_defs {
    pub type CHAR = i8;
    pub type WCHAR = u32;
    pub type OLECHAR = WCHAR;
    pub type LPSTR = *mut CHAR;
    pub type LPWSTR = *mut WCHAR;
    pub type LPCSTR = *const CHAR;
    pub type LPCWSTR = *const WCHAR;
    pub type BSTR = *mut OLECHAR;
    pub type LPBSTR = *mut BSTR;
    pub type HRESULT = i32;
}

pub use os_defs::*;
