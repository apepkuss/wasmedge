use crate::types::{HostRegistration, WasmEdgeProposal};
use wasmedge_sys::ffi as we_ffi;

pub struct ConfigureContext {
    pub(crate) raw: *mut we_ffi::WasmEdge_ConfigureContext,
}
impl ConfigureContext {
    pub fn create() -> ConfigureContext {
        ConfigureContext {
            raw: unsafe { we_ffi::WasmEdge_ConfigureCreate() },
        }
    }

    pub fn add_host_registration(&mut self, host: HostRegistration) {
        unsafe { we_ffi::WasmEdge_ConfigureAddHostRegistration(self.raw, host) }
    }

    pub fn remove_host_registration(&mut self, host: HostRegistration) {
        unsafe { we_ffi::WasmEdge_ConfigureRemoveHostRegistration(self.raw, host) }
    }

    pub fn has_host_registration(&self, host: HostRegistration) -> bool {
        {
            unsafe { we_ffi::WasmEdge_ConfigureHasHostRegistration(self.raw, host) }
        }
    }

    pub fn add_proposal(&mut self, prop: WasmEdgeProposal) {
        unsafe { we_ffi::WasmEdge_ConfigureAddProposal(self.raw, prop) }
    }
}
impl Drop for ConfigureContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_ConfigureDelete(self.raw) }
        }
    }
}
