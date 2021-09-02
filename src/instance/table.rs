use crate::context::store::StoreContext;
use crate::error::WasmEdgeResult;
use crate::types::*;
use crate::utils::check;
use std::marker::PhantomData;
use std::mem;
use wasmedge_sys::ffi as we_ffi;

pub struct TableInstanceContext<'a> {
    pub(crate) raw: *mut we_ffi::WasmEdge_TableInstanceContext,
    pub(crate) _marker: PhantomData<&'a StoreContext<'a>>,
}
impl<'a> TableInstanceContext<'a> {
    pub fn create(ref_type: WasmEdgeRefType, limit: WasmEdgeLimit) -> Option<Self> {
        let raw = unsafe { we_ffi::WasmEdge_TableInstanceCreate(ref_type, limit) };
        match raw.is_null() {
            true => None,
            false => Some(TableInstanceContext {
                raw,
                _marker: PhantomData,
            }),
        }
    }

    pub fn get_ref_type(&self) -> WasmEdgeRefType {
        unsafe { we_ffi::WasmEdge_TableInstanceGetRefType(self.raw) }
    }

    pub fn set_data(&mut self, data: WasmEdgeValue, offset: usize) -> WasmEdgeResult<()> {
        unsafe {
            check(we_ffi::WasmEdge_TableInstanceSetData(
                self.raw,
                data,
                offset as u32,
            ))
        }
    }

    pub fn get_data(&self, data: &mut WasmEdgeValue, offset: usize) -> WasmEdgeResult<()> {
        unsafe {
            check(we_ffi::WasmEdge_TableInstanceGetData(
                self.raw,
                data as *mut _,
                offset as u32,
            ))
        }
    }

    pub fn get_size(&self) -> usize {
        unsafe { we_ffi::WasmEdge_TableInstanceGetSize(self.raw) as usize }
    }

    pub fn grow(&mut self, size: usize) -> WasmEdgeResult<()> {
        unsafe { check(we_ffi::WasmEdge_TableInstanceGrow(self.raw, size as u32)) }
    }
}
impl<'a> Drop for TableInstanceContext<'a> {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_TableInstanceDelete(self.raw) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::*;
    use std::os::raw::c_void;

    #[test]
    fn test_instance_table() {
        // Table instance creation
        let limit = WasmEdgeLimit {
            HasMax: false,
            Min: 10,
            Max: 0,
        };
        let result =
            TableInstanceContext::create(WasmEdgeRefType::WasmEdge_RefType_ExternRef, limit);
        assert!(result.is_some());
        let tab_ctx = result.unwrap();
        assert!(!tab_ctx.raw.is_null());

        let limit = WasmEdgeLimit {
            HasMax: true,
            Min: 10,
            Max: 0,
        };
        let result =
            TableInstanceContext::create(WasmEdgeRefType::WasmEdge_RefType_ExternRef, limit);
        assert!(result.is_some());
        let mut tab_ctx = result.unwrap();
        assert!(!tab_ctx.raw.is_null());

        // Table instance get reference type
        assert_eq!(
            tab_ctx.get_ref_type(),
            WasmEdgeRefType::WasmEdge_RefType_ExternRef
        );

        // Table instance set data
        let ptr = &mut tab_ctx as *mut _ as *mut c_void;
        let val = WasmEdgeValueGenExternRef(ptr);
        let result = tab_ctx.set_data(val, 5);
        assert!(result.is_ok());
        // ? error: Mismatched value type. Expected: externref , Got: funcref
        // let tmp_val = WasmEdgeValueGenFuncRef(2);
        // let result = tab_ctx.set_data(tmp_val, 6);
        // assert!(result.is_err());

        // Table instance get data
        let mut val = WasmEdgeValueGenI32(0);
        let result = tab_ctx.get_data(&mut val, 5);
        assert!(result.is_ok());
        let x = WasmEdgeValueGetExternRef(val) as *const TableInstanceContext;
        let y = &tab_ctx as *const TableInstanceContext;
        assert!(std::ptr::eq(x, y));
        // let result = tab_ctx.get_data(&mut val, 15); // ! error: execution failed: out of bounds table access, Code: 0x87
        // assert!(result.is_err());

        // Table instance get size and grow
        assert_eq!(tab_ctx.get_size(), 10);
        let result = tab_ctx.grow(8);
        assert!(result.is_ok());
        assert_eq!(tab_ctx.get_size(), 18);
        let result = tab_ctx.grow(8);
        assert!(result.is_ok());
        assert_eq!(tab_ctx.get_size(), 26);

        let val = WasmEdgeValueGenExternRef((&mut tab_ctx) as *mut _ as *mut c_void);
        let result = tab_ctx.set_data(val, 15);
        assert!(result.is_ok());
        let mut val = WasmEdgeValueGenI32(0);
        let result = tab_ctx.get_data(&mut val, 15);
        assert!(result.is_ok());
        let x = WasmEdgeValueGetExternRef(val) as *const TableInstanceContext;
        let y = &tab_ctx as *const TableInstanceContext;
        assert!(std::ptr::eq(x, y));
    }
}
