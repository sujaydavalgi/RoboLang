//! Escalation model for distributed decisions.

use crate::types::{DecisionEscalation, DecisionLayer};
use serde::{Deserialize, Serialize};

/// Escalation trigger reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationReason {
    LocalRecoveryFailed,
    InsufficientCapability,
    TrustCompromised,
    MissionRiskIncreased,
    HumanApprovalRequired,
    PolicyViolation,
    OfflineLimitReached,
}

/// Build escalation chain from local entity upward.
pub fn build_escalation_chain(
    entity_id: &str,
    reason: EscalationReason,
    start_layer: DecisionLayer,
) -> Vec<DecisionEscalation> {
    // Description:
    //     Construct the standard escalation path from a starting layer.
    //
    // Parameters:
    // - `entity_id` — originating entity
    // - `reason` — why escalation is needed
    // - `start_layer` — layer that initiated escalation
    //
    // Returns:
    // Ordered escalation steps.
    //
    // Options:
    // None.
    //
    // Example:
    // let chain = build_escalation_chain("Rover001", EscalationReason::LocalRecoveryFailed, DecisionLayer::LocalEntity);

    let reason_str = format!("{reason:?}");
    let layers = match start_layer {
        DecisionLayer::Reflex | DecisionLayer::LocalEntity => vec![
            (DecisionLayer::LocalEntity, DecisionLayer::GroupFleet),
            (DecisionLayer::GroupFleet, DecisionLayer::ControlCenter),
        ],
        DecisionLayer::GroupFleet => {
            vec![(DecisionLayer::GroupFleet, DecisionLayer::ControlCenter)]
        }
        DecisionLayer::ControlCenter => vec![],
    };
    layers
        .into_iter()
        .enumerate()
        .map(|(i, (from, to))| DecisionEscalation {
            from_layer: from,
            to_layer: to,
            reason: reason_str.clone(),
            entity_id: entity_id.into(),
            pending_approval: to == DecisionLayer::ControlCenter,
            escalation_id: format!("esc-{entity_id}-{i}"),
        })
        .collect()
}

/// Approve a pending escalation at control center (delegates to persistent store).
pub fn approve_escalation(
    escalation: &mut DecisionEscalation,
    approver: &str,
) -> Result<(), String> {
    crate::escalation_store::approve_escalation_with_store(escalation, approver)
}
