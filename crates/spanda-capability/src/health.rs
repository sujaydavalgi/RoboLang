//! Health check types, evaluation, and traceability.

pub use spanda_runtime::health_primitives::{
    apply_fleet_health_checks, evaluate_health_checks, evaluate_runtime_health,
};
pub use spanda_runtime::health_types::{
    HealthCheckResult, HealthReport, HealthStatus, HealthTraceRow,
};

use spanda_ast::foundations::HealthPolicyDecl;
use spanda_ast::nodes::Program;

/// Generate health traceability matrix.
pub fn health_traceability(program: &Program) -> Vec<HealthTraceRow> {
    // Build traceability rows linking health checks to policy reactions.
    //
    // Parameters:
    // - `program` — parsed program AST
    //
    // Returns:
    // Traceability matrix rows for health checks and policy actions.
    //
    // Options:
    // None.
    //
    // Example:
    // let rows = health_traceability(&program);

    let report = evaluate_health_checks(program);
    let Program::Program {
        health_policies, ..
    } = program;

    let policy_actions: std::collections::HashMap<String, String> = health_policies
        .iter()
        .flat_map(|p| {
            let HealthPolicyDecl::HealthPolicyDecl {
                name, reactions, ..
            } = p;
            reactions.iter().map(move |reaction| {
                let action = reaction
                    .body
                    .iter()
                    .map(|s| format!("{s:?}"))
                    .collect::<Vec<_>>()
                    .join("; ");
                (format!("{name}:{}", reaction.status), action)
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
        let report = evaluate_runtime_health(&["GPSDegraded".into()], &[], &program);
        assert!(report
            .checks
            .iter()
            .any(|c| c.status == HealthStatus::Degraded));
    }
}
