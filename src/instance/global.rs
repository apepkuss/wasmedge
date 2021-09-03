use crate::context::store::StoreContext;
use crate::types::*;
use std::marker::PhantomData;
use wasmedge_sys::ffi as we_ffi;

pub struct GlobalInstanceContext<'store, 'vm: 'store> {
    pub(crate) raw: *mut we_ffi::WasmEdge_GlobalInstanceContext,
    pub(crate) _marker: PhantomData<&'store StoreContext<'vm>>,
    pub(crate) _drop: bool,
}
impl<'store, 'vm: 'store> GlobalInstanceContext<'store, 'vm> {
    pub fn create(value: WasmEdgeValue, mutable: WasmEdgeMutability) -> Option<Self> {
        let raw = unsafe { we_ffi::WasmEdge_GlobalInstanceCreate(value, mutable) };
        match raw.is_null() {
            true => None,
            false => Some(GlobalInstanceContext {
                raw,
                _marker: PhantomData,
                _drop: true,
            }),
        }
    }

    pub fn val_type(&self) -> WasmEdgeValType {
        unsafe { we_ffi::WasmEdge_GlobalInstanceGetValType(self.raw) }
    }

    pub fn mutability(&self) -> WasmEdgeMutability {
        unsafe { we_ffi::WasmEdge_GlobalInstanceGetMutability(self.raw) }
    }

    pub fn value(&self) -> WasmEdgeValue {
        unsafe { we_ffi::WasmEdge_GlobalInstanceGetValue(self.raw) }
    }

    pub fn value_mut(&self, value: WasmEdgeValue) {
        unsafe { we_ffi::WasmEdge_GlobalInstanceSetValue(self.raw, value) }
    }
}
impl<'store, 'vm: 'store> Drop for GlobalInstanceContext<'store, 'vm> {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            if self._drop {
                unsafe { we_ffi::WasmEdge_GlobalInstanceDelete(self.raw) }
            } else {
                self.raw = std::ptr::null_mut();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::*;

    #[test]
    fn test_instance_global() {
        // Global instance creation
        let result = GlobalInstanceContext::create(
            WasmEdgeValueGenI64(55555555555),
            WasmEdgeMutability::WasmEdge_Mutability_Const,
        );
        assert!(result.is_some());
        let glob_c_ctx = result.unwrap();
        assert!(!glob_c_ctx.raw.is_null());
        let result = GlobalInstanceContext::create(
            WasmEdgeValueGenI64(66666666666),
            WasmEdgeMutability::WasmEdge_Mutability_Var,
        );
        assert!(result.is_some());
        let glob_v_ctx = result.unwrap();
        assert!(!glob_v_ctx.raw.is_null());

        // Global instance get value type
        assert_eq!(glob_c_ctx.val_type(), WasmEdgeValType::WasmEdge_ValType_I64);
        assert_eq!(glob_c_ctx.val_type(), WasmEdgeValType::WasmEdge_ValType_I64);

        // Global instance get mutability
        assert_eq!(
            glob_c_ctx.mutability(),
            WasmEdgeMutability::WasmEdge_Mutability_Const
        );
        assert_eq!(
            glob_v_ctx.mutability(),
            WasmEdgeMutability::WasmEdge_Mutability_Var
        );

        // Global instance get value
        let val = glob_c_ctx.value();
        assert_eq!(WasmEdgeValueGetI64(val), 55555555555);
        let val = glob_v_ctx.value();
        assert_eq!(WasmEdgeValueGetI64(val), 66666666666);

        // Global instance set value
        let val = WasmEdgeValueGenI64(77777777777);
        glob_c_ctx.value_mut(val);
        let val = glob_c_ctx.value();
        assert_eq!(WasmEdgeValueGetI64(val), 55555555555);
        let val = WasmEdgeValueGenI64(88888888888);
        glob_v_ctx.value_mut(val);
        let val = glob_v_ctx.value();
        assert_eq!(WasmEdgeValueGetI64(val), 88888888888);
        let val = WasmEdgeValueGenF32(12.345);
        glob_v_ctx.value_mut(val);
        let val = glob_v_ctx.value();
        assert_eq!(WasmEdgeValueGetI64(val), 88888888888);
    }
}
