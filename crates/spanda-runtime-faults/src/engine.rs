//! Runtime fault detection engine — scans programs and evaluates fault state.

use crate::detection::{
    collect_configured_monitors, detect_from_runtime_signals, evaluate_resource_pressure,
    evaluate_restart_loops, static_fault_scan,
};
use crate::types::{
    FaultScanOptions, FaultScanReport, FaultTimeline, RuntimeFault, RuntimeHealth,
    RuntimeHealthStatus, RuntimeReliabilityEvidence,
};
use spanda_ast::nodes::Program;

/// Scan a program for runtime fault configuration and active faults.
pub fn scan_program_faults(
    program: &Program,
    source_label: &str,
    options: &FaultScanOptions,
) -> FaultScanReport {
    // Scan a parsed program for configured monitors and simulated or live fault signals.
    //
    // Parameters:
    // - `program` — parsed Spanda program AST
    // - `source_label` — file path or label for the report
    // - `options` — scan options including fault injection flags
    //
    // Returns:
    // Aggregated fault scan report with health, faults, and timeline.
    //
    // Options:
    // Set injection flags to simulate fault conditions for testing.
    //
    // Example:
    // let report = scan_program_faults(&program, "rover.sd", &FaultScanOptions::default());

    let (hb, mw, rw, rp) = collect_configured_monitors(program);
    let mut faults = static_fault_scan(program);
    let runtime_faults = detect_from_runtime_signals(program, options);
    faults.extend(runtime_faults);

    let resource_faults = evaluate_resource_pressure(program, options);
    faults.extend(resource_faults);

    let restart_faults = evaluate_restart_loops(program, options);
    faults.extend(restart_faults);

    let timeline = build_timeline(&faults, options.sim_time_ms);
    let health = build_runtime_health(&faults, &hb, options.sim_time_ms);
    let passed = health.overall.severity_rank() < RuntimeHealthStatus::Critical.severity_rank();

    FaultScanReport {
        source: source_label.into(),
        health,
        faults,
        timeline,
        heartbeats_configured: hb,
        memory_watches_configured: mw,
        resource_watches_configured: rw,
        restart_policies_configured: rp,
        passed,
    }
}

/// Build runtime health snapshot from active faults.
pub fn build_runtime_health(
    faults: &[RuntimeFault],
    _heartbeat_count: &u32,
    uptime_ms: f64,
) -> RuntimeHealth {
    // Aggregate fault severities into an overall runtime health snapshot.
    //
    // Parameters:
    // - `faults` — active runtime faults
    // - `heartbeat_count` — number of configured heartbeat monitors
    // - `uptime_ms` — current uptime in milliseconds
    //
    // Returns:
    // Runtime health with overall status and active faults.
    //
    // Options:
    // None.
    //
    // Example:
    // let health = build_runtime_health(&faults, &2, 60_000.0);

    let overall = faults
        .iter()
        .map(|f| f.status)
        .max_by_key(|s| s.severity_rank())
        .unwrap_or(RuntimeHealthStatus::Healthy);

    RuntimeHealth {
        overall,
        heartbeats: Vec::new(),
        processes: Vec::new(),
        active_faults: faults.to_vec(),
        uptime_ms,
        boot_id: None,
    }
}

/// Build chronological fault timeline from detected faults.
pub fn build_timeline(faults: &[RuntimeFault], sim_time_ms: f64) -> Vec<FaultTimeline> {
    // Convert detected faults into a chronological timeline.
    //
    // Parameters:
    // - `faults` — detected runtime faults
    // - `sim_time_ms` — simulation time for ordering
    //
    // Returns:
    // Sorted fault timeline entries.
    //
    // Options:
    // None.
    //
    // Example:
    // let timeline = build_timeline(&faults, 0.0);

    let mut timeline: Vec<FaultTimeline> = faults
        .iter()
        .map(|f| FaultTimeline {
            timestamp_ms: f.detected_at_ms,
            event: f.kind.as_str().into(),
            fault_kind: Some(f.kind.clone()),
            target: f.target.clone(),
            status: f.status,
            detail: Some(f.message.clone()),
        })
        .collect();
    timeline.sort_by(|a, b| {
        a.timestamp_ms
            .partial_cmp(&b.timestamp_ms)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if timeline.is_empty() && sim_time_ms > 0.0 {
        timeline.push(FaultTimeline {
            timestamp_ms: sim_time_ms,
            event: "scan_complete".into(),
            fault_kind: None,
            target: String::new(),
            status: RuntimeHealthStatus::Healthy,
            detail: None,
        });
    }
    timeline
}

/// Extract runtime reliability evidence for assurance integration.
pub fn extract_reliability_evidence(
    report: &FaultScanReport,
    uptime_ms: f64,
) -> RuntimeReliabilityEvidence {
    // Build assurance evidence from a fault scan report.
    //
    // Parameters:
    // - `report` — completed fault scan report
    // - `uptime_ms` — system uptime in milliseconds
    //
    // Returns:
    // Runtime reliability evidence record.
    //
    // Options:
    // None.
    //
    // Example:
    // let evidence = extract_reliability_evidence(&report, 3600_000.0);

    let crash_count = report
        .faults
        .iter()
        .filter(|f| {
            matches!(
                f.kind,
                crate::types::RuntimeFaultKind::ProcessCrash
                    | crate::types::RuntimeFaultKind::RuntimePanic
                    | crate::types::RuntimeFaultKind::ProviderCrash
                    | crate::types::RuntimeFaultKind::PackageCrash
            )
        })
        .count() as u32;

    let reboot_count = report
        .faults
        .iter()
        .filter(|f| {
            matches!(
                f.kind,
                crate::types::RuntimeFaultKind::UnexpectedReboot
                    | crate::types::RuntimeFaultKind::OsReboot
            )
        })
        .count() as u32;

    let memory_leak = report
        .faults
        .iter()
        .any(|f| f.kind == crate::types::RuntimeFaultKind::MemoryLeak);

    RuntimeReliabilityEvidence {
        uptime_ms,
        crash_free_duration_ms: if crash_count == 0 { uptime_ms } else { 0.0 },
        reboot_count,
        unexpected_reboot_count: reboot_count,
        memory_stable: !memory_leak,
        watchdog_coverage: report.restart_policies_configured,
        restart_policies: report.restart_policies_configured,
        heartbeat_monitors: report.heartbeats_configured,
    }
}
