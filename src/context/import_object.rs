use crate::{
    context::vm::VMContext,
    instance::{
        function::HostFunctionContext, global::GlobalInstanceContext,
        memory::MemoryInstanceContext, table::TableInstanceContext,
    },
    types::WasmEdgeString,
};
use std::marker::PhantomData;
use wasmedge_sys::ffi as we_ffi;

#[derive(Clone)]
pub struct ImportObjectContext<'vm> {
    pub(crate) raw: *mut we_ffi::WasmEdge_ImportObjectContext,
    pub(crate) _marker: PhantomData<&'vm VMContext>,
}
impl<'vm> ImportObjectContext<'vm> {
    pub fn create(mod_name: &str, data: *mut std::os::raw::c_void) -> Option<ImportObjectContext> {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let raw = unsafe { we_ffi::WasmEdge_ImportObjectCreate(mod_name.raw, data) };
        match raw.is_null() {
            true => None,
            false => Some(ImportObjectContext {
                raw,
                _marker: PhantomData,
            }),
        }
    }

    pub fn create_tensorflow_import_object() -> ImportObjectContext<'vm> {
        ImportObjectContext {
            raw: unsafe { we_ffi::WasmEdge_Tensorflow_ImportObjectCreate() },
            _marker: PhantomData,
        }
    }

    pub fn create_tensorflowlite_import_object() -> ImportObjectContext<'vm> {
        ImportObjectContext {
            raw: unsafe { we_ffi::WasmEdge_TensorflowLite_ImportObjectCreate() },
            _marker: PhantomData,
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
                cargs.as_ptr(),
                cargs.len() as u32,
                cenvs.as_ptr(),
                cenvs.len() as u32,
                cdirs.as_ptr(),
                cdirs.len() as u32,
                cpreopens.as_ptr(),
                cpreopens.len() as u32,
            )
        };
    }

    pub fn add_host_function(&mut self, name: &str, host_func_ctx: &mut HostFunctionContext) {
        let name = WasmEdgeString::from_str(name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
        unsafe {
            we_ffi::WasmEdge_ImportObjectAddHostFunction(self.raw, name.raw, host_func_ctx.raw)
        }
    }

    pub fn add_table(&mut self, name: &str, table_ctx: &mut TableInstanceContext) {
        let name = WasmEdgeString::from_str(name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
        unsafe { we_ffi::WasmEdge_ImportObjectAddTable(self.raw, name.raw, table_ctx.raw) }
    }

    pub fn add_memory(&mut self, name: &str, mem_ctx: &mut MemoryInstanceContext) {
        let name = WasmEdgeString::from_str(name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
        unsafe {
            we_ffi::WasmEdge_ImportObjectAddMemory(self.raw, name.raw, mem_ctx.raw);
        }
    }

    pub fn add_global(&mut self, name: &str, global_ctx: &mut GlobalInstanceContext) {
        let name = WasmEdgeString::from_str(name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
        unsafe {
            we_ffi::WasmEdge_ImportObjectAddGlobal(self.raw, name.raw, global_ctx.raw);
        }
    }
}
impl<'vm> Drop for ImportObjectContext<'vm> {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_ImportObjectDelete(self.raw) }
        }
    }
}
