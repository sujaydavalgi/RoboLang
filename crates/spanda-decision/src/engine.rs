//! Distributed decision engine — orchestrates layers, trees, and traceability.

use crate::authority::{
    default_safety_boundaries, extract_decision_authorities, validate_against_policy,
};
use crate::escalation::{build_escalation_chain, EscalationReason};
use crate::offline::extract_offline_policies;
use crate::policy_cache::build_policy_cache;
use crate::trees::{evaluate_tree, extract_decision_trees, DecisionTreeSpec};
use crate::types::{
    DecisionLayer, DecisionSecurityEnvelope, DecisionType, DistributedDecisionRecord,
};
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use std::collections::HashMap;

/// Context for evaluating a distributed decision.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionContext {
    pub entity_id: String,
    pub mission: Option<String>,
    pub layer: DecisionLayer,
    pub action: String,
    pub signals: HashMap<String, bool>,
    pub offline_minutes: u32,
    pub policy_version: String,
}

/// Full distributed decision evaluation report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DistributedDecisionReport {
    pub entity_id: String,
    pub mission: Option<String>,
    pub authorities: Vec<crate::types::DecisionAuthority>,
    pub trees: Vec<DecisionTreeSpec>,
    pub offline_policies: Vec<crate::offline::OfflinePolicySpec>,
    pub decisions: Vec<DistributedDecisionRecord>,
    pub policy_cache: crate::policy_cache::LocalPolicyCache,
    pub safety_boundaries: Vec<crate::types::DecisionBoundary>,
    pub passed: bool,
    pub messages: Vec<String>,
}

/// Evaluate distributed decisions for a program and context.
pub fn evaluate_distributed_decisions(
    program: &Program,
    context: &DecisionContext,
) -> DistributedDecisionReport {
    // Description:
    //     Run the full distributed decision pipeline for a program.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `context` — decision evaluation context
    //
    // Returns:
    // Report with authorities, trees, decisions, and validation messages.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = evaluate_distributed_decisions(&program, &ctx);

    let authorities = extract_decision_authorities(program);
    let trees = extract_decision_trees(program);
    let offline_policies = extract_offline_policies(program);
    let safety_boundaries = default_safety_boundaries();
    let mut messages = Vec::new();
    let mut decisions = Vec::new();
    let mut passed = true;

    let policy_cache = build_policy_cache(
        &context.entity_id,
        offline_policies
            .iter()
            .map(|o| crate::offline::offline_to_decision_policy(o, &context.policy_version))
            .collect(),
        safety_boundaries.iter().map(|b| b.action.clone()).collect(),
    );

    for tree in &trees {
        if let Some(result) = evaluate_tree(tree, &context.signals) {
            let record = build_decision_record(
                context,
                DecisionType::Recovery,
                &result.actions.join(", "),
                &result.tree_hash,
                &[],
            );
            decisions.push(record);
            messages.push(format!(
                "tree '{}' matched condition '{}' → [{}]",
                result.tree_name,
                result.condition_matched,
                result.actions.join(", ")
            ));
        }
    }

    if let Some(auth) = authorities
        .iter()
        .find(|a| a.entity_id == context.entity_id)
    {
        if !crate::authority::entity_may_decide_locally(auth, &context.action) {
            passed = false;
            messages.push(format!(
                "action '{}' requires central approval for entity '{}'",
                context.action, context.entity_id
            ));
            let escalations = build_escalation_chain(
                &context.entity_id,
                EscalationReason::HumanApprovalRequired,
                context.layer,
            );
            let mut record = build_decision_record(
                context,
                DecisionType::Policy,
                &context.action,
                "",
                &escalations,
            );
            record.escalation_path = escalations;
            decisions.push(record);
        }
    }

    for boundary in &safety_boundaries {
        if boundary.action == context.action && context.layer as u8 > boundary.max_layer as u8 {
            passed = false;
            messages.push(boundary.reason.clone());
        }
    }

    if context.offline_minutes > 0 {
        for op in &offline_policies {
            if let Err(e) = crate::offline::validate_offline_action(
                op,
                &context.action,
                context.offline_minutes,
            ) {
                passed = false;
                messages.push(e);
            }
        }
    }

    if let Some(policy) = policy_cache.get_policy("default") {
        if let Err(e) = validate_against_policy(policy, &context.action, context.layer) {
            passed = false;
            messages.push(e);
        }
    }

    DistributedDecisionReport {
        entity_id: context.entity_id.clone(),
        mission: context.mission.clone(),
        authorities,
        trees,
        offline_policies,
        decisions,
        policy_cache,
        safety_boundaries,
        passed,
        messages,
    }
}

fn build_decision_record(
    context: &DecisionContext,
    decision_type: DecisionType,
    selected_action: &str,
    tree_hash: &str,
    escalations: &[crate::types::DecisionEscalation],
) -> DistributedDecisionRecord {
    let now = context.policy_version.parse::<f64>().unwrap_or(0.0);
    DistributedDecisionRecord {
        decision_id: format!(
            "dd-{}-{}",
            context.entity_id,
            selected_action.replace(' ', "_")
        ),
        layer: context.layer,
        decision_type,
        entity_id: context.entity_id.clone(),
        mission: context.mission.clone(),
        timestamp_ms: now,
        inputs: serde_json::json!(context.signals),
        policy_version: context.policy_version.clone(),
        local_context: serde_json::json!({ "offline_minutes": context.offline_minutes }),
        selected_action: selected_action.into(),
        rejected_alternatives: vec![],
        safety_validation: serde_json::json!({ "passed": true }),
        trust_validation: serde_json::json!({ "passed": true }),
        escalation_path: escalations.to_vec(),
        outcome: None,
        security: DecisionSecurityEnvelope {
            entity_id: context.entity_id.clone(),
            authority_scope: format!("{:?}", context.layer),
            policy_version: context.policy_version.clone(),
            decision_tree_hash: if tree_hash.is_empty() {
                None
            } else {
                Some(tree_hash.into())
            },
            timestamp_ms: now,
            nonce: format!("n-{}", selected_action.len()),
            signature: None,
            safety_validation_passed: true,
            trust_validation_passed: true,
            audit_record_id: None,
        },
    }
}

/// Format distributed decision report for CLI.
pub fn format_distributed_report(report: &DistributedDecisionReport, json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(report).unwrap_or_default();
    }
    let mut out = format!(
        "Distributed decisions for {} ({})\n",
        report.entity_id,
        if report.passed { "PASSED" } else { "FAILED" }
    );
    out.push_str(&format!("  Authorities: {}\n", report.authorities.len()));
    out.push_str(&format!("  Decision trees: {}\n", report.trees.len()));
    out.push_str(&format!("  Records: {}\n", report.decisions.len()));
    for msg in &report.messages {
        out.push_str(&format!("  - {msg}\n"));
    }
    for d in &report.decisions {
        out.push_str(&format!(
            "\n  [{}] {:?} @ {:?}: {}\n",
            d.decision_id, d.decision_type, d.layer, d.selected_action
        ));
    }
    out
}
