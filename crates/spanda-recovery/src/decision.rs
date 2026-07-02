//! Recovery decision engine — intelligent recovery authorization and strategy selection.
//!
use crate::policy::{evaluate_policy, policy_for_entity};
use crate::types::{
    EntityRecoveryPolicy, OrchestratorStrategy, RecoveryDecision, RecoveryEscalationLevel,
};
use spanda_config::entity::{EntityHealthStatus, EntityRecord, EntityRegistry};
use spanda_runtime::recovery_primitives::classify_failure;
use spanda_runtime::recovery_types::FailureClassification;

/// Make a recovery decision for an entity failure.
pub fn decide_recovery(
    entity: &EntityRecord,
    failure: &str,
    policies: &[EntityRecoveryPolicy],
    registry: &EntityRegistry,
    classification: Option<FailureClassification>,
) -> RecoveryDecision {
    // Make a recovery decision for an entity failure.
    //
    // Parameters:
    // - `entity` — failed entity record
    // - `failure` — failure description
    // - `policies` — entity recovery policies
    // - `registry` — entity registry for backup selection
    // - `classification` — optional failure classification
    //
    // Returns:
    // Recovery decision with explanations.
    //
    // Options:
    // None.
    //
    // Example:
    // let decision = decide_recovery(&entity, "gps_loss", &policies, &registry, None);

    let classification = classification.unwrap_or_else(|| classify_failure(failure));
    let policy = policy_for_entity(policies, &entity.id);
    let recommended = strategy_for_classification(classification, entity);
    let level = recommended.default_escalation_level();
    let mut explanations = Vec::new();

    let can_recover = !matches!(entity.health_status, EntityHealthStatus::Offline)
        || failure.contains("offline")
        || failure.contains("failed");
    if can_recover {
        explanations.push(format!(
            "Entity '{}' ({}) is eligible for recovery",
            entity.id,
            entity.entity_type.as_str()
        ));
    } else {
        explanations.push("Entity is offline with no recoverable state".into());
    }

    let should_recover = !failure.is_empty() && can_recover;
    if should_recover {
        explanations.push(format!("Failure '{}' warrants recovery action", failure));
    }

    let (policy_ok, policy_msgs) = if let Some(p) = policy {
        evaluate_policy(p, &recommended, level)
    } else {
        (
            true,
            vec!["No specific policy — using platform defaults".into()],
        )
    };
    explanations.extend(policy_msgs);

    let is_safe = policy_ok
        && !matches!(classification, FailureClassification::SafetyFailure)
        && level < RecoveryEscalationLevel::Level8EmergencyShutdown;
    if is_safe {
        explanations.push("Recovery action passes safety checks".into());
    } else if matches!(classification, FailureClassification::SafetyFailure) {
        explanations.push("Safety failure — only human-approved recovery permitted".into());
    }

    let requires_approval = policy.map(|p| p.requires_approval).unwrap_or(false)
        || level >= RecoveryEscalationLevel::Level7HumanIntervention
        || matches!(classification, FailureClassification::SafetyFailure);
    let is_authorized =
        !requires_approval || level <= RecoveryEscalationLevel::Level2RestartPackage;
    if requires_approval {
        explanations.push("Recovery requires operator approval".into());
    } else {
        explanations.push("Recovery authorized for automatic execution".into());
    }

    let automatic = is_authorized && is_safe && should_recover && policy_ok;
    let lowest_risk = lowest_risk_strategy(classification);
    let backup = find_backup_entity(registry, entity, classification);
    let mission_disruption = disruption_score(level);
    let downtime = estimated_downtime(level);

    if let Some(ref backup_id) = backup {
        explanations.push(format!("Backup entity '{}' available", backup_id));
    }

    explanations.push(format!(
        "Lowest-risk strategy: '{}' (disruption score {})",
        lowest_risk.label(),
        mission_disruption
    ));

    RecoveryDecision {
        can_recover,
        should_recover,
        is_safe,
        is_authorized,
        automatic,
        recommended_strategy: recommended,
        recommended_level: level,
        lowest_risk_strategy: lowest_risk,
        mission_disruption_score: mission_disruption,
        estimated_downtime_secs: downtime,
        backup_entity_id: backup,
        explanations,
    }
}

fn strategy_for_classification(
    classification: FailureClassification,
    entity: &EntityRecord,
) -> OrchestratorStrategy {
    match classification {
        FailureClassification::SensorFailure => OrchestratorStrategy::SwitchSensor,
        FailureClassification::ActuatorFailure => OrchestratorStrategy::Reinitialize,
        FailureClassification::ConnectivityFailure => OrchestratorStrategy::Reconnect,
        FailureClassification::ProviderFailure => OrchestratorStrategy::SwitchProvider,
        FailureClassification::PackageFailure => OrchestratorStrategy::RestartPackage,
        FailureClassification::MissionFailure => OrchestratorStrategy::TransferMission,
        FailureClassification::HealthDegradation => OrchestratorStrategy::GracefulDegradation,
        FailureClassification::FleetFailure => OrchestratorStrategy::RestartFleet,
        FailureClassification::SwarmFailure => OrchestratorStrategy::RestartFleet,
        FailureClassification::SafetyFailure => OrchestratorStrategy::HumanEscalation,
        FailureClassification::Unknown => {
            if entity.entity_type.as_str().contains("robot") {
                OrchestratorStrategy::RestartRobot
            } else {
                OrchestratorStrategy::Retry
            }
        }
    }
}

fn lowest_risk_strategy(classification: FailureClassification) -> OrchestratorStrategy {
    match classification {
        FailureClassification::SafetyFailure => OrchestratorStrategy::HumanEscalation,
        FailureClassification::ConnectivityFailure => OrchestratorStrategy::Retry,
        FailureClassification::SensorFailure => OrchestratorStrategy::Retry,
        _ => OrchestratorStrategy::GracefulDegradation,
    }
}

fn find_backup_entity(
    registry: &EntityRegistry,
    entity: &EntityRecord,
    classification: FailureClassification,
) -> Option<String> {
    let kind = &entity.entity_type;
    let all = registry.list();
    let candidates: Vec<_> = all
        .iter()
        .filter(|e| {
            e.id != entity.id
                && e.entity_type == *kind
                && !matches!(
                    e.health_status,
                    EntityHealthStatus::Critical | EntityHealthStatus::Offline
                )
        })
        .collect();

    if let Some(first) = candidates.into_iter().next() {
        return Some(first.id.clone());
    }

    if matches!(
        classification,
        FailureClassification::FleetFailure | FailureClassification::MissionFailure
    ) {
        return registry
            .list()
            .iter()
            .find(|e| {
                e.entity_type.as_str() == "robot"
                    && e.id != entity.id
                    && !matches!(e.health_status, EntityHealthStatus::Critical)
            })
            .map(|e| e.id.clone());
    }
    None
}

fn disruption_score(level: RecoveryEscalationLevel) -> u32 {
    match level {
        RecoveryEscalationLevel::Level0Retry => 5,
        RecoveryEscalationLevel::Level1RestartComponent => 15,
        RecoveryEscalationLevel::Level2RestartPackage => 25,
        RecoveryEscalationLevel::Level3RecoverDevice => 40,
        RecoveryEscalationLevel::Level4RecoverRobot => 55,
        RecoveryEscalationLevel::Level5MissionReassign => 70,
        RecoveryEscalationLevel::Level6FleetRedistribute => 80,
        RecoveryEscalationLevel::Level7HumanIntervention => 90,
        RecoveryEscalationLevel::Level8EmergencyShutdown => 100,
    }
}

fn estimated_downtime(level: RecoveryEscalationLevel) -> u64 {
    match level {
        RecoveryEscalationLevel::Level0Retry => 5,
        RecoveryEscalationLevel::Level1RestartComponent => 30,
        RecoveryEscalationLevel::Level2RestartPackage => 60,
        RecoveryEscalationLevel::Level3RecoverDevice => 120,
        RecoveryEscalationLevel::Level4RecoverRobot => 180,
        RecoveryEscalationLevel::Level5MissionReassign => 300,
        RecoveryEscalationLevel::Level6FleetRedistribute => 600,
        RecoveryEscalationLevel::Level7HumanIntervention => 1800,
        RecoveryEscalationLevel::Level8EmergencyShutdown => 60,
    }
}
