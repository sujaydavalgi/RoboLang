//! Replay integration — record and display fault events in mission traces.

use crate::types::{FaultTimeline, RuntimeFault, RuntimeFaultKind};
use spanda_runtime::replay::{MissionTrace, TraceFrame};

/// Fault event names recorded in mission traces.
pub const FAULT_EVENTS: &[&str] = &[
    "fault_crash",
    "fault_reboot",
    "fault_watchdog_timeout",
    "fault_memory_growth",
    "fault_restart_loop",
    "fault_resource_pressure",
    "fault_heartbeat_loss",
    "fault_deadlock",
    "fault_oom",
];

/// Record a runtime fault in a mission trace.
pub fn record_fault_in_trace(trace: &mut MissionTrace, fault: &RuntimeFault, sim_time_ms: f64) {
    // Append a fault event frame to a mission trace for replay analysis.
    //
    // Parameters:
    // - `trace` — mission trace to append to
    // - `fault` — detected runtime fault
    // - `sim_time_ms` — simulation timestamp
    //
    // Returns:
    // None (modifies trace in place).
    //
    // Options:
    // None.
    //
    // Example:
    // record_fault_in_trace(&mut trace, &fault, 1000.0);

    let event = fault_kind_to_trace_event(&fault.kind);
    let payload = serde_json::json!({
        "kind": fault.kind.as_str(),
        "target": fault.target,
        "status": fault.status.as_str(),
        "message": fault.message,
        "evidence": fault.evidence,
    });
    trace.record(sim_time_ms, event, payload);
}

/// Record all faults from a scan into a mission trace.
pub fn record_faults_in_trace(trace: &mut MissionTrace, faults: &[RuntimeFault], sim_time_ms: f64) {
    // Record all faults from a scan into a mission trace.
    //
    // Parameters:
    // - `trace` — mission trace to append to
    // - `faults` — list of detected faults
    // - `sim_time_ms` — base simulation timestamp
    //
    // Returns:
    // None (modifies trace in place).
    //
    // Options:
    // None.
    //
    // Example:
    // record_faults_in_trace(&mut trace, &faults, 0.0);

    for (i, fault) in faults.iter().enumerate() {
        record_fault_in_trace(trace, fault, sim_time_ms + i as f64 * 100.0);
    }
}

/// Extract fault timeline entries from a mission trace.
pub fn faults_from_trace(trace: &MissionTrace) -> Vec<FaultTimeline> {
    // Parse fault events from a mission trace into a fault timeline.
    //
    // Parameters:
    // - `trace` — mission trace to analyze
    //
    // Returns:
    // Chronological fault timeline extracted from trace frames.
    //
    // Options:
    // None.
    //
    // Example:
    // let timeline = faults_from_trace(&trace);

    trace
        .frames
        .iter()
        .filter(|f| is_fault_event(&f.event))
        .map(|f| {
            let kind = f
                .payload
                .get("kind")
                .and_then(|v| v.as_str())
                .map(trace_event_to_kind);
            let target = f
                .payload
                .get("target")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let detail = f
                .payload
                .get("message")
                .and_then(|v| v.as_str())
                .map(String::from);
            FaultTimeline {
                timestamp_ms: f.sim_time_ms,
                event: f.event.clone(),
                fault_kind: kind,
                target,
                status: crate::types::RuntimeHealthStatus::Unknown,
                detail,
            }
        })
        .collect()
}

/// Format fault frames from a trace for CLI display.
pub fn format_trace_faults(trace: &MissionTrace) -> String {
    // Format fault frames from a trace for CLI `--show-faults` output.
    //
    // Parameters:
    // - `trace` — mission trace to display
    //
    // Returns:
    // Human-readable fault summary from trace frames.
    //
    // Options:
    // None.
    //
    // Example:
    // let output = format_trace_faults(&trace);

    let faults = faults_from_trace(trace);
    if faults.is_empty() {
        return "No fault events in trace.".into();
    }
    let mut out = format!(
        "Fault events in {} ({} frames):\n",
        trace.source,
        faults.len()
    );
    for entry in &faults {
        out.push_str(&format!(
            "  {:.0}ms  {}  {}  {}\n",
            entry.timestamp_ms,
            entry.event,
            entry.target,
            entry.detail.as_deref().unwrap_or("")
        ));
    }
    out
}

/// Filter trace frames to fault events only.
pub fn fault_frames(trace: &MissionTrace) -> Vec<&TraceFrame> {
    trace
        .frames
        .iter()
        .filter(|f| is_fault_event(&f.event))
        .collect()
}

fn is_fault_event(event: &str) -> bool {
    event.starts_with("fault_") || FAULT_EVENTS.contains(&event)
}

fn fault_kind_to_trace_event(kind: &RuntimeFaultKind) -> &'static str {
    match kind {
        RuntimeFaultKind::ProcessCrash
        | RuntimeFaultKind::RuntimePanic
        | RuntimeFaultKind::ProviderCrash
        | RuntimeFaultKind::PackageCrash => "fault_crash",
        RuntimeFaultKind::UnexpectedReboot | RuntimeFaultKind::OsReboot => "fault_reboot",
        RuntimeFaultKind::WatchdogTimeout => "fault_watchdog_timeout",
        RuntimeFaultKind::MemoryLeak => "fault_memory_growth",
        RuntimeFaultKind::RestartLoop => "fault_restart_loop",
        RuntimeFaultKind::CpuOverload
        | RuntimeFaultKind::MemoryPressure
        | RuntimeFaultKind::DiskPressure => "fault_resource_pressure",
        RuntimeFaultKind::HeartbeatLoss => "fault_heartbeat_loss",
        RuntimeFaultKind::Deadlock | RuntimeFaultKind::TaskStarvation => "fault_deadlock",
        RuntimeFaultKind::OutOfMemory => "fault_oom",
        _ => "fault_event",
    }
}

fn trace_event_to_kind(s: &str) -> RuntimeFaultKind {
    match s {
        "process_crash" | "runtime_panic" | "provider_crash" | "package_crash" => {
            RuntimeFaultKind::ProcessCrash
        }
        "unexpected_reboot" | "os_reboot" => RuntimeFaultKind::UnexpectedReboot,
        "watchdog_timeout" => RuntimeFaultKind::WatchdogTimeout,
        "memory_leak" => RuntimeFaultKind::MemoryLeak,
        "restart_loop" => RuntimeFaultKind::RestartLoop,
        "heartbeat_loss" => RuntimeFaultKind::HeartbeatLoss,
        "out_of_memory" => RuntimeFaultKind::OutOfMemory,
        "deadlock" | "task_starvation" => RuntimeFaultKind::Deadlock,
        _ => RuntimeFaultKind::AbnormalShutdown,
    }
}
