use crate::context::store::StoreContext;
use crate::types::*;
use std::marker::PhantomData;
use wasmedge_sys::ffi as we_ffi;

pub struct GlobalInstanceContext<'store, 'vm> {
    pub(crate) raw: *mut we_ffi::WasmEdge_GlobalInstanceContext,
    pub(crate) _marker: PhantomData<&'store StoreContext<'vm>>,
}
impl<'store, 'vm> GlobalInstanceContext<'store, 'vm> {
    pub fn create(value: WasmEdgeValue, mutable: WasmEdgeMutability) -> Self {
        GlobalInstanceContext {
            raw: unsafe { we_ffi::WasmEdge_GlobalInstanceCreate(value, mutable) },
            _marker: PhantomData,
        }
    }
}
impl<'store, 'vm> Drop for GlobalInstanceContext<'store, 'vm> {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_GlobalInstanceDelete(self.raw) }
    }
}
