use crate::error::{WasmEdgeError, WasmEdgeResult};
use std::ffi::{CStr, CString};
use std::path::Path;
use wasmedge_sys::ffi as we_ffi;

#[cfg(unix)]
pub fn path_to_cstring(path: &Path) -> WasmEdgeResult<CString> {
    use std::os::unix::ffi::OsStrExt;

    Ok(CString::new(path.as_os_str().as_bytes())?)
}

#[cfg(windows)]
pub fn path_to_cstring(path: &Path) -> WasmEdgeResult<CString> {
    match path.to_str() {
        Some(s) => Ok(CString::new(s)?),
        None => {
            let message = format!("Couldn't convert path '{}' to UTF-8", path.display());
            Err(message.into())
        }
    }
}

pub fn check(result: we_ffi::WasmEdge_Result) -> WasmEdgeResult<u32> {
    unsafe {
        let code = we_ffi::WasmEdge_ResultGetCode(result);
        if we_ffi::WasmEdge_ResultOK(result) {
            return Ok(code);
        }
        let message = CStr::from_ptr(we_ffi::WasmEdge_ResultGetMessage(result))
            .to_string_lossy()
            .into_owned();
        Err(WasmEdgeError::new(code, message))
    }
}
