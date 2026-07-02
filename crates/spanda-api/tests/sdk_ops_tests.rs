//! SDK program-operation contract tests (no live server required for parsing).
use spanda_api::sdk_ops::{program_replay, program_simulation, ProgramRequest};
use spanda_api::state::ControlCenterState;
use std::path::PathBuf;

#[test]
fn program_request_parses_defaults() {
    let req: ProgramRequest = serde_json::from_str("{}").unwrap();
    assert!(!req.include_runtime);
    assert!(!req.execute);
    assert!(!req.deterministic);
    assert!(!req.playback);
    assert!(!req.decision_trace);
    assert!(!req.record_trace);
}

#[test]
fn program_request_parses_execution_flags() {
    let req: ProgramRequest =
        serde_json::from_str(r#"{"execute":true,"deterministic":true,"playback":true}"#).unwrap();
    assert!(req.execute);
    assert!(req.deterministic);
    assert!(req.playback);
}

#[test]
fn program_simulation_dry_run_hello_world() {
    let examples = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let state = ControlCenterState::new().with_config_path(examples.clone());
    let body = r#"{"file":"hello_world.sd"}"#;
    let resp = program_simulation(&state, body);
    assert_eq!(resp.status, 200);
    let json: serde_json::Value = serde_json::from_str(&resp.body).unwrap();
    assert_eq!(json["simulation"]["dry_run"], true);
    assert_eq!(json["simulation"]["status"], "planned");
}

#[test]
fn program_simulation_execute_hello_world() {
    let examples = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let state = ControlCenterState::new().with_config_path(examples);
    let body = r#"{"file":"hello_world.sd","execute":true}"#;
    let resp = program_simulation(&state, body);
    assert_eq!(resp.status, 200);
    let json: serde_json::Value = serde_json::from_str(&resp.body).unwrap();
    assert_eq!(json["simulation"]["dry_run"], false);
    assert_eq!(json["simulation"]["status"], "completed");
}

#[test]
fn program_replay_inspect_mission_trace() {
    let showcase = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/showcase/root_cause_analysis");
    let state = ControlCenterState::new().with_config_path(showcase.clone());
    let body = r#"{"file":"mission.trace"}"#;
    let resp = program_replay(&state, body);
    assert_eq!(resp.status, 200);
    let json: serde_json::Value = serde_json::from_str(&resp.body).unwrap();
    assert_eq!(json["replay"]["loaded"], true);
    assert!(json["replay"]["frame_count"].as_u64().unwrap_or(0) > 0);
}

#[test]
fn program_replay_playback_mission_trace() {
    let showcase = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/showcase/root_cause_analysis");
    let state = ControlCenterState::new().with_config_path(showcase);
    let body = r#"{"file":"mission.trace","playback":true}"#;
    let resp = program_replay(&state, body);
    assert_eq!(resp.status, 200);
    let json: serde_json::Value = serde_json::from_str(&resp.body).unwrap();
    assert_eq!(json["replay"]["mode"], "playback");
    assert!(json["replay"]["frames_applied"].as_u64().is_some());
}
