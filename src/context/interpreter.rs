use crate::error::WasmEdgeResult;
use crate::utils::check;
use crate::{
    context::{
        ast::ASTModuleContext, configure::ConfigureContext, import_object::ImportObjectContext,
        statistics::StatisticsContext, store::StoreContext,
    },
    types::{WasmEdgeString, WasmEdgeValue},
};
use std::mem;
use std::ptr;
use wasmedge_sys::ffi as we_ffi;

pub struct InterpreterContext {
    raw: *mut we_ffi::WasmEdge_InterpreterContext,
}
impl InterpreterContext {
    pub fn create(
        conf: Option<&ConfigureContext>,
        stat: Option<&mut StatisticsContext>,
    ) -> Option<Self> {
        let conf = match conf {
            Some(conf) => conf.raw,
            None => ptr::null(),
        };
        let stat = match stat {
            Some(stat) => stat.raw,
            None => ptr::null_mut(),
        };
        let raw = unsafe { we_ffi::WasmEdge_InterpreterCreate(conf, stat) };
        match raw.is_null() {
            true => None,
            false => Some(InterpreterContext { raw }),
        }
    }

    pub fn register_import_object_module(
        &mut self,
        store: &mut StoreContext,
        imp_obj: &ImportObjectContext,
    ) -> WasmEdgeResult<()> {
        unsafe {
            check(we_ffi::WasmEdge_InterpreterRegisterImport(
                self.raw,
                store.raw,
                imp_obj.raw,
            ))
        }
    }

    pub fn register_ast_module(
        &mut self,
        store: &mut StoreContext,
        ast_mod: &ASTModuleContext,
        mod_name: &str,
    ) -> WasmEdgeResult<()> {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        unsafe {
            check(we_ffi::WasmEdge_InterpreterRegisterModule(
                self.raw,
                store.raw,
                ast_mod.raw,
                mod_name.raw,
            ))
        }
    }

    pub fn instantiate(
        &mut self,
        store: &mut StoreContext,
        ast_mod: &ASTModuleContext,
    ) -> WasmEdgeResult<()> {
        unsafe {
            check(we_ffi::WasmEdge_InterpreterInstantiate(
                self.raw,
                store.raw,
                ast_mod.raw,
            ))
        }
    }

    /// Invoke a WASM function by name.
    pub fn invoke<'a>(
        &self,
        store: &mut StoreContext,
        func_name: &str,
        params: Option<&[WasmEdgeValue]>,
        buf: &'a mut [mem::MaybeUninit<WasmEdgeValue>],
    ) -> WasmEdgeResult<&'a [WasmEdgeValue]> {
        let func_name = WasmEdgeString::from_str(func_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());
        let (len, params) = match params {
            Some(params) => (params.len(), params.as_ptr()),
            None => (0, std::ptr::null()),
        };

        unsafe {
            check(we_ffi::WasmEdge_InterpreterInvoke(
                self.raw,
                store.raw,
                func_name.raw,
                params as *const _,
                len as u32,
                buf.as_mut_ptr() as *mut _,
                buf.len() as u32,
            ))?;

            Ok(mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()]))
        }
    }

    pub fn invoke_registered<'a>(
        &self,
        store: &mut StoreContext,
        mod_name: &str,
        func_name: &str,
        params: Option<&[WasmEdgeValue]>,
        buf: &'a mut [mem::MaybeUninit<WasmEdgeValue>],
    ) -> WasmEdgeResult<&'a [WasmEdgeValue]> {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let func_name = WasmEdgeString::from_str(func_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());

        let (len, params) = match params {
            Some(params) => (params.len(), params.as_ptr()),
            None => (0, std::ptr::null()),
        };

        unsafe {
            check(we_ffi::WasmEdge_InterpreterInvokeRegistered(
                self.raw,
                store.raw,
                mod_name.raw,
                func_name.raw,
                params as *const _,
                len as u32,
                buf.as_mut_ptr() as *mut _,
                buf.len() as u32,
            ))?;

            Ok(mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()]))
        }
    }
}
impl Drop for InterpreterContext {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_InterpreterDelete(self.raw) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{
        configure::ConfigureContext, import_object::ImportObjectContext, loader::LoaderContext,
        statistics::StatisticsContext, store::StoreContext, validator::Validator,
    };
    use crate::instance::function::{FunctionTypeContext, HostFunctionContext};
    use crate::types::*;
    use crate::value::*;

    const TPATH: &str = "/root/workspace/wasmedge-ml/wasmedge/tests/data/test.wasm";

    #[test]
    fn test_interpreter_create() {
        let mut conf = ConfigureContext::create();
        conf.add_proposal(WasmEdgeProposal::WasmEdge_Proposal_ReferenceTypes);

        // load and validate file
        let result = load_module(&conf);
        assert!(result.is_some());
        let ast_mod = result.unwrap();
        assert!(!ast_mod.raw.is_null());
        assert!(validate_module(&conf, &ast_mod));

        // statistics creation and deletion
        let result = StatisticsContext::create();
        assert!(result.is_some());
        let mut stat = result.unwrap();
        assert!(!stat.raw.is_null());

        // Statistics set cost table
        let mut cost_table = vec![20u64; 512];
        stat.set_cost_table(&mut cost_table);

        // Statistics set cost limit
        let limit = 100000000000000u64;
        stat.set_cost_limit(limit);

        // Interpreter creation
        let result = InterpreterContext::create(None, None);
        assert!(result.is_some());
        let result = InterpreterContext::create(Some(&conf), None);
        assert!(result.is_some());
        let result = InterpreterContext::create(None, Some(&mut stat));
        assert!(result.is_some());
        let result = InterpreterContext::create(Some(&conf), Some(&mut stat));
        assert!(result.is_some());
    }

    #[test]
    fn test_interpreter_register() {
        let mut conf = ConfigureContext::create();
        conf.add_proposal(WasmEdgeProposal::WasmEdge_Proposal_ReferenceTypes);

        // load and validate file
        let result = load_module(&conf);
        assert!(result.is_some());
        let ast_mod = result.unwrap();
        assert!(!ast_mod.raw.is_null());
        assert!(validate_module(&conf, &ast_mod));

        // statistics creation and deletion
        let result = StatisticsContext::create();
        assert!(result.is_some());
        let mut stat = result.unwrap();
        assert!(!stat.raw.is_null());

        // Statistics set cost table
        let mut cost_table = vec![20u64; 512];
        stat.set_cost_table(&mut cost_table);

        // Statistics set cost limit
        let limit = 100000000000000u64;
        stat.set_cost_limit(limit);

        // Interpreter creation
        let result = InterpreterContext::create(Some(&conf), Some(&mut stat));
        assert!(result.is_some());
        let mut interp = result.unwrap();
        assert!(!interp.raw.is_null());

        // register import object
        let result = create_extern_module("extern", false);
        assert!(result.is_some());
        let imp_obj = result.unwrap();
        assert!(!imp_obj.raw.is_null());
        // let result = create_extern_module("extern_interp", false); // ! error: module name conflict
        // assert!(result.is_some());
        // let imp_obj2 = result.unwrap();

        let mut store = StoreContext::create();
        assert!(interp
            .register_import_object_module(&mut store, &imp_obj)
            .is_ok());
        // assert!(interp
        //     .register_import_object_module(&mut store, &imp_obj2)
        //     .is_err());

        // register wasm module
        assert!(interp
            .register_ast_module(&mut store, &ast_mod, "module")
            .is_ok());

        // instantiate wasm module
        assert!(interp.instantiate(&mut store, &ast_mod).is_ok());
        // override instantiated wasm
        assert!(interp.instantiate(&mut store, &ast_mod).is_ok());

        // invoke functions
        let params = [WasmEdgeValueGenI32(123), WasmEdgeValueGenI32(456)];
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        let result = interp.invoke(&mut store, "func-mul-2", Some(&params), &mut buf);
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(246, WasmEdgeValueGetI32(returns[0]));
        assert_eq!(912, WasmEdgeValueGetI32(returns[1]));

        // ? Function type mismatch
        // let params = [WasmEdgeValueGenI64(123), WasmEdgeValueGenI32(456)];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // let result = interp.invoke(&mut store, "func-mul-2", Some(&params), &mut buf);
        // assert!(result.is_err());

        // ? Function not found
        // let params = [WasmEdgeValueGenI32(123), WasmEdgeValueGenI32(456)];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // let result = interp.invoke(&mut store, "func-mul-3", Some(&params), &mut buf);
        // assert!(result.is_err());

        // Discard result
        let params = [WasmEdgeValueGenI32(123), WasmEdgeValueGenI32(456)];
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<0>();
        let result = interp.invoke(&mut store, "func-mul-2", Some(&params), &mut buf);
        assert!(result.is_ok());
        let params = [WasmEdgeValueGenI32(123), WasmEdgeValueGenI32(456)];
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<1>();
        let result = interp.invoke(&mut store, "func-mul-2", Some(&params), &mut buf);
        assert!(result.is_ok());

        // ? Invoke functions call to host functions
        // // Get table and set external reference
        // let result = store.find_table("tab-ext");
        // assert!(result.is_some());
        // let mut table = result.unwrap();
        // assert!(!table.raw.is_null());

        // Invoke functions of registered module
        let mod_name = "extern";
        let mod_name2 = "error-name";
        let func_name = "func-add";
        let func_name2 = "func-add2";
        let mut test_value = 5000;
        let params = [
            WasmEdgeValueGenExternRef((&mut test_value) as *mut i32 as *mut std::os::raw::c_void),
            WasmEdgeValueGenI32(1500),
        ];
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<1>();
        let result =
            interp.invoke_registered(&mut store, mod_name, func_name, Some(&params), &mut buf);
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(6500i32, WasmEdgeValueGetI32(returns[0]));

        // ? Module not found
        // let result = interp.invoke_registered(&mut store, mod_name2, func_name, &params, &mut buf);
        // assert!(result.is_err());

        // ? Function not found
        // let result = interp.invoke_registered(&mut store, mod_name, func_name2, &params, &mut buf);
        // assert!(result.is_err());

        // Discard result
        let params = [
            WasmEdgeValueGenExternRef((&mut test_value) as *mut i32 as *mut std::os::raw::c_void),
            WasmEdgeValueGenI32(1500),
        ];
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<0>();
        let result =
            interp.invoke_registered(&mut store, mod_name, func_name, Some(&params), &mut buf);
        assert!(result.is_ok());

        // Invoke host function to terminate execution
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<1>();
        let result = interp.invoke_registered(&mut store, "extern", "func-term", None, &mut buf);
        assert!(result.is_ok());

        // ? Invoke host function to fail execution
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<1>();
        // let result = interp.invoke_registered(&mut store, "extern", "func-fail", None, &mut buf);
        // assert!(result.is_err());

        // ? Invoke host function with binding to functions
        // let mod_name = "extern-wrap";
        // let mut test_value = 1234;
        // let params = [
        //     WasmEdgeValueGenExternRef((&mut test_value) as *mut i32 as *mut std::os::raw::c_void),
        //     WasmEdgeValueGenI32(1500),
        // ];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<1>();
        // let result =
        //     interp.invoke_registered(&mut store, mod_name, "func-sub", Some(&params), &mut buf);
        // assert!(result.is_ok());
        // let returns = result.unwrap();
        // assert_eq!(-266i32, WasmEdgeValueGetI32(returns[0]));
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<1>();
        // let result = interp.invoke_registered(&mut store, mod_name, "func-term", None, &mut buf);
        // assert!(result.is_ok());
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<1>();
        // let result = interp.invoke_registered(&mut store, mod_name, "func-fail", None, &mut buf);
        // assert!(result.is_err());

        // Statistics get instruction count
        assert!(stat.get_instr_count() > 0);

        // Statistics get instruction per second
        assert!(stat.get_instr_per_second() > 0);

        // Statistics get total cost
        assert!(stat.get_total_cost() > 0);
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
            HostFunctionContext::create(&host_ftype, Some(extern_add_interp), 0)
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
            HostFunctionContext::create(&host_ftype, Some(extern_sub_interp), 0)
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
            HostFunctionContext::create(&host_ftype, Some(extern_mul_interp), 0)
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
            HostFunctionContext::create(&host_ftype, Some(extern_div_interp), 0)
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
            HostFunctionContext::create(&host_ftype, Some(extern_term_interp), 0)
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
            HostFunctionContext::create(&host_ftype, Some(extern_fail_interp), 0)
        };
        assert!(result.is_some());
        let mut host_func = result.unwrap();
        imp_obj.add_host_function(host_name, &mut host_func);

        Some(imp_obj)
    }

    #[no_mangle]
    unsafe extern "C" fn extern_add_interp(
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
    unsafe extern "C" fn extern_sub_interp(
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
    unsafe extern "C" fn extern_mul_interp(
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
    unsafe extern "C" fn extern_div_interp(
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
    unsafe extern "C" fn extern_term_interp(
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
    unsafe extern "C" fn extern_fail_interp(
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
