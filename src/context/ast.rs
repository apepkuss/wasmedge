use std::ptr;
use wasmedge_sys::ffi as we_ffi;

pub struct ASTModuleContext {
    pub(crate) raw: *mut we_ffi::WasmEdge_ASTModuleContext,
}
impl Drop for ASTModuleContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_ASTModuleDelete(self.raw) }
        }
    }
}
impl Default for ASTModuleContext {
    fn default() -> Self {
        ASTModuleContext {
            raw: ptr::null_mut(),
        }
    }
}
