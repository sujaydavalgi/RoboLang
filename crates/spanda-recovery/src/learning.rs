//! Recovery learning — rule-based knowledge base and historical statistics.
//!
use crate::types::{
    OrchestratorRecoveryEvidence, OrchestratorStrategy, RecoveryMetrics, RepeatedFailure,
    StrategyEffectiveness,
};
use spanda_assurance::recovery::load_merged_recovery_knowledge;
use spanda_runtime::recovery_types::{
    RecoveryKnowledgeBase, RecoveryKnowledgeEntry, RecoveryStatus,
};
use std::collections::HashMap;

/// In-memory recovery history store (rule-based learning, no ML).
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RecoveryHistoryStore {
    pub evidence: Vec<OrchestratorRecoveryEvidence>,
}

impl RecoveryHistoryStore {
    pub fn record(&mut self, evidence: OrchestratorRecoveryEvidence) {
        self.evidence.push(evidence);
    }

    pub fn recent(&self, limit: usize) -> Vec<&OrchestratorRecoveryEvidence> {
        self.evidence.iter().rev().take(limit).collect()
    }
}

/// Compute recovery metrics from history and knowledge base.
pub fn compute_metrics(
    history: &RecoveryHistoryStore,
    knowledge: &RecoveryKnowledgeBase,
) -> RecoveryMetrics {
    // Compute recovery metrics from history and knowledge base.
    //
    // Parameters:
    // - `history` — orchestrator evidence history
    // - `knowledge` — legacy knowledge base entries
    //
    // Returns:
    // Aggregated recovery metrics.
    //
    // Options:
    // None.
    //
    // Example:
    // let metrics = compute_metrics(&history, &knowledge);

    let total = history.evidence.len() as u64;
    let successful = history
        .evidence
        .iter()
        .filter(|e| e.status == RecoveryStatus::Success)
        .count() as u64;
    let failed = history
        .evidence
        .iter()
        .filter(|e| e.status == RecoveryStatus::Failed)
        .count() as u64;
    let success_rate = if total == 0 {
        knowledge
            .entries
            .iter()
            .map(|e| e.success_rate)
            .fold(0.0_f64, |a, b| a.max(b))
    } else {
        successful as f64 / total as f64
    };
    let average_duration = if total == 0 {
        0.0
    } else {
        history
            .evidence
            .iter()
            .map(|e| e.duration_secs as f64)
            .sum::<f64>()
            / total as f64
    };

    let mut strategy_stats: HashMap<String, (u64, u64, f64)> = HashMap::new();
    for entry in &history.evidence {
        let key = entry.strategy.label().to_string();
        let slot = strategy_stats.entry(key).or_insert((0, 0, 0.0));
        slot.0 += 1;
        if entry.status == RecoveryStatus::Success {
            slot.1 += 1;
        }
        slot.2 += entry.duration_secs as f64;
    }
    for kb_entry in &knowledge.entries {
        let slot = strategy_stats
            .entry(kb_entry.recovery_pattern.clone())
            .or_insert((0, 0, 0.0));
        if slot.0 == 0 {
            slot.1 = (kb_entry.success_rate * 100.0) as u64;
            slot.0 = 100;
        }
    }

    let mut most_effective: Vec<StrategyEffectiveness> = strategy_stats
        .into_iter()
        .map(
            |(strategy, (count, successes, total_dur))| StrategyEffectiveness {
                strategy: OrchestratorStrategy::Custom(strategy.clone()),
                success_rate: if count == 0 {
                    0.0
                } else {
                    successes as f64 / count as f64
                },
                average_duration_secs: if count == 0 {
                    0.0
                } else {
                    total_dur / count as f64
                },
                usage_count: count,
            },
        )
        .collect();
    most_effective.sort_by(|a, b| {
        b.success_rate
            .partial_cmp(&a.success_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    most_effective.truncate(10);

    let mut failure_counts: HashMap<(String, String), u64> = HashMap::new();
    for entry in &history.evidence {
        for entity_id in &entry.entities_involved {
            *failure_counts
                .entry((entry.root_cause.clone(), entity_id.clone()))
                .or_insert(0) += 1;
        }
    }
    let repeated_failures: Vec<RepeatedFailure> = failure_counts
        .into_iter()
        .filter(|(_, count)| *count >= 2)
        .map(|((pattern, entity_id), count)| RepeatedFailure {
            failure_pattern: pattern,
            entity_id,
            occurrence_count: count,
            last_seen: chrono::Utc::now().to_rfc3339(),
        })
        .collect();

    let recovery_confidence = if total == 0 {
        0.5
    } else {
        (success_rate * 0.7)
            + (if repeated_failures.is_empty() {
                0.3
            } else {
                0.1
            })
    };

    RecoveryMetrics {
        total_recoveries: total,
        successful_recoveries: successful,
        failed_recoveries: failed,
        success_rate,
        average_duration_secs: average_duration,
        most_effective_strategies: most_effective,
        repeated_failures,
        recovery_confidence,
    }
}

/// Recommend strategy from knowledge base for a failure pattern.
pub fn recommend_strategy(
    knowledge: &RecoveryKnowledgeBase,
    failure: &str,
) -> Option<RecoveryKnowledgeEntry> {
    let failure_lower = failure.to_ascii_lowercase();
    knowledge
        .entries
        .iter()
        .filter(|e| {
            failure_lower.contains(&e.failure_pattern.to_ascii_lowercase())
                || e.failure_pattern
                    .to_ascii_lowercase()
                    .contains(&failure_lower)
        })
        .max_by(|a, b| {
            a.success_rate
                .partial_cmp(&b.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
}

/// Record outcome into history store.
pub fn record_learning_outcome(
    history: &mut RecoveryHistoryStore,
    evidence: OrchestratorRecoveryEvidence,
) {
    history.record(evidence);
}

/// Load merged knowledge from program declarations and disk store.
pub fn load_knowledge(program: &spanda_ast::nodes::Program) -> RecoveryKnowledgeBase {
    load_merged_recovery_knowledge(program)
}
