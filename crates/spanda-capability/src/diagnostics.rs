//! Span-aware verification diagnostics for IDE and CLI integration.

use crate::{
    capability_traceability, check_minimum_capabilities, infer_robot_capabilities,
    lookup_capability,
};
use serde::{Deserialize, Serialize};
use spanda_ast::foundations::{
    HealthCheckDecl, KillSwitchDecl, RequiresCapabilityDecl, RequiresCapabilitySeverity,
};
use spanda_ast::nodes::{Program, RobotDecl, Span, TopicDecl};

/// Single verification diagnostic with source location.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationDiagnostic {
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub severity: String,
    pub category: String,
}

/// Collect capability, traceability, minimum-hardware, health, and kill-switch diagnostics.
pub fn collect_verification_diagnostics(program: &Program) -> Vec<VerificationDiagnostic> {
    let mut diags = Vec::new();
    let Program::Program {
        robots,
        requires_capabilities,
        kill_switches,
        health_checks,
        health_policies,
        ..
    } = program;

    let trace = capability_traceability(program);
    for err in &trace.errors {
        if let Some(d) = map_traceability_error(err, robots, requires_capabilities) {
            diags.push(d);
        }
    }
    for warn in &trace.warnings {
        if let Some(d) = map_traceability_warning(warn, robots) {
            diags.push(d);
        }
    }

    for row in &trace.capability_rows {
        if row.status == "FAIL" {
            if let Some(req) = requires_capabilities
                .iter()
                .find(|r| r.capability == row.capability)
            {
                diags.push(diag(
                    format!(
                        "Capability '{}' not satisfied by robot '{}'",
                        row.capability, row.provided_by
                    ),
                    req.span.start.line,
                    req.span.start.column,
                    "warning",
                    "capability",
                ));
            }
        }
    }

    let minimum = check_minimum_capabilities(program);
    for err in &minimum.errors {
        if let Some(d) = map_minimum_error(err, requires_capabilities, robots) {
            diags.push(d);
        }
    }
    for warn in &minimum.warnings {
        diags.push(diag(
            warn.clone(),
            1,
            1,
            "warning",
            "minimum-hardware",
        ));
    }

    for req in requires_capabilities {
        if lookup_capability(&req.capability).is_none() {
            diags.push(diag(
                format!("Unknown capability '{}'", req.capability),
                req.span.start.line,
                req.span.start.column,
                severity_for(req.severity),
                "capability",
            ));
        }
    }

    for ks in kill_switches {
        diags.extend(kill_switch_diagnostics(ks, program));
    }
    for robot in robots {
        let RobotDecl::RobotDecl {
            name,
            kill_switches: robot_kill_switches,
            actuators,
            ..
        } = robot;
        for ks in robot_kill_switches {
            diags.extend(kill_switch_diagnostics(ks, program));
        }
        if robot_kill_switches.is_empty() && kill_switches.is_empty() {
            let has_drive = actuators.iter().any(|a| {
                let spanda_ast::nodes::ActuatorDecl::ActuatorDecl { actuator_type, .. } = a;
                actuator_type.contains("Drive")
            });
            if has_drive {
                let span = robot_span(robot);
                diags.push(diag(
                    format!("Robot '{name}' has actuators but no kill_switch handler"),
                    span.start.line,
                    span.start.column,
                    "info",
                    "kill-switch",
                ));
            }
        }
    }

    if !health_checks.is_empty() && health_policies.is_empty() {
        for hc in health_checks {
            let HealthCheckDecl::HealthCheckDecl { span, name, .. } = hc;
            diags.push(diag(
                format!("Health check '{name}' has no matching health_policy"),
                span.start.line,
                span.start.column,
                "info",
                "health",
            ));
        }
    }

    let robot_reports = infer_robot_capabilities(program);
    for report in robot_reports {
        for row in report.rows {
            if row.status == "PARTIAL" {
                if let Some(robot) = robots.iter().find(|r| robot_name(r) == report.robot) {
                    let span = robot_span(robot);
                    diags.push(diag(
                        format!(
                            "Capability '{}' partially satisfied on robot '{}'",
                            row.capability, report.robot
                        ),
                        span.start.line,
                        span.start.column,
                        "warning",
                        "capability",
                    ));
                }
            }
        }
    }

    diags
}

fn diag(message: String, line: u32, column: u32, severity: &str, category: &str) -> VerificationDiagnostic {
    VerificationDiagnostic {
        message,
        line,
        column,
        severity: severity.into(),
        category: category.into(),
    }
}

fn severity_for(severity: RequiresCapabilitySeverity) -> &'static str {
    match severity {
        RequiresCapabilitySeverity::Error => "error",
        RequiresCapabilitySeverity::Warning => "warning",
        RequiresCapabilitySeverity::Info => "info",
    }
}

fn map_traceability_error(
    err: &str,
    robots: &[RobotDecl],
    requires: &[RequiresCapabilityDecl],
) -> Option<VerificationDiagnostic> {
    if let Some(rest) = err.strip_prefix("Robot '") {
        if let Some((name, tail)) = rest.split_once("' uses undeclared hardware '") {
            if let Some(robot) = robots.iter().find(|r| robot_name(r) == name) {
                let span = robot_span(robot);
                return Some(diag(
                    err.to_string(),
                    span.start.line,
                    span.start.column,
                    "error",
                    "traceability",
                ));
            }
            let _ = tail;
        }
    }
    if let Some(cap) = err.strip_prefix("Unknown capability '").and_then(|s| s.strip_suffix('\'')) {
        if let Some(req) = requires.iter().find(|r| r.capability == cap) {
            return Some(diag(
                err.to_string(),
                req.span.start.line,
                req.span.start.column,
                "error",
                "capability",
            ));
        }
    }
    None
}

fn map_traceability_warning(warn: &str, robots: &[RobotDecl]) -> Option<VerificationDiagnostic> {
    if let Some(rest) = warn.strip_prefix("Actuator '") {
        if let Some((actuator, tail)) = rest.split_once("' on robot '") {
            if let Some(rname) = tail.strip_suffix("' has no safety gate") {
                if let Some(robot) = robots.iter().find(|r| robot_name(r) == rname) {
                    let span = robot_span(robot);
                    return Some(diag(
                        warn.to_string(),
                        span.start.line,
                        span.start.column,
                        "warning",
                        "traceability",
                    ));
                }
            }
            let _ = actuator;
        }
    }
    None
}

fn map_minimum_error(
    err: &str,
    requires: &[RequiresCapabilityDecl],
    robots: &[RobotDecl],
) -> Option<VerificationDiagnostic> {
    for req in requires {
        if err.contains(&req.capability) {
            return Some(diag(
                err.to_string(),
                req.span.start.line,
                req.span.start.column,
                severity_for(req.severity),
                "minimum-hardware",
            ));
        }
    }
    for robot in robots {
        if err.contains(robot_name(robot)) {
            let span = robot_span(robot);
            return Some(diag(
                err.to_string(),
                span.start.line,
                span.start.column,
                "error",
                "minimum-hardware",
            ));
        }
    }
    None
}

fn kill_switch_diagnostics(ks: &KillSwitchDecl, program: &Program) -> Vec<VerificationDiagnostic> {
    let KillSwitchDecl::KillSwitchDecl {
        name,
        remote_signed,
        span,
        ..
    } = ks;
    let mut diags = Vec::new();
    if *remote_signed && !program_has_signed_comm(program) {
        diags.push(diag(
            format!(
                "Kill switch '{name}' requires remote_signed but no signed secure comm or topic policy is declared"
            ),
            span.start.line,
            span.start.column,
            "warning",
            "kill-switch",
        ));
    }
    diags
}

fn program_has_signed_comm(program: &Program) -> bool {
    let Program::Program { robots, .. } = program;
    robots.iter().any(|robot| {
        let RobotDecl::RobotDecl {
            secure_comm,
            topics,
            trust,
            ..
        } = robot;
        secure_comm.is_some()
            || trust.is_some()
            || topics.iter().any(|topic| {
                let TopicDecl::TopicDecl { secure, .. } = topic;
                secure
                    .as_ref()
                    .is_some_and(|s| s.signed || s.signed_required)
            })
    })
}

fn robot_name(robot: &RobotDecl) -> &str {
    let RobotDecl::RobotDecl { name, .. } = robot;
    name
}

fn robot_span(robot: &RobotDecl) -> Span {
    let RobotDecl::RobotDecl { span, .. } = robot;
    *span
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_lexer::tokenize;
    use spanda_parser::parse;

    fn parse_source(source: &str) -> Program {
        parse(tokenize(source).expect("tokenize")).expect("parse")
    }

    #[test]
    fn remote_kill_switch_without_signed_policy_warns() {
        let source = r#"
kill_switch EmergencyStop {
    priority: critical;
    remote_signed;
    action { emergency_stop; }
}
"#;
        let program = parse_source(source);
        let diags = collect_verification_diagnostics(&program);
        assert!(diags.iter().any(|d| d.category == "kill-switch" && d.severity == "warning"));
    }

    #[test]
    fn undeclared_hardware_produces_spanned_error() {
        let source = r#"
robot Rover {
    uses hardware MissingBoard;
    actuator wheels: DifferentialDrive;
}
"#;
        let program = parse_source(source);
        let diags = collect_verification_diagnostics(&program);
        assert!(diags.iter().any(|d| d.category == "traceability" && d.line > 1));
    }
}
