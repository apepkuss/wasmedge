use crate::context::configure::ConfigureContext;
use crate::error::WasmEdgeResult;
use crate::utils::{check, path_to_cstring};
use std::path::Path;
use wasmedge_sys::ffi as we_ffi;

pub struct Compiler {
    pub(crate) raw: *mut we_ffi::WasmEdge_CompilerContext,
}
impl Compiler {
    pub fn create(conf: ConfigureContext) -> Option<Self> {
        let raw = unsafe { we_ffi::WasmEdge_CompilerCreate(conf.raw) };
        match raw.is_null() {
            true => None,
            false => Some(Compiler { raw }),
        }
    }

    pub fn compile<P: AsRef<Path>>(&mut self, in_path: P, out_path: P) -> WasmEdgeResult<()> {
        let in_path = path_to_cstring(in_path.as_ref())?;
        let out_path = path_to_cstring(out_path.as_ref())?;
        unsafe {
            check(we_ffi::WasmEdge_CompilerCompile(
                self.raw,
                in_path.as_ptr(),
                out_path.as_ptr(),
            ))
        }
    }
}
impl Drop for Compiler {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_CompilerDelete(self.raw) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::configure::ConfigureContext;

    const TPATH: &str = "/root/workspace/wasmedge-ml/wasmedge/tests/data/test.wasm";

    #[test]
    fn test_compiler() {
        let conf = ConfigureContext::create();
        // create compiler
        let result = Compiler::create(conf);
        assert!(result.is_some());
        let mut compiler = result.unwrap();
        // compile
        assert!(compiler.compile(TPATH, "test.so").is_ok());
        assert!(compiler
            .compile("not_exist.wasm", "not_exit.wasm.so")
            .is_err());
    }
}
