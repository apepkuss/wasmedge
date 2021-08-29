use crate::context::store::StoreContext;
use crate::types::*;
use std::marker::PhantomData;
use wasmedge_sys::ffi as we_ffi;

pub struct TableInstanceContext<'store, 'vm> {
    pub(crate) raw: *mut we_ffi::WasmEdge_TableInstanceContext,
    pub(crate) _marker: PhantomData<&'store StoreContext<'vm>>,
}
impl<'store, 'vm> TableInstanceContext<'store, 'vm> {
    pub fn create(ref_type: WasmEdgeRefType, limit: WasmEdgeLimit) -> Self {
        TableInstanceContext {
            raw: unsafe { we_ffi::WasmEdge_TableInstanceCreate(ref_type, limit) },
            _marker: PhantomData,
        }
    }
}
impl<'store, 'vm> Drop for TableInstanceContext<'store, 'vm> {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_TableInstanceDelete(self.raw) }
    }
}
