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

pub fn check(result: we_ffi::WasmEdge_Result) -> WasmEdgeResult<()> {
    unsafe {
        if !we_ffi::WasmEdge_ResultOK(result) {
            let code = we_ffi::WasmEdge_ResultGetCode(result) as usize;
            let message = CStr::from_ptr(we_ffi::WasmEdge_ResultGetMessage(result))
                .to_string_lossy()
                .into_owned();
            return Err(WasmEdgeError { code, message });
        }
    }
    Ok(())
}

pub fn string_to_c_array(args: &[&str]) -> *const *const std::os::raw::c_char {
    let mut v = vec![];
    for &arg in args {
        let s = CString::new(arg).unwrap();
        v.push(s.as_ptr());
    }
    let p = v.as_ptr();
    std::mem::forget(v);
    p
}
