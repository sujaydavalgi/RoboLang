//! Decision-backed implementation of the runtime decision boundary.

use spanda_ast::nodes::Program;
use spanda_runtime::decision_runtime::{
    DecisionRuntime, DecisionTreeEvalResult, FleetConsensusEvalResult,
};
use std::collections::HashMap;

use crate::{evaluate_tree, extract_decision_trees, resolve_consensus, ConsensusStrategy, ConsensusVote};

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
}
