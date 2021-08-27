use crate::context::{ast::ASTModuleContext, configure::ConfigureContext};
use crate::error::WasmEdgeResult;
use crate::utils::check;
use wasmedge_sys::ffi as we_ffi;

pub struct Validator {
    pub(crate) raw: *mut we_ffi::WasmEdge_ValidatorContext,
}
impl Validator {
    pub fn create(conf: &ConfigureContext) -> Option<Self> {
        let raw = unsafe { we_ffi::WasmEdge_ValidatorCreate(conf.raw) };
        match raw.is_null() {
            true => None,
            false => Some(Validator { raw }),
        }
    }

    pub fn validate(&mut self, ast_mod: &ASTModuleContext) -> WasmEdgeResult<u32> {
        unsafe { check(we_ffi::WasmEdge_ValidatorValidate(self.raw, ast_mod.raw)) }
    }
}
impl Drop for Validator {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_ValidatorDelete(self.raw) }
        }
    }
}