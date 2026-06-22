//! Health check types, evaluation, and traceability.

use serde::{Deserialize, Serialize};
use spanda_ast::foundations::{HealthCheckDecl, HealthPolicyDecl};
use spanda_ast::nodes::Program;

/// Component health status values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Warning,
    Critical,
    Failed,
    Unknown,
    Offline,
    Unsafe,
}

/// Result of evaluating a single health check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub name: String,
    pub target: String,
    pub target_kind: String,
    pub metric: String,
    pub operator: String,
    pub threshold: String,
    pub status: HealthStatus,
    pub message: Option<String>,
}

/// Aggregated health report for a program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthReport {
    pub checks: Vec<HealthCheckResult>,
    pub overall: HealthStatus,
    pub policies: Vec<String>,
}

/// Health traceability matrix row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthTraceRow {
    pub component: String,
    pub health_check: String,
    pub metric: String,
    pub threshold: String,
    pub status: String,
    pub action: Option<String>,
}

/// Evaluate static health checks from program declarations.
pub fn evaluate_health_checks(program: &Program) -> HealthReport {
    let Program::Program {
        health_checks,
        health_policies,
        ..
    } = program;

    let mut checks = Vec::new();
    for hc in health_checks {
        let HealthCheckDecl::HealthCheckDecl {
            name,
            target,
            target_kind,
            conditions,
            ..
        } = hc;
        for cond in conditions {
            checks.push(HealthCheckResult {
                name: name.clone(),
                target: target.clone(),
                target_kind: target_kind.clone(),
                metric: cond.metric.clone(),
                operator: cond.operator.clone(),
                threshold: cond.threshold.clone(),
                status: HealthStatus::Unknown,
                message: Some(format!("Static check: {} {} {}", cond.metric, cond.operator, cond.threshold)),
            });
        }
    }

    // Evaluate robot-level health checks embedded in robots.
    let Program::Program { robots, .. } = program;
    for robot in robots {
        let spanda_ast::nodes::RobotDecl::RobotDecl {
            name,
            health_checks: robot_checks,
            ..
        } = robot;
        for hc in robot_checks {
            let HealthCheckDecl::HealthCheckDecl {
                name: hc_name,
                conditions,
                ..
            } = hc;
            for cond in conditions {
                checks.push(HealthCheckResult {
                    name: hc_name.clone(),
                    target: name.clone(),
                    target_kind: "robot".into(),
                    metric: cond.metric.clone(),
                    operator: cond.operator.clone(),
                    threshold: cond.threshold.clone(),
                    status: HealthStatus::Unknown,
                    message: None,
                });
            }
        }
    }

    let policies: Vec<String> = health_policies
        .iter()
        .map(|p| {
            let HealthPolicyDecl::HealthPolicyDecl { name, .. } = p;
            name.clone()
        })
        .collect();

    let overall = if checks.is_empty() {
        HealthStatus::Unknown
    } else {
        HealthStatus::Healthy
    };

    HealthReport {
        checks,
        overall,
        policies,
    }
}

/// Evaluate health checks against runtime fault and event signals from hardware monitoring.
pub fn evaluate_runtime_health(
    faults: &[String],
    active_events: &[String],
    program: &Program,
) -> HealthReport {
    let mut report = evaluate_health_checks(program);
    let fault_lower: Vec<String> = faults.iter().map(|f| f.to_ascii_lowercase()).collect();
    let event_lower: Vec<String> = active_events.iter().map(|f| f.to_ascii_lowercase()).collect();

    for check in &mut report.checks {
        check.status = runtime_status_for_metric(&check.metric, &fault_lower, &event_lower);
        check.message = Some(format!("Runtime status: {:?}", check.status));
    }

    report.overall = if report.checks.iter().any(|c| {
        matches!(
            c.status,
            HealthStatus::Critical | HealthStatus::Failed | HealthStatus::Unsafe
        )
    }) {
        HealthStatus::Critical
    } else if report.checks.iter().any(|c| {
        matches!(
            c.status,
            HealthStatus::Degraded | HealthStatus::Warning | HealthStatus::Offline
        )
    }) || !faults.is_empty()
    {
        HealthStatus::Degraded
    } else if report.checks.is_empty() {
        HealthStatus::Unknown
    } else {
        HealthStatus::Healthy
    };

    report
}

fn runtime_status_for_metric(
    metric: &str,
    faults: &[String],
    events: &[String],
) -> HealthStatus {
    let metric_lower = metric.to_ascii_lowercase();
    let signals: Vec<&str> = faults.iter().chain(events.iter()).map(String::as_str).collect();

    if metric_lower.contains("emergency_stop") {
        return if signals.iter().any(|s| s.contains("emergency") || s.contains("kill")) {
            HealthStatus::Unsafe
        } else {
            HealthStatus::Healthy
        };
    }

    if metric_lower.contains("gps") {
        if signals.iter().any(|s| s.contains("gps") && s.contains("critical")) {
            return HealthStatus::Critical;
        }
        if signals.iter().any(|s| s.contains("gps")) {
            return HealthStatus::Degraded;
        }
        return HealthStatus::Healthy;
    }

    if metric_lower.contains("camera") {
        if signals.iter().any(|s| s.contains("camera")) {
            return HealthStatus::Offline;
        }
        return HealthStatus::Healthy;
    }

    if metric_lower.contains("battery") {
        if signals.iter().any(|s| s.contains("critical")) {
            return HealthStatus::Critical;
        }
        return HealthStatus::Healthy;
    }

    if signals.iter().any(|s| s.contains("critical") || s.contains("unsafe")) {
        HealthStatus::Critical
    } else if signals.iter().any(|s| s.contains("degraded") || s.contains("offline")) {
        HealthStatus::Degraded
    } else if signals.is_empty() {
        HealthStatus::Healthy
    } else {
        HealthStatus::Warning
    }
}

/// Generate health traceability matrix.
pub fn health_traceability(program: &Program) -> Vec<HealthTraceRow> {
    let report = evaluate_health_checks(program);
    let Program::Program { health_policies, .. } = program;

    let policy_actions: std::collections::HashMap<String, String> = health_policies
        .iter()
        .flat_map(|p| {
            let HealthPolicyDecl::HealthPolicyDecl { name, reactions, .. } = p;
            reactions.iter().map(move |(status, action)| {
                (format!("{name}:{status}"), action.clone())
            })
        })
        .collect();

    report
        .checks
        .iter()
        .map(|c| {
            let action_key = format!("{}:{:?}", c.name, c.status);
            HealthTraceRow {
                component: c.target.clone(),
                health_check: c.name.clone(),
                metric: c.metric.clone(),
                threshold: c.threshold.clone(),
                status: format!("{:?}", c.status),
                action: policy_actions.get(&action_key).cloned(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_lexer::tokenize;
    use spanda_parser::parse;

    fn parse_source(source: &str) -> spanda_ast::nodes::Program {
        parse(tokenize(source).expect("tokenize")).expect("parse")
    }

    #[test]
    fn health_check_parsing_and_report() {
        let source = r#"
health_check RoverHealth for robot Rover {
    check battery.level > 20%;
    check gps.status == Healthy;
}

health_policy SafetyPolicy {
    on Critical { enter degraded_mode; }
    on Failed { emergency_stop; }
}
"#;
        let program = parse_source(source);
        let report = evaluate_health_checks(&program);
        assert!(!report.checks.is_empty());
        assert!(!report.policies.is_empty());
    }

    #[test]
    fn runtime_health_marks_gps_fault_degraded() {
        let source = r#"
health_check RoverHealth for robot Rover {
    check gps.status == Healthy;
}
"#;
        let program = parse_source(source);
        let report = evaluate_runtime_health(
            &["GPSDegraded".into()],
            &[],
            &program,
        );
        assert!(report.checks.iter().any(|c| c.status == HealthStatus::Degraded));
    }
}
