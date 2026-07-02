//! Recovery Orchestrator REST API contract tests.
use spanda_api::recovery_ops::{
    recovery_explain, recovery_history, recovery_playbooks, recovery_plan, recovery_policies,
    recovery_simulate, RecoveryRequest,
};
use spanda_api::state::ControlCenterState;
use std::path::PathBuf;

fn showcase_state() -> ControlCenterState {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let mut state = ControlCenterState::new().with_config_path(root.join("packages/basic_project"));
    state.program_path = Some(root.join("showcase/self_healing/rover.sd"));
    state.reload_config().expect("reload config");
    state
}

#[test]
fn recovery_playbooks_lists_defaults() {
    let state = ControlCenterState::new();
    let resp = recovery_playbooks(&state);
    assert_eq!(resp.status, 200);
    let json: serde_json::Value = serde_json::from_str(&resp.body).unwrap();
    let playbooks = json["playbooks"].as_array().expect("playbooks array");
    assert!(playbooks.iter().any(|p| p["name"] == "battery_low"));
}

#[test]
fn recovery_history_returns_envelope() {
    let state = ControlCenterState::new();
    let resp = recovery_history(&state);
    assert_eq!(resp.status, 200);
    let json: serde_json::Value = serde_json::from_str(&resp.body).unwrap();
    assert!(json["history"].is_array());
}

#[test]
fn recovery_plan_with_rover_program() {
    let state = showcase_state();
    let body = serde_json::to_string(&RecoveryRequest {
        failure: Some("gps_loss".into()),
        entity_id: Some("robot-1".into()),
        ..Default::default()
    })
    .unwrap();
    let resp = recovery_plan(&state, &body);
    assert_eq!(resp.status, 200);
    let json: serde_json::Value = serde_json::from_str(&resp.body).unwrap();
    assert!(json["report"]["plans"].is_array());
}

#[test]
fn recovery_simulate_with_failure() {
    let state = showcase_state();
    let body = r#"{"failure":"sensor_failure"}"#;
    let resp = recovery_simulate(&state, &body);
    assert_eq!(resp.status, 200);
    let json: serde_json::Value = serde_json::from_str(&resp.body).unwrap();
    assert!(json["report"]["simulation_mode"].is_string());
}

#[test]
fn recovery_explain_returns_decision_envelope() {
    let state = showcase_state();
    let body = r#"{"failure":"gps_loss"}"#;
    let resp = recovery_explain(&state, &body);
    assert_eq!(resp.status, 200);
    let json: serde_json::Value = serde_json::from_str(&resp.body).unwrap();
    assert_eq!(json["failure"], "gps_loss");
}

#[test]
fn recovery_policies_without_config_returns_empty() {
    let state = ControlCenterState::new();
    let resp = recovery_policies(&state);
    assert_eq!(resp.status, 200);
    let json: serde_json::Value = serde_json::from_str(&resp.body).unwrap();
    assert!(json["policies"].is_array());
}
