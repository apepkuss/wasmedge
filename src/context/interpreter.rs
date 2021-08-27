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
