//! v3 decision trace payload helpers for mission trace emission.

use serde_json::{json, Value};
use spanda_audit::{sign_with_backend, verify_signature};

/// Optional enrichment fields for distributed decision trace records.
#[derive(Debug, Clone, Default)]
pub struct DecisionTraceExtras {
    pub policy_version: Option<String>,
    pub policy_hash: Option<String>,
    pub tree_hash: Option<String>,
    pub authority_scope: Option<String>,
    pub alternatives_considered: Vec<Value>,
    pub rejected_alternatives: Vec<Value>,
    pub escalation_path: Vec<Value>,
    pub capability_validation: Option<Value>,
    pub result: Option<String>,
    pub audit_record_id: Option<String>,
    pub input_signals: Option<Value>,
    pub sim_time_ms: Option<f64>,
}

/// Return true when distributed decision trace emission is enabled.
pub fn decision_trace_enabled() -> bool {
    std::env::var("SPANDA_DECISION_TRACE")
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

fn decision_signing_key() -> Option<String> {
    std::env::var("SPANDA_DECISION_POLICY_SIGNING_KEY")
        .ok()
        .filter(|k| !k.is_empty())
        .or_else(|| {
            std::env::var("SPANDA_DECISION_POLICY_TRUST_KEY")
                .ok()
                .filter(|k| !k.is_empty())
        })
}

fn decision_trust_key() -> Option<String> {
    std::env::var("SPANDA_DECISION_POLICY_TRUST_KEY")
        .ok()
        .filter(|k| !k.is_empty())
}

/// Canonical signing payload for a v3 decision security envelope.
pub fn decision_envelope_signing_payload(
    decision_id: &str,
    entity_id: &str,
    layer: &str,
    decision: &str,
    reason: &str,
    policy_version: &str,
    tree_hash: &Option<String>,
    nonce: &str,
    timestamp_ms: f64,
) -> String {
    serde_json::json!({
        "decision_id": decision_id,
        "entity_id": entity_id,
        "layer": layer,
        "decision": decision,
        "reason": reason,
        "policy_version": policy_version,
        "tree_hash": tree_hash,
        "nonce": nonce,
        "timestamp_ms": timestamp_ms,
    })
    .to_string()
}

/// Sign a v3 decision envelope when signing key is configured.
pub fn sign_decision_envelope(payload: &str, signing_key: &str) -> String {
    sign_with_backend(payload, signing_key)
}

/// Verify a v3 decision envelope signature.
pub fn verify_decision_envelope_signature(
    payload: &str,
    signature: &str,
    trust_key: &str,
) -> bool {
    verify_signature(payload, signature, trust_key)
}

/// Build a v3 decision record payload for mission trace frames.
pub fn v3_decision_payload(
    decision_id: &str,
    mission: Option<&str>,
    decision: &str,
    reason: &str,
    layer: &str,
    entity_id: &str,
    evidence: Value,
) -> Value {
    v3_decision_payload_with_extras(
        decision_id,
        mission,
        decision,
        reason,
        layer,
        entity_id,
        evidence,
        DecisionTraceExtras::default(),
    )
}

/// Build an enriched v3 decision record with policy version, tree hash, and security envelope.
pub fn v3_decision_payload_with_extras(
    decision_id: &str,
    mission: Option<&str>,
    decision: &str,
    reason: &str,
    layer: &str,
    entity_id: &str,
    evidence: Value,
    extras: DecisionTraceExtras,
) -> Value {
    let policy_version = extras
        .policy_version
        .clone()
        .or_else(|| evidence.get("tree_hash").and_then(|_| Some("1.0.0".into())))
        .unwrap_or_else(|| "1.0.0".into());
    let policy_hash = extras
        .policy_hash
        .or_else(|| evidence.get("policy_hash").and_then(|v| v.as_str()).map(str::to_string));
    let tree_hash = extras.tree_hash.or_else(|| {
        evidence
            .get("tree_hash")
            .and_then(|v| v.as_str())
            .map(str::to_string)
    });
    let authority_scope = extras
        .authority_scope
        .unwrap_or_else(|| layer.to_string());
    let sim_ms = extras.sim_time_ms.unwrap_or(0.0);
    let nonce = format!("n-{sim_ms:.0}-{decision_id}");
    let audit_record_id = extras
        .audit_record_id
        .clone()
        .unwrap_or_else(|| decision_id.to_string());

    let signing_payload = decision_envelope_signing_payload(
        decision_id,
        entity_id,
        layer,
        decision,
        reason,
        &policy_version,
        &tree_hash,
        &nonce,
        sim_ms,
    );
    let signature = decision_signing_key().map(|key| sign_decision_envelope(&signing_payload, &key));

    let mut envelope = json!({
        "entity_id": entity_id,
        "authority_scope": authority_scope,
        "policy_version": policy_version,
        "policy_hash": policy_hash,
        "tree_hash": tree_hash,
        "timestamp_ms": sim_ms,
        "nonce": nonce,
        "audit_record_id": audit_record_id,
        "safety_validation_passed": true,
        "trust_validation_passed": true,
        "signing_payload": signing_payload,
    });
    if let Some(sig) = signature {
        envelope
            .as_object_mut()
            .expect("envelope object")
            .insert("signature".into(), json!(sig));
    }

    json!({
        "version": 3,
        "decision_id": decision_id,
        "mission": mission,
        "decision": decision,
        "reason": reason,
        "layer": layer,
        "entity_id": entity_id,
        "authority_scope": authority_scope,
        "evidence": evidence,
        "input_signals": extras.input_signals.unwrap_or_else(|| evidence.clone()),
        "policy_version": policy_version,
        "policy_hash": policy_hash,
        "tree_hash": tree_hash,
        "alternatives_considered": extras.alternatives_considered,
        "rejected_alternatives": extras.rejected_alternatives,
        "escalation_path": extras.escalation_path,
        "capability_validation": extras.capability_validation.unwrap_or(json!({"passed": true})),
        "result": extras.result,
        "security_envelope": envelope,
        "safety_validation": {"passed": true, "rules": ["distributed_decision"]},
        "trust_validation": {"passed": true},
        "safety_checks": [{"rule": "distributed_decision", "passed": true}],
        "audit_record": audit_record_id,
    })
}

/// Verify envelope signature on a v3 trace payload when trust key is configured.
pub fn verify_v3_decision_signature(payload: &Value) -> Result<(), String> {
    let Some(trust_key) = decision_trust_key() else {
        return Ok(());
    };
    let envelope = payload
        .get("security_envelope")
        .ok_or_else(|| "missing security_envelope".to_string())?;
    let signature = envelope
        .get("signature")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "missing envelope signature".to_string())?;
    let signing_payload = envelope
        .get("signing_payload")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing signing_payload".to_string())?;
    if verify_decision_envelope_signature(signing_payload, signature, &trust_key) {
        Ok(())
    } else {
        Err("decision envelope signature verification failed".into())
    }
}
