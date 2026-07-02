//! Decision-backed implementation of the runtime decision boundary.

use spanda_ast::nodes::Program;
use spanda_runtime::decision_runtime::{
    DecisionActionVerdict, DecisionRuntime, DecisionTreeEvalResult, FleetConsensusEvalResult,
};
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    build_escalation_chain, default_safety_boundaries, entity_may_decide_locally,
    evaluate_tree, extract_decision_authorities, extract_decision_trees, resolve_offline_policies,
    resolve_consensus, validate_offline_action, ConsensusStrategy,
    ConsensusVote, DecisionLayer, EscalationReason,
};

fn normalize_decision_action_key(action: &str) -> String {
    let lower = action.to_lowercase();
    for prefix in [
        "enter ",
        "pause ",
        "reduce_speed ",
        "switch_to ",
        "request_",
        "trigger ",
        "stop_",
        "cut_",
    ] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            return rest.replace(' ', "_");
        }
    }
    lower.replace(' ', "_")
}

/// Register the full decision runtime for fleet agents and platform callers.
pub fn register_platform_runtime() {
    spanda_runtime::decision_runtime::set_platform_decision_runtime(Arc::new(DecisionBackedRuntime));
}

/// Full distributed decision runtime delegating to `spanda-decision`.
#[derive(Debug, Default, Clone, Copy)]
pub struct DecisionBackedRuntime;

impl DecisionRuntime for DecisionBackedRuntime {
    fn evaluate_trees(
        &self,
        program: &Program,
        signals: &HashMap<String, bool>,
    ) -> Vec<DecisionTreeEvalResult> {
        extract_decision_trees(program)
            .iter()
            .filter_map(|spec| evaluate_tree(spec, signals))
            .map(|r| DecisionTreeEvalResult {
                tree_name: r.tree_name,
                layer: format!("{:?}", r.layer).to_lowercase(),
                condition_matched: r.condition_matched,
                actions: r.actions,
                tree_hash: r.tree_hash,
            })
            .collect()
    }

    fn resolve_fleet_consensus(
        &self,
        votes: &[(String, String, f64)],
        quorum_fraction: f64,
    ) -> FleetConsensusEvalResult {
        let consensus_votes: Vec<ConsensusVote> = votes
            .iter()
            .map(|(entity, action, weight)| ConsensusVote {
                entity_id: entity.clone(),
                action: action.clone(),
                trust_weight: *weight,
                timestamp_ms: 0.0,
            })
            .collect();
        let result = resolve_consensus(
            ConsensusStrategy::TrustWeightedVoting,
            &consensus_votes,
            quorum_fraction,
        );
        FleetConsensusEvalResult {
            strategy: format!("{:?}", result.strategy),
            selected_action: result.selected_action,
            quorum_met: result.quorum_met,
            vote_count: result.votes.len(),
        }
    }

    fn authorize_action(
        &self,
        program: &Program,
        entity_id: &str,
        action: &str,
        offline_minutes: u32,
        central_connected: bool,
    ) -> DecisionActionVerdict {
        let action_key = normalize_decision_action_key(action);
        let policy_version = "1.0.0".to_string();

        if !central_connected {
            for policy in resolve_offline_policies(program) {
                if let Err(reason) = crate::offline::validate_offline_policy_trust(&policy) {
                    return DecisionActionVerdict {
                        permitted: false,
                        reason,
                        requires_escalation: false,
                        escalation_id: None,
                        policy_version: Some(policy.policy_version.clone()),
                    };
                }
                if let Err(reason) =
                    validate_offline_action(&policy, &action_key, offline_minutes)
                {
                    return DecisionActionVerdict {
                        permitted: false,
                        reason,
                        requires_escalation: false,
                        escalation_id: None,
                        policy_version: Some(policy.policy_version.clone()),
                    };
                }
            }
        }

        for boundary in default_safety_boundaries() {
            if action_key.contains(&boundary.action) || action.contains(&boundary.action) {
                if boundary.requires_approval {
                    let chain = build_escalation_chain(
                        entity_id,
                        EscalationReason::HumanApprovalRequired,
                        DecisionLayer::LocalEntity,
                    );
                    return DecisionActionVerdict {
                        permitted: false,
                        reason: boundary.reason.clone(),
                        requires_escalation: true,
                        escalation_id: chain.last().map(|e| e.escalation_id.clone()),
                        policy_version: Some(policy_version.clone()),
                    };
                }
            }
        }

        if let Some(auth) = extract_decision_authorities(program)
            .into_iter()
            .find(|a| a.entity_id == entity_id)
        {
            if auth
                .requires_central_approval
                .iter()
                .any(|a| a == &action_key || action.contains(a))
            {
                let chain = build_escalation_chain(
                    entity_id,
                    EscalationReason::HumanApprovalRequired,
                    DecisionLayer::LocalEntity,
                );
                return DecisionActionVerdict {
                    permitted: false,
                    reason: format!("action '{action_key}' requires central approval"),
                    requires_escalation: true,
                    escalation_id: chain.last().map(|e| e.escalation_id.clone()),
                    policy_version: Some(policy_version.clone()),
                };
            }
            if !auth.local_actions.is_empty()
                && !entity_may_decide_locally(&auth, &action_key)
                && !auth.local_actions.iter().any(|a| action.contains(a))
            {
                return DecisionActionVerdict {
                    permitted: false,
                    reason: format!("action '{action_key}' not in local_decision_authority"),
                    requires_escalation: true,
                    escalation_id: Some(format!("esc-{entity_id}-local")),
                    policy_version: Some(policy_version.clone()),
                };
            }
        }

        DecisionActionVerdict {
            permitted: true,
            reason: "authorized".into(),
            requires_escalation: false,
            escalation_id: None,
            policy_version: Some(policy_version),
        }
    }
}
