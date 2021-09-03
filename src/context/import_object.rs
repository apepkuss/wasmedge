use crate::{
    context::vm::VMContext,
    instance::{
        function::HostFunctionContext, global::GlobalInstanceContext,
        memory::MemoryInstanceContext, table::TableInstanceContext,
    },
    types::WasmEdgeString,
    utils::string_to_c_array,
};
use std::marker::PhantomData;
use std::ptr;
use wasmedge_sys::ffi as we_ffi;

#[derive(Clone)]
pub struct ImportObjectContext<'vm> {
    pub(crate) raw: *mut we_ffi::WasmEdge_ImportObjectContext,
    pub(crate) _marker: PhantomData<&'vm VMContext>,
    pub(crate) _drop: bool,
}
impl<'a> ImportObjectContext<'a> {
    pub fn create(mod_name: &str, data: *mut std::os::raw::c_void) -> Option<ImportObjectContext> {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let raw = unsafe { we_ffi::WasmEdge_ImportObjectCreate(mod_name.raw, data) };
        match raw.is_null() {
            true => None,
            false => Some(ImportObjectContext {
                raw,
                _marker: PhantomData,
                _drop: true,
            }),
        }
    }

    pub fn create_wasi(
        args: Option<&[&str]>,
        envs: Option<&[&str]>,
        dirs: Option<&[&str]>,
        preopens: Option<&[&str]>,
    ) -> Option<ImportObjectContext<'a>> {
        // Option<*mut we_ffi::WasmEdge_ImportObjectContext> {

        let (args_len, args) = match args {
            Some(args) => (args.len() as u32, string_to_c_array(args)),
            None => (0, ptr::null()),
        };
        let (envs_len, envs) = match envs {
            Some(envs) => (envs.len() as u32, string_to_c_array(envs)),
            None => (0, ptr::null()),
        };
        let (dirs_len, dirs) = match dirs {
            Some(dirs) => (dirs.len() as u32, string_to_c_array(dirs)),
            None => (0, ptr::null()),
        };
        let (preopens_len, preopens) = match preopens {
            Some(preopens) => (preopens.len() as u32, string_to_c_array(preopens)),
            None => (0, ptr::null()),
        };
        let raw = unsafe {
            we_ffi::WasmEdge_ImportObjectCreateWASI(
                args,
                args_len,
                envs,
                envs_len,
                dirs,
                dirs_len,
                preopens,
                preopens_len,
            )
        };

        match raw.is_null() {
            true => None,
            // false => Some(raw),
            false => Some(ImportObjectContext {
                raw,
                _marker: PhantomData,
                _drop: true,
            }),
        }
    }

    pub fn create_tensorflow_import_object() -> ImportObjectContext<'a> {
        ImportObjectContext {
            raw: unsafe { we_ffi::WasmEdge_Tensorflow_ImportObjectCreate() },
            _marker: PhantomData,
            _drop: true,
        }
    }

    pub fn create_tensorflowlite_import_object() -> ImportObjectContext<'a> {
        ImportObjectContext {
            raw: unsafe { we_ffi::WasmEdge_TensorflowLite_ImportObjectCreate() },
            _marker: PhantomData,
            _drop: true,
        }
    }

    pub fn init_wasi(&self, args: &[&str], envs: &[&str], dirs: &[&str], preopens: &[&str]) {
        let mut cargs = vec![];
        for &arg in args.iter() {
            cargs.push(arg.as_ptr() as *const _);
        }
        let mut cenvs = vec![];
        for &env in envs.iter() {
            cenvs.push(env.as_ptr() as *const _);
        }
        let mut cdirs = vec![];
        for &dir in dirs.iter() {
            cdirs.push(dir.as_ptr() as *const _);
        }
        let mut cpreopens = vec![];
        for &pre in preopens.iter() {
            cpreopens.push(pre.as_ptr() as *const _);
        }
        unsafe {
            we_ffi::WasmEdge_ImportObjectInitWASI(
                self.raw,
                cargs.as_ptr() as *const _,
                cargs.len() as u32,
                cenvs.as_ptr() as *const _,
                cenvs.len() as u32,
                cdirs.as_ptr() as *const _,
                cdirs.len() as u32,
                cpreopens.as_ptr() as *const _,
                cpreopens.len() as u32,
            )
        };
    }

    pub fn add_host_function(&mut self, name: &str, host_func_ctx: &mut HostFunctionContext) {
        let name = WasmEdgeString::from_str(name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
        unsafe {
            we_ffi::WasmEdge_ImportObjectAddHostFunction(self.raw, name.raw, host_func_ctx.raw);
            host_func_ctx.raw = std::ptr::null_mut();
        }
    }

    pub fn add_table(&mut self, name: &str, table_ctx: &mut TableInstanceContext) {
        let name = WasmEdgeString::from_str(name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
        unsafe {
            we_ffi::WasmEdge_ImportObjectAddTable(self.raw, name.raw, table_ctx.raw);
            table_ctx.raw = std::ptr::null_mut();
        }
    }

    pub fn add_memory(&mut self, name: &str, mem_ctx: &mut MemoryInstanceContext) {
        let name = WasmEdgeString::from_str(name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
        unsafe {
            we_ffi::WasmEdge_ImportObjectAddMemory(self.raw, name.raw, mem_ctx.raw);
            mem_ctx.raw = std::ptr::null_mut();
        }
    }

    pub fn add_global(&mut self, name: &str, global_ctx: &mut GlobalInstanceContext) {
        let name = WasmEdgeString::from_str(name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
        unsafe {
            we_ffi::WasmEdge_ImportObjectAddGlobal(self.raw, name.raw, global_ctx.raw);
            global_ctx.raw = std::ptr::null_mut();
        }
    }
}
impl<'a> Drop for ImportObjectContext<'a> {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            if self._drop {
                unsafe { we_ffi::WasmEdge_ImportObjectDelete(self.raw) }
            } else {
                self.raw = ptr::null_mut();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::context::{configure::ConfigureContext, vm::VMContext};
    use crate::instance::function::*;
    use crate::types::*;
    use crate::value::*;
    use std::ffi::CString;
    use std::os::raw::c_void;
    use std::ptr;

    #[test]
    fn test_context_import_object() {
        let host_name = "extern";

        // Create import object with name ""
        let result = ImportObjectContext::create("", ptr::null_mut());
        assert!(result.is_some());
        let imp_obj = result.unwrap();
        assert!(!imp_obj.raw.is_null());

        // Create import object with name "extern"
        let result = ImportObjectContext::create(host_name, ptr::null_mut());
        assert!(result.is_some());
        let mut imp_obj = result.unwrap();
        assert!(!imp_obj.raw.is_null());

        // Add host function "func-add": {externref, i32} -> {i32}
        let params = [
            WasmEdgeValType::WasmEdge_ValType_ExternRef,
            WasmEdgeValType::WasmEdge_ValType_I32,
        ];
        let returns = [WasmEdgeValType::WasmEdge_ValType_I32];
        let result = FunctionTypeContext::create(Some(&params), Some(&returns));
        assert!(result.is_some());
        let func_type = result.unwrap();
        assert!(!func_type.raw.is_null());
        let result = HostFunctionContext::create(&func_type, Some(extern_add_impobj), 0);
        assert!(result.is_some());
        let mut host_func = result.unwrap();
        assert!(!host_func.raw.is_null());
        let host_name = "func-add";
        imp_obj.add_host_function(host_name, &mut host_func);

        // Add host table "table"
        let table_limit = WasmEdgeLimit {
            HasMax: true,
            Min: 10,
            Max: 20,
        };
        let result =
            TableInstanceContext::create(WasmEdgeRefType::WasmEdge_RefType_FuncRef, table_limit);
        assert!(result.is_some());
        let mut host_table = result.unwrap();
        assert!(!host_table.raw.is_null());
        imp_obj.add_table("table", &mut host_table);

        // Add host memory "memory"
        let mem_limit = WasmEdgeLimit {
            HasMax: true,
            Min: 1,
            Max: 2,
        };
        let result = MemoryInstanceContext::create(mem_limit);
        assert!(result.is_some());
        let mut host_memory = result.unwrap();
        assert!(!host_memory.raw.is_null());
        imp_obj.add_memory("memory", &mut host_memory);

        // Add host global "global_i32": const 666
        let result = GlobalInstanceContext::create(
            WasmEdgeValueGenI32(666),
            WasmEdgeMutability::WasmEdge_Mutability_Const,
        );
        assert!(result.is_some());
        let mut host_global = result.unwrap();
        imp_obj.add_global("global_i32", &mut host_global);
    }

    #[test]
    fn test_context_import_object_create_wasi() {
        let args = ["arg1", "arg2"];
        let envs = ["ENV1=VAL1", "ENV2=VAL2", "ENV3=VAL3"];
        let dirs = [".:."];
        let preopens = ["apiTestData", "Makefile", "CMakeFiles", "ssvmAPICoreTests"];

        // Create WASI
        let result = ImportObjectContext::create_wasi(
            Some(&args),
            Some(&envs),
            Some(&dirs),
            Some(&preopens),
        );
        assert!(result.is_some());
        let imp_obj = result.unwrap();
        assert!(!imp_obj.raw.is_null());
    }

    #[test]
    fn test_context_import_object_init_wasi_in_vm() {
        let args = ["arg1", "arg2"];
        let envs = ["ENV1=VAL1", "ENV2=VAL2", "ENV3=VAL3"];
        let dirs = [".:."];
        let preopens = ["apiTestData", "Makefile", "CMakeFiles", "ssvmAPICoreTests"];

        // Initialize WASI in VM.
        let mut conf = ConfigureContext::create();
        conf.add_host_registration(HostRegistration::WasmEdge_HostRegistration_Wasi);
        let vm = VMContext::create(Some(&conf), None);
        assert!(!vm.raw.is_null());
        let result = vm.get_import_object(HostRegistration::WasmEdge_HostRegistration_Wasi);
        assert!(result.is_some());
        let imp_obj = result.unwrap();
        assert!(!imp_obj.raw.is_null());
        imp_obj.init_wasi(&args, &envs, &dirs, &preopens);
    }

    #[test]
    fn test_context_import_object_init_process_in_vm() {
        // Initialize wasmedge_process in VM.
        let mut conf = ConfigureContext::create();
        conf.add_host_registration(HostRegistration::WasmEdge_HostRegistration_WasmEdge_Process);
        let vm = VMContext::create(Some(&conf), None);
        assert!(!vm.raw.is_null());
        let result =
            vm.get_import_object(HostRegistration::WasmEdge_HostRegistration_WasmEdge_Process);
        assert!(result.is_some());
        let imp_obj = result.unwrap();
        assert!(!imp_obj.raw.is_null());
    }

    #[no_mangle]
    unsafe extern "C" fn extern_add_impobj(
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
    unsafe extern "C" fn extern_sub_impobj(
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
    unsafe extern "C" fn extern_mul_impobj(
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
    unsafe extern "C" fn extern_div_impobj(
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
    unsafe extern "C" fn extern_term_impobj(
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
    unsafe extern "C" fn extern_fail_impobj(
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
