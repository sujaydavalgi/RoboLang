//! Fleet / swarm consensus strategies for distributed decisions.

use serde::{Deserialize, Serialize};

/// Consensus strategy for group-level decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusStrategy {
    CoordinatorDecision,
    Quorum,
    Majority,
    TrustWeightedVoting,
    LeaderFollower,
    BackupLeaderPromotion,
}

/// Vote cast by a fleet member.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsensusVote {
    pub entity_id: String,
    pub action: String,
    pub trust_weight: f64,
    pub timestamp_ms: f64,
}

/// Result of a consensus round.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub strategy: ConsensusStrategy,
    pub selected_action: String,
    pub votes: Vec<ConsensusVote>,
    pub quorum_met: bool,
    pub leader_entity: Option<String>,
}

/// Resolve consensus among fleet members.
pub fn resolve_consensus(
    strategy: ConsensusStrategy,
    votes: &[ConsensusVote],
    quorum_fraction: f64,
) -> ConsensusResult {
    // Description:
    //     Apply the chosen consensus strategy to member votes.
    //
    // Parameters:
    // - `strategy` — consensus algorithm
    // - `votes` — member votes
    // - `quorum_fraction` — minimum participation (0.0–1.0)
    //
    // Returns:
    // Consensus result with selected action.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = resolve_consensus(ConsensusStrategy::Majority, &votes, 0.5);

    let leader_entity = votes.first().map(|v| v.entity_id.clone());
    let selected = match &strategy {
        ConsensusStrategy::CoordinatorDecision | ConsensusStrategy::LeaderFollower => votes
            .first()
            .map(|v| v.action.clone())
            .unwrap_or_else(|| "no_action".into()),
        ConsensusStrategy::Majority | ConsensusStrategy::Quorum => majority_action(votes),
        ConsensusStrategy::TrustWeightedVoting => trust_weighted_action(votes),
        ConsensusStrategy::BackupLeaderPromotion => votes
            .get(1)
            .map(|v| v.action.clone())
            .or_else(|| votes.first().map(|v| v.action.clone()))
            .unwrap_or_else(|| "promote_backup".into()),
    };
    let quorum_met =
        !votes.is_empty() && (votes.len() as f64 / votes.len().max(1) as f64) >= quorum_fraction;
    ConsensusResult {
        strategy,
        selected_action: selected,
        votes: votes.to_vec(),
        quorum_met,
        leader_entity,
    }
}

fn majority_action(votes: &[ConsensusVote]) -> String {
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for v in votes {
        *counts.entry(v.action.clone()).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .map(|(a, _)| a)
        .unwrap_or_else(|| "no_action".into())
}

fn trust_weighted_action(votes: &[ConsensusVote]) -> String {
    let mut weights: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for v in votes {
        *weights.entry(v.action.clone()).or_default() += v.trust_weight;
    }
    weights
        .into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(a, _)| a)
        .unwrap_or_else(|| "no_action".into())
}
