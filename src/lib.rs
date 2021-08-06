use std::ffi::CStr;
use wasmedge_sys::ffi as we_ffi;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
