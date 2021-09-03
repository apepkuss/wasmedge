use crate::{
    context::{
        configure::ConfigureContext, import_object::ImportObjectContext, store::StoreContext,
    },
    error::WasmEdgeResult,
    instance::function::FunctionTypeContext,
    types::*,
    utils::{check, path_to_cstring},
};
use std::marker::PhantomData;
use std::mem;
use std::path::Path;
use std::ptr;
use wasmedge_sys::ffi as we_ffi;

pub struct VMContext {
    pub(crate) raw: *mut we_ffi::WasmEdge_VMContext,
}
impl VMContext {
    pub fn create(
        conf: Option<&ConfigureContext>,
        store: Option<&mut StoreContext>,
    ) -> Option<VMContext> {
        let conf = match conf {
            Some(conf) => conf.raw,
            None => ptr::null(),
        };
        let store = match store {
            Some(store) => store.raw,
            None => ptr::null_mut(),
        };
        let vm = unsafe { we_ffi::WasmEdge_VMCreate(conf, store) };
        match vm.is_null() {
            true => None,
            false => Some(VMContext { raw: vm }),
        }
    }

    pub fn register_module_from_import_object(
        &mut self,
        import_ctx: &ImportObjectContext,
    ) -> WasmEdgeResult<()> {
        unsafe {
            check(we_ffi::WasmEdge_VMRegisterModuleFromImport(
                self.raw,
                import_ctx.raw,
            ))
        }
    }

    pub fn register_module_from_file<P: AsRef<Path>>(
        &mut self,
        mod_name: &str,
        path: P,
    ) -> WasmEdgeResult<()> {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let path = path_to_cstring(path.as_ref())?;
        unsafe {
            check(we_ffi::WasmEdge_VMRegisterModuleFromFile(
                self.raw,
                mod_name.raw,
                path.as_ptr(),
            ))
        }
    }

    pub fn run_wasm_from_file<'vm, P: AsRef<Path>>(
        &mut self,
        path: P,
        func_name: &str,
        params: &[WasmEdgeValue],
        buf: &'vm mut [mem::MaybeUninit<WasmEdgeValue>],
    ) -> WasmEdgeResult<&'vm [WasmEdgeValue]> {
        let path = path_to_cstring(path.as_ref())?;
        unsafe {
            let func_name = WasmEdgeString::from_str(func_name)
                .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());

            let result = check(we_ffi::WasmEdge_VMRunWasmFromFile(
                self.raw,
                path.as_ptr(),
                func_name.raw,
                params.as_ptr() as *const WasmEdgeValue,
                params.len() as u32,
                buf.as_mut_ptr() as *mut WasmEdgeValue,
                buf.len() as u32,
            ));

            match result {
                Ok(_) => Ok(mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()])),
                Err(err) => Err(err),
            }
        }
    }

    pub fn run_wasm_from_buffer<'vm>(
        &mut self,
        buf: &[u8],
        func_name: &str,
        params: &[WasmEdgeValue],
        returns: &'vm mut [mem::MaybeUninit<WasmEdgeValue>],
    ) -> WasmEdgeResult<&'vm [WasmEdgeValue]> {
        let func_name = WasmEdgeString::from_str(func_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());

        unsafe {
            let result = check(we_ffi::WasmEdge_VMRunWasmFromBuffer(
                self.raw,
                buf.as_ptr(),
                buf.len() as u32,
                func_name.raw,
                params.as_ptr() as *const WasmEdgeValue,
                params.len() as u32,
                returns.as_mut_ptr() as *mut WasmEdgeValue,
                returns.len() as u32,
            ));

            match result {
                Ok(_) => Ok(mem::MaybeUninit::slice_assume_init_ref(
                    &returns[..returns.len()],
                )),
                Err(err) => Err(err),
            }
        }
    }

    pub fn function_type(&self, func_name: &str) -> Option<FunctionTypeContext> {
        let func_name = WasmEdgeString::from_str(func_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());
        let result = unsafe { we_ffi::WasmEdge_VMGetFunctionType(self.raw, func_name.raw) };
        if result.is_null() {
            return None;
        }

        Some(FunctionTypeContext { raw: result })
    }

    pub fn function_type_registered(
        &self,
        mod_name: &str,
        func_name: &str,
    ) -> Option<FunctionTypeContext> {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let func_name = WasmEdgeString::from_str(func_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());
        let result = unsafe {
            we_ffi::WasmEdge_VMGetFunctionTypeRegistered(self.raw, mod_name.raw, func_name.raw)
        };
        if result.is_null() {
            return None;
        }

        Some(FunctionTypeContext { raw: result })
    }

    pub fn function_list_len(&self) -> usize {
        unsafe { we_ffi::WasmEdge_VMGetFunctionListLength(self.raw) as usize }
    }

    pub fn get_import_object(&self, reg: HostRegistration) -> Option<ImportObjectContext> {
        let raw = unsafe { we_ffi::WasmEdge_VMGetImportModuleContext(self.raw, reg) };
        match raw.is_null() {
            true => None,
            false => Some(ImportObjectContext {
                raw,
                _marker: PhantomData,
                _drop: false,
            }),
        }
    }

    pub fn execute_registered<'vm>(
        &self,
        mod_name: &str,
        func_name: &str,
        params: &[WasmEdgeValue],
        buf: &'vm mut [mem::MaybeUninit<WasmEdgeValue>],
    ) -> WasmEdgeResult<&'vm [WasmEdgeValue]> {
        unsafe {
            let mod_name = WasmEdgeString::from_str(mod_name)
                .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
            let func_name = WasmEdgeString::from_str(func_name)
                .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());

            let result = check(we_ffi::WasmEdge_VMExecuteRegistered(
                self.raw,
                mod_name.raw,
                func_name.raw,
                params.as_ptr() as *const WasmEdgeValue,
                params.len() as u32,
                buf.as_mut_ptr() as *mut WasmEdgeValue,
                buf.len() as u32,
            ));

            match result {
                Ok(_) => Ok(mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()])),
                Err(err) => Err(err),
            }
        }
    }

    pub fn store_context(&self) -> StoreContext {
        let store_ctx = unsafe { we_ffi::WasmEdge_VMGetStoreContext(self.raw) };
        StoreContext {
            raw: store_ctx,
            _marker: PhantomData,
        }
    }
}
impl Drop for VMContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_VMDelete(self.raw) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        context::{
            ast::ASTModuleContext, configure::ConfigureContext, loader::LoaderContext,
            store::StoreContext, validator::Validator,
        },
        instance::function::*,
        types::*,
        value::*,
    };
    use std::mem;

    const TPATH: &str = "/root/workspace/wasmedge-ml/wasmedge/tests/data/test.wasm";

    #[test]
    fn test_context_vm_run_wasm_from_file() {
        let mut conf = ConfigureContext::create();
        conf.add_host_registration(HostRegistration::WasmEdge_HostRegistration_Wasi);

        let result = VMContext::create(Some(&conf), None);
        assert!(result.is_some());
        let mut vm = result.unwrap();
        assert!(!vm.raw.is_null());
        let wasm_name = "../../tests/data/add.wasm";
        let func_name = "add";
        let params = vec![WasmEdgeValueGenI32(2), WasmEdgeValueGenI32(8)];
        let mut buf: [mem::MaybeUninit<WasmEdgeValue>; 1] = mem::MaybeUninit::uninit_array();

        let result = vm.run_wasm_from_file(wasm_name, func_name, params.as_slice(), &mut buf);
        assert!(result.is_ok());
        let values = result.unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(WasmEdgeValueGetI32(values[0]), 10);
    }

    #[test]
    fn test_context_vm_basic() {
        let mut conf = ConfigureContext::create();
        conf.add_proposal(WasmEdgeProposal::WasmEdge_Proposal_ReferenceTypes);
        conf.add_host_registration(HostRegistration::WasmEdge_HostRegistration_Wasi);
        let mut store = StoreContext::create();
        let mut imp_obj = create_extern_module("extern", false);

        // WASM from file
        let result = std::fs::read(TPATH);
        assert!(result.is_ok());
        let buf = result.unwrap();
        assert!(buf.len() > 0);

        // Load and validate to wasm AST
        let result = load_module(&conf);
        assert!(result.is_some());
        let mut ast_mod = result.unwrap();
        assert!(!ast_mod.raw.is_null());
        assert!(validate_module(&conf, &ast_mod));
    }

    fn create_extern_module(name: &str, is_wrap: bool) -> Option<ImportObjectContext<'_>> {
        // create import object
        let result = ImportObjectContext::create(name, ptr::null_mut());
        assert!(result.is_some());
        let mut imp_obj = result.unwrap();

        let params = [
            WasmEdgeValType::WasmEdge_ValType_ExternRef,
            WasmEdgeValType::WasmEdge_ValType_I32,
        ];
        let returns = [WasmEdgeValType::WasmEdge_ValType_I32];
        let result = FunctionTypeContext::create(Some(&params), Some(&returns));
        assert!(result.is_some());
        let host_ftype = result.unwrap();
        assert!(!host_ftype.raw.is_null());

        // add host function "func-add"
        let host_name = "func-add";
        let result = if is_wrap {
            // WasmEdgeHostFunctionContext::create_binding(
            //     &host_ftype,
            //     extern_wrap,
            //     &extern_add as *mut std::os::raw::c_int as *mut std::os::raw::c_void,
            //     0,
            // )
            todo!()
        } else {
            HostFunctionContext::create(&host_ftype, Some(extern_add_vm), 0)
        };
        assert!(result.is_some());
        let mut host_func = result.unwrap();
        imp_obj.add_host_function(host_name, &mut host_func);

        // add host function "func-sub"
        let host_name = "func-sub";
        let result = if is_wrap {
            // WasmEdgeHostFunctionContext::create_binding(
            //     &host_ftype,
            //     extern_wrap,
            //     &extern_add as *mut std::os::raw::c_int as *mut std::os::raw::c_void,
            //     0,
            // )
            todo!()
        } else {
            HostFunctionContext::create(&host_ftype, Some(extern_sub_vm), 0)
        };
        assert!(result.is_some());
        let mut host_func = result.unwrap();
        imp_obj.add_host_function(host_name, &mut host_func);

        // add host function "func-mul"
        let host_name = "func-mul";
        let result = if is_wrap {
            // WasmEdgeHostFunctionContext::create_binding(
            //     &host_ftype,
            //     extern_wrap,
            //     &extern_add as *mut std::os::raw::c_int as *mut std::os::raw::c_void,
            //     0,
            // )
            todo!()
        } else {
            HostFunctionContext::create(&host_ftype, Some(extern_mul_vm), 0)
        };
        assert!(result.is_some());
        let mut host_func = result.unwrap();
        imp_obj.add_host_function(host_name, &mut host_func);

        // add host function "func-div"
        let host_name = "func-div";
        let result = if is_wrap {
            // WasmEdgeHostFunctionContext::create_binding(
            //     &host_ftype,
            //     extern_wrap,
            //     &extern_add as *mut std::os::raw::c_int as *mut std::os::raw::c_void,
            //     0,
            // )
            todo!()
        } else {
            HostFunctionContext::create(&host_ftype, Some(extern_div_vm), 0)
        };
        assert!(result.is_some());
        let mut host_func = result.unwrap();
        imp_obj.add_host_function(host_name, &mut host_func);

        let params = [
            WasmEdgeValType::WasmEdge_ValType_ExternRef,
            WasmEdgeValType::WasmEdge_ValType_I32,
        ];
        let returns = [WasmEdgeValType::WasmEdge_ValType_I32];
        let result = FunctionTypeContext::create(None, Some(&returns));
        assert!(result.is_some());
        let host_ftype = result.unwrap();
        assert!(!host_ftype.raw.is_null());

        // add host function "func-term"
        let host_name = "func-term";
        let result = if is_wrap {
            // WasmEdgeHostFunctionContext::create_binding(
            //     &host_ftype,
            //     extern_wrap,
            //     &extern_add as *mut std::os::raw::c_int as *mut std::os::raw::c_void,
            //     0,
            // )
            todo!()
        } else {
            HostFunctionContext::create(&host_ftype, Some(extern_term_vm), 0)
        };
        assert!(result.is_some());
        let mut host_func = result.unwrap();
        imp_obj.add_host_function(host_name, &mut host_func);

        // add host function "func-fail"
        let host_name = "func-fail";
        let result = if is_wrap {
            // WasmEdgeHostFunctionContext::create_binding(
            //     &host_ftype,
            //     extern_wrap,
            //     &extern_add as *mut std::os::raw::c_int as *mut std::os::raw::c_void,
            //     0,
            // )
            todo!()
        } else {
            HostFunctionContext::create(&host_ftype, Some(extern_fail_vm), 0)
        };
        assert!(result.is_some());
        let mut host_func = result.unwrap();
        imp_obj.add_host_function(host_name, &mut host_func);

        Some(imp_obj)
    }

    fn load_module(conf: &ConfigureContext) -> Option<ASTModuleContext> {
        let mut module = ASTModuleContext::default();
        let mut loader = LoaderContext::create(conf)?;
        loader.parse_from_file(&mut module, TPATH);
        Some(module)
    }

    fn validate_module(conf: &ConfigureContext, ast_mod: &ASTModuleContext) -> bool {
        let res = Validator::create(conf);
        match res {
            None => false,
            Some(mut validator) => {
                let res = validator.validate(ast_mod);
                match res {
                    Err(_) => false,
                    Ok(_) => true,
                }
            }
        }
    }

    #[no_mangle]
    unsafe extern "C" fn extern_add_vm(
        data: *mut std::os::raw::c_void,
        mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
        params: *const WasmEdgeValue,
        returns: *mut WasmEdgeValue,
    ) -> we_ffi::WasmEdge_Result {
        let params = std::slice::from_raw_parts(params, 2);
        let val1 = *(WasmEdgeValueGetExternRef(params[0]) as *const ::std::os::raw::c_int);
        let val2 = WasmEdgeValueGetI32(params[1]);
        let res = WasmEdgeValueGenI32(val1 + val2);
        returns.write(res);

        we_ffi::WasmEdge_Result { Code: 0 }
    }

    #[no_mangle]
    unsafe extern "C" fn extern_sub_vm(
        data: *mut std::os::raw::c_void,
        mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
        params: *const WasmEdgeValue,
        returns: *mut WasmEdgeValue,
    ) -> we_ffi::WasmEdge_Result {
        let params = std::slice::from_raw_parts(params, 2);
        let val1 = *(WasmEdgeValueGetExternRef(params[0]) as *const ::std::os::raw::c_int);
        let val2 = WasmEdgeValueGetI32(params[1]);
        let res = WasmEdgeValueGenI32(val1 - val2);
        returns.write(res);

        we_ffi::WasmEdge_Result { Code: 0 }
    }

    #[no_mangle]
    unsafe extern "C" fn extern_mul_vm(
        data: *mut std::os::raw::c_void,
        mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
        params: *const WasmEdgeValue,
        returns: *mut WasmEdgeValue,
    ) -> we_ffi::WasmEdge_Result {
        let params = std::slice::from_raw_parts(params, 2);
        let val1 = *(WasmEdgeValueGetExternRef(params[0]) as *const ::std::os::raw::c_int);
        let val2 = WasmEdgeValueGetI32(params[1]);
        let res = WasmEdgeValueGenI32(val1 * val2);
        returns.write(res);

        we_ffi::WasmEdge_Result { Code: 0 }
    }

    #[no_mangle]
    unsafe extern "C" fn extern_div_vm(
        data: *mut std::os::raw::c_void,
        mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
        params: *const WasmEdgeValue,
        returns: *mut WasmEdgeValue,
    ) -> we_ffi::WasmEdge_Result {
        let params = std::slice::from_raw_parts(params, 2);
        let val1 = *(WasmEdgeValueGetExternRef(params[0]) as *const ::std::os::raw::c_int);
        let val2 = WasmEdgeValueGetI32(params[1]);
        let res = WasmEdgeValueGenI32(val1 / val2);
        returns.write(res);

        we_ffi::WasmEdge_Result { Code: 0 }
    }

    #[no_mangle]
    unsafe extern "C" fn extern_term_vm(
        data: *mut std::os::raw::c_void,
        mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
        params: *const WasmEdgeValue,
        returns: *mut WasmEdgeValue,
    ) -> we_ffi::WasmEdge_Result {
        let res = WasmEdgeValueGenI32(1234);
        returns.write(res);

        we_ffi::WasmEdge_Result { Code: 1 }
    }

    #[no_mangle]
    unsafe extern "C" fn extern_fail_vm(
        data: *mut std::os::raw::c_void,
        mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
        params: *const WasmEdgeValue,
        returns: *mut WasmEdgeValue,
    ) -> we_ffi::WasmEdge_Result {
        let res = WasmEdgeValueGenI32(5678);
        returns.write(res);

        we_ffi::WasmEdge_Result { Code: 2 }
    }
}
