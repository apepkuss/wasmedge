#[derive(Debug)]
pub struct WasmEdgeError {
    code: u32,
    message: String,
}
impl WasmEdgeError {
    pub fn new<T: std::convert::Into<String>>(code: u32, message: T) -> Self {
        WasmEdgeError {
            code,
            message: message.into(),
        }
    }
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
            code: 49,
            message: e.to_string(),
        }
    }
}

pub type WasmEdgeResult<T> = Result<T, WasmEdgeError>;
