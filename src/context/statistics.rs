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

    pub fn set_cost_table(&mut self, cost_arr: &mut [u64]) {
        unsafe {
            we_ffi::WasmEdge_StatisticsSetCostTable(
                self.raw,
                cost_arr.as_mut_ptr(),
                cost_arr.len() as u32,
            )
        }
    }

    pub fn set_cost_limit(&mut self, limit: u64) {
        unsafe { we_ffi::WasmEdge_StatisticsSetCostLimit(self.raw, limit) }
    }

    pub fn get_instr_count(&self) -> usize {
        unsafe { we_ffi::WasmEdge_StatisticsGetInstrCount(self.raw) as usize }
    }

    pub fn get_instr_per_second(&self) -> usize {
        unsafe { we_ffi::WasmEdge_StatisticsGetInstrPerSecond(self.raw) as usize }
    }

    pub fn get_total_cost(&self) -> usize {
        unsafe { we_ffi::WasmEdge_StatisticsGetTotalCost(self.raw) as usize }
    }
}
impl Drop for StatisticsContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_StatisticsDelete(self.raw) }
        }
    }
}
