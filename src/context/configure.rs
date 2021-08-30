use crate::types::{CompilerOptimizationLevel, HostRegistration, WasmEdgeProposal};
use wasmedge_sys::{ffi as we_ffi, WasmEdge_ConfigureCompilerIsCostMeasuring};

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

    pub fn has_proposal(&self, prop: WasmEdgeProposal) -> bool {
        unsafe { we_ffi::WasmEdge_ConfigureHasProposal(self.raw, prop) }
    }

    pub fn add_proposal(&mut self, prop: WasmEdgeProposal) {
        unsafe { we_ffi::WasmEdge_ConfigureAddProposal(self.raw, prop) }
    }

    pub fn remove_proposal(&mut self, prop: WasmEdgeProposal) {
        unsafe { we_ffi::WasmEdge_ConfigureRemoveProposal(self.raw, prop) }
    }

    pub fn set_max_memory_page(&mut self, page: usize) {
        unsafe { we_ffi::WasmEdge_ConfigureSetMaxMemoryPage(self.raw, page as u32) }
    }

    pub fn get_max_memory_page(&self) -> usize {
        unsafe { we_ffi::WasmEdge_ConfigureGetMaxMemoryPage(self.raw) as usize }
    }

    pub fn set_opt_level(&mut self, level: CompilerOptimizationLevel) {
        unsafe { we_ffi::WasmEdge_ConfigureCompilerSetOptimizationLevel(self.raw, level) }
    }

    pub fn get_opt_level(&self) -> CompilerOptimizationLevel {
        unsafe { we_ffi::WasmEdge_ConfigureCompilerGetOptimizationLevel(self.raw) }
    }

    pub fn set_dump_ir(&mut self, is_dump: bool) {
        unsafe { we_ffi::WasmEdge_ConfigureCompilerSetDumpIR(self.raw, is_dump) }
    }

    pub fn is_dump_ir(&self) -> bool {
        unsafe { we_ffi::WasmEdge_ConfigureCompilerIsDumpIR(self.raw) }
    }

    pub fn set_instruction_counting(&mut self, is_count: bool) {
        unsafe { we_ffi::WasmEdge_ConfigureCompilerSetInstructionCounting(self.raw, is_count) }
    }

    pub fn is_instruction_counting(&self) -> bool {
        unsafe { we_ffi::WasmEdge_ConfigureCompilerIsInstructionCounting(self.raw) }
    }

    pub fn set_cost_measuring(&mut self, is_measure: bool) {
        unsafe { we_ffi::WasmEdge_ConfigureCompilerSetCostMeasuring(self.raw, is_measure) }
    }

    pub fn is_cost_measuring(&self) -> bool {
        unsafe { WasmEdge_ConfigureCompilerIsCostMeasuring(self.raw) }
    }
}
impl Drop for ConfigureContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_ConfigureDelete(self.raw) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configure_proposals() {
        let mut config = ConfigureContext::create();

        config.add_proposal(WasmEdgeProposal::WasmEdge_Proposal_SIMD);
        config.add_proposal(WasmEdgeProposal::WasmEdge_Proposal_Memory64);
        assert!(config.has_proposal(WasmEdgeProposal::WasmEdge_Proposal_SIMD));
        assert!(config.has_proposal(WasmEdgeProposal::WasmEdge_Proposal_Memory64));
        config.remove_proposal(WasmEdgeProposal::WasmEdge_Proposal_SIMD);
        assert!(!config.has_proposal(WasmEdgeProposal::WasmEdge_Proposal_SIMD));
        assert!(config.has_proposal(WasmEdgeProposal::WasmEdge_Proposal_Memory64));
    }

    #[test]
    fn test_host_registration() {
        let mut config = ConfigureContext::create();
        config.add_host_registration(HostRegistration::WasmEdge_HostRegistration_Wasi);
        assert!(config.has_host_registration(HostRegistration::WasmEdge_HostRegistration_Wasi));
        config.remove_host_registration(HostRegistration::WasmEdge_HostRegistration_Wasi);
        assert!(!config.has_host_registration(HostRegistration::WasmEdge_HostRegistration_Wasi));
    }

    #[test]
    fn test_memory() {
        let mut config = ConfigureContext::create();

        config.set_max_memory_page(1234);
        assert_eq!(config.get_max_memory_page(), 1234);
    }

    #[test]
    fn test_aot_compiler() {
        let mut config = ConfigureContext::create();

        config.set_opt_level(CompilerOptimizationLevel::WasmEdge_CompilerOptimizationLevel_O0);
        assert_eq!(
            config.get_opt_level(),
            CompilerOptimizationLevel::WasmEdge_CompilerOptimizationLevel_O0,
        );
        config.set_dump_ir(true);
        assert!(config.is_dump_ir());
        config.set_instruction_counting(true);
        assert!(config.is_instruction_counting());
        config.set_cost_measuring(true);
        assert!(config.is_cost_measuring());
    }
}
