//! Runtime-faults-backed implementation of the runtime fault boundary.
//!
use spanda_runtime::fault_runtime::FaultRuntime;
use spanda_runtime::fault_types::{FaultScanOptions, FaultScanReport, RuntimeFault};
use spanda_runtime::replay::MissionTrace;
use spanda_ast::nodes::Program;

use crate::engine::scan_program_faults as store_scan_program_faults;
use spanda_runtime::fault_primitives::{
    faults_from_hardware_signals as kernel_faults_from_hardware_signals,
    record_fault_in_trace as kernel_record_fault_in_trace,
};

/// Full fault runtime delegating to `spanda-runtime-faults` scanners and trace helpers.
#[derive(Debug, Default, Clone, Copy)]
pub struct FaultBackedRuntime;

impl FaultRuntime for FaultBackedRuntime {
    fn faults_from_hardware_signals(
        &self,
        faults: &[String],
        events: &[String],
        sim_time_ms: f64,
    ) -> Vec<RuntimeFault> {
        kernel_faults_from_hardware_signals(faults, events, sim_time_ms)
    }

    fn scan_program_faults(
        &self,
        program: &Program,
        source_label: &str,
        options: &FaultScanOptions,
    ) -> FaultScanReport {
        store_scan_program_faults(program, source_label, options)
    }

    fn record_fault_in_trace(
        &self,
        trace: &mut MissionTrace,
        fault: &RuntimeFault,
        sim_time_ms: f64,
    ) {
        kernel_record_fault_in_trace(trace, fault, sim_time_ms);
    }
}
