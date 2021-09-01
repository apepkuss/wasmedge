use crate::{context::store::StoreContext, types::*};
use std::marker::PhantomData;
use wasmedge_sys::ffi as we_ffi;

pub struct MemoryInstanceContext<'a> {
    pub(crate) raw: *mut we_ffi::WasmEdge_MemoryInstanceContext,
    pub(crate) _marker: PhantomData<&'a StoreContext<'a>>,
}
impl<'a> MemoryInstanceContext<'a> {
    pub fn create(limit: WasmEdgeLimit) -> Self {
        MemoryInstanceContext {
            raw: unsafe { we_ffi::WasmEdge_MemoryInstanceCreate(limit) },
            _marker: PhantomData,
        }
    }
}
impl<'a> Drop for MemoryInstanceContext<'a> {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_MemoryInstanceDelete(self.raw) }
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
