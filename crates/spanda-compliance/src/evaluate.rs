//! Verify-time compliance profile evaluation.

use crate::profiles::{builtin_profile, list_builtin_profiles, ComplianceProfile};
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::{Expr, Program, RobotDecl, SafetyBlock, SafetyRule, UnitKind};
use spanda_readiness::{evaluate_readiness, ReadinessOptions};

/// Severity for a compliance violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceSeverity {
    Error,
    Warning,
}

/// Failed compliance requirement with context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub profile: String,
    pub requirement: String,
    pub severity: ComplianceSeverity,
    pub message: String,
}

/// Compliance profile evaluation report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplianceEvaluationReport {
    pub profile: String,
    pub program: String,
    pub description: String,
    pub template_notice: String,
    pub passed: bool,
    pub violations: Vec<ComplianceViolation>,
}

/// List built-in compliance profile names.
pub fn list_compliance_profiles() -> Vec<String> {
    list_builtin_profiles()
        .into_iter()
        .map(str::to_string)
        .collect()
}

/// Evaluate a built-in compliance profile against a program.
pub fn evaluate_compliance_profile(
    program: &Program,
    profile_name: &str,
    source_label: &str,
) -> Result<ComplianceEvaluationReport, String> {
    // Check profile template requirements against program declarations.
    //
    // Parameters:
    // - `program` — parsed program
    // - `profile_name` — built-in profile name
    // - `source_label` — file label
    //
    // Returns:
    // Compliance evaluation report or unknown-profile error.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = evaluate_compliance_profile(&program, "warehouse", "rover.sd")?;

    let profile = builtin_profile(profile_name)
        .ok_or_else(|| format!("Unknown compliance profile '{profile_name}'"))?;
    let mut violations = Vec::new();

    if profile.requires_kill_switch {
        check_kill_switch(&profile, program, &mut violations);
    }
    if profile.min_readiness_score > 0 {
        check_readiness(&profile, program, &mut violations);
    }
    if !profile.required_capabilities.is_empty() {
        check_capabilities(&profile, program, &mut violations);
    }
    if profile.min_health_checks > 0 {
        check_health_checks(&profile, program, &mut violations);
    }
    if profile.requires_assurance_case {
        check_assurance_case(&profile, program, &mut violations);
    }
    if let Some(limit) = profile.max_speed_mps {
        check_max_speed(&profile, program, limit, &mut violations);
    }
    if let Some(range) = profile.operation_hours.as_deref() {
        check_operation_hours(&profile, range, &mut violations);
    }
    if profile.requires_secure_comm {
        check_secure_comm(&profile, program, &mut violations);
    }
    if profile.requires_tamper_policy {
        check_tamper_policy(&profile, program, &mut violations);
    }

    let passed = if profile.warn_only {
        true
    } else {
        violations
            .iter()
            .all(|violation| violation.severity != ComplianceSeverity::Error)
    };

    Ok(ComplianceEvaluationReport {
        profile: profile.name.clone(),
        program: source_label.into(),
        description: profile.description.clone(),
        template_notice: profile.template_notice.to_string(),
        passed,
        violations,
    })
}

/// Format a compliance evaluation report for CLI output.
pub fn format_compliance_report(report: &ComplianceEvaluationReport, json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(report).unwrap_or_else(|e| e.to_string());
    }

    let mut lines = vec![
        format!(
            "Compliance profile: {} on {}",
            report.profile, report.program
        ),
        report.description.clone(),
        report.template_notice.clone(),
        if report.passed {
            "Result: PASS".into()
        } else {
            "Result: FAIL".into()
        },
    ];
    if report.violations.is_empty() {
        lines.push("No violations.".into());
    } else {
        for violation in &report.violations {
            lines.push(format!(
                "  [{:?}] {} — {}",
                violation.severity, violation.requirement, violation.message
            ));
        }
    }
    lines.join("\n")
}

fn check_kill_switch(
    profile: &ComplianceProfile,
    program: &Program,
    violations: &mut Vec<ComplianceViolation>,
) {
    let Program::Program {
        kill_switches,
        robots,
        ..
    } = program;
    let robot_switches = robots.iter().any(|robot| {
        let RobotDecl::RobotDecl { kill_switches, .. } = robot;
        !kill_switches.is_empty()
    });
    if kill_switches.is_empty() && !robot_switches {
        push_violation(
            profile,
            "requires_kill_switch",
            ComplianceSeverity::Error,
            "program must declare at least one kill_switch",
            violations,
        );
    }
}

fn check_readiness(
    profile: &ComplianceProfile,
    program: &Program,
    violations: &mut Vec<ComplianceViolation>,
) {
    let readiness = evaluate_readiness(program, &ReadinessOptions::default());
    if readiness.score.total < profile.min_readiness_score {
        push_violation(
            profile,
            "min_readiness_score",
            severity_for(profile, ComplianceSeverity::Error),
            format!(
                "readiness score {}/{} below profile minimum {}",
                readiness.score.total,
                readiness.score.maximum,
                profile.min_readiness_score
            ),
            violations,
        );
    }
}

fn check_capabilities(
    profile: &ComplianceProfile,
    program: &Program,
    violations: &mut Vec<ComplianceViolation>,
) {
    let Program::Program { robots, .. } = program;
    if robots.is_empty() {
        push_violation(
            profile,
            "required_capabilities",
            ComplianceSeverity::Error,
            "profile requires robots with declared capabilities",
            violations,
        );
        return;
    }
    for robot in robots {
        let RobotDecl::RobotDecl {
            name,
            exposes_capabilities,
            ..
        } = robot;
        for capability in &profile.required_capabilities {
            if !exposes_capabilities.iter().any(|value| value == capability) {
                push_violation(
                    profile,
                    "required_capabilities",
                    ComplianceSeverity::Error,
                    format!("robot {name} missing required capability `{capability}`"),
                    violations,
                );
            }
        }
    }
}

fn check_health_checks(
    profile: &ComplianceProfile,
    program: &Program,
    violations: &mut Vec<ComplianceViolation>,
) {
    let Program::Program { health_checks, .. } = program;
    if health_checks.len() < profile.min_health_checks {
        push_violation(
            profile,
            "min_health_checks",
            ComplianceSeverity::Error,
            format!(
                "profile requires at least {} health_check declarations (found {})",
                profile.min_health_checks,
                health_checks.len()
            ),
            violations,
        );
    }
}

fn check_assurance_case(
    profile: &ComplianceProfile,
    program: &Program,
    violations: &mut Vec<ComplianceViolation>,
) {
    let Program::Program { assurance_cases, .. } = program;
    if assurance_cases.is_empty() {
        push_violation(
            profile,
            "requires_assurance_case",
            ComplianceSeverity::Error,
            "profile requires at least one assurance_case declaration",
            violations,
        );
    }
}

fn check_max_speed(
    profile: &ComplianceProfile,
    program: &Program,
    limit_mps: f64,
    violations: &mut Vec<ComplianceViolation>,
) {
    let Program::Program { robots, .. } = program;
    for robot in robots {
        let RobotDecl::RobotDecl { name, safety, .. } = robot;
        let Some(safety) = safety else {
            push_violation(
                profile,
                "max_speed",
                ComplianceSeverity::Error,
                format!("robot {name} has no safety block with max_speed"),
                violations,
            );
            continue;
        };
        let SafetyBlock::SafetyBlock { rules, .. } = safety;
        let mut robot_limit = None;
        for rule in rules {
            if let SafetyRule::MaxSpeedRule { value, unit, .. } = rule {
                if let Some(mps) = expr_to_mps(value, *unit) {
                    robot_limit = Some(robot_limit.map_or(mps, |current: f64| current.max(mps)));
                }
            }
        }
        let Some(robot_limit) = robot_limit else {
            push_violation(
                profile,
                "max_speed",
                ComplianceSeverity::Error,
                format!("robot {name} missing max_speed safety rule"),
                violations,
            );
            continue;
        };
        if robot_limit > limit_mps {
            push_violation(
                profile,
                "max_speed",
                ComplianceSeverity::Error,
                format!(
                    "robot {name} max_speed {robot_limit:.2} m/s exceeds profile limit {limit_mps:.2} m/s"
                ),
                violations,
            );
        }
    }
}

fn check_operation_hours(
    profile: &ComplianceProfile,
    range: &str,
    violations: &mut Vec<ComplianceViolation>,
) {
    let valid = range.contains(':') && range.contains('-');
    if !valid {
        push_violation(
            profile,
            "operation_hours",
            ComplianceSeverity::Warning,
            format!("operation_hours `{range}` is not a valid HH:MM-HH:MM range"),
            violations,
        );
    }
}

fn check_secure_comm(
    profile: &ComplianceProfile,
    program: &Program,
    violations: &mut Vec<ComplianceViolation>,
) {
    let Program::Program { robots, .. } = program;
    let robot_secure = robots.iter().any(|robot| {
        let RobotDecl::RobotDecl {
            secure_comm,
            signed_records,
            trust_boundaries,
            ..
        } = robot;
        secure_comm.is_some() || !signed_records.is_empty() || !trust_boundaries.is_empty()
    });
    if !robot_secure {
        push_violation(
            profile,
            "requires_secure_comm",
            ComplianceSeverity::Error,
            "profile requires secure_comm, signed records, or trust boundaries",
            violations,
        );
    }
}

fn check_tamper_policy(
    profile: &ComplianceProfile,
    program: &Program,
    violations: &mut Vec<ComplianceViolation>,
) {
    let (has_policy, branch_count) = spanda_tamper::tamper_policy_coverage(program);
    if !has_policy || branch_count == 0 {
        push_violation(
            profile,
            "requires_tamper_policy",
            ComplianceSeverity::Error,
            "profile requires at least one tamper_policy response branch",
            violations,
        );
    }
}

fn severity_for(profile: &ComplianceProfile, default: ComplianceSeverity) -> ComplianceSeverity {
    if profile.warn_only {
        ComplianceSeverity::Warning
    } else {
        default
    }
}

fn push_violation(
    profile: &ComplianceProfile,
    requirement: &str,
    severity: ComplianceSeverity,
    message: impl Into<String>,
    violations: &mut Vec<ComplianceViolation>,
) {
    violations.push(ComplianceViolation {
        profile: profile.name.clone(),
        requirement: requirement.into(),
        severity: severity_for(profile, severity),
        message: message.into(),
    });
}

fn expr_to_mps(value: &Expr, unit: UnitKind) -> Option<f64> {
    let number = match value {
        Expr::UnitLiteralExpr { value, unit, .. } => {
            return match unit {
                UnitKind::MPerS => Some(*value),
                UnitKind::KmPerH => Some(*value / 3.6),
                UnitKind::Mph => Some(*value * 0.44704),
                _ => None,
            };
        }
        Expr::LiteralExpr {
            value: spanda_ast::nodes::LiteralValue::Number(n),
            ..
        } => *n,
        _ => return None,
    };
    match unit {
        UnitKind::MPerS => Some(number),
        UnitKind::KmPerH => Some(number / 3.6),
        UnitKind::Mph => Some(number * 0.44704),
        _ => None,
    }
}
