use crate::types::WasmEdgeValue;
use wasmedge_sys::ffi as we_ffi;

pub fn WasmEdgeValueGenI32(val: i32) -> WasmEdgeValue {
    unsafe { we_ffi::WasmEdge_ValueGenI32(val) }
}

pub fn WasmEdgeValueGenI64(val: i64) -> WasmEdgeValue {
    unsafe { we_ffi::WasmEdge_ValueGenI64(val) }
}

pub fn WasmEdgeValueGenExternRef(ptr: *mut ::std::os::raw::c_void) -> WasmEdgeValue {
    unsafe { we_ffi::WasmEdge_ValueGenExternRef(ptr) }
}

pub fn WasmEdgeValueGetI32(val: WasmEdgeValue) -> i32 {
    unsafe { we_ffi::WasmEdge_ValueGetI32(val) }
}

pub fn WasmEdgeValueGetI64(val: WasmEdgeValue) -> i64 {
    unsafe { we_ffi::WasmEdge_ValueGetI64(val) }
}

pub fn WasmEdgeValueGetExternRef(val: WasmEdgeValue) -> *mut ::std::os::raw::c_void {
    unsafe { we_ffi::WasmEdge_ValueGetExternRef(val) }
}
