#[cfg(windows)]
mod os_defs {
    pub use winapi::shared{ntdef::{LPSTR, LPWSTR}, wtypes::BSTR};
}

#[not(cfg(windows))]
mod os_defs {
    use crate::os::raw::c_long;

    pub type LPCSTR = *const CHAR;
    pub type LPCWSTR = *const WCHAR;
    pub type BSTR = *mut u16;
    pub type HRESULT = LONG;
    pub type LONG = c_long;
}

pub use os_defs::*;
