//! Recovery validation — post-recovery health, readiness, trust, and mission gates.
//!
use crate::types::OrchestratorValidationResult;
use spanda_assurance::recovery::{evaluate_recovery_readiness, validate_recovery_plan};
use spanda_ast::nodes::Program;
use spanda_config::entity::{EntityRegistry, EntityTrustStatus};
use spanda_runtime::recovery_types::RecoveryPlan;

/// Validate recovery plan through all required gates.
pub fn validate_recovery(
    program: &Program,
    plan: &RecoveryPlan,
    registry: &EntityRegistry,
    entity_id: &str,
    validation_rules: &[String],
) -> OrchestratorValidationResult {
    // Validate recovery plan through all required gates.
    //
    // Parameters:
    // - `program` — Spanda program
    // - `plan` — legacy recovery plan
    // - `registry` — entity registry
    // - `entity_id` — target entity
    // - `validation_rules` — required validation gate names
    //
    // Returns:
    // Aggregated validation result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = validate_recovery(&program, &plan, &registry, "robot-1", &rules);

    let safe_actions = validate_recovery_plan(program, plan);
    let readiness = evaluate_recovery_readiness(program, plan);
    let mut messages = Vec::new();

    let health_verification = validation_rules.is_empty()
        || validation_rules.iter().any(|r| r == "health")
            && safe_actions.iter().all(|a| a.hardware_verification.passed);
    if !health_verification {
        messages.push("Health verification failed".into());
    }

    let capability_verification = safe_actions.is_empty()
        || safe_actions
            .iter()
            .all(|a| a.capability_verification.passed);
    if !capability_verification {
        messages.push("Capability verification failed".into());
    }

    let hardware_verification =
        safe_actions.is_empty() || safe_actions.iter().all(|a| a.hardware_verification.passed);
    if !hardware_verification {
        messages.push("Hardware verification failed".into());
    }

    let readiness_verification = readiness.recovery_ready;
    if !readiness_verification {
        messages.push(format!(
            "Readiness verification failed: {}",
            readiness.blockers.join(", ")
        ));
    }

    let trust_verification = registry
        .get(entity_id)
        .map(|e| {
            !matches!(
                e.trust_status,
                EntityTrustStatus::Untrusted | EntityTrustStatus::Compromised
            )
        })
        .unwrap_or(true);
    if !trust_verification {
        messages.push("Trust verification failed — entity untrusted".into());
    }

    let security_verification = safe_actions.iter().all(|a| a.safety_validation.passed);
    if !security_verification {
        messages.push("Security/safety validation failed".into());
    }

    let mission_validation = !plan
        .classification
        .eq(&spanda_runtime::recovery_types::FailureClassification::MissionFailure)
        || safe_actions.iter().any(|a| a.approved);
    if !mission_validation {
        messages.push("Mission validation requires approval".into());
    }

    let passed = health_verification
        && capability_verification
        && hardware_verification
        && readiness_verification
        && trust_verification
        && security_verification
        && mission_validation;

    if passed {
        messages.push("All validation gates passed".into());
    }

    OrchestratorValidationResult {
        passed,
        health_verification,
        capability_verification,
        hardware_verification,
        readiness_verification,
        trust_verification,
        security_verification,
        mission_validation,
        messages,
    }
}
