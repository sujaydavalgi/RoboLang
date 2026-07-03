//! Rule enforcement integration tests for distributed decision architecture.

use spanda_decision::{
    clear_nonce_registry, default_safety_boundaries, entity_may_decide_locally,
    evaluate_distributed_decisions, evaluate_tree, extract_decision_authorities,
    extract_decision_trees, extract_offline_policies, high_risk_requires_central_approval,
    local_action_respects_kill_switch, local_action_respects_safety_boundaries,
    offline_decision_expired, policy_hash, reflex_may_act_without_central,
    register_decision_nonce, resolve_split_brain, sign_offline_policy,
    tamper_policy_for_test, tree_hash, untrusted_entity_may_not_takeover,
    validate_decision_trace_payload, validate_offline_policy_trust,
    verify_decision_tree_hash, verify_offline_policy_signature, CompetingDecision,
    DecisionContext, DecisionLayer, NonceRegistry,
};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn parse_sd(source: &str) -> spanda_ast::nodes::Program {
    parse(tokenize(source).expect("tokenize")).expect("parse")
}

#[test]
fn reflex_safety_actions_run_without_central_approval() {
    assert!(reflex_may_act_without_central("emergency_stop", DecisionLayer::Reflex));
    assert!(reflex_may_act_without_central("stop_motor", DecisionLayer::Reflex));
    assert!(reflex_may_act_without_central("kill_switch", DecisionLayer::Reflex));
    assert!(!reflex_may_act_without_central("emergency_stop", DecisionLayer::ControlCenter));
    assert!(!reflex_may_act_without_central("update_firmware", DecisionLayer::Reflex));
}

#[test]
fn local_decisions_cannot_bypass_safety_validation() {
    let err = local_action_respects_safety_boundaries(
        "override_safety_policy",
        DecisionLayer::LocalEntity,
    )
    .expect_err("should block");
    assert!(err.contains("Safety policy override"));
}

#[test]
fn local_decisions_cannot_bypass_kill_switch() {
    let err = local_action_respects_kill_switch("disable_kill_switch").expect_err("blocked");
    assert!(err.contains("kill switch"));
}

#[test]
fn local_decisions_cannot_bypass_trust_policy() {
    let program = parse_sd(
        r#"
        robot Rover001 {
            local_decision_authority [emergency_stop];
            requires_central_approval [accept_unknown_device];
        }
        "#,
    );
    let err = high_risk_requires_central_approval(&program, "Rover001", "accept_unknown_device")
        .expect_err("blocked");
    assert!(err.contains("central approval"));
}

#[test]
fn high_risk_actions_require_central_authorization() {
    let program = parse_sd(
        r#"
        robot Rover001 {
            local_decision_authority [degraded_mode];
            requires_central_approval [update_firmware, override_safety_policy];
        }
        "#,
    );
    let ctx = DecisionContext {
        entity_id: "Rover001".into(),
        action: "update_firmware".into(),
        layer: DecisionLayer::LocalEntity,
        ..Default::default()
    };
    let report = evaluate_distributed_decisions(&program, &ctx);
    assert!(!report.passed);
    assert!(report.messages.iter().any(|m| m.contains("central approval")));
}

#[test]
fn offline_decisions_expire_beyond_max_duration() {
    let program = parse_sd(
        r#"
        offline_policy RoverOffline {
            max_duration = 30 min;
            allowed_actions [return_home];
            forbidden_actions [disable_safety];
        }
        "#,
    );
    let policies = extract_offline_policies(&program);
    let err = offline_decision_expired(&policies[0], 45).expect_err("expired");
    assert!(err.contains("expired"));
}

#[test]
fn cached_policies_must_be_signed_when_required() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var("SPANDA_DECISION_REQUIRE_SIGNED_OFFLINE_POLICY", "1");
    std::env::set_var("SPANDA_DECISION_POLICY_TRUST_KEY", "rule-test-key");
    let program = parse_sd(
        r#"
        offline_policy RoverOffline {
            max_duration = 30 min;
            allowed_actions [pause_mission];
            forbidden_actions [disable_safety];
        }
        "#,
    );
    let policies = extract_offline_policies(&program);
    let err = validate_offline_policy_trust(&policies[0]).expect_err("unsigned");
    assert!(err.contains("signed"));
    std::env::remove_var("SPANDA_DECISION_REQUIRE_SIGNED_OFFLINE_POLICY");
    std::env::remove_var("SPANDA_DECISION_POLICY_TRUST_KEY");
}

#[test]
fn decision_tree_hash_must_match() {
    let program = parse_sd(
        r#"
        decision_tree GPSLoss local {
            when gps.status == Failed {
                enter degraded_mode;
            }
        }
        "#,
    );
    let trees = extract_decision_trees(&program);
    let hash = tree_hash(&trees[0]);
    assert!(verify_decision_tree_hash(&trees[0], &hash).is_ok());
    assert!(verify_decision_tree_hash(&trees[0], "deadbeef00000000").is_err());
}

#[test]
fn replayed_decisions_are_rejected() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempfile::tempdir().expect("tempdir");
    std::env::set_var(
        "SPANDA_DECISION_NONCE_CACHE",
        temp.path().join("nonces.json").display().to_string(),
    );
    clear_nonce_registry();
    register_decision_nonce("rule-test-nonce").expect("first");
    let err = register_decision_nonce("rule-test-nonce").unwrap_err();
    assert!(err.contains("replayed"));
    clear_nonce_registry();
    std::env::remove_var("SPANDA_DECISION_NONCE_CACHE");
}

#[test]
fn untrusted_entities_cannot_issue_takeover_decisions() {
    let program = parse_sd(
        r#"
        robot UntrustedBot {
            local_decision_authority [pause_mission];
            requires_central_approval [fleet_takeover];
        }
        "#,
    );
    let auth = extract_decision_authorities(&program)[0].clone();
    let err = untrusted_entity_may_not_takeover(&auth, "fleet_takeover", &["TrustedLeader"])
        .expect_err("blocked");
    assert!(err.contains("untrusted"));
}

#[test]
fn split_brain_conflicts_resolve_using_precedence() {
    let decisions = vec![
        CompetingDecision {
            layer_precedence: "fleet_consensus".into(),
            entity_id: "A".into(),
            action: "continue_mission".into(),
            reason: "leader A".into(),
        },
        CompetingDecision {
            layer_precedence: "safety_kill_switch".into(),
            entity_id: "B".into(),
            action: "emergency_stop".into(),
            reason: "safety override".into(),
        },
    ];
    let resolution = resolve_split_brain(&decisions).expect("resolved");
    assert_eq!(resolution.winner.action, "emergency_stop");
    assert_eq!(resolution.precedence_applied, "safety_kill_switch");
}

#[test]
fn policy_tampering_changes_hash() {
    let program = parse_sd(
        r#"
        offline_policy RoverOffline {
            max_duration = 30 min;
            allowed_actions [pause_mission];
            forbidden_actions [disable_safety];
            policy_version = "1.0.0";
        }
        "#,
    );
    let policies = extract_offline_policies(&program);
    let h1 = policy_hash(&policies[0]);
    let tampered = tamper_policy_for_test(&policies[0]);
    let h2 = policy_hash(&tampered);
    assert_ne!(h1, h2);
}

#[test]
fn signed_policy_detects_tampering() {
    let program = parse_sd(
        r#"
        offline_policy RoverOffline {
            max_duration = 30 min;
            allowed_actions [pause_mission];
            forbidden_actions [disable_safety];
            policy_version = "1.0.0";
        }
        "#,
    );
    let mut policies = extract_offline_policies(&program);
    let key = "tamper-test-key";
    policies[0].signature = Some(sign_offline_policy(&policies[0], key));
    assert!(verify_offline_policy_signature(&policies[0], key));
    let tampered = tamper_policy_for_test(&policies[0]);
    assert!(!verify_offline_policy_signature(&tampered, key));
}

#[test]
fn decision_trace_payload_requires_proof_fields() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempfile::tempdir().expect("tempdir");
    std::env::set_var(
        "SPANDA_DECISION_NONCE_CACHE",
        temp.path().join("nonces.json").display().to_string(),
    );
    clear_nonce_registry();
    let complete = json!({
        "decision_id": "dd-001",
        "entity_id": "Rover001",
        "layer": "local_entity",
        "policy_version": "1.0.0",
        "decision": "enter_degraded_mode",
        "rejected_alternatives": ["pause_mission"],
        "security_envelope": {
            "entity_id": "Rover001",
            "policy_version": "1.0.0",
            "nonce": "trace-nonce-001",
            "safety_validation_passed": true,
            "trust_validation_passed": true
        }
    });
    let result = validate_decision_trace_payload(&complete);
    assert!(result.valid, "missing: {:?}, errors: {:?}", result.missing_fields, result.errors);

    let incomplete = json!({ "decision_id": "dd-002" });
    let bad = validate_decision_trace_payload(&incomplete);
    assert!(!bad.valid);
    clear_nonce_registry();
    std::env::remove_var("SPANDA_DECISION_NONCE_CACHE");
}

#[test]
fn entity_may_decide_locally_respects_central_approval() {
    let program = parse_sd(
        r#"
        robot Rover001 {
            local_decision_authority [emergency_stop, degraded_mode];
            requires_central_approval [update_firmware];
        }
        "#,
    );
    let auth = extract_decision_authorities(&program)[0].clone();
    assert!(entity_may_decide_locally(&auth, "emergency_stop"));
    assert!(!entity_may_decide_locally(&auth, "update_firmware"));
}

#[test]
fn safety_boundaries_include_kill_switch_and_firmware() {
    let boundaries = default_safety_boundaries();
    assert!(boundaries.iter().any(|b| b.action.contains("kill_switch")));
    assert!(boundaries.iter().any(|b| b.action.contains("firmware")));
}

#[test]
fn decision_tree_evaluates_gps_loss_recovery() {
    let program = parse_sd(
        r#"
        decision_tree GPSLossRecovery local {
            when gps.status == Failed {
                if visual_odometry.available {
                    switch_to visual_odometry;
                    reduce_speed 0.5 m/s;
                    enter degraded_mode;
                }
            }
        }
        "#,
    );
    let trees = extract_decision_trees(&program);
    let mut signals = HashMap::new();
    signals.insert("gps.status == Failed".into(), true);
    signals.insert("visual_odometry.available".into(), true);
    let result = evaluate_tree(&trees[0], &signals).expect("match");
    assert!(result.actions.iter().any(|a| a.contains("degraded")));
}

#[test]
fn nonce_registry_isolates_replays_in_tests() {
    let mut reg = NonceRegistry::default();
    reg.register_nonce("n1").expect("ok");
    assert!(reg.register_nonce("n1").is_err());
    reg.clear();
    reg.register_nonce("n1").expect("ok after clear");
}
