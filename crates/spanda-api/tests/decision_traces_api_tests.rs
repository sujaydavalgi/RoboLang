//! REST API tests for distributed decision trace listing.

use spanda_api::decision_ops::{list_decision_policy_cache, list_decision_traces};
use spanda_api::handlers::handle_request;
use spanda_api::sdk_ops::program_simulation;
use spanda_api::state::ControlCenterState;
use spanda_decision::DecisionBackedRuntime;
use spanda_deploy_http::HttpRequest;
use spanda_interpreter::{run_program, RunOptions};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use std::path::PathBuf;
use std::sync::Arc;

fn showcase_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/showcase/distributed_decisions")
}

#[test]
fn list_decision_traces_returns_v3_frames_after_sim() {
    let root = showcase_root();
    let source_path = root.join("smoke.sd");
    let trace_path = root.join("smoke.trace");
    let _ = std::fs::remove_file(&trace_path);
    let source = std::fs::read_to_string(&source_path).expect("read smoke.sd");
    let program = parse(tokenize(&source).unwrap()).expect("parse smoke.sd");
    std::env::set_var("SPANDA_DECISION_TRACE", "1");
    let result = run_program(
        &program,
        RunOptions {
            inject_health_faults: true,
            decision_trace: true,
            record_trace: true,
            trace_output: Some(trace_path.to_string_lossy().into_owned()),
            decision_runtime: Some(Arc::new(DecisionBackedRuntime)),
            max_loop_iterations: 3,
            ..Default::default()
        },
    )
    .expect("sim");
    assert!(result.mission_trace.is_some() || trace_path.exists());
    std::env::remove_var("SPANDA_DECISION_TRACE");

    let mut state = ControlCenterState::new().with_config_path(root.clone());
    state.program_path = Some(source_path.clone());
    let response = list_decision_traces(&state, "");
    assert_eq!(response.status, 200, "{}", response.body);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert!(json["count"].as_u64().unwrap_or(0) > 0);
    let frames = json["frames"].as_array().expect("frames array");
    assert!(frames.iter().any(|f| f["payload"]["version"].as_u64() == Some(3)));

    let (http, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "GET".into(),
            path: "/v1/decisions/traces".into(),
            body: String::new(),
            authorization: None,
        },
        "",
    );
    assert_eq!(http.status, 200);
    assert!(http.body.contains("\"frames\""));
    let _ = std::fs::remove_file(&trace_path);
}

#[test]
fn list_decision_policy_cache_returns_json() {
    let response = list_decision_policy_cache(&ControlCenterState::new(), "");
    assert_eq!(response.status, 200, "{}", response.body);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert_eq!(json["api_version"], "v1");
    assert!(json["policies"].is_object());
}

#[test]
fn program_simulation_emits_decision_trace_for_showcase() {
    let root = showcase_root().join("obstacle_reflex_stop");
    let source_path = root.join("main.sd");
    let trace_path = root.join("main.trace");
    let _ = std::fs::remove_file(&trace_path);
    let mut state = ControlCenterState::new().with_config_path(root.clone());
    state.program_path = Some(source_path.clone());
    let body = r#"{"execute":true,"decision_trace":true,"record_trace":true,"inject_health_faults":true}"#;
    let response = program_simulation(&state, body);
    assert_eq!(response.status, 200, "{}", response.body);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert_eq!(json["simulation"]["decision_trace"], true);
    assert!(json["simulation"]["has_trace"].as_bool().unwrap_or(false));
    let _ = std::fs::remove_file(&trace_path);
}
