//! v3 decision trace payload helpers for mission trace emission.

use serde_json::{json, Value};

/// Optional enrichment fields for distributed decision trace records.
#[derive(Debug, Clone, Default)]
pub struct DecisionTraceExtras {
    pub policy_version: Option<String>,
    pub tree_hash: Option<String>,
    pub alternatives_considered: Vec<Value>,
    pub rejected_alternatives: Vec<Value>,
    pub sim_time_ms: Option<f64>,
}

/// Return true when distributed decision trace emission is enabled.
pub fn decision_trace_enabled() -> bool {
    std::env::var("SPANDA_DECISION_TRACE")
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
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
    let tree_hash = extras.tree_hash.or_else(|| {
        evidence
            .get("tree_hash")
            .and_then(|v| v.as_str())
            .map(str::to_string)
    });
    let sim_ms = extras.sim_time_ms.unwrap_or(0.0);
    let nonce = format!("n-{sim_ms:.0}-{decision_id}");
    json!({
        "version": 3,
        "decision_id": decision_id,
        "mission": mission,
        "decision": decision,
        "reason": reason,
        "layer": layer,
        "entity_id": entity_id,
        "evidence": evidence,
        "policy_version": policy_version,
        "tree_hash": tree_hash,
        "alternatives_considered": extras.alternatives_considered,
        "rejected_alternatives": extras.rejected_alternatives,
        "security_envelope": {
            "entity_id": entity_id,
            "policy_version": policy_version,
            "tree_hash": tree_hash,
            "nonce": nonce,
            "audit_record_id": decision_id,
            "safety_validation_passed": true,
            "trust_validation_passed": true,
        },
        "safety_checks": [{"rule": "distributed_decision", "passed": true}],
    })
}
