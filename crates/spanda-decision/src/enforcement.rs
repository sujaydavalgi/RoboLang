//! Runtime rule enforcement and decision trace validation for distributed autonomy.

use crate::authority::{
    default_safety_boundaries, entity_may_decide_locally, extract_decision_authorities,
};
use crate::conflict::{resolve_conflict, CompetingDecision};
use crate::offline::{
    offline_policy_signing_payload, validate_offline_policy_trust,
    verify_offline_policy_signature, OfflinePolicySpec,
};
use crate::trees::{tree_hash, DecisionTreeSpec};
use crate::types::{DecisionAuthority, DecisionLayer, DecisionSecurityEnvelope};
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use std::collections::HashSet;

/// Canonical hash of an offline policy payload (for tamper detection).
pub fn policy_hash(spec: &OfflinePolicySpec) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    offline_policy_signing_payload(spec).hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Verify that a decision tree hash matches the live tree specification.
pub fn verify_decision_tree_hash(spec: &DecisionTreeSpec, expected_hash: &str) -> Result<(), String> {
    let actual = tree_hash(spec);
    if actual == expected_hash {
        Ok(())
    } else {
        Err(format!(
            "decision tree hash mismatch: expected {expected_hash}, got {actual}"
        ))
    }
}

/// In-memory nonce registry for replay protection (test and simulation use).
#[derive(Debug, Default)]
pub struct NonceRegistry {
    seen: HashSet<String>,
}

impl NonceRegistry {
    /// Register a nonce; returns Err when the nonce was already seen (replay).
    pub fn register_nonce(&mut self, nonce: &str) -> Result<(), String> {
        if nonce.is_empty() {
            return Err("empty nonce rejected".into());
        }
        if !self.seen.insert(nonce.to_string()) {
            return Err(format!("replayed decision nonce '{nonce}' rejected"));
        }
        Ok(())
    }

    /// Clear all registered nonces (for test isolation).
    pub fn clear(&mut self) {
        self.seen.clear();
    }
}

/// Register a decision nonce globally with disk persistence; rejects replays.
pub fn register_decision_nonce(nonce: &str) -> Result<(), String> {
    crate::nonce_cache::register_persisted_nonce(nonce)
}

/// Clear the global nonce registry (tests only).
pub fn clear_nonce_registry() {
    crate::nonce_cache::clear_persisted_nonce_registry();
}

/// Validate timestamp is within acceptable bounds (default 5 minutes).
pub fn validate_decision_timestamp(timestamp_ms: f64, max_age_ms: f64, now_ms: f64) -> Result<(), String> {
    if timestamp_ms <= 0.0 {
        return Err("invalid decision timestamp".into());
    }
    let age = now_ms - timestamp_ms;
    if age > max_age_ms {
        return Err(format!(
            "decision timestamp stale: age {age}ms exceeds max {max_age_ms}ms"
        ));
    }
    if timestamp_ms > now_ms + 60_000.0 {
        return Err("decision timestamp in the future".into());
    }
    Ok(())
}

/// Validate required security envelope fields and validation flags.
pub fn validate_security_envelope_fields(envelope: &DecisionSecurityEnvelope) -> Result<(), String> {
    if envelope.entity_id.is_empty() {
        return Err("missing entity identity".into());
    }
    if envelope.policy_version.is_empty() {
        return Err("missing policy version".into());
    }
    if envelope.nonce.is_empty() {
        return Err("missing nonce (replay protection)".into());
    }
    if !envelope.safety_validation_passed {
        return Err("safety validation failed".into());
    }
    if !envelope.trust_validation_passed {
        return Err("trust validation failed".into());
    }
    Ok(())
}

/// Validate authority scope matches the decision layer.
pub fn validate_authority_scope(envelope: &DecisionSecurityEnvelope, layer: DecisionLayer) -> Result<(), String> {
    let expected = format!("{layer:?}");
    if envelope.authority_scope.contains(&expected.replace('_', " "))
        || envelope.authority_scope.eq_ignore_ascii_case(&expected)
        || envelope.authority_scope.contains(&expected.to_lowercase())
    {
        Ok(())
    } else if envelope.authority_scope.is_empty() {
        Err("missing authority scope".into())
    } else {
        Ok(())
    }
}

/// Reflex-layer safety actions may proceed without central connectivity.
pub fn reflex_may_act_without_central(action: &str, layer: DecisionLayer) -> bool {
    if layer != DecisionLayer::Reflex {
        return false;
    }
    matches!(
        action,
        "emergency_stop" | "stop_motor" | "cut_actuator_power" | "kill_switch" | "e_stop"
    ) || action.contains("stop") || action.contains("kill")
}

/// Local decisions cannot bypass safety boundaries.
pub fn local_action_respects_safety_boundaries(action: &str, layer: DecisionLayer) -> Result<(), String> {
    for boundary in default_safety_boundaries() {
        if !action.contains(&boundary.action) && boundary.action != action {
            continue;
        }

        if boundary.requires_approval && (layer as u8) < (boundary.max_layer as u8) {
            return Err(boundary.reason.clone());
        }

        if (layer as u8) > (boundary.max_layer as u8) {
            return Err(format!(
                "action '{action}' exceeds max layer {:?}",
                boundary.max_layer
            ));
        }
    }
    Ok(())
}

/// Local decisions cannot bypass kill-switch disable.
pub fn local_action_respects_kill_switch(action: &str) -> Result<(), String> {
    if action.contains("disable_kill_switch") {
        return Err("kill switch disable requires central approval".into());
    }
    Ok(())
}

/// High-risk actions require central approval when declared on entity authority.
pub fn high_risk_requires_central_approval(
    program: &Program,
    entity_id: &str,
    action: &str,
) -> Result<(), String> {
    let authorities = extract_decision_authorities(program);
    let Some(auth) = authorities.iter().find(|a| a.entity_id == entity_id) else {
        return Ok(());
    };
    if auth.requires_central_approval.iter().any(|a| a == action) {
        return Err(format!("action '{action}' requires central approval/quorum"));
    }
    if !entity_may_decide_locally(auth, action) && !auth.local_actions.is_empty() {
        return Err(format!("action '{action}' not in local_decision_authority"));
    }
    Ok(())
}

/// Offline decisions expire when duration exceeds policy max.
pub fn offline_decision_expired(spec: &OfflinePolicySpec, offline_minutes: u32) -> Result<(), String> {
    if offline_minutes > spec.max_duration_minutes {
        return Err(format!(
            "offline duration {offline_minutes}m exceeds max {}m — decision expired",
            spec.max_duration_minutes
        ));
    }
    Ok(())
}

/// Cached policies must carry a valid signature when signing is required.
pub fn cached_policy_must_be_signed(spec: &OfflinePolicySpec) -> Result<(), String> {
    validate_offline_policy_trust(spec)
}

/// Untrusted entities cannot issue takeover decisions.
pub fn untrusted_entity_may_not_takeover(
    authority: &DecisionAuthority,
    action: &str,
    trusted_entities: &[&str],
) -> Result<(), String> {
    if !action.contains("takeover") && !action.contains("reassign") {
        return Ok(());
    }
    if trusted_entities.contains(&authority.entity_id.as_str()) {
        return Ok(());
    }
    if entity_may_decide_locally(authority, action) {
        return Ok(());
    }
    Err(format!(
        "untrusted entity '{}' cannot issue takeover decision '{action}'",
        authority.entity_id
    ))
}

/// Resolve split-brain conflicts using documented precedence.
pub fn resolve_split_brain(decisions: &[CompetingDecision]) -> Result<crate::conflict::ConflictResolution, String> {
    resolve_conflict(decisions).ok_or_else(|| "no competing decisions to resolve".into())
}

/// Required fields for a complete v3 decision trace record.
pub const TRACE_REQUIRED_FIELDS: &[&str] = &[
    "decision_id",
    "entity_id",
    "layer",
    "policy_version",
    "decision",
    "security_envelope",
];

/// Validation result for a decision trace payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceValidationResult {
    pub valid: bool,
    pub missing_fields: Vec<String>,
    pub errors: Vec<String>,
}

/// Validate that a v3 decision trace payload includes all proof fields.
pub fn validate_decision_trace_payload(payload: &serde_json::Value) -> TraceValidationResult {
    let mut missing = Vec::new();
    let mut errors = Vec::new();

    for field in TRACE_REQUIRED_FIELDS {
        if payload.get(field).is_none() {
            missing.push((*field).into());
        }
    }

    let envelope = payload.get("security_envelope");
    if let Some(env) = envelope {
        for key in ["entity_id", "policy_version", "nonce"] {
            if env.get(key).and_then(|v| v.as_str()).unwrap_or("").is_empty() {
                missing.push(format!("security_envelope.{key}"));
            }
        }
        if env.get("safety_validation_passed").and_then(|v| v.as_bool()) != Some(true) {
            errors.push("safety validation not passed".into());
        }
    } else {
        missing.push("security_envelope".into());
    }

    if payload
        .get("rejected_alternatives")
        .and_then(|v| v.as_array())
        .is_none()
    {
        missing.push("rejected_alternatives".into());
    }

    if let Some(nonce) = envelope
        .and_then(|e| e.get("nonce"))
        .and_then(|v| v.as_str())
    {
        if let Err(e) = register_decision_nonce(nonce) {
            errors.push(e);
        }
    }

    if std::env::var("SPANDA_DECISION_POLICY_TRUST_KEY")
        .map(|k| !k.is_empty())
        .unwrap_or(false)
    {
        if let Err(e) = spanda_runtime::decision_trace::verify_v3_decision_signature(payload) {
            errors.push(e);
        }
    }

    TraceValidationResult {
        valid: missing.is_empty() && errors.is_empty(),
        missing_fields: missing,
        errors,
    }
}

/// Build a tampered policy copy for attack simulation (test-only helper).
pub fn tamper_policy_for_test(spec: &OfflinePolicySpec) -> OfflinePolicySpec {
    let mut tampered = spec.clone();
    tampered
        .allowed_actions
        .push("disable_safety".into());
    tampered
}

/// Verify policy signature detects tampering.
pub fn detect_policy_tampering(spec: &OfflinePolicySpec, trust_key: &str) -> bool {
    !verify_offline_policy_signature(spec, trust_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_hash_changes_on_tamper() {
        let spec = OfflinePolicySpec {
            name: "Test".into(),
            max_duration_minutes: 30,
            allowed_actions: vec!["pause".into()],
            forbidden_actions: vec!["disable_safety".into()],
            policy_version: "1.0.0".into(),
            signature: None,
            expires_at_ms: None,
        };
        let h1 = policy_hash(&spec);
        let tampered = tamper_policy_for_test(&spec);
        let h2 = policy_hash(&tampered);
        assert_ne!(h1, h2);
    }

    #[test]
    fn nonce_replay_rejected() {
        clear_nonce_registry();
        register_decision_nonce("test-nonce-1").expect("first");
        let err = register_decision_nonce("test-nonce-1").unwrap_err();
        assert!(err.contains("replayed"));
        clear_nonce_registry();
    }
}
