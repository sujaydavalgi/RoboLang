//! Tamper diagnosis from mission traces with recovery recommendations.

use crate::detect::{TamperFinding, TamperSeverity, TamperStatus};
use crate::runtime::{generate_runtime_tamper_check, MissionTrace, TraceFrame};
use serde::{Deserialize, Serialize};
use spanda_spoofing::trace::{analyze_trace_spoofing, SpoofingAlert, DEFAULT_MAX_GROUND_SPEED_M_S};
use std::collections::BTreeSet;

/// Output format for tamper diagnosis reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TamperDiagnosisFormat {
    #[default]
    Text,
    Json,
}

/// One tamper-relevant event on the mission timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TamperTimelineEvent {
    pub sim_time_ms: f64,
    pub event: String,
    pub detail: String,
    pub severity: TamperSeverity,
}

/// Full tamper diagnosis report for a mission trace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TamperDiagnosisReport {
    pub source: String,
    pub tamper_status: TamperStatus,
    pub trust_score: u32,
    pub tamper_source: String,
    pub affected_components: Vec<String>,
    pub impact: String,
    pub findings: Vec<TamperFinding>,
    pub spoofing_alerts: Vec<SpoofingAlert>,
    pub timeline: Vec<TamperTimelineEvent>,
    pub recovery_recommendations: Vec<String>,
    pub passed: bool,
}

/// Diagnose tamper source, impact, and recovery actions from a mission trace.
pub fn diagnose_tamper_trace(trace: &MissionTrace, source_label: &str) -> TamperDiagnosisReport {
    // Compose runtime tamper findings, spoofing alerts, and operator guidance.
    //
    // Parameters:
    // - `trace` — mission trace with security and sensor frames
    // - `source_label` — file label for the report
    //
    // Returns:
    // Tamper diagnosis report with timeline and recovery recommendations.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = diagnose_tamper_trace(&trace, "intrusion.trace");

    let tamper = generate_runtime_tamper_check(trace, source_label);
    let spoof_trace = spanda_spoofing::trace::MissionTrace {
        version: trace.version,
        source: trace.source.clone(),
        deterministic: trace.deterministic,
        frames: trace
            .frames
            .iter()
            .map(|frame| spanda_spoofing::trace::TraceFrame {
                sim_time_ms: frame.sim_time_ms,
                event: frame.event.clone(),
                payload: frame.payload.clone(),
            })
            .collect(),
    };
    let spoofing_alerts = analyze_trace_spoofing(&spoof_trace, DEFAULT_MAX_GROUND_SPEED_M_S);
    let timeline = build_tamper_timeline(trace, &tamper.findings, &spoofing_alerts);
    let affected_components = collect_affected_components(trace, &tamper.findings, &spoofing_alerts);
    let tamper_source = derive_tamper_source(&tamper, &spoofing_alerts);
    let impact = derive_impact(&tamper.status, &spoofing_alerts);
    let recovery_recommendations =
        build_recovery_recommendations(&tamper, &spoofing_alerts, &affected_components);
    let passed = tamper.passed && spoofing_alerts.iter().all(|alert| {
        !matches!(
            alert.severity,
            spanda_spoofing::trace::SpoofingSeverity::High
                | spanda_spoofing::trace::SpoofingSeverity::Critical
        )
    });

    TamperDiagnosisReport {
        source: source_label.into(),
        tamper_status: tamper.status,
        trust_score: tamper.trust_score,
        tamper_source,
        affected_components,
        impact,
        findings: tamper.findings,
        spoofing_alerts,
        timeline,
        recovery_recommendations,
        passed,
    }
}

/// Format a tamper diagnosis report for CLI output.
pub fn format_tamper_diagnosis(report: &TamperDiagnosisReport, format: TamperDiagnosisFormat) -> String {
    // Render tamper diagnosis as JSON or human-readable text.
    //
    // Parameters:
    // - `report` — tamper diagnosis report
    // - `format` — text or JSON output
    //
    // Returns:
    // Formatted report string.
    //
    // Options:
    // None.
    //
    // Example:
    // println!("{}", format_tamper_diagnosis(&report, TamperDiagnosisFormat::Text));

    match format {
        TamperDiagnosisFormat::Json => {
            serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".into())
        }
        TamperDiagnosisFormat::Text => format_tamper_diagnosis_text(report),
    }
}

fn format_tamper_diagnosis_text(report: &TamperDiagnosisReport) -> String {
    let mut lines = vec![
        format!("Tamper diagnosis: {}", report.source),
        format!("Status: {:?} (trust {}/100)", report.tamper_status, report.trust_score),
        format!("Tamper source: {}", report.tamper_source),
        format!("Impact: {}", report.impact),
    ];

    if !report.affected_components.is_empty() {
        lines.push(String::new());
        lines.push("Affected components:".into());
        for component in &report.affected_components {
            lines.push(format!("  - {component}"));
        }
    }

    if !report.timeline.is_empty() {
        lines.push(String::new());
        lines.push("Timeline:".into());
        for event in &report.timeline {
            lines.push(format!(
                "  T+{:.0}ms [{:?}] {} — {}",
                event.sim_time_ms, event.severity, event.event, event.detail
            ));
        }
    }

    if !report.findings.is_empty() {
        lines.push(String::new());
        lines.push("Tamper findings:".into());
        for finding in &report.findings {
            lines.push(format!(
                "  [{:?}] {} — {}",
                finding.severity, finding.category, finding.message
            ));
        }
    }

    if !report.spoofing_alerts.is_empty() {
        lines.push(String::new());
        lines.push("Spoofing alerts:".into());
        for alert in &report.spoofing_alerts {
            lines.push(format!(
                "  [{:?}] {} ({:.0}%) — {}",
                alert.severity,
                alert.sensor,
                alert.confidence * 100.0,
                alert.message
            ));
        }
    }

    if !report.recovery_recommendations.is_empty() {
        lines.push(String::new());
        lines.push("Recovery recommendations:".into());
        for action in &report.recovery_recommendations {
            lines.push(format!("  * {action}"));
        }
    }

    lines.push(String::new());
    lines.push(format!(
        "Result: {}",
        if report.passed { "PASS" } else { "FAIL" }
    ));
    lines.join("\n")
}

fn build_tamper_timeline(
    trace: &MissionTrace,
    findings: &[TamperFinding],
    spoofing_alerts: &[SpoofingAlert],
) -> Vec<TamperTimelineEvent> {
    let mut timeline = Vec::new();

    for frame in &trace.frames {
        if !frame_is_tamper_relevant(frame) {
            continue;
        }
        timeline.push(TamperTimelineEvent {
            sim_time_ms: frame.sim_time_ms,
            event: frame.event.clone(),
            detail: frame.payload.to_string(),
            severity: severity_for_frame(frame),
        });
    }

    if timeline.is_empty() {
        for finding in findings {
            timeline.push(TamperTimelineEvent {
                sim_time_ms: parse_sim_time_from_evidence(finding.evidence.as_deref()),
                event: finding.category.clone(),
                detail: finding.message.clone(),
                severity: finding.severity,
            });
        }
    }

    for alert in spoofing_alerts {
        if let Some(sim_time_ms) = alert.sim_time_ms {
            timeline.push(TamperTimelineEvent {
                sim_time_ms,
                event: format!("spoofing.{}", alert.sensor),
                detail: alert.message.clone(),
                severity: spoofing_to_tamper_severity(alert.severity),
            });
        }
    }

    timeline.sort_by(|left, right| {
        left.sim_time_ms
            .partial_cmp(&right.sim_time_ms)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    timeline
}

fn collect_affected_components(
    trace: &MissionTrace,
    findings: &[TamperFinding],
    spoofing_alerts: &[SpoofingAlert],
) -> Vec<String> {
    let mut components = BTreeSet::new();

    for frame in &trace.frames {
        if let Some(obj) = frame.payload.as_object() {
            for key in ["agent", "robot", "capability", "target", "sensor", "module"] {
                if let Some(value) = obj.get(key).and_then(|field| field.as_str()) {
                    components.insert(format!("{key}:{value}"));
                }
            }
        }
    }

    if components.is_empty() && !findings.is_empty() {
        components.insert("runtime".into());
    }

    for alert in spoofing_alerts {
        components.insert(format!("sensor:{}", alert.sensor));
    }

    components.into_iter().collect()
}

fn derive_tamper_source(
    tamper: &crate::detect::TamperReport,
    spoofing_alerts: &[SpoofingAlert],
) -> String {
    if spoofing_alerts.iter().any(|alert| {
        matches!(
            alert.severity,
            spanda_spoofing::trace::SpoofingSeverity::High
                | spanda_spoofing::trace::SpoofingSeverity::Critical
        )
    }) {
        return "GPS or sensor spoofing detected in mission trace".into();
    }

    if tamper
        .findings
        .iter()
        .any(|finding| finding.category == "capability_monitor")
    {
        return "Unauthorized capability use or agent intrusion".into();
    }

    if tamper.findings.is_empty() {
        return "No tamper indicators in trace".into();
    }

    "Runtime tamper or integrity violation".into()
}

fn derive_impact(status: &TamperStatus, spoofing_alerts: &[SpoofingAlert]) -> String {
    if matches!(status, TamperStatus::Compromised)
        || spoofing_alerts.iter().any(|alert| {
            matches!(
                alert.severity,
                spanda_spoofing::trace::SpoofingSeverity::Critical
            )
        })
    {
        return "Critical — mission trust compromised; halt actuators and require operator approval"
            .into();
    }

    if matches!(status, TamperStatus::Tampered) {
        return "High — unauthorized runtime behavior detected; enter safe mode and audit".into();
    }

    if matches!(status, TamperStatus::Suspicious) {
        return "Medium — suspicious signals recorded; continue in degraded mode with monitoring".into();
    }

    "Low — no material tamper impact detected".into()
}

fn build_recovery_recommendations(
    tamper: &crate::detect::TamperReport,
    spoofing_alerts: &[SpoofingAlert],
    affected_components: &[String],
) -> Vec<String> {
    let mut actions = Vec::new();

    if tamper
        .findings
        .iter()
        .any(|finding| finding.category == "capability_monitor")
    {
        actions.push("Review agent capability grants and enable capability_enforced where missing".into());
        actions.push("Audit denied actions and rotate agent credentials if intrusion is suspected".into());
    }

    if !spoofing_alerts.is_empty() {
        actions.push("Cross-check GPS fixes against IMU odometry and declared geofence bounds".into());
        actions.push("Run spanda spoof-check on the mission trace and source program".into());
    }

    if tamper.trust_score < 70 {
        actions.push("Compare deployed program hash against approved baseline with spanda integrity".into());
    }

    if affected_components.iter().any(|c| c.starts_with("agent:")) {
        actions.push("Isolate affected agents and verify trust_boundary policies".into());
    }

    actions.push("Replay trace with spanda replay --deterministic to confirm findings".into());
    actions.push("Record tamper response in audit trail and escalate per tamper_policy".into());
    actions
}

fn frame_is_tamper_relevant(frame: &TraceFrame) -> bool {
    let event_lower = frame.event.to_ascii_lowercase();
    if [
        "tamper",
        "intrusion",
        "capability_denied",
        "security_audit",
        "unauthorized",
        "spoof",
    ]
    .iter()
    .any(|needle| event_lower.contains(needle))
    {
        return true;
    }

    frame
        .payload
        .as_object()
        .map(payload_indicates_tamper_or_denial)
        .unwrap_or(false)
}

fn payload_indicates_tamper_or_denial(obj: &serde_json::Map<String, serde_json::Value>) -> bool {
    if obj
        .get("denied")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        return true;
    }
    if obj
        .get("unauthorized")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        return true;
    }
    if obj
        .get("tampered")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        return true;
    }

    matches!(
        obj.get("kind")
            .or_else(|| obj.get("event"))
            .and_then(|value| value.as_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some(kind) if kind.contains("denied") || kind.contains("spoof") || kind.contains("tamper")
    )
}

fn severity_for_frame(frame: &TraceFrame) -> TamperSeverity {
    let event_lower = frame.event.to_ascii_lowercase();
    if event_lower.contains("critical") || event_lower.contains("compromise") {
        return TamperSeverity::Critical;
    }
    if event_lower.contains("denied") || event_lower.contains("unauthorized") {
        return TamperSeverity::High;
    }
    TamperSeverity::Medium
}

fn parse_sim_time_from_evidence(evidence: Option<&str>) -> f64 {
    let Some(evidence) = evidence else {
        return 0.0;
    };
    evidence
        .split("sim_time_ms=")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|value| value.trim_end_matches(',').parse().ok())
        .unwrap_or(0.0)
}

fn spoofing_to_tamper_severity(
    severity: spanda_spoofing::trace::SpoofingSeverity,
) -> TamperSeverity {
    match severity {
        spanda_spoofing::trace::SpoofingSeverity::Info => TamperSeverity::Info,
        spanda_spoofing::trace::SpoofingSeverity::Low => TamperSeverity::Low,
        spanda_spoofing::trace::SpoofingSeverity::Medium => TamperSeverity::Medium,
        spanda_spoofing::trace::SpoofingSeverity::High => TamperSeverity::High,
        spanda_spoofing::trace::SpoofingSeverity::Critical => TamperSeverity::Critical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn diagnoses_runtime_intrusion_trace() {
        let path = format!(
            "{}/../../examples/showcase/runtime_intrusion/intrusion.trace",
            env!("CARGO_MANIFEST_DIR")
        );
        let raw = fs::read_to_string(&path).expect("read intrusion trace");
        let trace: MissionTrace = serde_json::from_str(&raw).expect("parse trace");
        let report = diagnose_tamper_trace(&trace, "intrusion.trace");
        assert!(!report.passed);
        assert!(report.tamper_source.contains("intrusion")
            || report.tamper_source.contains("capability"));
        assert!(report
            .affected_components
            .iter()
            .any(|component| component.contains("Intruder") || component.contains("Rover")));
    }

    #[test]
    fn diagnoses_gps_spoofing_trace() {
        let path = format!(
            "{}/../../examples/showcase/gps_spoofing/spoof.trace",
            env!("CARGO_MANIFEST_DIR")
        );
        let raw = fs::read_to_string(&path).expect("read spoof trace");
        let trace: MissionTrace = serde_json::from_str(&raw).expect("parse trace");
        let report = diagnose_tamper_trace(&trace, "spoof.trace");
        assert!(!report.passed);
        assert!(report.tamper_source.to_ascii_lowercase().contains("spoof"));
        assert!(!report.spoofing_alerts.is_empty());
    }
}
