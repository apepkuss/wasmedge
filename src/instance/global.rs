use crate::types::*;
use wasmedge_sys::ffi as we_ffi;

pub struct GlobalInstanceContext {
    pub(crate) raw: *mut we_ffi::WasmEdge_GlobalInstanceContext,
}
impl GlobalInstanceContext {
    pub fn create(value: WasmEdgeValue, mutable: WasmEdgeMutability) -> Self {
        GlobalInstanceContext {
            raw: unsafe { we_ffi::WasmEdge_GlobalInstanceCreate(value, mutable) },
        }
    }
}
impl Drop for GlobalInstanceContext {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_GlobalInstanceDelete(self.raw) }
    }
}
