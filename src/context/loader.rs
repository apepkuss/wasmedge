use crate::context::{ast::ASTModuleContext, configure::ConfigureContext};
use crate::utils::path_to_cstring;
use std::path::Path;
use wasmedge_sys::ffi as we_ffi;

pub struct LoaderContext {
    pub(crate) raw: *mut we_ffi::WasmEdge_LoaderContext,
}
impl LoaderContext {
    pub fn create(conf: &ConfigureContext) -> Option<Self> {
        let raw = unsafe { we_ffi::WasmEdge_LoaderCreate(conf.raw) };
        match raw.is_null() {
            true => None,
            false => Some(LoaderContext { raw }),
        }
    }

    pub fn parse_from_file<P: AsRef<Path>>(&mut self, module: &mut ASTModuleContext, path: P) {
        let path = path_to_cstring(path.as_ref()).unwrap();
        unsafe { we_ffi::WasmEdge_LoaderParseFromFile(self.raw, &mut module.raw, path.as_ptr()) };
    }
}
impl Drop for LoaderContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_LoaderDelete(self.raw) }
        }
    }
}
