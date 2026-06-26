//! Merge operational policy evaluation into readiness reports.

use crate::evaluate::{evaluate_policy_with_options, PolicyEvaluationReport, PolicySeverity};
use spanda_ast::nodes::Program;
use spanda_readiness::{
    ReadinessFactorScore, ReadinessIssue, ReadinessOptions, ReadinessReport, ReadinessSeverity,
    ReadinessStatus,
};

/// Weight applied to the operational policy readiness factor.
const POLICY_FACTOR_WEIGHT: u32 = 8;

/// Evaluate an operational policy and merge results into a readiness report.
pub fn evaluate_readiness_with_policy(
    program: &Program,
    source_label: &str,
    options: &ReadinessOptions,
    policy_name: &str,
    mut report: ReadinessReport,
) -> Result<(ReadinessReport, PolicyEvaluationReport), String> {
    // Run policy checks with the same readiness options, then fold outcomes into the report.
    //
    // Parameters:
    // - `program` — parsed `.sd` program
    // - `source_label` — file label for reports
    // - `options` — readiness evaluation options shared with policy rules
    // - `policy_name` — operational policy to enforce
    // - `report` — base readiness report from `evaluate_readiness`
    //
    // Returns:
    // Updated readiness report and policy evaluation report.
    //
    // Options:
    // None.
    //
    // Example:
    // let (report, policy) = evaluate_readiness_with_policy(&program, "rover.sd", &options, "WarehousePolicy", report)?;

    let policy_report =
        evaluate_policy_with_options(program, policy_name, source_label, Some(options))?;
    apply_policy_to_readiness(&mut report, &policy_report);
    Ok((report, policy_report))
}

/// Fold policy violations into readiness score, issues, and mission readiness.
pub fn apply_policy_to_readiness(
    report: &mut ReadinessReport,
    policy_report: &PolicyEvaluationReport,
) {
    // Map policy violations to readiness issues and adjust the weighted score.
    //
    // Parameters:
    // - `report` — readiness report to update in place
    // - `policy_report` — operational policy evaluation output
    //
    // Returns:
    // None.
    //
    // Options:
    // None.
    //
    // Example:
    // apply_policy_to_readiness(&mut report, &policy_report);

    let error_count = policy_report
        .violations
        .iter()
        .filter(|violation| violation.severity == PolicySeverity::Error)
        .count();
    let warning_count = policy_report
        .violations
        .iter()
        .filter(|violation| violation.severity == PolicySeverity::Warning)
        .count();

    for violation in &policy_report.violations {
        let severity = match violation.severity {
            PolicySeverity::Error => ReadinessSeverity::High,
            PolicySeverity::Warning => ReadinessSeverity::Medium,
        };
        report.issues.push(ReadinessIssue {
            factor: "OperationalPolicy".into(),
            severity,
            message: format!(
                "policy {} rule {} — {}",
                violation.policy, violation.rule, violation.message
            ),
            suggested_action: Some(format!(
                "Fix operational policy `{}` rule `{}`",
                violation.policy, violation.rule
            )),
        });
    }

    let policy_score = if policy_report.passed {
        100
    } else {
        100u32
            .saturating_sub((error_count as u32).saturating_mul(25))
            .saturating_sub((warning_count as u32).saturating_mul(10))
    };
    report.score.factors.push(ReadinessFactorScore {
        factor: "OperationalPolicy".into(),
        score: policy_score,
        weight: POLICY_FACTOR_WEIGHT,
        weighted: (policy_score as f64) * (POLICY_FACTOR_WEIGHT as f64) / 100.0,
    });
    report.score.total = recompute_weighted_total(&report.score.factors);

    if !policy_report.passed {
        report.mission_ready = false;
        if report.status == ReadinessStatus::Ready {
            report.status = ReadinessStatus::NotReady;
        }
    } else if report.mission_ready
        && report.issues.iter().any(|issue| {
            matches!(
                issue.severity,
                ReadinessSeverity::Medium | ReadinessSeverity::Low
            )
        })
    {
        report.status = ReadinessStatus::Degraded;
    }
}

/// Build a deployment gate entry from an operational policy report.
pub fn operational_policy_gate(
    policy_report: &PolicyEvaluationReport,
) -> spanda_readiness::DeploymentGate {
    // Convert a policy evaluation into a named deployment gate.
    //
    // Parameters:
    // - `policy_report` — operational policy evaluation output
    //
    // Returns:
    // Deployment gate with pass/fail and summary message.
    //
    // Options:
    // None.
    //
    // Example:
    // let gate = operational_policy_gate(&policy_report);

    let violation_count = policy_report.violations.len();
    spanda_readiness::DeploymentGate {
        name: format!("operational_policy:{}", policy_report.policy),
        passed: policy_report.passed,
        message: if policy_report.passed {
            format!("operational policy {} passed", policy_report.policy)
        } else {
            format!(
                "operational policy {} failed with {violation_count} violation(s)",
                policy_report.policy
            )
        },
    }
}

fn recompute_weighted_total(factors: &[ReadinessFactorScore]) -> u32 {
    // Recompute the weighted readiness total after appending a policy factor.
    let weight_sum: u32 = factors.iter().map(|factor| factor.weight).sum();
    if weight_sum == 0 {
        return 0;
    }
    let weighted: f64 = factors.iter().map(|factor| factor.weighted).sum();
    (weighted * 100.0 / weight_sum as f64).round() as u32
}
