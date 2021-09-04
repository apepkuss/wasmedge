use crate::{
    context::{
        ast::ASTModuleContext, configure::ConfigureContext, import_object::ImportObjectContext,
        statistics::StatisticsContext, store::StoreContext,
    },
    error::WasmEdgeResult,
    instance::function::FunctionTypeContext,
    types::*,
    utils::{check, path_to_cstring},
};
use std::ffi::CStr;
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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> WasmEdgeResult<()> {
        let path = path_to_cstring(path.as_ref())?;
        unsafe { check(we_ffi::WasmEdge_VMLoadWasmFromFile(self.raw, path.as_ptr())) }
    }

    pub fn load_from_buffer(&mut self, buf: &[u8]) -> WasmEdgeResult<()> {
        unsafe {
            check(we_ffi::WasmEdge_VMLoadWasmFromBuffer(
                self.raw,
                buf.as_ptr() as *const _,
                buf.len() as u32,
            ))
        }
    }

    pub fn load_from_ast(&mut self, ast_mod: &ASTModuleContext) -> WasmEdgeResult<()> {
        unsafe {
            check(we_ffi::WasmEdge_VMLoadWasmFromASTModule(
                self.raw,
                ast_mod.raw,
            ))
        }
    }

    pub fn validate(&self) -> WasmEdgeResult<()> {
        unsafe { check(we_ffi::WasmEdge_VMValidate(self.raw)) }
    }

    pub fn instantiate(&self) -> WasmEdgeResult<()> {
        unsafe { check(we_ffi::WasmEdge_VMInstantiate(self.raw)) }
    }

    pub fn cleanup(&mut self) {
        unsafe { we_ffi::WasmEdge_VMCleanup(self.raw) }
    }

    pub fn execute<'vm>(
        &self,
        func_name: &str,
        params: &[WasmEdgeValue],
        buf: &'vm mut [mem::MaybeUninit<WasmEdgeValue>],
    ) -> WasmEdgeResult<&'vm [WasmEdgeValue]> {
        let func_name = WasmEdgeString::from_str(func_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());
        unsafe {
            check(we_ffi::WasmEdge_VMExecute(
                self.raw,
                func_name.raw,
                params.as_ptr() as *const _,
                params.len() as u32,
                buf.as_mut_ptr() as *mut _,
                buf.len() as u32,
            ))?;

            Ok(mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()]))
        }
    }

    pub fn execute_registered<'vm>(
        &self,
        mod_name: &str,
        func_name: &str,
        params: &[WasmEdgeValue],
        buf: &'vm mut [mem::MaybeUninit<WasmEdgeValue>],
    ) -> WasmEdgeResult<&'vm [WasmEdgeValue]> {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let func_name = WasmEdgeString::from_str(func_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());
        unsafe {
            check(we_ffi::WasmEdge_VMExecuteRegistered(
                self.raw,
                mod_name.raw,
                func_name.raw,
                params.as_ptr() as *const _,
                params.len() as u32,
                buf.as_mut_ptr() as *mut _,
                buf.len() as u32,
            ))?;

            Ok(mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()]))
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

    pub fn register_module_from_buffer(
        &mut self,
        mod_name: &str,
        buf: &[u8],
    ) -> WasmEdgeResult<()> {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        unsafe {
            check(we_ffi::WasmEdge_VMRegisterModuleFromBuffer(
                self.raw,
                mod_name.raw,
                buf.as_ptr(),
                buf.len() as u32,
            ))
        }
    }

    pub fn register_module_from_ast(
        &mut self,
        mod_name: &str,
        ast_mod: &ASTModuleContext,
    ) -> WasmEdgeResult<()> {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        unsafe {
            check(we_ffi::WasmEdge_VMRegisterModuleFromASTModule(
                self.raw,
                mod_name.raw,
                ast_mod.raw,
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
        let func_name = WasmEdgeString::from_str(func_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());

        unsafe {
            check(we_ffi::WasmEdge_VMRunWasmFromFile(
                self.raw,
                path.as_ptr(),
                func_name.raw,
                params.as_ptr() as *const WasmEdgeValue,
                params.len() as u32,
                buf.as_mut_ptr() as *mut WasmEdgeValue,
                buf.len() as u32,
            ))?;

            Ok(mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()]))
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
            check(we_ffi::WasmEdge_VMRunWasmFromBuffer(
                self.raw,
                buf.as_ptr(),
                buf.len() as u32,
                func_name.raw,
                params.as_ptr() as *const WasmEdgeValue,
                params.len() as u32,
                returns.as_mut_ptr() as *mut WasmEdgeValue,
                returns.len() as u32,
            ))?;

            Ok(mem::MaybeUninit::slice_assume_init_ref(
                &returns[..returns.len()],
            ))
        }
    }

    pub fn run_wasm_from_ast<'vm>(
        &mut self,
        ast_mod: &ASTModuleContext,
        func_name: &str,
        params: &[WasmEdgeValue],
        buf: &'vm mut [mem::MaybeUninit<WasmEdgeValue>],
    ) -> WasmEdgeResult<&'vm [WasmEdgeValue]> {
        let func_name = WasmEdgeString::from_str(func_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());

        unsafe {
            check(we_ffi::WasmEdge_VMRunWasmFromASTModule(
                self.raw,
                ast_mod.raw,
                func_name.raw,
                params.as_ptr() as *const _,
                params.len() as u32,
                buf.as_mut_ptr() as *mut _,
                buf.len() as u32,
            ))?;

            Ok(mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()]))
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

    pub fn function_names(
        &self,
        buf: &mut [mem::MaybeUninit<we_ffi::WasmEdge_String>],
    ) -> Option<Vec<String>> {
        let max_len = self.function_list_len();
        match 0 < buf.len() && buf.len() <= max_len {
            true => unsafe {
                we_ffi::WasmEdge_VMGetFunctionList(
                    self.raw,
                    buf.as_mut_ptr() as *mut _,
                    std::ptr::null_mut(),
                    buf.len() as u32,
                );

                let names = mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()]);
                let mut s_vec = vec![];
                for s in names {
                    let slice = CStr::from_ptr(s.Buf as *const _);
                    s_vec.push(slice.to_string_lossy().into_owned());
                }
                Some(s_vec)
            },
            false => None,
        }
    }

    // pub fn function_types()

    pub fn import_object(&self, reg: HostRegistration) -> Option<ImportObjectContext> {
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

    pub fn store_context(&self) -> Option<StoreContext> {
        let raw = unsafe { we_ffi::WasmEdge_VMGetStoreContext(self.raw) };
        match raw.is_null() {
            true => None,
            false => Some(StoreContext {
                raw,
                _marker: PhantomData,
                _drop: false,
            }),
        }
    }

    pub fn statistics_context(&self) -> Option<StatisticsContext> {
        let raw = unsafe { we_ffi::WasmEdge_VMGetStatisticsContext(self.raw) };
        match raw.is_null() {
            true => None,
            false => Some(StatisticsContext {
                raw,
                _marker: PhantomData,
                _drop: false,
            }),
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
        let result = create_extern_module("extern", false);
        assert!(result.is_some());
        let imp_obj = result.unwrap();
        assert!(!imp_obj.raw.is_null());

        // WASM from file
        let result = std::fs::read(TPATH);
        assert!(result.is_ok());
        let mod_buf = result.unwrap();
        assert!(mod_buf.len() > 0);

        // Load and validate to wasm AST
        let result = load_module(&conf);
        assert!(result.is_some());
        let mut ast_mod = result.unwrap();
        assert!(!ast_mod.raw.is_null());
        assert!(validate_module(&conf, &ast_mod));

        // VM creation and deletion
        let mut result: Option<VMContext>;
        result = VMContext::create(None, None);
        assert!(result.is_some());
        result = VMContext::create(Some(&conf), None);
        assert!(result.is_some());
        result = VMContext::create(None, Some(&mut store));
        assert!(result.is_some());
        result = VMContext::create(Some(&conf), Some(&mut store));
        assert!(result.is_some());
        let mut vm = result.unwrap();
        assert!(!vm.raw.is_null());

        // VM register module from import
        let result = vm.register_module_from_import_object(&imp_obj);
        assert!(result.is_ok());
        // ! error: instantiation failed: module name conflict, Code: 0x60
        // let result = vm.register_module_from_import_object(&imp_obj);
        // assert!(result.is_err());

        // VM register module from buffer
        let result = vm.register_module_from_buffer("reg-wasm-buffer", &mod_buf);
        assert!(result.is_ok());

        // VM register module from AST module
        let result = vm.register_module_from_ast("reg-wasm-ast", &ast_mod);
        assert!(result.is_ok());
        // ! error: instantiation failed: module name conflict, Code: 0x60
        // let result = vm.register_module_from_ast("reg-wasm-ast", &ast_mod);
        // assert!(result.is_err());

        let mod_name = "reg-wasm-buffer";
        let func_name = "func-mul-2";
        let func_name2 = "func-mul-3";
        // VM run wasm from file
        let params = [WasmEdgeValueGenI32(123), WasmEdgeValueGenI32(456)];
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        let result = vm.run_wasm_from_file(TPATH, func_name, &params, &mut buf);
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(246, WasmEdgeValueGetI32(returns[0]));
        assert_eq!(912, WasmEdgeValueGetI32(returns[1]));

        // ! error: loading failed: invalid path, Code: 0x20
        // let result = vm.run_wasm_from_file("no-file", func_name, &params, &mut buf);
        // assert!(result.is_err());

        // Function not found
        // ! error: wasmedge runtime failed: wasm function not found, Code: 0x05
        // let result = vm.run_wasm_from_file(TPATH, func_name2, &params, &mut buf);
        // assert!(result.is_err());

        // Discard result
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<1>();
        let result = vm.run_wasm_from_file(TPATH, func_name, &params, &mut buf);
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(246, WasmEdgeValueGetI32(returns[0]));
        // Discard result
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<0>();
        let result = vm.run_wasm_from_file(TPATH, func_name, &params, &mut buf);
        assert!(result.is_ok());

        // VM run wasm from buffer
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        let result = vm.run_wasm_from_buffer(&mod_buf, func_name, &params, &mut buf);
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(246, WasmEdgeValueGetI32(returns[0]));
        assert_eq!(912, WasmEdgeValueGetI32(returns[1]));
        // ! error
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // let result = vm.run_wasm_from_buffer(&mod_buf, func_name, &params, &mut buf);
        // assert!(result.is_err());

        // Function type mismatch
        // ! error: execution failed: function signature mismatch, Code: 0x83
        // let params = [WasmEdgeValueGenI32(123)];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // let result = vm.run_wasm_from_buffer(&mod_buf, func_name, &params, &mut buf);
        // assert!(result.is_err());

        // Function type mismatch
        // ! error: execution failed: function signature mismatch, Code: 0x83
        // let params = [WasmEdgeValueGenI64(123), WasmEdgeValueGenI32(456)];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // let result = vm.run_wasm_from_file(TPATH, func_name, &params, &mut buf);
        // assert!(result.is_err());

        // Discard result
        let params = [WasmEdgeValueGenI32(123), WasmEdgeValueGenI32(456)];
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<1>();
        let result = vm.run_wasm_from_file(TPATH, func_name, &params, &mut buf);
        assert!(result.is_ok());
        // Discard result
        let params = [WasmEdgeValueGenI32(123), WasmEdgeValueGenI32(456)];
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<0>();
        let result = vm.run_wasm_from_file(TPATH, func_name, &params, &mut buf);
        assert!(result.is_ok());

        // VM run wasm from AST module
        let params = [WasmEdgeValueGenI32(123), WasmEdgeValueGenI32(456)];
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        let result = vm.run_wasm_from_ast(&ast_mod, func_name, &params, &mut buf);
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(246, WasmEdgeValueGetI32(returns[0]));
        assert_eq!(912, WasmEdgeValueGetI32(returns[1]));

        // Function type mismatch
        // ! error: execution failed: function signature mismatch, Code: 0x83
        // let params = [WasmEdgeValueGenI32(123)];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // let result = vm.run_wasm_from_ast(&ast_mod, func_name, &params, &mut buf);
        // assert!(result.is_err());

        // Function type mismatch
        // ! error: execution failed: function signature mismatch, Code: 0x83
        // let params = [];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // let result = vm.run_wasm_from_ast(&ast_mod, func_name, &params, &mut buf);
        // assert!(result.is_err());

        // Function type mismatch
        // ! error: execution failed: function signature mismatch, Code: 0x83
        // let params = [WasmEdgeValueGenI64(123), WasmEdgeValueGenI32(456)];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // let result = vm.run_wasm_from_ast(&ast_mod, func_name, &params, &mut buf);
        // assert!(result.is_err());

        // Function not found
        // ! error: wasmedge runtime failed: wasm function not found, Code: 0x05
        // let params = [WasmEdgeValueGenI64(123), WasmEdgeValueGenI32(456)];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // let result = vm.run_wasm_from_ast(&ast_mod, func_name2, &params, &mut buf);
        // assert!(result.is_err());

        // Discard result
        // ! error: execution failed: function signature mismatch, Code: 0x83
        // let params = [WasmEdgeValueGenI64(123), WasmEdgeValueGenI32(456)];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<1>();
        // let result = vm.run_wasm_from_ast(&ast_mod, func_name, &params, &mut buf);
        // assert!(result.is_ok());

        // VM load wasm from file
        assert!(vm.load_from_file(TPATH).is_ok());
        // ! error: loading failed: invalid path, Code: 0x20
        // assert!(vm.load_from_file("file").is_err());

        // VM load wasm from buffer
        assert!(vm.load_from_buffer(&mod_buf).is_ok());

        // VM load wasm from AST module
        assert!(vm.load_from_ast(&ast_mod).is_ok());

        // VM validate
        vm.cleanup();
        // ! error: wasmedge runtime failed: wrong VM workflow, Code: 0x04
        // assert!(vm.instantiate().is_err());
        assert!(vm.load_from_ast(&ast_mod).is_ok());
        assert!(vm.validate().is_ok());
        assert!(vm.instantiate().is_ok());

        // VM execute
        vm.cleanup();
        let params = [WasmEdgeValueGenI32(123), WasmEdgeValueGenI32(456)];
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        let result = vm.run_wasm_from_ast(&ast_mod, func_name, &params, &mut buf);
        assert!(result.is_ok());
        // Inited phase
        // ! error: should be Err
        assert!(vm.execute(func_name, &params, &mut buf).is_ok());
        // loaded phase
        assert!(vm.load_from_ast(&ast_mod).is_ok());
        // ! error: should be Err
        assert!(vm.execute(func_name, &params, &mut buf).is_ok());
        // validated phase
        assert!(vm.validate().is_ok());
        // ! error: should be Err
        assert!(vm.execute(func_name, &params, &mut buf).is_ok());
        // Instantiated phase
        assert!(vm.instantiate().is_ok());
        let result = vm.execute(func_name, &params, &mut buf);
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(246, WasmEdgeValueGetI32(returns[0]));
        assert_eq!(912, WasmEdgeValueGetI32(returns[1]));
        // ! error: should be Err
        assert!(vm.execute(func_name, &params, &mut buf).is_ok());

        // Function type mismatch
        // ! error: execution failed: function signature mismatch, Code: 0x83
        // let params = [WasmEdgeValueGenI32(123)];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // assert!(vm
        //     .run_wasm_from_ast(&ast_mod, func_name, &params, &mut buf)
        //     .is_err());

        // Function type mismatch
        // ! error: execution failed: function signature mismatch, Code: 0x83
        // let params = [];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // assert!(vm
        //     .run_wasm_from_ast(&ast_mod, func_name, &params, &mut buf)
        //     .is_err());

        // Function type mismatch
        // ! error: execution failed: function signature mismatch, Code: 0x83
        // let params = [WasmEdgeValueGenI64(123), WasmEdgeValueGenI32(456)];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // assert!(vm
        //     .run_wasm_from_ast(&ast_mod, func_name, &params, &mut buf)
        //     .is_err());

        // Function not found
        // ! error: wasmedge runtime failed: wasm function not found, Code: 0x05
        // let params = [WasmEdgeValueGenI32(123), WasmEdgeValueGenI32(456)];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // assert!(vm
        //     .run_wasm_from_ast(&ast_mod, func_name2, &params, &mut buf)
        //     .is_err());

        // Discard result
        let params = [WasmEdgeValueGenI32(123), WasmEdgeValueGenI32(456)];
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<1>();
        assert!(vm
            .run_wasm_from_ast(&ast_mod, func_name, &params, &mut buf)
            .is_ok());

        // Discard result
        let params = [WasmEdgeValueGenI32(123), WasmEdgeValueGenI32(456)];
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<0>();
        assert!(vm
            .run_wasm_from_ast(&ast_mod, func_name, &params, &mut buf)
            .is_ok());
    }

    #[test]
    fn test_context_vm_execute_registered() {
        let mut conf = ConfigureContext::create();
        conf.add_proposal(WasmEdgeProposal::WasmEdge_Proposal_ReferenceTypes);
        conf.add_host_registration(HostRegistration::WasmEdge_HostRegistration_Wasi);
        let mut store = StoreContext::create();

        // WASM from file
        let result = std::fs::read(TPATH);
        assert!(result.is_ok());
        let mod_buf = result.unwrap();
        assert!(mod_buf.len() > 0);

        // Load and validate to wasm AST
        let result = load_module(&conf);
        assert!(result.is_some());
        let ast_mod = result.unwrap();
        assert!(!ast_mod.raw.is_null());
        assert!(validate_module(&conf, &ast_mod));

        // VM creation and deletion
        let result = VMContext::create(Some(&conf), Some(&mut store));
        assert!(result.is_some());
        let mut vm = result.unwrap();
        assert!(!vm.raw.is_null());

        // VM register module from import object
        let result = create_extern_module("extern", false);
        assert!(result.is_some());
        let imp_obj = result.unwrap();
        assert!(!imp_obj.raw.is_null());
        let result = vm.register_module_from_import_object(&imp_obj);
        assert!(result.is_ok());

        let mod_name = "reg-wasm-buffer";
        let mod_name2 = "reg-wasm-error";
        let func_name = "func-mul-2";
        let func_name2 = "func-mul-3";

        // VM register module from buffer
        let result = vm.register_module_from_buffer(mod_name, &mod_buf);
        assert!(result.is_ok());

        let params = [WasmEdgeValueGenI32(123), WasmEdgeValueGenI32(456)];
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        let result = vm.execute_registered(mod_name, func_name, &params, &mut buf);
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(246, WasmEdgeValueGetI32(returns[0]));
        assert_eq!(912, WasmEdgeValueGetI32(returns[1]));

        // Function type mismatch
        // ! error: execution failed: function signature mismatch, Code: 0x83
        // let params = [WasmEdgeValueGenI32(123)];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // let result = vm.execute_registered(mod_name, func_name, &params, &mut buf);
        // assert!(result.is_err());

        // Function type mismatch
        // ! error: execution failed: function signature mismatch, Code: 0x83
        // let params = [];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // let result = vm.execute_registered(mod_name, func_name, &params, &mut buf);
        // assert!(result.is_err());

        // Function type mismatch
        // ! error: execution failed: function signature mismatch, Code: 0x83
        // let params = [WasmEdgeValueGenI64(123), WasmEdgeValueGenI32(456)];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // let result = vm.execute_registered(mod_name, func_name, &params, &mut buf);
        // assert!(result.is_err());

        // Module not found
        // ! error: execution failed: wrong instance address, Code: 0x80
        // let params = [WasmEdgeValueGenI32(123), WasmEdgeValueGenI32(456)];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // let result = vm.execute_registered(mod_name2, func_name, &params, &mut buf);
        // assert!(result.is_err());

        // Function not found
        // ! error: wasmedge runtime failed: wasm function not found, Code: 0x05
        // let params = [WasmEdgeValueGenI32(123), WasmEdgeValueGenI32(456)];
        // let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<2>();
        // let result = vm.execute_registered(mod_name, func_name2, &params, &mut buf);
        // assert!(result.is_err());

        // Discard result
        let params = [WasmEdgeValueGenI32(123), WasmEdgeValueGenI32(456)];
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<1>();
        let result = vm.execute_registered(mod_name, func_name, &params, &mut buf);
        assert!(result.is_ok());

        // Discard result
        let params = [WasmEdgeValueGenI32(123), WasmEdgeValueGenI32(456)];
        let mut buf = mem::MaybeUninit::<WasmEdgeValue>::uninit_array::<0>();
        let result = vm.execute_registered(mod_name, func_name, &params, &mut buf);
        assert!(result.is_ok());
    }

    #[test]
    fn test_context_vm_function() {
        let mod_name = "reg-wasm-buffer";
        let mod_name2 = "reg-wasm-error";
        let func_name = "func-mul-2";
        let func_name2 = "func-mul-3";

        let mut conf = ConfigureContext::create();
        conf.add_proposal(WasmEdgeProposal::WasmEdge_Proposal_ReferenceTypes);
        conf.add_host_registration(HostRegistration::WasmEdge_HostRegistration_Wasi);
        let mut store = StoreContext::create();

        // create extern import object module
        let result = create_extern_module("extern", false);
        assert!(result.is_some());
        let imp_obj = result.unwrap();
        assert!(!imp_obj.raw.is_null());

        // WASM from file
        let result = std::fs::read(TPATH);
        assert!(result.is_ok());
        let mod_buf = result.unwrap();
        assert!(mod_buf.len() > 0);

        // VM creation and deletion
        let result = VMContext::create(Some(&conf), Some(&mut store));
        assert!(result.is_some());
        let mut vm = result.unwrap();
        assert!(!vm.raw.is_null());

        // VM register module from import object
        let result = vm.register_module_from_import_object(&imp_obj);
        assert!(result.is_ok());

        // VM register module from buffer
        let result = vm.register_module_from_buffer(mod_name, &mod_buf);
        assert!(result.is_ok());

        // Load and validate to wasm AST
        let result = load_module(&conf);
        assert!(result.is_some());
        let ast_mod = result.unwrap();
        assert!(!ast_mod.raw.is_null());
        assert!(validate_module(&conf, &ast_mod));

        // VM load wasm from AST module
        assert!(vm.load_from_ast(&ast_mod).is_ok());
        assert!(vm.validate().is_ok());
        assert!(vm.instantiate().is_ok());

        // VM get function type
        assert!(vm.function_type(func_name).is_some());
        assert!(vm.function_type(func_name2).is_none());

        // VM get function type registered
        assert!(vm.function_type_registered(mod_name, func_name).is_some());
        assert!(vm.function_type_registered(mod_name2, func_name).is_none());
        assert!(vm.function_type_registered(mod_name, func_name2).is_none());

        // VM get function list
        assert_eq!(vm.function_list_len(), 11);
        let mut buf = mem::MaybeUninit::<we_ffi::WasmEdge_String>::uninit_array::<11>();
        let result = vm.function_names(&mut buf);
        assert!(result.is_some());
        let names = result.unwrap();
        assert_eq!(
            &names[..],
            [
                "func-1",
                "func-2",
                "func-3",
                "func-4",
                "func-add",
                "func-call-indirect",
                "func-host-add",
                "func-host-div",
                "func-host-mul",
                "func-host-sub",
                "func-mul-2"
            ]
        );
    }

    #[test]
    fn test_context_vm_get() {
        let mod_name = "reg-wasm-buffer";
        let mod_name2 = "reg-wasm-error";
        let func_name = "func-mul-2";
        let func_name2 = "func-mul-3";

        let mut conf = ConfigureContext::create();
        conf.add_proposal(WasmEdgeProposal::WasmEdge_Proposal_ReferenceTypes);
        conf.add_host_registration(HostRegistration::WasmEdge_HostRegistration_Wasi);
        let mut store = StoreContext::create();

        // create extern import object module
        let result = create_extern_module("extern", false);
        assert!(result.is_some());
        let imp_obj = result.unwrap();
        assert!(!imp_obj.raw.is_null());

        // WASM from file
        let result = std::fs::read(TPATH);
        assert!(result.is_ok());
        let mod_buf = result.unwrap();
        assert!(mod_buf.len() > 0);

        // VM creation and deletion
        let result = VMContext::create(Some(&conf), Some(&mut store));
        assert!(result.is_some());
        let mut vm = result.unwrap();
        assert!(!vm.raw.is_null());

        // VM register module from import object
        let result = vm.register_module_from_import_object(&imp_obj);
        assert!(result.is_ok());

        // VM register module from buffer
        let result = vm.register_module_from_buffer(mod_name, &mod_buf);
        assert!(result.is_ok());

        // Load and validate to wasm AST
        let result = load_module(&conf);
        assert!(result.is_some());
        let ast_mod = result.unwrap();
        assert!(!ast_mod.raw.is_null());
        assert!(validate_module(&conf, &ast_mod));

        // VM load wasm from AST module
        assert!(vm.load_from_ast(&ast_mod).is_ok());
        assert!(vm.validate().is_ok());
        assert!(vm.instantiate().is_ok());

        // VM get import module
        assert!(vm
            .import_object(HostRegistration::WasmEdge_HostRegistration_Wasi)
            .is_some());
        assert!(vm
            .import_object(HostRegistration::WasmEdge_HostRegistration_WasmEdge_Process)
            .is_none());

        // VM get store
        assert!(vm.store_context().is_some());

        // VM get statistics
        assert!(vm.statistics_context().is_some());
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
