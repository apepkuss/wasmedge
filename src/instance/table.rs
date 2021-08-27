use crate::types::*;
use wasmedge_sys::ffi as we_ffi;

pub struct TableInstanceContext {
    pub(crate) raw: *mut we_ffi::WasmEdge_TableInstanceContext,
}
impl TableInstanceContext {
    pub fn create(ref_type: WasmEdgeRefType, limit: WasmEdgeLimit) -> TableInstanceContext {
        TableInstanceContext {
            raw: unsafe { we_ffi::WasmEdge_TableInstanceCreate(ref_type, limit) },
        }
    }
}
impl Drop for TableInstanceContext {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_TableInstanceDelete(self.raw) }
    }
}
