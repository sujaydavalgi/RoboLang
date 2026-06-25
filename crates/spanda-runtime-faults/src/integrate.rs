//! Integration with readiness, assurance, diagnosis, and recovery systems.

use crate::types::{
    CrashEvent, FaultDiagnosis, FaultRecoveryAction, FaultScanReport, RuntimeFault,
    RuntimeFaultKind, RuntimeHealthStatus,
};
use spanda_ast::fault_decl::RuntimeFaultTriggerDecl;
use spanda_ast::nodes::Program;
use spanda_readiness::types::{
    ReadinessIssue, ReadinessReport, ReadinessSeverity, ReadinessStatus,
};

/// Apply runtime fault impact to a readiness report.
pub fn apply_fault_readiness_impact(report: &mut ReadinessReport, fault_report: &FaultScanReport) {
    // Lower readiness score and add issues based on detected runtime faults.
    //
    // Parameters:
    // - `report` — readiness report to modify in place
    // - `fault_report` — fault scan results
    //
    // Returns:
    // None (modifies report in place).
    //
    // Options:
    // None.
    //
    // Example:
    // apply_fault_readiness_impact(&mut readiness, &fault_report);

    for fault in &fault_report.faults {
        let (severity, action) = fault_readiness_impact(fault);
        report.issues.push(ReadinessIssue {
            factor: "Runtime".into(),
            severity,
            message: fault.message.clone(),
            suggested_action: Some(action),
        });
    }

    if fault_report.faults.iter().any(|f| {
        matches!(
            f.kind,
            RuntimeFaultKind::RestartLoop | RuntimeFaultKind::ProcessCrash
        ) && f.status == RuntimeHealthStatus::Critical
    }) {
        report.status = ReadinessStatus::NotReady;
    } else if fault_report
        .faults
        .iter()
        .any(|f| f.status.severity_rank() >= RuntimeHealthStatus::Degraded.severity_rank())
    {
        if report.status == ReadinessStatus::Ready {
            report.status = ReadinessStatus::Degraded;
        }
    }

    if fault_report
        .faults
        .iter()
        .any(|f| f.kind == RuntimeFaultKind::MemoryLeak)
    {
        if let Some(score) = report
            .score
            .factors
            .iter_mut()
            .find(|f| f.factor == "Health")
        {
            score.score = score.score.saturating_sub(15);
        }
    }

    if fault_report
        .faults
        .iter()
        .any(|f| f.kind == RuntimeFaultKind::UnexpectedReboot)
    {
        report.issues.push(ReadinessIssue {
            factor: "Runtime".into(),
            severity: ReadinessSeverity::High,
            message: "Unexpected reboot requires operator review before deployment".into(),
            suggested_action: Some("Run post-reboot diagnostics".into()),
        });
    }
}

/// Diagnose a runtime fault with causal explanation.
pub fn diagnose_fault(fault: &RuntimeFault) -> FaultDiagnosis {
    // Produce a human-readable diagnosis for a runtime fault.
    //
    // Parameters:
    // - `fault` — detected runtime fault
    //
    // Returns:
    // Diagnosis with what/when/why/affected/recovery fields.
    //
    // Options:
    // None.
    //
    // Example:
    // let diagnosis = diagnose_fault(&fault);

    let likely_cause = match fault.kind {
        RuntimeFaultKind::ProcessCrash => "Abnormal process exit or unhandled signal".into(),
        RuntimeFaultKind::RuntimePanic => "Unhandled panic in runtime thread".into(),
        RuntimeFaultKind::MemoryLeak => {
            "Sustained memory growth exceeding configured threshold".into()
        }
        RuntimeFaultKind::UnexpectedReboot | RuntimeFaultKind::OsReboot => {
            "Power loss, kernel panic, or watchdog-forced reboot".into()
        }
        RuntimeFaultKind::WatchdogTimeout => "Task or pipeline exceeded watchdog timeout".into(),
        RuntimeFaultKind::OutOfMemory => "Memory allocation failed — system OOM".into(),
        RuntimeFaultKind::Deadlock | RuntimeFaultKind::TaskStarvation => {
            "Scheduler starvation or blocked queue".into()
        }
        RuntimeFaultKind::RestartLoop => "Repeated crash/restart within policy window".into(),
        RuntimeFaultKind::HeartbeatLoss => "Runtime heartbeat not received within timeout".into(),
        RuntimeFaultKind::CpuOverload
        | RuntimeFaultKind::MemoryPressure
        | RuntimeFaultKind::DiskPressure => "Resource usage exceeded configured threshold".into(),
        _ => "Runtime fault detected — see evidence for details".into(),
    };

    FaultDiagnosis {
        what: format!("{} on '{}'", fault.kind.as_str(), fault.target),
        when_ms: fault.detected_at_ms,
        likely_cause,
        affected: vec![fault.target.clone()],
        recovery_successful: None,
    }
}

/// Diagnose all faults in a scan report.
pub fn diagnose_fault_report(fault_report: &FaultScanReport) -> Vec<FaultDiagnosis> {
    // Diagnose every fault in a scan report.
    //
    // Parameters:
    // - `fault_report` — completed fault scan report
    //
    // Returns:
    // List of fault diagnoses.
    //
    // Options:
    // None.
    //
    // Example:
    // let diagnoses = diagnose_fault_report(&report);

    fault_report.faults.iter().map(diagnose_fault).collect()
}

/// Recommend recovery actions for a fault, respecting safety validation.
pub fn recommend_recovery(fault: &RuntimeFault, program: &Program) -> Vec<FaultRecoveryAction> {
    // Map fault kind to allowed recovery actions from program policies.
    //
    // Parameters:
    // - `fault` — detected runtime fault
    // - `program` — parsed program with recovery policies
    //
    // Returns:
    // List of recommended recovery actions.
    //
    // Options:
    // Recovery actions always require safety validation.
    //
    // Example:
    // let actions = recommend_recovery(&fault, &program);

    let Program::Program {
        runtime_fault_triggers,
        ..
    } = program;

    let mut actions = Vec::new();

    for trigger in runtime_fault_triggers {
        let RuntimeFaultTriggerDecl::RuntimeFaultTriggerDecl { event, body, .. } = trigger;
        if event_matches_fault(event, &fault.kind) {
            for action in body {
                actions.push(FaultRecoveryAction {
                    action: action.clone(),
                    target: fault.target.clone(),
                    requires_approval: action.contains("operator") || action.contains("approval"),
                    safety_validated: !action.contains("kill_switch")
                        || program_has_safety(program),
                });
            }
        }
    }

    if actions.is_empty() {
        actions.extend(default_recovery_for_kind(&fault.kind, &fault.target));
    }

    actions
}

/// Build crash event from a runtime fault.
pub fn fault_to_crash_event(fault: &RuntimeFault) -> Option<CrashEvent> {
    // Convert a process crash fault into a structured crash event.
    //
    // Parameters:
    // - `fault` — runtime fault (must be crash-related)
    //
    // Returns:
    // Crash event if fault is crash-related, otherwise None.
    //
    // Options:
    // None.
    //
    // Example:
    // let event = fault_to_crash_event(&fault);

    match fault.kind {
        RuntimeFaultKind::ProcessCrash
        | RuntimeFaultKind::RuntimePanic
        | RuntimeFaultKind::ProviderCrash
        | RuntimeFaultKind::PackageCrash => Some(CrashEvent {
            process: fault.target.clone(),
            exit_code: fault.evidence.exit_code.unwrap_or(-1),
            signal: None,
            panic_message: fault.evidence.stack_trace.clone(),
            timestamp_ms: fault.detected_at_ms,
            abnormal: true,
        }),
        _ => None,
    }
}

fn fault_readiness_impact(fault: &RuntimeFault) -> (ReadinessSeverity, String) {
    match fault.status {
        RuntimeHealthStatus::Crashed | RuntimeHealthStatus::Critical => (
            ReadinessSeverity::Critical,
            "Block deployment until fault is resolved".into(),
        ),
        RuntimeHealthStatus::Rebooted => (
            ReadinessSeverity::High,
            "Run post-reboot diagnostics".into(),
        ),
        RuntimeHealthStatus::Degraded => (
            ReadinessSeverity::Medium,
            "Review degraded runtime before mission start".into(),
        ),
        RuntimeHealthStatus::Warning => (
            ReadinessSeverity::Low,
            "Monitor runtime health during mission".into(),
        ),
        _ => (ReadinessSeverity::Info, "No action required".into()),
    }
}

fn event_matches_fault(event: &str, kind: &RuntimeFaultKind) -> bool {
    let event_lower = event.to_lowercase();
    match kind {
        RuntimeFaultKind::ProcessCrash | RuntimeFaultKind::RuntimePanic => {
            event_lower.contains("crash")
        }
        RuntimeFaultKind::MemoryLeak => event_lower.contains("memory"),
        RuntimeFaultKind::UnexpectedReboot | RuntimeFaultKind::OsReboot => {
            event_lower.contains("reboot")
        }
        RuntimeFaultKind::RestartLoop => event_lower.contains("restart"),
        RuntimeFaultKind::WatchdogTimeout => event_lower.contains("watchdog"),
        RuntimeFaultKind::HeartbeatLoss => event_lower.contains("heartbeat"),
        _ => false,
    }
}

fn default_recovery_for_kind(kind: &RuntimeFaultKind, target: &str) -> Vec<FaultRecoveryAction> {
    match kind {
        RuntimeFaultKind::ProcessCrash | RuntimeFaultKind::ProviderCrash => {
            vec![FaultRecoveryAction {
                action: format!("restart {target}"),
                target: target.into(),
                requires_approval: false,
                safety_validated: true,
            }]
        }
        RuntimeFaultKind::RestartLoop => vec![FaultRecoveryAction {
            action: "enter SafeMode".into(),
            target: target.into(),
            requires_approval: true,
            safety_validated: true,
        }],
        RuntimeFaultKind::MemoryLeak => vec![FaultRecoveryAction {
            action: "notify_operator".into(),
            target: target.into(),
            requires_approval: true,
            safety_validated: true,
        }],
        RuntimeFaultKind::ActuatorDriverCrash => vec![FaultRecoveryAction {
            action: "trigger kill switch".into(),
            target: target.into(),
            requires_approval: false,
            safety_validated: true,
        }],
        _ => vec![FaultRecoveryAction {
            action: "enter degraded_mode".into(),
            target: target.into(),
            requires_approval: false,
            safety_validated: true,
        }],
    }
}

fn program_has_safety(program: &Program) -> bool {
    let Program::Program { robots, .. } = program;
    robots.iter().any(|r| {
        let spanda_ast::nodes::RobotDecl::RobotDecl { safety, .. } = r;
        safety.is_some()
    })
}
