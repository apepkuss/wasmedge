use crate::context::store::StoreContext;
use crate::types::*;
use std::marker::PhantomData;
use wasmedge_sys::ffi as we_ffi;

pub struct TableInstanceContext<'a> {
    pub(crate) raw: *mut we_ffi::WasmEdge_TableInstanceContext,
    pub(crate) _marker: PhantomData<&'a StoreContext<'a>>,
}
impl<'a> TableInstanceContext<'a> {
    pub fn create(ref_type: WasmEdgeRefType, limit: WasmEdgeLimit) -> Self {
        TableInstanceContext {
            raw: unsafe { we_ffi::WasmEdge_TableInstanceCreate(ref_type, limit) },
            _marker: PhantomData,
        }
    }
}
impl<'a> Drop for TableInstanceContext<'a> {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_TableInstanceDelete(self.raw) }
    }
}
