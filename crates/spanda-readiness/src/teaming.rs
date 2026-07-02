//! Human/robot teaming verification — approval, escalation, and fallback paths.

use crate::approval::{verify_approvals, ApprovalVerifyReport};
use serde::{Deserialize, Serialize};
use spanda_ast::assurance_decl::{ContinuityPolicyDecl, RecoveryPolicyDecl};
use spanda_ast::nodes::Program;

/// Escalation path extracted from recovery/continuity policies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HumanEscalation {
    pub policy: String,
    pub trigger: String,
    pub actions: Vec<String>,
}

/// Human/robot teaming verification report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HumanTeamingReport {
    pub program: String,
    pub approvals: ApprovalVerifyReport,
    pub escalations: Vec<HumanEscalation>,
    pub fallback_actions: Vec<String>,
    pub passed: bool,
}

/// Verify human approval and escalation paths in a program.
pub fn evaluate_human_teaming(program: &Program, source_label: &str) -> HumanTeamingReport {
    let approvals = verify_approvals(program);
    let escalations = extract_escalations(program);
    let fallback_actions = extract_fallback_actions(program);
    let passed = approvals.compatible;
    HumanTeamingReport {
        program: source_label.into(),
        approvals,
        escalations,
        fallback_actions,
        passed,
    }
}

fn extract_escalations(program: &Program) -> Vec<HumanEscalation> {
    let Program::Program {
        recovery_policies,
        continuity_policies,
        ..
    } = program;
    let mut out = Vec::new();
    for policy in recovery_policies {
        let RecoveryPolicyDecl::RecoveryPolicyDecl { name, branches, .. } = policy;
        for branch in branches {
            out.push(HumanEscalation {
                policy: name.clone(),
                trigger: branch.condition.clone(),
                actions: branch.actions.clone(),
            });
        }
    }
    for policy in continuity_policies {
        let ContinuityPolicyDecl::ContinuityPolicyDecl { name, branches, .. } = policy;
        for branch in branches {
            out.push(HumanEscalation {
                policy: name.clone(),
                trigger: branch.condition.clone(),
                actions: branch.actions.clone(),
            });
        }
    }
    out
}

fn extract_fallback_actions(program: &Program) -> Vec<String> {
    let Program::Program {
        recovery_policies,
        health_policies,
        ..
    } = program;
    let mut actions = Vec::new();
    for policy in health_policies {
        let spanda_ast::foundations::HealthPolicyDecl::HealthPolicyDecl { reactions, .. } = policy;
        for reaction in reactions {
            actions.push(format!(
                "health:{} -> {} statements",
                reaction.status,
                reaction.body.len()
            ));
        }
    }
    for policy in recovery_policies {
        let RecoveryPolicyDecl::RecoveryPolicyDecl { branches, .. } = policy;
        for branch in branches {
            for action in &branch.actions {
                if action.contains("safe") || action.contains("degraded") || action.contains("mode") {
                    actions.push(format!("recovery:{} -> {}", branch.condition, action));
                }
            }
        }
    }
    actions
}

/// Format human teaming report for CLI output.
pub fn format_human_teaming(report: &HumanTeamingReport, json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(report).unwrap_or_else(|e| e.to_string());
    }
    format!(
        "Human/robot teaming: {}\nApproval rows: {} compatible={}\nEscalations: {}\nFallback actions: {}\nPassed: {}",
        report.program,
        report.approvals.rows.len(),
        report.approvals.compatible,
        report.escalations.len(),
        report.fallback_actions.len(),
        report.passed
    )
}
