//! Tamper policy extraction and runtime matching for interpreter dispatch.

use serde::{Deserialize, Serialize};
use spanda_ast::assurance_decl::{TamperPolicyBranch, TamperPolicyDecl};
use spanda_ast::nodes::Program;

/// Tamper severity aligned with platform maturity taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TamperSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Declarative tamper response policy extracted from a program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TamperPolicySpec {
    pub name: String,
    pub triggers: Vec<(String, Vec<String>)>,
}

fn normalize_tamper_action(action: &str) -> String {
    action
        .replace("( )", "()")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" (", "(")
        .replace(") ", ")")
}

/// Extract tamper policies declared in a parsed program.
pub fn extract_tamper_policies(program: &Program) -> Vec<TamperPolicySpec> {
    let Program::Program {
        tamper_policies, ..
    } = program;
    tamper_policies
        .iter()
        .map(|decl| {
            let TamperPolicyDecl::TamperPolicyDecl { name, branches, .. } = decl;
            TamperPolicySpec {
                name: name.clone(),
                triggers: branches
                    .iter()
                    .map(
                        |TamperPolicyBranch {
                             condition, actions, ..
                         }| {
                            (
                                condition.clone(),
                                actions
                                    .iter()
                                    .map(|action| normalize_tamper_action(action))
                                    .collect(),
                            )
                        },
                    )
                    .collect(),
            }
        })
        .collect()
}

/// Resolve tamper policy actions for a runtime signal and severity.
pub fn actions_for_tamper_event(
    policies: &[TamperPolicySpec],
    signal: &str,
    severity: TamperSeverity,
) -> Vec<String> {
    let mut actions = Vec::new();
    for policy in policies {
        for (condition, branch_actions) in &policy.triggers {
            if tamper_condition_matches(condition, signal, severity) {
                actions.extend(branch_actions.iter().cloned());
            }
        }
    }
    actions
}

/// Report whether a program declares tamper response policies.
pub fn tamper_policy_coverage(program: &Program) -> (bool, usize) {
    let policies = extract_tamper_policies(program);
    let branch_count = policies
        .iter()
        .map(|policy| policy.triggers.len())
        .sum::<usize>();
    (!policies.is_empty(), branch_count)
}

fn tamper_condition_matches(condition: &str, signal: &str, severity: TamperSeverity) -> bool {
    let normalized = condition.to_lowercase();
    let signal_lower = signal.to_lowercase();
    let severity_label = format!("{:?}", severity).to_lowercase();

    if normalized.starts_with("tamper.severity.") {
        let expected = normalized.trim_start_matches("tamper.severity.");
        return expected == severity_label || expected == "any";
    }

    if normalized.starts_with("tamper.signal.") {
        let expected = normalized.trim_start_matches("tamper.signal.");
        return signal_lower.contains(expected) || expected == "any";
    }

    if normalized == "gps.spoofed" {
        return signal_lower.contains("gps.spoofed") || signal_lower.contains("spoof");
    }

    signal_lower.contains(&normalized)
        || normalized
            .split('.')
            .all(|part| signal_lower.contains(part))
}
