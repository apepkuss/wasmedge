use crate::types::WasmEdgeValue;
use wasmedge_sys::ffi as we_ffi;

#[allow(non_snake_case)]
pub fn WasmEdgeValueGenI32(val: i32) -> WasmEdgeValue {
    unsafe { we_ffi::WasmEdge_ValueGenI32(val) }
}

#[allow(non_snake_case)]
pub fn WasmEdgeValueGenI64(val: i64) -> WasmEdgeValue {
    unsafe { we_ffi::WasmEdge_ValueGenI64(val) }
}

#[allow(non_snake_case)]
pub fn WasmEdgeValueGenExternRef(ptr: *mut ::std::os::raw::c_void) -> WasmEdgeValue {
    unsafe { we_ffi::WasmEdge_ValueGenExternRef(ptr) }
}

#[allow(non_snake_case)]
pub fn WasmEdgeValueGenFuncRef(idx: usize) -> WasmEdgeValue {
    unsafe { we_ffi::WasmEdge_ValueGenFuncRef(idx as u32) }
}

#[allow(non_snake_case)]
pub fn WasmEdgeValueGenF32(val: f32) -> WasmEdgeValue {
    unsafe { we_ffi::WasmEdge_ValueGenF32(val) }
}

#[allow(non_snake_case)]
pub fn WasmEdgeValueGenF64(val: f64) -> WasmEdgeValue {
    unsafe { we_ffi::WasmEdge_ValueGenF64(val) }
}

#[allow(non_snake_case)]
pub fn WasmEdgeValueGetI32(val: WasmEdgeValue) -> i32 {
    unsafe { we_ffi::WasmEdge_ValueGetI32(val) }
}

#[allow(non_snake_case)]
pub fn WasmEdgeValueGetI64(val: WasmEdgeValue) -> i64 {
    unsafe { we_ffi::WasmEdge_ValueGetI64(val) }
}

#[allow(non_snake_case)]
pub fn WasmEdgeValueGetExternRef(val: WasmEdgeValue) -> *mut ::std::os::raw::c_void {
    unsafe { we_ffi::WasmEdge_ValueGetExternRef(val) }
}

#[allow(non_snake_case)]
pub fn WasmEdgeValueGetF32(val: WasmEdgeValue) -> f32 {
    unsafe { we_ffi::WasmEdge_ValueGetF32(val) }
}

#[allow(non_snake_case)]
pub fn WasmEdgeValueGetF64(val: WasmEdgeValue) -> f64 {
    unsafe { we_ffi::WasmEdge_ValueGetF64(val) }
}
