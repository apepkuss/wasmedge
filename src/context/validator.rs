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

    pub fn validate(&self, ast_mod: &ASTModuleContext) -> WasmEdgeResult<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{configure::ConfigureContext, loader::LoaderContext};
    use crate::types::WasmEdgeProposal;

    const TPATH: &str = "/root/workspace/wasmedge-ml/wasmedge/tests/data/test.wasm";

    #[test]
    fn test_validator() {
        let mut conf = ConfigureContext::create();
        conf.add_proposal(WasmEdgeProposal::WasmEdge_Proposal_ReferenceTypes);
        let result = Validator::create(&conf);
        assert!(result.is_some());
        let validator = result.unwrap();

        // load and parse file
        let result = load_module(&conf);
        assert!(result.is_some());
        let ast_mod = result.unwrap();
        assert!(!ast_mod.raw.is_null());

        // validation
        assert!(validator.validate(&ast_mod).is_ok());
    }

    fn load_module(conf: &ConfigureContext) -> Option<ASTModuleContext> {
        let mut module = ASTModuleContext::default();
        let mut loader = LoaderContext::create(conf)?;
        loader.parse_from_file(&mut module, TPATH);
        Some(module)
    }
}
