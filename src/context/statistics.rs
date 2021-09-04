use crate::context::vm::VMContext;
use std::marker::PhantomData;
use wasmedge_sys::ffi as we_ffi;

pub struct StatisticsContext<'vm> {
    pub(crate) raw: *mut we_ffi::WasmEdge_StatisticsContext,
    pub(crate) _marker: PhantomData<&'vm VMContext>,
    pub(crate) _drop: bool,
}
impl<'vm> StatisticsContext<'vm> {
    pub fn create() -> Option<Self> {
        let raw = unsafe { we_ffi::WasmEdge_StatisticsCreate() };
        match raw.is_null() {
            true => None,
            false => Some(StatisticsContext {
                raw,
                _marker: PhantomData,
                _drop: true,
            }),
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
impl<'vm> Drop for StatisticsContext<'vm> {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            if self._drop {
                unsafe { we_ffi::WasmEdge_StatisticsDelete(self.raw) }
            } else {
                self.raw = std::ptr::null_mut();
            }
        }
    }
}
