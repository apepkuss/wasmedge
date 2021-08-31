use crate::utils::check;
use crate::{
    context::{
        ast::ASTModuleContext, configure::ConfigureContext, statistics::StatisticsContext,
        store::StoreContext,
    },
    types::WasmEdgeString,
};
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
        imp_obj: *const we_ffi::WasmEdge_ImportObjectContext,
    ) -> bool {
        let res = unsafe {
            check(we_ffi::WasmEdge_InterpreterRegisterImport(
                self.raw, store.raw, imp_obj,
            ))
        };
        match res {
            Err(_) => false,
            Ok(_) => true,
        }
    }

    pub fn register_ast_module(
        &mut self,
        store: &mut StoreContext,
        ast_mod: &ASTModuleContext,
        mod_name: &str,
    ) -> bool {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let res = unsafe {
            check(we_ffi::WasmEdge_InterpreterRegisterModule(
                self.raw,
                store.raw,
                ast_mod.raw,
                mod_name.raw,
            ))
        };
        match res {
            Err(_) => false,
            Ok(_) => true,
        }
    }

    pub fn instantiate(&mut self, store: &mut StoreContext, ast_mod: &ASTModuleContext) -> bool {
        let res = unsafe {
            check(we_ffi::WasmEdge_InterpreterInstantiate(
                self.raw,
                store.raw,
                ast_mod.raw,
            ))
        };
        match res {
            Err(_) => false,
            Ok(_) => true,
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
    fn test_interpreter() {
        // register import object
        let result = create_extern_module("extern_interp", false);
        assert!(result.is_some());
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

        let param = [
            WasmEdgeValType::WasmEdge_ValType_ExternRef,
            WasmEdgeValType::WasmEdge_ValType_I32,
        ];
        let result = [WasmEdgeValType::WasmEdge_ValType_I32];
        let host_ftype = FunctionTypeContext::create(Some(&param), &result);

        // add host function "func-add-interp"
        let host_name = "func-add-interp";
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
