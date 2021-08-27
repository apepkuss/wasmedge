use wasmedge_sys::ffi as we_ffi;

pub struct StatisticsContext {
    pub(crate) raw: *mut we_ffi::WasmEdge_StatisticsContext,
}
impl StatisticsContext {
    pub fn create() -> Option<Self> {
        let raw = unsafe { we_ffi::WasmEdge_StatisticsCreate() };
        match raw.is_null() {
            true => None,
            false => Some(StatisticsContext { raw }),
        }
    }
}
impl Drop for StatisticsContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_StatisticsDelete(self.raw) }
        }
    }
}
