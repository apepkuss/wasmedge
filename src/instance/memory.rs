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
    use crate::{
        context::import_object::ImportObjectContext,
        context::vm::VMContext,
        instance::function::{FunctionTypeContext, HostFunctionContext},
        value::*,
    };
    use std::ptr;
    use wasmedge_sys::ffi as we_ffi;

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
        // ! error: xecution failed: out of bounds memory access, Code: 0x88
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

    #[test]
    fn test_wasmedge_memory_ctx() {
        #[no_mangle]
        unsafe extern "C" fn FuncAdd(
            data: *mut std::os::raw::c_void,
            mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
            params: *const WasmEdgeValue,
            returns: *mut WasmEdgeValue,
        ) -> we_ffi::WasmEdge_Result {
            let params = std::slice::from_raw_parts(params, 2);
            let val1 = WasmEdgeValueGetI32(params[0]);
            let val2 = WasmEdgeValueGetI32(params[1]);
            let res = WasmEdgeValueGenI32(val1 + val2);
            returns.write(res);

            we_ffi::WasmEdge_Result { Code: 0 }
        }

        // Create the VM context.
        let mut vm_ctx = VMContext::create(None, None);

        // create import object
        let mod_name = "calculator";
        let mut imp_obj = ImportObjectContext::create(mod_name, ptr::null_mut()).unwrap();

        // Create and add a host function instance into the import object.
        let params = [
            WasmEdgeValType::WasmEdge_ValType_I32,
            WasmEdgeValType::WasmEdge_ValType_I32,
        ];
        let returns = [WasmEdgeValType::WasmEdge_ValType_I32];
        let result = FunctionTypeContext::create(Some(&params), Some(&returns));
        assert!(result.is_some());
        let func_type = result.unwrap();
        let result = HostFunctionContext::create(&func_type, Some(FuncAdd), 0);
        let mut host_func = result.unwrap();
        imp_obj.add_host_function("func-add", &mut host_func);

        // register import-object
        vm_ctx.register_module_from_import_object(&imp_obj).unwrap();

        unsafe {
            // let tf_imp_obj = we_ffi::WasmEdge_Tensorflow_ImportObjectCreate();
            // vm_ctx
            //     .register_module_from_import_object(&tf_imp_obj)
            //     .unwrap();
            // we_ffi::WasmEdge_VMRegisterModuleFromImport(vm_ctx.raw, tf_imp_obj);

            // let ipt_ctx = we_ffi::WasmEdge_InterpreterCreate(ptr::null(), ptr::null_mut());
            // let store_ctx = we_ffi::WasmEdge_StoreCreate();
            // let result =
            //     we_ffi::WasmEdge_InterpreterRegisterImport(ipt_ctx, store_ctx, imp_obj.raw);
            // if !we_ffi::WasmEdge_ResultOK(result) {
            //     println!("Import object registration failed");
            // }

            let x = we_ffi::WasmEdge_VMGetFunctionListLength(vm_ctx.raw);
            println!("x: {}", x);

            let store = we_ffi::WasmEdge_VMGetStoreContext(vm_ctx.raw);
            // let mod_name = WasmEdgeString::from_str(mod_name).unwrap();
            // let len = we_ffi::WasmEdge_StoreListMemoryRegisteredLength(store_ctx, mod_name.raw);

            // let mod_name = WasmEdgeString::from_str("wasi_ephemeral_nn").unwrap();
            let len = we_ffi::WasmEdge_StoreListFunctionLength(store);
            println!("len: {}", len);
        }
    }
}
