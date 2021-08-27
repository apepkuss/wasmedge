use crate::{
    context::vm::VMContext,
    error::WasmEdgeResult,
    instance::{function::FunctionInstanceContext, memory::MemoryInstanceContext},
    types::WasmEdgeString,
    types::WasmEdgeValue,
    utils::check,
};
use std::marker::PhantomData;
use std::mem;
use wasmedge_sys::ffi as we_ffi;

pub struct StoreContext<'vm> {
    pub(crate) raw: *mut we_ffi::WasmEdge_StoreContext,
    pub(crate) _marker: PhantomData<&'vm VMContext>,
}
impl<'vm> StoreContext<'vm> {
    pub fn create() -> StoreContext<'vm> {
        StoreContext {
            raw: unsafe { we_ffi::WasmEdge_StoreCreate() },
            _marker: PhantomData,
        }
    }

    pub fn find_function(&self, func_name: &str) -> Option<FunctionInstanceContext> {
        let func_name = WasmEdgeString::from_str(func_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());
        let raw = unsafe { we_ffi::WasmEdge_StoreFindFunction(self.raw, func_name.raw) };
        match raw.is_null() {
            true => None,
            false => Some(FunctionInstanceContext {
                raw,
                _marker: PhantomData,
            }),
        }
    }

    pub fn find_memory(&self, mem_name: &str) -> MemoryInstanceContext {
        let mem_name = WasmEdgeString::from_str(mem_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mem_name).as_str());
        let mem = unsafe { we_ffi::WasmEdge_StoreFindMemory(self.raw, mem_name.raw) };
        MemoryInstanceContext {
            raw: mem,
            _marker: PhantomData,
        }
    }

    pub fn find_memory_registered(&self, mod_name: &str, mem_name: &str) -> MemoryInstanceContext {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let mem_name = WasmEdgeString::from_str(mem_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mem_name).as_str());
        let mem = unsafe {
            we_ffi::WasmEdge_StoreFindMemoryRegistered(self.raw, mod_name.raw, mem_name.raw)
        };
        MemoryInstanceContext {
            raw: mem,
            _marker: PhantomData,
        }
    }

    pub fn list_function_len(&self) -> usize {
        unsafe { we_ffi::WasmEdge_StoreListFunctionLength(self.raw) as usize }
    }

    pub fn list_function<'a>(
        &self,
        names: &'a mut [mem::MaybeUninit<WasmEdgeString>],
    ) -> (usize, &'a [WasmEdgeString]) {
        unsafe {
            let len = we_ffi::WasmEdge_StoreListFunction(
                self.raw,
                names.as_ptr() as *mut _,
                names.len() as u32,
            );

            (
                len as usize,
                mem::MaybeUninit::slice_assume_init_ref(&names[..names.len()]),
            )
        }
    }

    pub fn list_function_registered_len(&self, mod_name: &str) -> usize {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        unsafe {
            we_ffi::WasmEdge_StoreListFunctionRegisteredLength(self.raw, mod_name.raw) as usize
        }
    }

    pub fn list_table_len(&self) -> usize {
        unsafe { we_ffi::WasmEdge_StoreListTableLength(self.raw) as usize }
    }

    pub fn list_table_registered_len(&self, mod_name: &str) -> usize {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        unsafe { we_ffi::WasmEdge_StoreListTableRegisteredLength(self.raw, mod_name.raw) as usize }
    }

    pub fn list_global_len(&self) -> usize {
        unsafe { we_ffi::WasmEdge_StoreListGlobalLength(self.raw) as usize }
    }

    pub fn list_global_registered_len(&self, mod_name: &str) -> usize {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        unsafe { we_ffi::WasmEdge_StoreListGlobalRegisteredLength(self.raw, mod_name.raw) as usize }
    }

    pub fn list_memory_len(&self) -> usize {
        let len = unsafe { we_ffi::WasmEdge_StoreListMemoryLength(self.raw) };
        len as usize
    }

    pub fn list_memory(
        &self,
        buf: &mut [mem::MaybeUninit<we_ffi::WasmEdge_String>],
    ) -> (usize, Vec<String>) {
        let len = unsafe {
            we_ffi::WasmEdge_StoreListMemory(
                self.raw,
                buf.as_mut_ptr() as *mut we_ffi::WasmEdge_String,
                buf.len() as u32,
            )
        };
        let s_vec = unsafe { mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()]) };
        let mut names = vec![];
        for s in s_vec {
            let str = WasmEdgeString { raw: *s };
            let cow = str.to_string_lossy();
            names.push(cow.into_owned())
        }
        (len as usize, names)
    }

    pub fn list_memory_registered_len(&self, mod_name: &str) -> usize {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let len =
            unsafe { we_ffi::WasmEdge_StoreListMemoryRegisteredLength(self.raw, mod_name.raw) };
        len as usize
    }

    pub fn list_memory_registered(
        &self,
        mod_name: &str,
        buf: &mut [mem::MaybeUninit<we_ffi::WasmEdge_String>],
    ) -> (usize, Vec<String>) {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let len = unsafe {
            we_ffi::WasmEdge_StoreListMemoryRegistered(
                self.raw,
                mod_name.raw,
                buf.as_mut_ptr() as *mut we_ffi::WasmEdge_String,
                buf.len() as u32,
            )
        };
        let s_vec = unsafe { mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()]) };
        let mut names = vec![];
        for s in s_vec {
            let str = WasmEdgeString { raw: *s };
            let cow = str.to_string_lossy();
            names.push(cow.into_owned())
        }
        (len as usize, names)
    }

    pub fn list_module_len(&self) -> usize {
        let len = unsafe { we_ffi::WasmEdge_StoreListModuleLength(self.raw) };
        len as usize
    }
}
impl<'vm> Drop for StoreContext<'vm> {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_StoreDelete(self.raw) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        context::{
            ast::ASTModuleContext, configure::ConfigureContext, interpreter::InterpreterContext,
            loader::LoaderContext, validator::Validator,
        },
        instance::function::FunctionTypeContext,
        types::*,
        value::*,
    };
    use std::ffi::CString;
    use std::ptr;
    use wasmedge_sys::ffi as we_ffi;

    const TPATH: &str = "/root/workspace/wasmedge-ml/wasmedge/tests/data/test.wasm";

    #[test]
    fn test_wasmedge_store() {
        // create contexts
        let mut conf = ConfigureContext::create();
        conf.add_host_registration(HostRegistration::WasmEdge_HostRegistration_Wasi);
        conf.add_host_registration(HostRegistration::WasmEdge_HostRegistration_WasmEdge_Process);
        conf.add_proposal(WasmEdgeProposal::WasmEdge_Proposal_ReferenceTypes);
        let mut store = StoreContext::create();
        let mod_name = vec!["module", "extern", "no-such-module"];
        let err_name = "invalid-instance-name";

        // Store list exports before instantiation
        assert_eq!(store.list_function_len(), 0);
        assert_eq!(store.list_table_len(), 0);
        assert_eq!(store.list_memory_len(), 0);
        assert_eq!(store.list_global_len(), 0);
        assert_eq!(store.list_function_registered_len(mod_name[1]), 0);
        assert_eq!(store.list_table_registered_len(mod_name[1]), 0);
        assert_eq!(store.list_memory_registered_len(mod_name[1]), 0);
        assert_eq!(store.list_global_registered_len(mod_name[1]), 0);
        assert_eq!(store.list_module_len(), 0);

        // Register host module and instantiate wasm module
        let res = create_extern_module("extern", false);
        assert!(res.is_some());
        let imp_obj = res.unwrap();
        assert!(!imp_obj.is_null());
        let res = load_module(&conf);
        assert!(res.is_some());
        let mut ast_mod = res.unwrap();
        assert!(!ast_mod.raw.is_null());
        assert!(validate_module(&conf, &ast_mod));
        assert!(instantiate_module(&conf, &mut store, &ast_mod, imp_obj));

        // Store list function exports
        assert_eq!(store.list_function_len(), 11);
        let mut names = mem::MaybeUninit::uninit_array::<4>();
        let (len, names) = store.list_function(&mut names);
        assert_eq!(len, 11);
        for name in names.into_iter() {
            drop(name);
        }
        let mut names = mem::MaybeUninit::uninit_array::<15>();
        let (len, names) = store.list_function(&mut names);
        assert_eq!(len, 11);

        // store find function
        let res = store.find_function(names[7].to_string_lossy().into_owned().as_str());
        assert!(res.is_none());

        // unsafe { we_ffi::WasmEdge_ImportObjectDelete(imp_obj) };
    }

    fn create_extern_module(
        name: &str,
        is_wrap: bool,
    ) -> Option<*mut we_ffi::WasmEdge_ImportObjectContext> {
        unsafe {
            // let host_name = WasmEdgeString::from_str(name).unwrap().into_raw();
            let x = CString::new(name).unwrap();
            let host_name = we_ffi::WasmEdge_StringCreateByCString(x.as_ptr());
            let imp_obj = we_ffi::WasmEdge_ImportObjectCreate(host_name, ptr::null_mut());

            let param = [
                WasmEdgeValType::WasmEdge_ValType_ExternRef,
                WasmEdgeValType::WasmEdge_ValType_I32,
            ];
            let result = [WasmEdgeValType::WasmEdge_ValType_I32];
            let host_ftype = FunctionTypeContext::create(Some(&param), &result);

            // Add host function "func-add"
            let host_func =
                we_ffi::WasmEdge_HostFunctionCreate(host_ftype.raw, Some(extern_add), 0);
            // let res = HostFunctionContext::create(host_ftype, Some(extern_add), 0);
            // assert!(res.is_some());
            // let host_func = res.unwrap();
            // println!("host_func.raw: {:?}", host_func.raw);
            let host_name = WasmEdgeString::from_str("func-add").unwrap();
            we_ffi::WasmEdge_ImportObjectAddHostFunction(imp_obj, host_name.raw, host_func);

            // add host function "func-sub"
            let host_func =
                we_ffi::WasmEdge_HostFunctionCreate(host_ftype.raw, Some(extern_sub), 0);
            let host_name = WasmEdgeString::from_str("func-sub").unwrap();
            we_ffi::WasmEdge_ImportObjectAddHostFunction(imp_obj, host_name.raw, host_func);

            // add host function "func-mul"
            let host_func =
                we_ffi::WasmEdge_HostFunctionCreate(host_ftype.raw, Some(extern_mul), 0);
            let host_name = WasmEdgeString::from_str("func-mul").unwrap();
            we_ffi::WasmEdge_ImportObjectAddHostFunction(imp_obj, host_name.raw, host_func);

            // add host function "func-div"
            let host_func =
                we_ffi::WasmEdge_HostFunctionCreate(host_ftype.raw, Some(extern_div), 0);
            let host_name = WasmEdgeString::from_str("func-div").unwrap();
            we_ffi::WasmEdge_ImportObjectAddHostFunction(imp_obj, host_name.raw, host_func);

            let param = [
                WasmEdgeValType::WasmEdge_ValType_ExternRef,
                WasmEdgeValType::WasmEdge_ValType_I32,
            ];
            let result = [WasmEdgeValType::WasmEdge_ValType_I32];
            let host_ftype = FunctionTypeContext::create(None, &result);

            // add host function "func-term"
            let host_func =
                we_ffi::WasmEdge_HostFunctionCreate(host_ftype.raw, Some(extern_term), 0);
            let host_name = WasmEdgeString::from_str("func-term").unwrap();
            we_ffi::WasmEdge_ImportObjectAddHostFunction(imp_obj, host_name.raw, host_func);

            // add host function "func-fail"
            let host_func =
                we_ffi::WasmEdge_HostFunctionCreate(host_ftype.raw, Some(extern_fail), 0);
            let host_name = WasmEdgeString::from_str("func-fail").unwrap();
            we_ffi::WasmEdge_ImportObjectAddHostFunction(imp_obj, host_name.raw, host_func);

            Some(imp_obj)
        }
        // // create import object
        // let mut imp_obj = ImportObjectContext::create(name, ptr::null_mut())?;

        // let param = [
        //     WasmEdgeValType::WasmEdge_ValType_ExternRef,
        //     WasmEdgeValType::WasmEdge_ValType_I32,
        // ];
        // let result = [WasmEdgeValType::WasmEdge_ValType_I32];
        // let host_ftype = WasmEdgeFunctionTypeContext::create(&param, &result);

        // // add host function "func-add"
        // let host_name = "func-add";
        // let mut host_func: WasmEdgeHostFunctionContext = if is_wrap {
        //     // WasmEdgeHostFunctionContext::create_binding(
        //     //     &host_ftype,
        //     //     extern_wrap,
        //     //     &extern_add as *mut std::os::raw::c_int as *mut std::os::raw::c_void,
        //     //     0,
        //     // )
        //     todo!()
        // } else {
        //     WasmEdgeHostFunctionContext::create(&host_ftype, Some(extern_add), 0)
        // };
        // imp_obj.add_host_function(host_name, &mut host_func);
        // Some(imp_obj)
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

    fn instantiate_module(
        conf: &ConfigureContext,
        store: &mut StoreContext,
        ast_mod: &ASTModuleContext,
        imp_obj: *const we_ffi::WasmEdge_ImportObjectContext,
    ) -> bool {
        let res = InterpreterContext::create(Some(conf), None);
        if res.is_none() {
            return false;
        }
        let mut interp = res.unwrap();
        if !interp.register_import_object_module(store, imp_obj) {
            return false;
        }
        if !interp.register_ast_module(store, ast_mod, "module") {
            return false;
        }
        if !interp.instantiate(store, ast_mod) {
            return false;
        }
        true
    }

    #[no_mangle]
    unsafe extern "C" fn extern_add(
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
    unsafe extern "C" fn extern_sub(
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
    unsafe extern "C" fn extern_mul(
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
    unsafe extern "C" fn extern_div(
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
    unsafe extern "C" fn extern_term(
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
    unsafe extern "C" fn extern_fail(
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
