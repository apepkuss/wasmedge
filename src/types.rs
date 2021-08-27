use crate::error::WasmEdgeResult;
use std::ffi::{CStr, CString};
use wasmedge_sys::ffi as we_ffi;

pub type WasmEdgeValue = we_ffi::WasmEdge_Value;
pub type WasmEdgeValType = we_ffi::WasmEdge_ValType;
pub type WasmEdgeProposal = we_ffi::WasmEdge_Proposal;
pub type HostRegistration = we_ffi::WasmEdge_HostRegistration;
pub type WasmEdgeLimit = we_ffi::WasmEdge_Limit;
pub type WasmEdgeRefType = we_ffi::WasmEdge_RefType;
pub type WasmEdgeMutability = we_ffi::WasmEdge_Mutability;

pub struct WasmEdgeString {
    pub(crate) raw: we_ffi::WasmEdge_String,
}
impl WasmEdgeString {
    pub fn from_str(s: &str) -> WasmEdgeResult<WasmEdgeString> {
        let cstring = CString::new(s)?;
        Ok(WasmEdgeString {
            raw: unsafe { we_ffi::WasmEdge_StringCreateByCString(cstring.as_ptr()) },
        })
    }

    pub fn from_buffer(buf: &[i8]) -> WasmEdgeString {
        WasmEdgeString {
            raw: unsafe { we_ffi::WasmEdge_StringCreateByBuffer(buf.as_ptr(), buf.len() as u32) },
        }
    }

    pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        let cstr = unsafe { CStr::from_ptr(self.raw.Buf) };
        cstr.to_string_lossy()
    }

    pub fn into_raw(&self) -> we_ffi::WasmEdge_String {
        self.raw
    }
}
impl Drop for WasmEdgeString {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_StringDelete(self.raw) }
    }
}
