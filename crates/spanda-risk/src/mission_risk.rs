//! Mission risk assessment composition from readiness, coverage, trust, and what-if engines.

use serde::{Deserialize, Serialize};
use spanda_assurance::evaluate_recovery_coverage;
use spanda_ast::nodes::Program;
use spanda_contract::verify_contract;
use spanda_hardware::{verify_program_compatibility, VerifyOptions};
use spanda_readiness::{evaluate_readiness, evaluate_safety_coverage, ReadinessOptions};
use spanda_trust::{evaluate_composite_trust, CompositeTrustOptions};
use spanda_whatif::{run_what_if_analysis, WhatIfOptions};

/// Output format for mission risk reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MissionRiskFormat {
    #[default]
    Text,
    Json,
}

/// Individual risk contributor with weight and local score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionRiskFactor {
    pub name: String,
    pub weight: u32,
    pub score: u32,
    pub detail: String,
}

/// Composite mission risk score (0–100, higher = more risk).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionRiskScore {
    pub total: u32,
    pub tier: String,
}

/// Full mission deployment risk assessment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionRiskAssessment {
    pub program: String,
    pub score: MissionRiskScore,
    pub factors: Vec<MissionRiskFactor>,
    pub mitigations: Vec<String>,
}

/// Evaluate deployment risk for a Spanda program.
pub fn evaluate_mission_risk(
    program: &Program,
    source: &str,
    source_label: &str,
) -> MissionRiskAssessment {
    let readiness = evaluate_readiness(
        program,
        &ReadinessOptions {
            simulate: true,
            include_runtime: true,
            ..ReadinessOptions::default()
        },
    );
    let safety = evaluate_safety_coverage(program, source_label);
    let recovery = evaluate_recovery_coverage(program, source_label);
    let contract = verify_contract(program, source_label);
    let trust = evaluate_composite_trust(
        program,
        source,
        source_label,
        &CompositeTrustOptions::default(),
    );
    let hardware = verify_program_compatibility(program, &VerifyOptions::default());
    let what_if = run_what_if_analysis(program, source_label, &WhatIfOptions::default());

    let readiness_pct = pct(readiness.score.total, readiness.score.maximum);
    let readiness_risk = if readiness.mission_ready {
        (100 - readiness_pct) / 2
    } else {
        100 - readiness_pct
    };
    let safety_risk = 100 - safety.overall_coverage_pct.min(100);
    let recovery_risk = 100 - recovery.coverage_pct.min(100);
    let trust_risk = 100 - trust.score.min(100);
    let contract_risk = contract_risk_score(&contract);
    let what_if_risk = what_if_risk_score(&what_if);
    let hardware_risk = if hardware.compatible {
        0
    } else {
        hardware.errors().count().saturating_mul(12).min(100) as u32
    };
    let fleet_risk = if program_has_fleet(program) && recovery.uncovered > 0 {
        40
    } else if program_has_fleet(program) {
        15
    } else {
        0
    };

    let mut factors = vec![
        factor(
            "readiness",
            25,
            readiness_risk,
            &format!(
                "mission_ready={}, score {}/{}",
                readiness.mission_ready, readiness.score.total, readiness.score.maximum
            ),
        ),
        factor(
            "safety_coverage",
            20,
            safety_risk,
            &format!(
                "coverage {}%, {} scenarios",
                safety.overall_coverage_pct,
                safety.scenarios.len()
            ),
        ),
        factor(
            "recovery_coverage",
            20,
            recovery_risk,
            &format!(
                "coverage {}%, {} uncovered",
                recovery.coverage_pct, recovery.uncovered
            ),
        ),
        factor(
            "trust",
            15,
            trust_risk,
            &format!("composite trust {}/100 tier={}", trust.score, trust.tier),
        ),
        factor(
            "mission_contract",
            10,
            contract_risk,
            &format!(
                "passed={}, {} checks",
                contract.passed,
                contract.checks.len()
            ),
        ),
        factor(
            "what_if_scenarios",
            10,
            what_if_risk,
            &format!(
                "{} high-risk of {} scenarios",
                what_if.summary.high_risk, what_if.summary.total
            ),
        ),
    ];

    if hardware_risk > 0 {
        factors.push(factor(
            "hardware_margin",
            0,
            hardware_risk,
            &format!("{} compatibility issue(s)", hardware.errors().count()),
        ));
    }
    if fleet_risk > 0 {
        factors.push(factor(
            "fleet_dependency",
            0,
            fleet_risk,
            "fleet declared; recovery gaps increase coordination risk",
        ));
    }

    let weighted = weighted_risk(&factors);
    let tier = risk_tier(weighted);
    let mut mitigations = Vec::new();
    mitigations.extend(safety.recommendations);
    mitigations.extend(
        recovery
            .missing_paths
            .iter()
            .map(|gap| gap.recommendation.clone()),
    );
    mitigations.extend(contract.issues);
    if !readiness.mission_ready {
        mitigations.push("Improve readiness score before deployment".into());
    }
    if trust_risk > 40 {
        mitigations.push(format!(
            "Review trust posture (tier={}) before field rollout",
            trust.tier
        ));
    }
    mitigations.sort();
    mitigations.dedup();

    MissionRiskAssessment {
        program: source_label.into(),
        score: MissionRiskScore {
            total: weighted,
            tier,
        },
        factors,
        mitigations,
    }
}

/// Format a mission risk assessment for CLI output.
pub fn format_mission_risk(report: &MissionRiskAssessment, format: MissionRiskFormat) -> String {
    match format {
        MissionRiskFormat::Json => {
            serde_json::to_string_pretty(report).unwrap_or_else(|e| e.to_string())
        }
        MissionRiskFormat::Text => format_mission_risk_text(report),
    }
}

fn factor(name: &str, weight: u32, score: u32, detail: &str) -> MissionRiskFactor {
    MissionRiskFactor {
        name: name.into(),
        weight,
        score: score.min(100),
        detail: detail.into(),
    }
}

fn weighted_risk(factors: &[MissionRiskFactor]) -> u32 {
    let weighted_sum: u32 = factors
        .iter()
        .filter(|f| f.weight > 0)
        .map(|f| f.score.saturating_mul(f.weight))
        .sum();
    let weight_total: u32 = factors
        .iter()
        .filter(|f| f.weight > 0)
        .map(|f| f.weight)
        .sum();
    if weight_total == 0 {
        return 0;
    }
    (weighted_sum / weight_total).min(100)
}

fn risk_tier(total: u32) -> String {
    match total {
        0..=25 => "low".into(),
        26..=50 => "medium".into(),
        51..=75 => "high".into(),
        _ => "critical".into(),
    }
}

fn pct(part: u32, whole: u32) -> u32 {
    if whole == 0 {
        return 0;
    }
    ((part as f64 / whole as f64) * 100.0).round() as u32
}

fn contract_risk_score(contract: &spanda_contract::ContractVerificationReport) -> u32 {
    if !contract.passed {
        return 85;
    }
    let failed = contract.checks.iter().filter(|c| !c.passed).count() as u32;
    (failed * 20).min(100)
}

fn what_if_risk_score(what_if: &spanda_whatif::WhatIfReport) -> u32 {
    if what_if.summary.total == 0 {
        return 0;
    }
    ((what_if.summary.high_risk as f64 / what_if.summary.total as f64) * 100.0).round() as u32
}

fn program_has_fleet(program: &Program) -> bool {
    let Program::Program { fleets, .. } = program;
    !fleets.is_empty()
}

fn format_mission_risk_text(report: &MissionRiskAssessment) -> String {
    let mut lines = vec![
        format!("Mission risk: {}", report.program),
        format!(
            "Risk score: {}/100 ({})",
            report.score.total, report.score.tier
        ),
        String::new(),
        "Contributors:".into(),
    ];
    for factor in &report.factors {
        lines.push(format!(
            "  {} — {} (weight {}, score {})",
            factor.name, factor.detail, factor.weight, factor.score
        ));
    }
    if !report.mitigations.is_empty() {
        lines.push(String::new());
        lines.push("Recommended mitigations:".into());
        for item in &report.mitigations {
            lines.push(format!("  - {item}"));
        }
    }
    lines.join("\n")
}
