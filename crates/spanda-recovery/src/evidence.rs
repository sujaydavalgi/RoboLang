//! Recovery evidence generation — immutable audit records for recovery operations.
//!
use crate::types::{
    OrchestratedRecoveryPlan, OrchestratorRecoveryEvidence, OrchestratorValidationResult,
    RecoveryTimelineEvent,
};
use chrono::Utc;
use spanda_runtime::recovery_types::{RecoveryResult, RecoveryStatus};

/// Generate immutable evidence from a completed recovery operation.
pub fn generate_evidence(
    plan: &OrchestratedRecoveryPlan,
    validation: &OrchestratorValidationResult,
    result: Option<&RecoveryResult>,
    duration_secs: u64,
) -> OrchestratorRecoveryEvidence {
    // Generate immutable evidence from a completed recovery operation.
    //
    // Parameters:
    // - `plan` — orchestrated recovery plan
    // - `validation` — validation result
    // - `result` — optional legacy execution result
    // - `duration_secs` — total recovery duration
    //
    // Returns:
    // Immutable evidence record.
    //
    // Options:
    // None.
    //
    // Example:
    // let evidence = generate_evidence(&plan, &validation, result.as_ref(), 120);

    let now = Utc::now().to_rfc3339();
    let status = result.map(|r| r.status).unwrap_or(if validation.passed {
        RecoveryStatus::Success
    } else {
        RecoveryStatus::Failed
    });

    let mut entities_involved = vec![plan.entity_id.clone()];
    entities_involved.extend(plan.upstream_impact.clone());
    entities_involved.extend(plan.downstream_impact.clone());
    entities_involved.sort();
    entities_involved.dedup();

    let mut automatic_decisions = plan.decision.explanations.clone();
    if plan.decision.automatic {
        automatic_decisions.push("Automatic recovery authorized".into());
    }

    let operator_actions = if plan.decision.is_authorized && !plan.decision.automatic {
        vec!["Operator approval recorded".into()]
    } else {
        Vec::new()
    };

    OrchestratorRecoveryEvidence {
        evidence_id: format!("rcv-{}", chrono::Utc::now().timestamp_millis()),
        root_cause: plan.failure.clone(),
        strategy: plan
            .strategies
            .first()
            .cloned()
            .unwrap_or(plan.decision.recommended_strategy.clone()),
        timeline: build_timeline(plan, duration_secs, &now),
        entities_involved,
        safety_validation: if validation.security_verification {
            "PASS".into()
        } else {
            "FAIL".into()
        },
        readiness_result: if validation.readiness_verification {
            "PASS".into()
        } else {
            "FAIL".into()
        },
        trust_result: if validation.trust_verification {
            "PASS".into()
        } else {
            "FAIL".into()
        },
        operator_actions,
        automatic_decisions,
        mission_impact: format!(
            "disruption_score={} upstream={} downstream={}",
            plan.decision.mission_disruption_score,
            plan.upstream_impact.len(),
            plan.downstream_impact.len()
        ),
        duration_secs,
        status,
        timestamp: now,
    }
}

fn build_timeline(
    plan: &OrchestratedRecoveryPlan,
    total_secs: u64,
    timestamp: &str,
) -> Vec<RecoveryTimelineEvent> {
    let phase_duration = total_secs.saturating_mul(1000) / 5;
    vec![
        RecoveryTimelineEvent {
            phase: "detect".into(),
            description: format!("Detected failure: {}", plan.failure),
            timestamp: timestamp.to_string(),
            duration_ms: phase_duration,
        },
        RecoveryTimelineEvent {
            phase: "diagnose".into(),
            description: plan.diagnosis.clone(),
            timestamp: timestamp.to_string(),
            duration_ms: phase_duration,
        },
        RecoveryTimelineEvent {
            phase: "plan".into(),
            description: format!(
                "Plan {} with {} strategies",
                plan.plan_id,
                plan.strategies.len()
            ),
            timestamp: timestamp.to_string(),
            duration_ms: phase_duration,
        },
        RecoveryTimelineEvent {
            phase: "validate".into(),
            description: "Validation gates executed".into(),
            timestamp: timestamp.to_string(),
            duration_ms: phase_duration,
        },
        RecoveryTimelineEvent {
            phase: "complete".into(),
            description: format!("Recovery completed — risk {}", plan.risk),
            timestamp: timestamp.to_string(),
            duration_ms: phase_duration,
        },
    ]
}
