//! Unified spoof-check report generation for programs and mission traces.

use crate::coverage::{analyze_spoofing_coverage, SpoofingCoverageCheck};
use crate::trace::{
    analyze_trace_spoofing, MissionTrace, SpoofingAlert, SpoofingSeverity,
    DEFAULT_MAX_GROUND_SPEED_M_S,
};
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use std::fs;
use std::path::Path;

/// Output format for spoofing reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpoofingFormat {
    #[default]
    Text,
    Json,
}

/// Whether spoof-check analyzed a program or a mission trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpoofingSourceKind {
    Program,
    Trace,
}

/// Full spoof-check report for CLI and automation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpoofingReport {
    pub source: String,
    pub kind: SpoofingSourceKind,
    pub coverage_score: Option<u32>,
    pub coverage_checks: Vec<SpoofingCoverageCheck>,
    pub alerts: Vec<SpoofingAlert>,
    pub passed: bool,
    #[serde(default)]
    pub ml_alerts_merged: u32,
    #[serde(default)]
    pub suppressed_low_confidence: u32,
    #[serde(default)]
    pub requires_operator_confirmation: bool,
}

/// Run spoof-check on a parsed program (static coverage analysis).
pub fn generate_program_spoof_check(program: &Program, source_label: &str) -> SpoofingReport {
    // Evaluate spoofing detection coverage for a Spanda program artifact.
    //
    // Parameters:
    // - `program` — parsed program
    // - `source_label` — file label for the report
    //
    // Returns:
    // Spoofing readiness report with coverage score and gap alerts.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = generate_program_spoof_check(&program, "rover.sd");

    let (coverage_score, coverage_checks) = analyze_spoofing_coverage(program);
    let mut alerts = Vec::new();

    for check in &coverage_checks {
        if check.passed {
            continue;
        }

        let severity = match check.id.as_str() {
            "gps_sensor" | "cross_sensor_fusion" | "spoof_handler" => SpoofingSeverity::High,
            "gps_health_bounds" => SpoofingSeverity::Medium,
            _ => SpoofingSeverity::Low,
        };

        alerts.push(SpoofingAlert {
            sensor: "program".into(),
            severity,
            confidence: if check.passed { 0.0 } else { 0.8 },
            message: check.label.clone(),
            evidence: check.detail.clone().unwrap_or_else(|| check.id.clone()),
            sim_time_ms: None,
        });
    }

    let passed = coverage_score >= 60
        && coverage_checks
            .iter()
            .find(|check| check.id == "gps_sensor")
            .map(|check| check.passed)
            .unwrap_or(false)
        && coverage_checks
            .iter()
            .find(|check| check.id == "spoof_handler")
            .map(|check| check.passed)
            .unwrap_or(false);

    SpoofingReport {
        source: source_label.into(),
        kind: SpoofingSourceKind::Program,
        coverage_score: Some(coverage_score),
        coverage_checks,
        alerts,
        passed,
        ml_alerts_merged: 0,
        suppressed_low_confidence: 0,
        requires_operator_confirmation: false,
    }
}

/// Run spoof-check on a mission trace file (runtime plausibility analysis).
pub fn generate_trace_spoof_check(trace: &MissionTrace, source_label: &str) -> SpoofingReport {
    // Evaluate a recorded mission trace for GPS spoofing and plausibility violations.
    //
    // Parameters:
    // - `trace` — mission trace
    // - `source_label` — file label for the report
    //
    // Returns:
    // Spoofing report with runtime alerts.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = generate_trace_spoof_check(&trace, "mission.trace");

    let mut alerts = analyze_trace_spoofing(trace, DEFAULT_MAX_GROUND_SPEED_M_S);
    let ml_before = alerts.len();
    crate::ml::merge_ml_spoofing_alerts(trace, &mut alerts);
    let ml_alerts_merged = alerts.len().saturating_sub(ml_before) as u32;
    let suppressed_low_confidence = crate::confidence::apply_spoofing_confidence_filter(&mut alerts);
    let requires_operator_confirmation = crate::confidence::requires_operator_confirmation(&alerts);
    let passed = !alerts.iter().any(|alert| {
        matches!(
            alert.severity,
            SpoofingSeverity::High | SpoofingSeverity::Critical
        )
    });

    SpoofingReport {
        source: source_label.into(),
        kind: SpoofingSourceKind::Trace,
        coverage_score: None,
        coverage_checks: Vec::new(),
        alerts,
        passed,
        ml_alerts_merged,
        suppressed_low_confidence,
        requires_operator_confirmation,
    }
}

/// Load a path and run the appropriate spoof-check analysis.
pub fn analyze_path(path: &Path) -> Result<SpoofingReport, String> {
    // Dispatch spoof-check to program or trace analysis based on file extension.
    //
    // Parameters:
    // - `path` — `.sd` program or `.trace` mission file
    //
    // Returns:
    // Spoofing report, or I/O / parse error message.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = analyze_path(Path::new("rover.sd"))?;

    let label = path.display().to_string();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if extension == "trace" {
        let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
        let trace: MissionTrace = serde_json::from_str(&raw).map_err(|error| error.to_string())?;
        return Ok(generate_trace_spoof_check(&trace, &label));
    }

    Err(format!(
        "Unsupported spoof-check input `{}` (expected .sd or .trace)",
        label
    ))
}

/// Format a spoofing report for CLI output.
pub fn format_spoofing_report(report: &SpoofingReport, format: SpoofingFormat) -> String {
    // Render spoofing report as JSON or human-readable text.
    //
    // Parameters:
    // - `report` — spoof-check report
    // - `format` — text or JSON output
    //
    // Returns:
    // Formatted report string.
    //
    // Options:
    // None.
    //
    // Example:
    // println!("{}", format_spoofing_report(&report, SpoofingFormat::Text));

    match format {
        SpoofingFormat::Json => {
            serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".into())
        }
        SpoofingFormat::Text => format_spoofing_text(report),
    }
}

fn format_spoofing_text(report: &SpoofingReport) -> String {
    let mut lines = vec![
        format!("Spoofing check: {}", report.source),
        format!("Kind: {:?}", report.kind),
    ];

    if let Some(score) = report.coverage_score {
        lines.push(format!("Coverage score: {score}/100"));
        lines.push(String::new());
        lines.push("Coverage checks:".into());
        for check in &report.coverage_checks {
            let mark = if check.passed { "ok" } else { "gap" };
            lines.push(format!("  [{mark}] {} ({})", check.label, check.weight));
            if let Some(detail) = &check.detail {
                lines.push(format!("       {detail}"));
            }
        }
    }

    if report.ml_alerts_merged > 0 {
        lines.push(format!("ML alerts merged: {}", report.ml_alerts_merged));
    }
    if report.suppressed_low_confidence > 0 {
        lines.push(format!(
            "Suppressed low-confidence alerts: {}",
            report.suppressed_low_confidence
        ));
    }
    if report.requires_operator_confirmation {
        lines.push(
            "Operator confirmation required before destructive tamper response (set SPANDA_OPERATOR_APPROVAL=1 in sim to bypass)."
                .into(),
        );
    }

    if !report.alerts.is_empty() {
        lines.push(String::new());
        lines.push("Alerts:".into());
        for alert in &report.alerts {
            lines.push(format!(
                "  [{:?}] {} (confidence {:.0}%) — {}",
                alert.severity,
                alert.sensor,
                alert.confidence * 100.0,
                alert.message
            ));
            lines.push(format!("       evidence: {}", alert.evidence));
        }
    } else {
        lines.push(String::new());
        lines.push("No spoofing alerts.".into());
    }

    lines.push(String::new());
    lines.push(format!(
        "Result: {}",
        if report.passed { "PASS" } else { "FAIL" }
    ));
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_lexer::tokenize;
    use spanda_parser::parse;

    #[test]
    fn program_report_marks_incomplete_stack_as_failed() {
        let source = r#"
hardware R { sensors [ GPS ]; actuators [ DifferentialDrive ]; }
robot Rover { sensor gps: GPS; }
deploy Rover to R;
"#;
        let program = parse(tokenize(source).unwrap()).unwrap();
        let report = generate_program_spoof_check(&program, "bare.sd");
        assert!(!report.passed);
        assert!(report.coverage_score.unwrap_or(100) < 60);
    }
}
