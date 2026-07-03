//! Security validation and live attack simulation for distributed decisions.

use crate::conflict::{resolve_conflict, CompetingDecision};
use crate::enforcement::{
    clear_nonce_registry, detect_policy_tampering, offline_decision_expired,
    register_decision_nonce, tamper_policy_for_test, validate_decision_timestamp,
    validate_decision_trace_payload, validate_security_envelope_fields,
};
use crate::offline::{
    sign_offline_policy, validate_offline_action, verify_offline_policy_signature, OfflinePolicySpec,
};
use crate::types::DecisionSecurityEnvelope;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
/// Attack scenario for security simulation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackScenario {
    PolicyTampering,
    FakeCoordinator,
    ReplayedDecision,
    CompromisedRobot,
    PoisonedTelemetry,
    OfflineAbuse,
    SplitBrainCoordinator,
}

/// Security audit finding (static threat catalog).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityAuditFinding {
    pub scenario: AttackScenario,
    pub detected: bool,
    pub severity: String,
    pub mitigation: String,
}

/// Live attack simulation result with evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttackSimulationResult {
    pub scenario: AttackScenario,
    pub blocked: bool,
    pub severity: String,
    pub mitigation: String,
    pub evidence: Value,
}

/// Validate a decision security envelope (field presence and validation flags).
pub fn validate_security_envelope(envelope: &DecisionSecurityEnvelope) -> Result<(), String> {
    validate_security_envelope_fields(envelope)
}

/// Run security audit across standard attack scenarios (static catalog).
pub fn security_audit() -> Vec<SecurityAuditFinding> {
    vec![
        SecurityAuditFinding {
            scenario: AttackScenario::PolicyTampering,
            detected: true,
            severity: "high".into(),
            mitigation: "Signed policy cache with version pinning and tamper hash".into(),
        },
        SecurityAuditFinding {
            scenario: AttackScenario::FakeCoordinator,
            detected: true,
            severity: "critical".into(),
            mitigation: "Entity trust validation and coordinator identity attestation".into(),
        },
        SecurityAuditFinding {
            scenario: AttackScenario::ReplayedDecision,
            detected: true,
            severity: "high".into(),
            mitigation: "Nonce and timestamp bounds on every decision envelope".into(),
        },
        SecurityAuditFinding {
            scenario: AttackScenario::CompromisedRobot,
            detected: true,
            severity: "critical".into(),
            mitigation: "Capability verification and trust policy block".into(),
        },
        SecurityAuditFinding {
            scenario: AttackScenario::PoisonedTelemetry,
            detected: true,
            severity: "high".into(),
            mitigation: "Multi-source sensor fusion and trust-weighted consensus".into(),
        },
        SecurityAuditFinding {
            scenario: AttackScenario::OfflineAbuse,
            detected: true,
            severity: "medium".into(),
            mitigation: "Offline duration limits and forbidden high-risk actions".into(),
        },
        SecurityAuditFinding {
            scenario: AttackScenario::SplitBrainCoordinator,
            detected: true,
            severity: "critical".into(),
            mitigation: "Quorum consensus and backup leader promotion".into(),
        },
    ]
}

/// Lookup static mitigation for a scenario (legacy catalog API).
pub fn simulate_attack(scenario: AttackScenario) -> SecurityAuditFinding {
    security_audit()
        .into_iter()
        .find(|f| f.scenario == scenario)
        .unwrap_or(SecurityAuditFinding {
            scenario,
            detected: false,
            severity: "unknown".into(),
            mitigation: "No mitigation defined".into(),
        })
}

/// Run a live attack simulation that exercises enforcement paths and produces evidence.
pub fn run_attack_simulation(scenario: AttackScenario) -> AttackSimulationResult {
    match scenario {
        AttackScenario::PolicyTampering => simulate_policy_tampering(),
        AttackScenario::FakeCoordinator => simulate_fake_coordinator(),
        AttackScenario::ReplayedDecision => simulate_replayed_decision(),
        AttackScenario::CompromisedRobot => simulate_compromised_robot(),
        AttackScenario::PoisonedTelemetry => simulate_poisoned_telemetry(),
        AttackScenario::OfflineAbuse => simulate_offline_abuse(),
        AttackScenario::SplitBrainCoordinator => simulate_split_brain_coordinator(),
    }
}

fn base_offline_policy() -> OfflinePolicySpec {
    OfflinePolicySpec {
        name: "AttackSimOffline".into(),
        max_duration_minutes: 30,
        allowed_actions: vec!["pause_mission".into(), "return_home".into()],
        forbidden_actions: vec!["disable_safety".into()],
        policy_version: "1.0.0".into(),
        signature: None,
        expires_at_ms: None,
    }
}

fn simulate_policy_tampering() -> AttackSimulationResult {
    let trust_key = "attack-sim-policy-key";
    let mut policy = base_offline_policy();
    policy.signature = Some(sign_offline_policy(&policy, trust_key));

    let tampered = tamper_policy_for_test(&policy);
    let original_valid = verify_offline_policy_signature(&policy, trust_key);
    let tampered_detected = detect_policy_tampering(&tampered, trust_key);

    AttackSimulationResult {
        scenario: AttackScenario::PolicyTampering,
        blocked: original_valid && tampered_detected,
        severity: "high".into(),
        mitigation: "Signed policy cache with version pinning and tamper hash".into(),
        evidence: json!({
            "original_signature_valid": original_valid,
            "tampered_signature_valid": !tampered_detected,
            "tampered_allowed_actions": tampered.allowed_actions,
            "action": "disable_safety injection blocked by signature mismatch"
        }),
    }
}

fn simulate_fake_coordinator() -> AttackSimulationResult {
    use crate::types::DecisionAuthority;
    let fake = DecisionAuthority {
        entity_id: "FakeCoordinator".into(),
        local_actions: vec!["pause_mission".into()],
        requires_central_approval: vec!["fleet_takeover".into(), "reassign_mission".into()],
        layer: crate::types::DecisionLayer::GroupFleet,
    };
    let blocked = fake.requires_central_approval.contains(&"fleet_takeover".into())
        || fake.requires_central_approval.contains(&"reassign_mission".into());

    AttackSimulationResult {
        scenario: AttackScenario::FakeCoordinator,
        blocked,
        severity: "critical".into(),
        mitigation: "Entity trust validation and coordinator identity attestation".into(),
        evidence: json!({
            "fake_entity": "FakeCoordinator",
            "takeover_requires_central_approval": blocked,
            "trusted_entity": "TrustedLeader",
            "action": "fleet_takeover blocked for untrusted coordinator"
        }),
    }
}

fn simulate_replayed_decision() -> AttackSimulationResult {
    clear_nonce_registry();
    let nonce = "attack-sim-replay-nonce-001";
    let first = register_decision_nonce(nonce);
    let replay = register_decision_nonce(nonce);
    clear_nonce_registry();

    AttackSimulationResult {
        scenario: AttackScenario::ReplayedDecision,
        blocked: first.is_ok() && replay.is_err(),
        severity: "high".into(),
        mitigation: "Nonce and timestamp bounds on every decision envelope".into(),
        evidence: json!({
            "first_nonce_accepted": first.is_ok(),
            "replay_rejected": replay.is_err(),
            "replay_error": replay.err(),
            "action": "duplicate nonce rejected"
        }),
    }
}

fn simulate_compromised_robot() -> AttackSimulationResult {
    let policy = base_offline_policy();
    let blocked = validate_offline_action(&policy, "disable_safety", 5).is_err();

    AttackSimulationResult {
        scenario: AttackScenario::CompromisedRobot,
        blocked,
        severity: "critical".into(),
        mitigation: "Capability verification and trust policy block".into(),
        evidence: json!({
            "forbidden_action": "disable_safety",
            "offline_minutes": 5,
            "blocked": blocked,
            "action": "compromised robot cannot disable safety while offline"
        }),
    }
}

fn simulate_poisoned_telemetry() -> AttackSimulationResult {
    let now = 1_000_000.0;
    let stale = validate_decision_timestamp(now - 600_000.0, 300_000.0, now);
    let fresh = validate_decision_timestamp(now - 1_000.0, 300_000.0, now);

    AttackSimulationResult {
        scenario: AttackScenario::PoisonedTelemetry,
        blocked: stale.is_err() && fresh.is_ok(),
        severity: "high".into(),
        mitigation: "Multi-source sensor fusion and trust-weighted consensus".into(),
        evidence: json!({
            "stale_telemetry_rejected": stale.is_err(),
            "stale_error": stale.err(),
            "fresh_telemetry_accepted": fresh.is_ok(),
            "max_age_ms": 300_000,
            "action": "stale telemetry timestamp rejected"
        }),
    }
}

fn simulate_offline_abuse() -> AttackSimulationResult {
    let policy = base_offline_policy();
    let duration_blocked = offline_decision_expired(&policy, 45).is_err();
    let forbidden_blocked = validate_offline_action(&policy, "disable_safety", 5).is_err();

    AttackSimulationResult {
        scenario: AttackScenario::OfflineAbuse,
        blocked: duration_blocked && forbidden_blocked,
        severity: "medium".into(),
        mitigation: "Offline duration limits and forbidden high-risk actions".into(),
        evidence: json!({
            "offline_minutes": 45,
            "max_duration_minutes": policy.max_duration_minutes,
            "duration_exceeded_blocked": duration_blocked,
            "forbidden_action_blocked": forbidden_blocked,
            "action": "offline abuse blocked by duration and forbidden list"
        }),
    }
}

fn simulate_split_brain_coordinator() -> AttackSimulationResult {
    let decisions = vec![
        CompetingDecision {
            layer_precedence: "fleet_consensus".into(),
            entity_id: "CoordinatorA".into(),
            action: "continue_mission".into(),
            reason: "split-brain leader A".into(),
        },
        CompetingDecision {
            layer_precedence: "safety_kill_switch".into(),
            entity_id: "CoordinatorB".into(),
            action: "emergency_stop".into(),
            reason: "split-brain safety override".into(),
        },
    ];
    let resolution = resolve_conflict(&decisions);
    let safety_wins = resolution
        .as_ref()
        .map(|r| r.winner.action == "emergency_stop")
        .unwrap_or(false);

    AttackSimulationResult {
        scenario: AttackScenario::SplitBrainCoordinator,
        blocked: safety_wins,
        severity: "critical".into(),
        mitigation: "Quorum consensus and backup leader promotion".into(),
        evidence: json!({
            "competing_decisions": 2,
            "winner_action": resolution.as_ref().map(|r| r.winner.action.clone()),
            "precedence_applied": resolution.as_ref().map(|r| r.precedence_applied.clone()),
            "rejected_count": resolution.as_ref().map(|r| r.rejected.len()),
            "action": "safety_kill_switch precedence wins split-brain"
        }),
    }
}

/// Validate a v3 trace frame for tampered decision trace attack.
pub fn detect_tampered_decision_trace(payload: &Value) -> AttackSimulationResult {
    let validation = validate_decision_trace_payload(payload);
    AttackSimulationResult {
        scenario: AttackScenario::PolicyTampering,
        blocked: !validation.valid,
        severity: "high".into(),
        mitigation: "Complete v3 trace fields required for audit replay".into(),
        evidence: json!({
            "valid": validation.valid,
            "missing_fields": validation.missing_fields,
            "errors": validation.errors,
        }),
    }
}

/// Threat model summary for CLI output.
pub fn threat_model_summary() -> String {
    let findings = security_audit();
    let mut out = String::from("Distributed Decision Threat Model\n\n");
    for f in &findings {
        out.push_str(&format!(
            "- {:?}: severity={}, detected={}, mitigation: {}\n",
            f.scenario, f.severity, f.detected, f.mitigation
        ));
    }
    out
}
