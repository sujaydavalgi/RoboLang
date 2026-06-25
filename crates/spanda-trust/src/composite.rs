//! Composite trust rollup across packages, device posture, integrity, identity, and safety.

use serde::{Deserialize, Serialize};
use spanda_ast::nodes::{ImportDecl, Program, RobotDecl};
use spanda_package::evaluate_package_trust;
use spanda_readiness::{audit_program, evaluate_safety_coverage};
use spanda_security::validate::security_audit;
use spanda_tamper::detect::{generate_tamper_check, TamperStatus};
use spanda_tamper::integrity::{generate_integrity_report, ArtifactIntegrityStatus, IntegrityReport};
use std::path::Path;

/// Output format for composite trust reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompositeTrustFormat {
    #[default]
    Text,
    Json,
    Markdown,
}

/// Options for composite trust evaluation.
#[derive(Debug, Clone, Default)]
pub struct CompositeTrustOptions {
    pub project_root: Option<std::path::PathBuf>,
}

/// One weighted trust category in a composite report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustCategory {
    pub name: String,
    pub score: u32,
    pub weight: u32,
    pub weighted: u32,
    pub detail: String,
}

/// Composite trust report for a mission program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeTrustReport {
    pub program: String,
    pub score: u32,
    pub tier: String,
    pub integrity_status: String,
    pub categories: Vec<TrustCategory>,
    pub recommendations: Vec<String>,
    pub passed: bool,
}

/// Evaluate composite trust for a parsed mission program.
pub fn evaluate_composite_trust(
    program: &Program,
    source: &str,
    source_label: &str,
    options: &CompositeTrustOptions,
) -> CompositeTrustReport {
    // Blend package, device, firmware, configuration, identity, and safety trust signals.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `source` — raw program source for security audit
    // - `source_label` — file label for reports
    // - `options` — optional project root for package trust lookups
    //
    // Returns:
    // Weighted composite trust report with category breakdown.
    //
    // Options:
    // `CompositeTrustOptions::project_root`.
    //
    // Example:
    // let report = evaluate_composite_trust(&program, &source, "rover.sd", &CompositeTrustOptions::default());

    let tamper = generate_tamper_check(program, source_label);
    let integrity = generate_integrity_report(program, source_label, None, None);
    let project_root = options.project_root.as_deref();

    let package_score = package_trust_score(program, project_root);
    let device_score = device_integrity_score(program);
    let firmware_score = artifact_group_score(&integrity, &["deploy", "hardware"]);
    let configuration_score = artifact_group_score(
        &integrity,
        &[
            "policy",
            "kill_switch",
            "health",
            "tamper_policy",
            "mission",
        ],
    );
    let identity_score = identity_validation_score(program, source);
    let safety_score = safety_integrity_score(program, source_label);

    let categories = vec![
        category(
            "package_trust",
            package_score.0,
            20,
            &package_score.1,
        ),
        category("device_integrity", device_score.0, 20, &device_score.1),
        category("firmware_integrity", firmware_score.0, 15, &firmware_score.1),
        category(
            "configuration_integrity",
            configuration_score.0,
            20,
            &configuration_score.1,
        ),
        category("identity_validation", identity_score.0, 15, &identity_score.1),
        category("safety_integrity", safety_score.0, 10, &safety_score.1),
    ];

    let weighted_sum: u32 = categories.iter().map(|category| category.weighted).sum();
    let score = weighted_sum;
    let tier = trust_tier(score);
    let integrity_status = tamper_status_label(tamper.status);
    let mut recommendations = synthesize_recommendations(&categories, &tamper.status);
    if score < 60 {
        recommendations.push("Composite trust below pass threshold (60)".into());
    }
    let passed = score >= 60 && tamper.passed;

    CompositeTrustReport {
        program: source_label.to_string(),
        score,
        tier,
        integrity_status,
        categories,
        recommendations,
        passed,
    }
}

/// Render a composite trust report for CLI output.
pub fn format_composite_trust(report: &CompositeTrustReport, format: CompositeTrustFormat) -> String {
    // Format composite trust output as text, JSON, or Markdown.
    //
    // Parameters:
    // - `report` — composite trust report
    // - `format` — output format selector
    //
    // Returns:
    // Rendered report string.
    //
    // Options:
    // `CompositeTrustFormat`.
    //
    // Example:
    // let text = format_composite_trust(&report, CompositeTrustFormat::Text);

    match format {
        CompositeTrustFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        CompositeTrustFormat::Markdown => format_markdown(report),
        CompositeTrustFormat::Text => format_text(report),
    }
}

fn category(name: &str, score: u32, weight: u32, detail: &str) -> TrustCategory {
    let weighted = score.saturating_mul(weight) / 100;
    TrustCategory {
        name: name.to_string(),
        score,
        weight,
        weighted,
        detail: detail.to_string(),
    }
}

fn package_trust_score(program: &Program, project_root: Option<&Path>) -> (u32, String) {
    let packages = package_names_from_imports(program);
    if packages.is_empty() {
        return (
            70,
            "no registry packages referenced; neutral baseline".into(),
        );
    }

    let reports: Vec<_> = packages
        .iter()
        .map(|name| evaluate_package_trust(name, None, project_root))
        .collect();
    let total: u32 = reports.iter().map(|report| report.score).sum();
    let average = total / reports.len() as u32;
    let detail = reports
        .iter()
        .map(|report| format!("{}={}/{}", report.package, report.score, report.max_score))
        .collect::<Vec<_>>()
        .join(", ");
    (average, detail)
}

fn package_names_from_imports(program: &Program) -> Vec<String> {
    let mut names = Vec::new();
    for import in program.imports() {
        let ImportDecl::ImportDecl { path, .. } = import;
        if let Some(package) = import_path_to_package(path) {
            if !names.iter().any(|existing| existing == &package) {
                names.push(package);
            }
        }
    }
    names
}

fn import_path_to_package(path: &str) -> Option<String> {
    match path {
        "trust.jetson" => Some("spanda-trust-jetson".into()),
        "trust.pi" => Some("spanda-trust-pi".into()),
        "positioning.gps" => Some("spanda-gps".into()),
        "fusion.sensor" => Some("spanda-fusion".into()),
        other if other.starts_with("spanda-") => Some(other.to_string()),
        _ => None,
    }
}

fn device_integrity_score(program: &Program) -> (u32, String) {
    let imports = program.imports();
    let secure_boot = imports.iter().any(|import| {
        let ImportDecl::ImportDecl { path, .. } = import;
        matches!(path.as_str(), "trust.jetson" | "trust.pi")
    });
    if secure_boot {
        return (
            100,
            "secure boot contract import (trust.jetson or trust.pi)".into(),
        );
    }

    let Program::Program {
        hardware_profiles,
        deployments,
        ..
    } = program;
    if !hardware_profiles.is_empty() && !deployments.is_empty() {
        return (
            80,
            "hardware profile and deploy declared; attestation adapter optional".into(),
        );
    }
    if !hardware_profiles.is_empty() {
        return (65, "hardware profile declared without deploy binding".into());
    }
    (50, "no device attestation or hardware profile".into())
}

fn artifact_group_score(report: &IntegrityReport, kinds: &[&str]) -> (u32, String) {
    let artifacts: Vec<_> = report
        .artifacts
        .iter()
        .filter(|artifact| kinds.contains(&artifact.artifact_type.as_str()))
        .collect();
    if artifacts.is_empty() {
        return (50, format!("no {} artifacts hashed", kinds.join("/")));
    }

    let trusted = artifacts
        .iter()
        .map(|artifact| artifact_status_points(artifact.status))
        .sum::<u32>();
    let score = trusted / artifacts.len() as u32;
    let modified = artifacts
        .iter()
        .filter(|artifact| artifact.status == ArtifactIntegrityStatus::Modified)
        .count();
    let detail = format!(
        "artifacts={} points={} modified={} passed={}",
        artifacts.len(),
        trusted,
        modified,
        report.passed
    );
    (score, detail)
}

fn artifact_status_points(status: ArtifactIntegrityStatus) -> u32 {
    match status {
        ArtifactIntegrityStatus::Trusted => 100,
        ArtifactIntegrityStatus::Unknown => 85,
        ArtifactIntegrityStatus::Modified => 0,
    }
}

fn identity_validation_score(program: &Program, source: &str) -> (u32, String) {
    let audit = security_audit(source).unwrap_or_default();
    let mut score = 100u32;
    let mut errors = 0u32;
    let mut warnings = 0u32;
    for finding in &audit.findings {
        match finding.severity {
            spanda_security::validate::SecuritySeverity::Error => {
                errors += 1;
                score = score.saturating_sub(25);
            }
            spanda_security::validate::SecuritySeverity::Warning => {
                warnings += 1;
                score = score.saturating_sub(8);
            }
            spanda_security::validate::SecuritySeverity::Info => {
                score = score.saturating_sub(2);
            }
        }
    }

    let boundaries = trust_boundary_count(program);
    if boundaries > 0 {
        score = score.max(75);
    }
    let detail = format!(
        "trust_boundaries={} audit_errors={} audit_warnings={}",
        boundaries, errors, warnings
    );
    (score.min(100), detail)
}

fn trust_boundary_count(program: &Program) -> u32 {
    let Program::Program { robots, .. } = program;
    robots
        .iter()
        .map(|robot| {
            let RobotDecl::RobotDecl {
                trust_boundaries, ..
            } = robot;
            trust_boundaries.len() as u32
        })
        .sum()
}

fn safety_integrity_score(program: &Program, source_label: &str) -> (u32, String) {
    let coverage = evaluate_safety_coverage(program, source_label);
    let audit = audit_program(program, source_label);
    let score = coverage
        .overall_coverage_pct
        .saturating_sub(audit.critical_count.saturating_mul(20))
        .saturating_sub(audit.high_count.saturating_mul(8))
        .min(100);
    let detail = format!(
        "coverage={}% critical={} high={}",
        coverage.overall_coverage_pct, audit.critical_count, audit.high_count
    );
    (score, detail)
}

fn trust_tier(score: u32) -> String {
    if score >= 80 {
        "trusted".into()
    } else if score >= 60 {
        "acceptable".into()
    } else {
        "low".into()
    }
}

fn tamper_status_label(status: TamperStatus) -> String {
    match status {
        TamperStatus::Trusted => "Trusted".into(),
        TamperStatus::Suspicious => "Modified".into(),
        TamperStatus::Tampered | TamperStatus::Compromised => "Modified".into(),
        TamperStatus::Unknown => "Unknown".into(),
    }
}

fn synthesize_recommendations(categories: &[TrustCategory], tamper_status: &TamperStatus) -> Vec<String> {
    let mut recommendations = Vec::new();
    for category in categories {
        if category.score < 60 {
            recommendations.push(format!(
                "Improve {} (score {}): {}",
                category.name, category.score, category.detail
            ));
        }
    }
    match tamper_status {
        TamperStatus::Tampered | TamperStatus::Compromised | TamperStatus::Suspicious => {
            recommendations.push("Run spanda tamper-check and spanda diagnose tamper on traces".into());
        }
        TamperStatus::Unknown => {
            recommendations.push("Establish integrity baseline with spanda integrity --baseline".into());
        }
        TamperStatus::Trusted => {}
    }
    recommendations
}

fn format_text(report: &CompositeTrustReport) -> String {
    let mut lines = vec![
        format!("Composite trust: {}", report.program),
        format!(
            "Score: {}/100 tier={} status={} passed={}",
            report.score, report.tier, report.integrity_status, report.passed
        ),
        String::new(),
        "Categories:".into(),
    ];
    for category in &report.categories {
        lines.push(format!(
            "  - {}: {}/100 (weight {}%, weighted {}) — {}",
            category.name,
            category.score,
            category.weight,
            category.weighted,
            category.detail
        ));
    }
    if !report.recommendations.is_empty() {
        lines.push(String::new());
        lines.push("Recommendations:".into());
        for recommendation in &report.recommendations {
            lines.push(format!("  - {recommendation}"));
        }
    }
    lines.join("\n")
}

fn format_markdown(report: &CompositeTrustReport) -> String {
    let mut lines = vec![
        format!("# Composite trust — {}", report.program),
        String::new(),
        format!(
            "**Score:** {}/100 · **Tier:** {} · **Status:** {} · **Passed:** {}",
            report.score, report.tier, report.integrity_status, report.passed
        ),
        String::new(),
        "| Category | Score | Weight | Detail |".into(),
        "|----------|-------|--------|--------|".into(),
    ];
    for category in &report.categories {
        lines.push(format!(
            "| {} | {}/100 | {}% | {} |",
            category.name, category.score, category.weight, category.detail
        ));
    }
    if !report.recommendations.is_empty() {
        lines.push(String::new());
        lines.push("## Recommendations".into());
        for recommendation in &report.recommendations {
            lines.push(format!("- {recommendation}"));
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_lexer::tokenize;
    use spanda_parser::parse;

    fn parse_fixture(source: &str) -> Program {
        parse(tokenize(source).expect("tokenize")).expect("parse")
    }

    #[test]
    fn composite_trust_scores_mission_program() {
        let source = r#"
hardware RoverV1 { sensors [ GPS ]; actuators [ DifferentialDrive ]; }
kill_switch EmergencyStop { priority: critical; action { emergency_stop; } }
robot Rover {
  uses hardware RoverV1;
  sensor gps: GPS;
  actuator wheels: DifferentialDrive;
  safety { max_speed = 0.5 m/s; }
  behavior patrol() { wheels.drive(0.1 m/s); }
}
deploy Rover to RoverV1;
"#;
        let program = parse_fixture(source);
        let report = evaluate_composite_trust(
            &program,
            source,
            "rover.sd",
            &CompositeTrustOptions::default(),
        );
        assert!(report.score > 0);
        assert_eq!(report.categories.len(), 6);
        assert!(!report.tier.is_empty());
    }

    #[test]
    fn secure_boot_import_raises_device_integrity() {
        let source = r#"
import trust.jetson;
hardware Jetson { sensors [ GPS ]; actuators [ DifferentialDrive ]; }
robot Rover {
  uses hardware Jetson;
  sensor gps: GPS;
  actuator wheels: DifferentialDrive;
  behavior patrol() { wheels.drive(0.1 m/s); }
}
"#;
        let program = parse_fixture(source);
        let report = evaluate_composite_trust(
            &program,
            source,
            "jetson.sd",
            &CompositeTrustOptions::default(),
        );
        let device = report
            .categories
            .iter()
            .find(|category| category.name == "device_integrity")
            .expect("device_integrity");
        assert_eq!(device.score, 100);
    }
}
