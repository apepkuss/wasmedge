use std::ffi::CStr;
use wasmedge_sys::{WasmEdge_Result, WasmEdge_ResultGetCode, WasmEdge_ResultGetMessage};

#[derive(Debug)]
pub struct WasmEdgeError {
    /// 0x00: Success
    /// 0x01: Terminated -> Success
    /// 0x02: Failed
    /// 0x03: NullError
    /// 0x20: File not found
    pub code: usize,
    pub message: String,
}
impl std::fmt::Display for WasmEdgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
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
            code: 3,
            message: e.to_string(),
        }
    }
}
impl From<WasmEdge_Result> for WasmEdgeError {
    fn from(result: WasmEdge_Result) -> WasmEdgeError {
        let code = unsafe { WasmEdge_ResultGetCode(result) as usize };
        let message = unsafe {
            let c_str = CStr::from_ptr(WasmEdge_ResultGetMessage(result));
            c_str.to_string_lossy().into_owned()
        };
        WasmEdgeError { code, message }
    }
}

pub type WasmEdgeResult<T> = Result<T, WasmEdgeError>;

#[cfg(test)]
mod tests {
    use super::*;
    use wasmedge_sys::WasmEdge_Result;

    #[test]
    fn test_result() {
        let mut res: WasmEdge_Result;
        let mut err: WasmEdgeError;
        res = WasmEdge_Result { Code: 0 };
        err = WasmEdgeError::from(res);
        assert_eq!(err.code, 0);
        assert!(!err.message.is_empty());
        res = WasmEdge_Result { Code: 1 };
        err = WasmEdgeError::from(res);
        assert_eq!(err.code, 1);
        assert!(!err.message.is_empty());
        res = WasmEdge_Result { Code: 2 };
        err = WasmEdgeError::from(res);
        assert_eq!(err.code, 2);
        assert!(!err.message.is_empty());
    }
}
