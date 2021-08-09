#![feature(maybe_uninit_uninit_array, maybe_uninit_extra, maybe_uninit_slice)]

use std::ffi::{CStr, CString};
use std::fmt;
use std::mem;
use std::path::Path;
use wasmedge_sys::{ffi as we_ffi, WasmEdge_VMRunWasmFromFile};

#[derive(Debug)]
pub struct WasmEdgeError {
    code: u32,
    message: String,
}
impl fmt::Display for WasmEdgeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        self.message.fmt(f)
    }
}
impl std::error::Error for WasmEdgeError {}
impl From<String> for WasmEdgeError {
    fn from(message: String) -> WasmEdgeError {
        WasmEdgeError { code: 49, message }
    }
}
impl From<std::ffi::NulError> for WasmEdgeError {
    fn from(e: std::ffi::NulError) -> WasmEdgeError {
        WasmEdgeError {
            code: 49,
            message: e.to_string(),
        }
    }
}

pub type WasmEdgeResult<T> = Result<T, WasmEdgeError>;

fn check(result: we_ffi::WasmEdge_Result) -> WasmEdgeResult<u32> {
    unsafe {
        let code = we_ffi::WasmEdge_ResultGetCode(result);
        if we_ffi::WasmEdge_ResultOK(result) {
            return Ok(code);
        }
        let message = CStr::from_ptr(we_ffi::WasmEdge_ResultGetMessage(result))
            .to_string_lossy()
            .into_owned();
        Err(WasmEdgeError { code, message })
    }
}

pub fn version() -> String {
    let c_buf = unsafe { we_ffi::WasmEdge_VersionGet() };
    let slice: &CStr = unsafe { CStr::from_ptr(c_buf) };
    let str_slice: &str = slice.to_str().unwrap();
    str_slice.to_owned()
}

pub fn major_version() -> usize {
    let major = unsafe { we_ffi::WasmEdge_VersionGetMajor() };
    major as usize
}

pub fn minor_version() -> usize {
    let minor = unsafe { we_ffi::WasmEdge_VersionGetMinor() };
    minor as usize
}

pub fn version_patch() -> usize {
    let patch = unsafe { we_ffi::WasmEdge_VersionGetPatch() };
    patch as usize
}

pub struct WasmEdgeConfigureContext {
    raw: *mut we_ffi::WasmEdge_ConfigureContext,
}
impl WasmEdgeConfigureContext {
    pub fn new() -> WasmEdgeConfigureContext {
        WasmEdgeConfigureContext {
            raw: unsafe { we_ffi::WasmEdge_ConfigureCreate() },
        }
    }
}
impl Drop for WasmEdgeConfigureContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_ConfigureDelete(self.raw) }
        }
    }
}

pub struct WasmEdgeStoreContext {
    raw: *mut we_ffi::WasmEdge_StoreContext,
}
impl WasmEdgeStoreContext {
    pub fn new() -> WasmEdgeStoreContext {
        WasmEdgeStoreContext {
            raw: unsafe { we_ffi::WasmEdge_StoreCreate() },
        }
    }
}
impl Drop for WasmEdgeStoreContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_StoreDelete(self.raw) }
        }
    }
}

pub fn add_host_registration(
    ctx: &mut WasmEdgeConfigureContext,
    host: we_ffi::WasmEdge_HostRegistration,
) {
    unsafe { we_ffi::WasmEdge_ConfigureAddHostRegistration(ctx.raw, host) }
}

pub fn remove_host_registration(
    ctx: &mut WasmEdgeConfigureContext,
    host: we_ffi::WasmEdge_HostRegistration,
) {
    unsafe { we_ffi::WasmEdge_ConfigureRemoveHostRegistration(ctx.raw, host) }
}

pub fn has_host_registration(
    ctx: &WasmEdgeConfigureContext,
    host: we_ffi::WasmEdge_HostRegistration,
) -> bool {
    {
        unsafe { we_ffi::WasmEdge_ConfigureHasHostRegistration(ctx.raw, host) }
    }
}

pub struct WasmEdgeVMContext {
    raw: *mut we_ffi::WasmEdge_VMContext,
}
impl WasmEdgeVMContext {
    pub fn new(
        conf_ctx: &WasmEdgeConfigureContext,
        store_ctx: &mut WasmEdgeStoreContext,
    ) -> WasmEdgeVMContext {
        WasmEdgeVMContext {
            raw: unsafe { we_ffi::WasmEdge_VMCreate(conf_ctx.raw, store_ctx.raw) },
        }
    }
}
impl Drop for WasmEdgeVMContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_VMDelete(self.raw) }
        }
    }
}

#[cfg(unix)]
fn path_to_cstring(path: &Path) -> WasmEdgeResult<CString> {
    use std::os::unix::ffi::OsStrExt;

    Ok(CString::new(path.as_os_str().as_bytes())?)
}

#[cfg(windows)]
fn path_to_cstring(path: &Path) -> WasmEdgeResult<CString> {
    match path.to_str() {
        Some(s) => Ok(CString::new(s)?),
        None => {
            let message = format!("Couldn't convert path '{}' to UTF-8", path.display());
            Err(message.into())
        }
    }
}

pub fn WasmEdgeValueGenI32(val: i32) -> we_ffi::WasmEdge_Value {
    unsafe { we_ffi::WasmEdge_ValueGenI32(val) }
}

pub fn WasmEdgeValueGenI64(val: i64) -> we_ffi::WasmEdge_Value {
    unsafe { we_ffi::WasmEdge_ValueGenI64(val) }
}

pub fn WasmEdgeValueGetI32(val: we_ffi::WasmEdge_Value) -> i32 {
    unsafe { we_ffi::WasmEdge_ValueGetI32(val) }
}

pub fn WasmEdgeValueGetI64(val: we_ffi::WasmEdge_Value) -> i64 {
    unsafe { we_ffi::WasmEdge_ValueGetI64(val) }
}

pub fn vm_run_wasm_from_file<P: AsRef<Path>>(
    ctx: &mut WasmEdgeVMContext,
    path: P,
    func_name: &str,
    params: &[we_ffi::WasmEdge_Value],
    returns: &mut [mem::MaybeUninit<we_ffi::WasmEdge_Value>],
) -> WasmEdgeResult<u32> {
    let path = path_to_cstring(path.as_ref())?;
    unsafe {
        let funcname = we_ffi::WasmEdge_StringCreateByCString(CString::new(func_name)?.as_ptr());

        let result = check(WasmEdge_VMRunWasmFromFile(
            ctx.raw,
            path.as_ptr(),
            funcname,
            params.as_ptr() as *const we_ffi::WasmEdge_Value,
            params.len() as u32,
            returns.as_mut_ptr() as *mut we_ffi::WasmEdge_Value,
            returns.len() as u32,
        ));
        mem::MaybeUninit::slice_assume_init_ref(&returns[..returns.len()]);
        we_ffi::WasmEdge_StringDelete(funcname);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
        assert!(major_version() >= 0);
        assert!(minor_version() >= 0);
        assert!(version_patch() >= 0);
    }
}
