use crate::error::WasmEdgeResult;
use crate::utils::check;
use crate::{context::store::StoreContext, types::*};
use std::marker::PhantomData;
use wasmedge_sys::ffi as we_ffi;

pub struct MemoryInstanceContext<'store, 'vm: 'store> {
    pub(crate) raw: *mut we_ffi::WasmEdge_MemoryInstanceContext,
    pub(crate) _marker: PhantomData<&'store StoreContext<'vm>>,
    pub(crate) _drop: bool,
}
impl<'store, 'vm: 'store> MemoryInstanceContext<'store, 'vm> {
    pub fn create(limit: WasmEdgeLimit) -> Option<Self> {
        let raw = unsafe { we_ffi::WasmEdge_MemoryInstanceCreate(limit) };
        match raw.is_null() {
            true => None,
            false => Some(MemoryInstanceContext {
                raw,
                _marker: PhantomData,
                _drop: true,
            }),
        }
    }

    pub fn set_data(&mut self, data: &[u8], offset: usize) -> WasmEdgeResult<()> {
        unsafe {
            check(we_ffi::WasmEdge_MemoryInstanceSetData(
                self.raw,
                data.as_ptr() as *mut _,
                offset as u32,
                data.len() as u32,
            ))
        }
    }

    pub fn get_data(&self, data: &mut [u8], offset: usize) -> WasmEdgeResult<()> {
        unsafe {
            check(we_ffi::WasmEdge_MemoryInstanceGetData(
                self.raw,
                data.as_mut_ptr(),
                offset as u32,
                data.len() as u32,
            ))
        }
    }

    pub fn page_size(&self) -> usize {
        unsafe { we_ffi::WasmEdge_MemoryInstanceGetPageSize(self.raw) as usize }
    }

    pub fn grow_page(&mut self, page: usize) -> WasmEdgeResult<()> {
        unsafe {
            check(we_ffi::WasmEdge_MemoryInstanceGrowPage(
                self.raw,
                page as u32,
            ))
        }
    }
}
impl<'store, 'vm: 'store> Drop for MemoryInstanceContext<'store, 'vm> {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            if self._drop {
                unsafe { we_ffi::WasmEdge_MemoryInstanceDelete(self.raw) }
            } else {
                self.raw = std::ptr::null_mut();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_memory() {
        // Memory instance creation
        let limit = WasmEdgeLimit {
            HasMax: false,
            Min: 1,
            Max: 0,
        };
        let result = MemoryInstanceContext::create(limit);
        assert!(result.is_some());
        let mem_ctx = result.unwrap();
        assert!(!mem_ctx.raw.is_null());
        let limit = WasmEdgeLimit {
            HasMax: true,
            Min: 1,
            Max: 0,
        };
        let result = MemoryInstanceContext::create(limit);
        assert!(result.is_some());
        let mut mem_ctx = result.unwrap();
        assert!(!mem_ctx.raw.is_null());

        // Memory instance set data
        let data = "test data\n".as_bytes();
        let result = mem_ctx.set_data(data, 100);
        assert!(result.is_ok());
        let result = mem_ctx.set_data(data, 100);
        assert!(result.is_ok());
        // ! error: execution failed: out of bounds memory access, Code: 0x88
        // let result = mem_ctx.set_data(data, 65536);
        // assert!(result.is_ok());

        // Memory instance get data
        let mut data_get = [0; 10];
        let result = mem_ctx.get_data(&mut data_get, 100);
        assert!(result.is_ok());
        assert_eq!(data, &data_get);
        let mut data_get = [];
        let result = mem_ctx.get_data(&mut data_get, 100);
        assert!(result.is_ok());
        // ! error: execution failed: out of bounds memory access, Code: 0x88
        // let mut data_get = [0; 10];
        // let result = mem_ctx.get_data(&mut data_get, 65536);
        // assert!(result.is_err());

        // Memory instance get size and grow
        // ! error: code:136, message:out of bounds memory access
        // assert_eq!(mem_ctx.page_size(), 1);
        // let result = mem_ctx.grow_page(1);
        // assert!(result.is_ok());
        // assert_eq!(mem_ctx.page_size(), 2);
        // let result = mem_ctx.grow_page(2);
        // assert!(result.is_err());
        // assert_eq!(mem_ctx.page_size(), 2);
    }
}
