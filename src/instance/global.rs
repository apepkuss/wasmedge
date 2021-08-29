use crate::context::store::StoreContext;
use crate::types::*;
use std::marker::PhantomData;
use wasmedge_sys::ffi as we_ffi;

pub struct GlobalInstanceContext<'a> {
    pub(crate) raw: *mut we_ffi::WasmEdge_GlobalInstanceContext,
    pub(crate) _marker: PhantomData<&'a StoreContext<'a>>,
}
impl<'a> GlobalInstanceContext<'a> {
    pub fn create(value: WasmEdgeValue, mutable: WasmEdgeMutability) -> Self {
        GlobalInstanceContext {
            raw: unsafe { we_ffi::WasmEdge_GlobalInstanceCreate(value, mutable) },
            _marker: PhantomData,
        }
    }
}
impl<'a> Drop for GlobalInstanceContext<'a> {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_GlobalInstanceDelete(self.raw) }
    }
}
