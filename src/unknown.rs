#[cfg(not(windows))]
use crate::os::HRESULT;

use tinycom::{com_interface, IID_IUnknown, IUnknown};

#[cfg(not(windows))]
// Steal the interface ID from IUnknown:
com_interface! {
    /// Insert complete object and deleting destructor on non-Windows platforms, where Dxc shims IUnknown in WinAdapter.
    /// This requires a virtual destructor (delete is actually used on the base class) which unfortunately makes the struct
    /// binary incompatible.
    ///
    /// See the third and fourth entry:
    /// ```cmd
    /// vtable for 'DxcLibrary' @ 0x7ffff7cbc5f8 (subobject @ 0x5555556bb9e0):
    /// [0]: 0x7ffff6a56d40 <DxcLibrary::QueryInterface(_GUID const&, void**)>
    /// [1]: 0x7ffff6a56d20 <DxcLibrary::AddRef()>
    /// [2]: 0x7ffff6a56d30 <DxcLibrary::Release()>
    /// [3]: 0x7ffff6b36bc0 <IUnknown::~IUnknown()>
    /// [4]: 0x7ffff6a57130 <DxcLibrary::~DxcLibrary()>
    /// [5]: 0x7ffff6a56d50 <DxcLibrary::SetMalloc(IMalloc*)>
    /// [6]: 0x7ffff6a56d60 <DxcLibrary::CreateBlobFromBlob(IDxcBlob*, unsigned int, unsigned int, IDxcBlob**)>
    /// [7]: 0x7ffff6a56d70 <DxcLibrary::CreateBlobFromFile(wchar_t const*, unsigned int*, IDxcBlobEncoding**)>
    /// [8]: 0x7ffff6a56d80 <DxcLibrary::CreateBlobWithEncodingFromPinned(void const*, unsigned int, unsigned int, IDxcBlobEncoding**)>
    /// [9]: 0x7ffff6a56d90 <DxcLibrary::CreateBlobWithEncodingOnHeapCopy(void const*, unsigned int, unsigned int, IDxcBlobEncoding**)>
    /// [10]: 0x7ffff6a56da0 <DxcLibrary::CreateBlobWithEncodingOnMalloc(void const*, IMalloc*, unsigned int, unsigned int, IDxcBlobEncoding**)>
    /// [11]: 0x7ffff6a56db0 <DxcLibrary::CreateIncludeHandler(IDxcIncludeHandler**)>
    /// [12]: 0x7ffff6a56dc0 <DxcLibrary::CreateStreamFromBlobReadOnly(IDxcBlob*, IStream**)>
    /// [13]: 0x7ffff6a56dd0 <DxcLibrary::GetBlobAsUtf8(IDxcBlob*, IDxcBlobEncoding**)>
    /// [14]: 0x7ffff6a56e90 <DxcLibrary::GetBlobAsUtf16(IDxcBlob*, IDxcBlobEncoding**)>
    /// ```
    interface IDxcUnknownShim: IUnknown {
        iid: IID_IUnknown,
        vtable: IDxcUnknownShimVtbl,
        fn complete_object_destructor() -> HRESULT;
        fn deleting_destructor() -> HRESULT;
    }
}

#[cfg(windows)]
com_interface! {
    /// Forwards to IUnknown. No-op on Windows
    interface IDxcUnknownShim: IUnknown {
        iid: IID_IUnknown,
        vtable: IDxcUnknownShimVtbl,
    }
}
