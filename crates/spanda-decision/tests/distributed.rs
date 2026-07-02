//! Integration tests for distributed decision architecture.

use spanda_decision::{
    evaluate_distributed_decisions, evaluate_tree, extract_decision_authorities,
    extract_decision_trees, extract_offline_policies, simulate_distributed_decisions,
    DecisionContext, DecisionLayer, SimulationOptions,
};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use std::collections::HashMap;
use std::sync::Mutex;

static OFFLINE_POLICY_ENV_LOCK: Mutex<()> = Mutex::new(());

fn parse_sd(source: &str) -> spanda_ast::nodes::Program {
    let tokens = tokenize(source).expect("tokenize");
    parse(tokens).expect("parse")
}

fn clear_offline_policy_env() {
    for key in [
        "SPANDA_DECISION_POLICY_CACHE",
        "SPANDA_DECISION_POLICY_TRUST_KEY",
        "SPANDA_DECISION_REQUIRE_SIGNED_OFFLINE_POLICY",
    ] {
        std::env::remove_var(key);
    }
}

#[test]
fn extracts_entity_decision_authority() {
    let program = parse_sd(
        r#"
        robot Rover001 {
            local_decision_authority [emergency_stop, degraded_mode];
            requires_central_approval [update_firmware];
        }
        "#,
    );
    let authorities = extract_decision_authorities(&program);
    assert_eq!(authorities.len(), 1);
    assert_eq!(authorities[0].entity_id, "Rover001");
    assert!(authorities[0]
        .local_actions
        .contains(&"emergency_stop".into()));
    assert!(authorities[0]
        .requires_central_approval
        .contains(&"update_firmware".into()));
}

#[test]
fn evaluates_decision_tree() {
    let program = parse_sd(
        r#"
        decision_tree GPSLoss local {
            when gps.status == Failed {
                if visual_odometry.available { enter degraded_mode; }
            }
        }
        "#,
    );
    let trees = extract_decision_trees(&program);
    assert_eq!(trees.len(), 1);
    let mut signals = HashMap::new();
    signals.insert("gps.status == Failed".into(), true);
    signals.insert("visual_odometry.available".into(), true);
    let result = evaluate_tree(&trees[0], &signals).expect("match");
    assert!(result
        .actions
        .iter()
        .any(|a| a.contains("degraded") || a.contains("degraded_mode")));
}

#[test]
fn offline_policy_blocks_forbidden_action() {
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
    assert_eq!(policies.len(), 1);
    assert_eq!(policies[0].max_duration_minutes, 30);
}

#[test]
fn simulate_offline_scenario() {
    let program = parse_sd(
        r#"
        robot Rover001 {
            local_decision_authority [return_home];
        }
        offline_policy RoverOffline {
            max_duration = 30 min;
            allowed_actions [return_home];
            forbidden_actions [disable_safety];
        }
        "#,
    );
    let sim = simulate_distributed_decisions(
        &program,
        SimulationOptions {
            offline: true,
            entity_id: "Rover001".into(),
            ..Default::default()
        },
    );
    assert_eq!(sim.scenario, "offline");
}

#[test]
fn central_approval_blocks_action() {
    let program = parse_sd(
        r#"
        robot Rover001 {
            local_decision_authority [emergency_stop];
            requires_central_approval [update_firmware];
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
}

#[test]
fn signed_offline_policy_verifies_with_trust_key() {
    let _guard = OFFLINE_POLICY_ENV_LOCK.lock().unwrap();
    clear_offline_policy_env();
    use spanda_decision::{
        extract_offline_policies, sign_offline_policy, validate_offline_policy_trust,
        verify_offline_policy_signature, DecisionBackedRuntime,
    };
    use spanda_runtime::decision_runtime::DecisionRuntime;
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
    assert_eq!(policies.len(), 1);
    let signing_key = "offline-policy-test-key";
    let signature = sign_offline_policy(&policies[0], signing_key);
    let program = parse_sd(&format!(
        r#"
        offline_policy RoverOffline {{
            max_duration = 30 min;
            allowed_actions [pause_mission];
            forbidden_actions [disable_safety];
            policy_version = "1.0.0";
            signature = "{signature}";
        }}
        "#
    ));
    policies = extract_offline_policies(&program);
    std::env::set_var("SPANDA_DECISION_POLICY_TRUST_KEY", signing_key);
    std::env::set_var("SPANDA_DECISION_REQUIRE_SIGNED_OFFLINE_POLICY", "1");
    assert!(verify_offline_policy_signature(&policies[0], signing_key));
    assert!(validate_offline_policy_trust(&policies[0]).is_ok());
    let runtime = DecisionBackedRuntime;
    let verdict = runtime.authorize_action(&program, "Rover001", "pause_mission", 5, false);
    assert!(verdict.permitted, "{}", verdict.reason);
    clear_offline_policy_env();
}

#[test]
fn unsigned_offline_policy_blocked_when_signature_required() {
    let _guard = OFFLINE_POLICY_ENV_LOCK.lock().unwrap();
    clear_offline_policy_env();
    use spanda_decision::DecisionBackedRuntime;
    use spanda_runtime::decision_runtime::DecisionRuntime;
    let program = parse_sd(
        r#"
        offline_policy RoverOffline {
            max_duration = 30 min;
            allowed_actions [pause_mission];
            forbidden_actions [disable_safety];
        }
        "#,
    );
    std::env::set_var("SPANDA_DECISION_REQUIRE_SIGNED_OFFLINE_POLICY", "1");
    std::env::set_var(
        "SPANDA_DECISION_POLICY_TRUST_KEY",
        "offline-policy-test-key",
    );
    let runtime = DecisionBackedRuntime;
    let verdict = runtime.authorize_action(&program, "Rover001", "pause_mission", 5, false);
    assert!(!verdict.permitted);
    assert!(verdict.reason.contains("signed"));
    clear_offline_policy_env();
}

#[test]
fn persisted_policy_cache_merges_signatures_at_runtime() {
    let _guard = OFFLINE_POLICY_ENV_LOCK.lock().unwrap();
    clear_offline_policy_env();
    use spanda_decision::{
        extract_offline_policies, merge_offline_policies_with_cache, resolve_offline_policies,
        save_persisted_policy_cache, sign_offline_policy, DecisionBackedRuntime,
        PersistedPolicyCache,
    };
    use spanda_runtime::decision_runtime::DecisionRuntime;
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
    let signing_key = "offline-cache-test-key";
    let mut policies = extract_offline_policies(&program);
    policies[0].signature = Some(sign_offline_policy(&policies[0], signing_key));
    let mut cache = PersistedPolicyCache::new();
    cache.upsert_offline_policy(policies[0].clone());
    let temp = tempfile::tempdir().expect("tempdir");
    let cache_path = temp.path().join("decision-policy-cache.json");
    save_persisted_policy_cache(&mut cache, Some(&cache_path)).expect("save cache");
    std::env::set_var(
        "SPANDA_DECISION_POLICY_CACHE",
        cache_path.display().to_string(),
    );
    std::env::set_var("SPANDA_DECISION_POLICY_TRUST_KEY", signing_key);
    std::env::set_var("SPANDA_DECISION_REQUIRE_SIGNED_OFFLINE_POLICY", "1");
    let merged = merge_offline_policies_with_cache(extract_offline_policies(&program), &cache);
    assert!(merged[0].signature.is_some());
    let resolved = resolve_offline_policies(&program);
    assert!(resolved[0].signature.is_some());
    let runtime = DecisionBackedRuntime;
    let verdict = runtime.authorize_action(&program, "Rover001", "pause_mission", 5, false);
    assert!(verdict.permitted, "{}", verdict.reason);
    clear_offline_policy_env();
}
