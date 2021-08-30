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
        conf_ctx: Option<&ConfigureContext>,
        store_ctx: Option<&mut StoreContext>,
    ) -> VMContext {
        match (conf_ctx, store_ctx) {
            (Some(conf_ctx), Some(store_ctx)) => VMContext {
                raw: unsafe { we_ffi::WasmEdge_VMCreate(conf_ctx.raw, store_ctx.raw) },
            },
            (Some(conf_ctx), None) => VMContext {
                raw: unsafe { we_ffi::WasmEdge_VMCreate(conf_ctx.raw, ptr::null_mut()) },
            },
            (None, Some(store_ctx)) => VMContext {
                raw: unsafe { we_ffi::WasmEdge_VMCreate(ptr::null(), store_ctx.raw) },
            },
            (None, None) => VMContext {
                raw: unsafe { we_ffi::WasmEdge_VMCreate(ptr::null(), ptr::null_mut()) },
            },
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

    pub fn importobject_module(&self, reg: HostRegistration) -> Option<ImportObjectContext> {
        let raw = unsafe { we_ffi::WasmEdge_VMGetImportModuleContext(self.raw, reg) };
        if raw.is_null() {
            return None;
        } else {
        }
        match raw.is_null() {
            true => None,
            false => Some(ImportObjectContext {
                raw,
                _marker: PhantomData,
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
        context::configure::ConfigureContext,
        types::{HostRegistration, WasmEdgeValue},
        value::*,
    };
    use std::mem;

    #[test]
    fn test_wasmedge_vm_run_wasm_from_file() {
        let mut conf_ctx = ConfigureContext::create();
        conf_ctx.add_host_registration(HostRegistration::WasmEdge_HostRegistration_Wasi);

        let mut vm_ctx = VMContext::create(Some(&conf_ctx), None);
        let wasm_name = "../../add.wasm";
        let func_name = "add";
        let params = vec![WasmEdgeValueGenI32(2), WasmEdgeValueGenI32(8)];
        let mut buf: [mem::MaybeUninit<WasmEdgeValue>; 1] = mem::MaybeUninit::uninit_array();

        let result = vm_ctx.run_wasm_from_file(wasm_name, func_name, params.as_slice(), &mut buf);
        assert!(result.is_ok());
        let values = result.unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(WasmEdgeValueGetI32(values[0]), 10);
    }
}
