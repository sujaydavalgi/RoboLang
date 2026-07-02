//! What-if scenario orchestration composing recovery, readiness, and chaos injection labels.

use serde::{Deserialize, Serialize};
use spanda_assurance::{simulate_failure_recovery, RecoveryReport, RecoveryStatus};
use spanda_ast::nodes::Program;
use spanda_chaos::{default_injections, normalize_injection};
use spanda_readiness::{evaluate_readiness, ReadinessOptions, ReadinessSeverity};

/// Output format for what-if reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WhatIfFormat {
    #[default]
    Text,
    Json,
}

/// Rollup counts for scenario risk bands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhatIfScenarioSummary {
    pub total: usize,
    pub mission_completion_likely: usize,
    pub high_risk: usize,
}

/// Per-scenario predicted outcome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhatIfScenarioResult {
    pub scenario: String,
    pub impact: String,
    pub risk: String,
    pub recovery_plan: String,
    pub probability: f64,
    pub mission_completion_likely: bool,
    pub recovery_passed: bool,
    pub readiness_passed: bool,
    pub details: Vec<String>,
}

/// What-if analysis report for a program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhatIfReport {
    pub program: String,
    pub scenarios: Vec<WhatIfScenarioResult>,
    pub summary: WhatIfScenarioSummary,
}

/// Options for what-if analysis.
#[derive(Debug, Clone, Default)]
pub struct WhatIfOptions {
    pub scenarios: Vec<String>,
    pub all: bool,
}

/// Normalize user-facing scenario labels to internal failure kinds.
pub fn normalize_scenario(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_ascii_lowercase().replace('_', "-");
    if let Some(kind) = normalize_injection(&normalized) {
        return Some(kind.to_string());
    }
    match normalized.as_str() {
        "gps-failure" => Some("gps".into()),
        "battery-failure" => Some("battery".into()),
        "connectivity-loss" | "connectivity-failure" => Some("connectivity".into()),
        "robot-failure" => Some("gps".into()),
        "fleet-failure" | "swarm-failure" => Some("connectivity".into()),
        "provider-failure" => Some("provider".into()),
        "package-failure" => Some("package".into()),
        _ => None,
    }
}

/// Infer default what-if scenarios from program structure.
pub fn default_scenarios(program: &Program) -> Vec<String> {
    default_injections(program)
        .into_iter()
        .map(|label| label.replace("-failure", "_failure"))
        .collect()
}

/// Run what-if analysis for selected or inferred failure scenarios.
pub fn run_what_if_analysis(
    program: &Program,
    source_label: &str,
    options: &WhatIfOptions,
) -> WhatIfReport {
    let labels = if options.all || options.scenarios.is_empty() {
        default_scenarios(program)
    } else {
        options.scenarios.clone()
    };

    let mut scenarios = Vec::new();
    for label in labels {
        if let Some(kind) = normalize_scenario(&label) {
            scenarios.push(analyze_scenario(program, &label, &kind));
        } else {
            scenarios.push(WhatIfScenarioResult {
                scenario: label.clone(),
                impact: "unknown".into(),
                risk: "high".into(),
                recovery_plan: "none".into(),
                probability: 0.5,
                mission_completion_likely: false,
                recovery_passed: false,
                readiness_passed: false,
                details: vec![format!("unknown scenario: {label}")],
            });
        }
    }

    let high_risk = scenarios
        .iter()
        .filter(|s| matches!(s.risk.as_str(), "high" | "critical"))
        .count();
    let mission_completion_likely = scenarios
        .iter()
        .filter(|s| s.mission_completion_likely)
        .count();
    let total = scenarios.len();
    WhatIfReport {
        program: source_label.into(),
        scenarios,
        summary: WhatIfScenarioSummary {
            total,
            mission_completion_likely,
            high_risk,
        },
    }
}

/// Format a what-if report for CLI output.
pub fn format_what_if_report(report: &WhatIfReport, format: WhatIfFormat) -> String {
    match format {
        WhatIfFormat::Json => {
            serde_json::to_string_pretty(report).unwrap_or_else(|e| e.to_string())
        }
        WhatIfFormat::Text => format_what_if_text(report),
    }
}

fn analyze_scenario(program: &Program, label: &str, kind: &str) -> WhatIfScenarioResult {
    let recovery = simulate_failure_recovery(program, kind, None);
    let readiness = evaluate_readiness(
        program,
        &ReadinessOptions {
            simulate: true,
            include_runtime: true,
            inject_health_faults: true,
            ..ReadinessOptions::default()
        },
    );

    let recovery_passed = recovery.passed;
    let readiness_passed = readiness.mission_ready
        || recovery.readiness.recovery_ready
        || !readiness_has_critical(&readiness);
    let mission_completion_likely = recovery_passed
        && readiness_passed
        && !recovery
            .results
            .iter()
            .any(|result| result.status == RecoveryStatus::Unsafe);

    let impact = scenario_impact(kind).to_string();
    let risk = scenario_risk(kind, &recovery, &readiness, mission_completion_likely);
    let recovery_plan = format_recovery_plan(&recovery);
    let probability = estimate_probability(kind, &recovery, &readiness, mission_completion_likely);

    let mut details = Vec::new();
    details.push(format!(
        "recovery: {} (plans={}, success_rate={:.0}%)",
        if recovery_passed { "pass" } else { "fail" },
        recovery.plans.len(),
        recovery.assurance.success_rate * 100.0
    ));
    details.push(format!(
        "readiness: mission_ready={}, score {}/{}",
        readiness.mission_ready, readiness.score.total, readiness.score.maximum
    ));

    WhatIfScenarioResult {
        scenario: label.into(),
        impact,
        risk,
        recovery_plan,
        probability,
        mission_completion_likely,
        recovery_passed,
        readiness_passed,
        details,
    }
}

fn format_what_if_text(report: &WhatIfReport) -> String {
    let mut lines = vec![
        format!("What-if analysis: {}", report.program),
        format!(
            "Summary: {} scenarios, {} likely complete, {} high risk",
            report.summary.total,
            report.summary.mission_completion_likely,
            report.summary.high_risk
        ),
        String::new(),
    ];

    for scenario in &report.scenarios {
        lines.push(format!("Scenario: {}", scenario.scenario));
        lines.push(format!("  impact: {}", scenario.impact));
        lines.push(format!("  risk: {}", scenario.risk));
        lines.push(format!("  recovery_plan: {}", scenario.recovery_plan));
        lines.push(format!(
            "  probability: {:.2} (heuristic v1)",
            scenario.probability
        ));
        lines.push(format!(
            "  mission_completion_likely: {}",
            scenario.mission_completion_likely
        ));
        for detail in &scenario.details {
            lines.push(format!("  - {detail}"));
        }
        lines.push(String::new());
    }

    lines.join("\n").trim_end().to_string()
}

fn scenario_impact(kind: &str) -> &'static str {
    match kind {
        "gps" => "navigation_degraded",
        "camera" => "perception_degraded",
        "lidar" => "obstacle_sensing_degraded",
        "battery" => "mission_duration_at_risk",
        "connectivity" => "central_link_lost",
        "provider" => "provider_dispatch_degraded",
        "package" => "dependency_integrity_at_risk",
        _ => "mission_degraded",
    }
}

fn scenario_risk(
    kind: &str,
    recovery: &RecoveryReport,
    readiness: &spanda_readiness::ReadinessReport,
    mission_completion_likely: bool,
) -> String {
    if !recovery.passed {
        return "critical".into();
    }
    if !mission_completion_likely {
        return "high".into();
    }
    if readiness_has_critical(readiness) {
        return "medium".into();
    }
    if recovery.readiness.risk.eq_ignore_ascii_case("high") {
        return "medium".into();
    }
    match kind {
        "battery" | "connectivity" => "medium".into(),
        _ => "low".into(),
    }
}

fn format_recovery_plan(recovery: &RecoveryReport) -> String {
    if let Some(plan) = recovery.plans.first() {
        let action = plan
            .actions
            .first()
            .map(|a| a.description.clone())
            .unwrap_or_else(|| "evaluate".into());
        return format!("{} → {}", plan.name, action);
    }
    if let Some(entry) = recovery.knowledge.entries.first() {
        return format!("{} → {}", entry.failure_pattern, entry.recovery_pattern);
    }
    "no_recovery_plan".into()
}

fn estimate_probability(
    kind: &str,
    recovery: &RecoveryReport,
    readiness: &spanda_readiness::ReadinessReport,
    mission_completion_likely: bool,
) -> f64 {
    let base = match kind {
        "gps" => 0.15,
        "camera" => 0.12,
        "lidar" => 0.14,
        "battery" => 0.18,
        "connectivity" => 0.22,
        "provider" => 0.10,
        "package" => 0.08,
        _ => 0.15,
    };
    let recovery_penalty = (1.0 - recovery.assurance.success_rate) * 0.35;
    let readiness_penalty = if readiness.mission_ready { 0.0 } else { 0.12 };
    let completion_adjust = if mission_completion_likely {
        -0.05
    } else {
        0.15
    };
    (base + recovery_penalty + readiness_penalty + completion_adjust).clamp(0.05, 0.95)
}

fn readiness_has_critical(readiness: &spanda_readiness::ReadinessReport) -> bool {
    readiness.issues.iter().any(|issue| {
        matches!(
            issue.severity,
            ReadinessSeverity::Critical | ReadinessSeverity::High
        )
    })
}
