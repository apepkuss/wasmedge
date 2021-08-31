use crate::context::{ast::ASTModuleContext, configure::ConfigureContext};
use crate::error::{WasmEdgeError, WasmEdgeResult};
use crate::utils::{check, path_to_cstring};
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

    pub fn parse_from_file<P: AsRef<Path>>(
        &mut self,
        ast_mod: &mut ASTModuleContext,
        path: P,
    ) -> WasmEdgeResult<()> {
        if !path.as_ref().exists() {
            return Err(WasmEdgeError {
                code: 0x20 as usize,
                message: format!(
                    "Loading failed: invalid file path: {}",
                    path.as_ref().to_string_lossy().into_owned()
                ),
            });
        }

        let path = path_to_cstring(path.as_ref())?;
        unsafe {
            check(we_ffi::WasmEdge_LoaderParseFromFile(
                self.raw,
                &mut ast_mod.raw,
                path.as_ptr(),
            ))?;
        }
        Ok(())
    }

    pub fn parse_from_buffer(
        &mut self,
        ast_mod: &mut ASTModuleContext,
        buf: &[u8],
    ) -> WasmEdgeResult<()> {
        unsafe {
            check(we_ffi::WasmEdge_LoaderParseFromBuffer(
                self.raw,
                &mut ast_mod.raw,
                buf.as_ptr(),
                buf.len() as u32,
            ))
        }
    }
}
impl Drop for LoaderContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_LoaderDelete(self.raw) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{context::ast::ASTModuleContext, types::WasmEdgeProposal};

    const TPATH: &str = "/root/workspace/wasmedge-ml/wasmedge/tests/data/test.wasm";

    #[test]
    fn test_loader_create() {
        let mut conf = ConfigureContext::create();
        conf.add_proposal(WasmEdgeProposal::WasmEdge_Proposal_ReferenceTypes);
        let loader = LoaderContext::create(&conf);
        assert!(loader.is_some());
    }

    #[test]
    fn test_loader_parse_from_file() {
        let mut conf = ConfigureContext::create();
        conf.add_proposal(WasmEdgeProposal::WasmEdge_Proposal_ReferenceTypes);
        let result = LoaderContext::create(&conf);
        assert!(result.is_some());
        let mut loader = result.unwrap();

        let mut ast_mod = ASTModuleContext::default();
        let result = loader.parse_from_file(&mut ast_mod, TPATH);
        assert!(result.is_ok());
        assert!(!ast_mod.raw.is_null());

        let mut ast_mod = ASTModuleContext::default();
        let result = loader.parse_from_file(&mut ast_mod, "file");
        assert!(result.is_err());
    }

    #[test]
    fn test_loader_parse_from_buffer() {
        let mut conf = ConfigureContext::create();
        conf.add_proposal(WasmEdgeProposal::WasmEdge_Proposal_ReferenceTypes);
        let result = LoaderContext::create(&conf);
        assert!(result.is_some());
        let mut loader = result.unwrap();

        let result = std::fs::read(TPATH);
        assert!(result.is_ok());
        let buf = result.unwrap();

        let mut ast_mod = ASTModuleContext::default();
        assert!(loader.parse_from_buffer(&mut ast_mod, &buf).is_ok());
        assert!(!ast_mod.raw.is_null())
    }
}
