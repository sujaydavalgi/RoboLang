//! Stable promotion gap-fix integration tests.

use spanda_decision::{
    approve_escalation_persisted, clear_persisted_nonce_registry, escalation_is_approved,
    layer_str_precedence_key, register_pending_escalation, register_persisted_nonce,
    resolve_conflict, sign_decision_tree, verify_decision_tree_signature, CompetingDecision,
    DecisionTreeSpec,
};
use spanda_runtime::decision_trace::{
    decision_envelope_signing_payload, sign_decision_envelope, v3_decision_payload_with_extras,
    verify_v3_decision_signature, DecisionTraceExtras,
};
use std::sync::Mutex;

static GAP_ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_temp_stores<F: FnOnce()>(f: F) {
    let _guard = GAP_ENV_LOCK.lock().unwrap();
    let temp = tempfile::tempdir().expect("tempdir");
    std::env::set_var(
        "SPANDA_DECISION_NONCE_CACHE",
        temp.path().join("nonces.json").display().to_string(),
    );
    std::env::set_var(
        "SPANDA_DECISION_ESCALATION_STORE",
        temp.path().join("escalations.json").display().to_string(),
    );
    clear_persisted_nonce_registry();
    f();
    std::env::remove_var("SPANDA_DECISION_NONCE_CACHE");
    std::env::remove_var("SPANDA_DECISION_ESCALATION_STORE");
}

#[test]
fn persisted_nonce_registry_rejects_replays() {
    with_temp_stores(|| {
        register_persisted_nonce("stable-gap-nonce-1").expect("first");
        let err = register_persisted_nonce("stable-gap-nonce-1").unwrap_err();
        assert!(err.contains("replayed"));
    });
}

#[test]
fn escalation_store_persists_approval() {
    with_temp_stores(|| {
        register_pending_escalation("esc-test-1", "Rover001", "update_firmware", "high risk")
            .expect("register");
        assert!(!escalation_is_approved("esc-test-1"));
        approve_escalation_persisted("esc-test-1", "operator", Some("Rover001")).expect("approve");
        assert!(escalation_is_approved("esc-test-1"));
    });
}

#[test]
fn decision_tree_ed25519_signature() {
    let spec = DecisionTreeSpec {
        name: "GPSLoss".into(),
        scope: "local".into(),
        layer: spanda_decision::DecisionLayer::LocalEntity,
        version: "1.0.0".into(),
        branches: vec![],
        signature: None,
    };
    let key = "tree-sign-test-key";
    let sig = sign_decision_tree(&spec, key);
    let mut signed = spec.clone();
    signed.signature = Some(sig);
    assert!(verify_decision_tree_signature(&signed, key));
}

#[test]
fn resolve_conflict_wired_precedence() {
    let decisions = vec![
        CompetingDecision {
            layer_precedence: "local_optimization".into(),
            entity_id: "Rover".into(),
            action: "continue_mission".into(),
            reason: "local".into(),
        },
        CompetingDecision {
            layer_precedence: "safety_kill_switch".into(),
            entity_id: "Rover".into(),
            action: "emergency_stop".into(),
            reason: "reflex".into(),
        },
    ];
    let resolution = resolve_conflict(&decisions).expect("resolved");
    assert_eq!(resolution.winner.action, "emergency_stop");
}

#[test]
fn v3_envelope_signature_roundtrip() {
    let key = "envelope-sign-test-key";
    std::env::set_var("SPANDA_DECISION_POLICY_SIGNING_KEY", key);
    std::env::set_var("SPANDA_DECISION_POLICY_TRUST_KEY", key);
    let payload = v3_decision_payload_with_extras(
        "d-test-1",
        Some("mission"),
        "enter_degraded_mode",
        "gps loss",
        "local_entity",
        "Rover001",
        serde_json::json!({}),
        DecisionTraceExtras {
            sim_time_ms: Some(100.0),
            ..Default::default()
        },
    );
    verify_v3_decision_signature(&payload).expect("valid signature");
    std::env::remove_var("SPANDA_DECISION_POLICY_SIGNING_KEY");
    std::env::remove_var("SPANDA_DECISION_POLICY_TRUST_KEY");
}

#[test]
fn layer_str_precedence_maps_reflex_and_fleet() {
    assert_eq!(layer_str_precedence_key("reflex"), "local_immediate_safety");
    assert_eq!(layer_str_precedence_key("group_fleet"), "fleet_coordination");
    assert_eq!(layer_str_precedence_key("safety_kill_switch"), "safety_kill_switch");
}

#[test]
fn envelope_signing_payload_is_stable() {
    let payload = decision_envelope_signing_payload(
        "d-1",
        "Rover",
        "local_entity",
        "pause",
        "gps",
        "1.0.0",
        &Some("abc".into()),
        "n-1",
        100.0,
    );
    let sig = sign_decision_envelope(&payload, "test-key");
    assert!(!sig.is_empty());
}
