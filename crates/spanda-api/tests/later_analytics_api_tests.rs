//! REST API tests for LATER differentiation analytics endpoints.

use spanda_api::handlers::handle_request;
use spanda_api::state::ControlCenterState;
use spanda_deploy_http::HttpRequest;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("repo root")
}

fn handle_get(state: &mut ControlCenterState, path: &str) -> spanda_deploy_http::HttpResponse {
    let (response, _) = handle_request(
        state,
        &HttpRequest {
            method: "GET".into(),
            path: path.into(),
            body: String::new(),
            authorization: None,
        },
        "",
    );
    response
}

#[test]
fn later_analytics_endpoints_return_reports() {
    let root = repo_root();
    let cases = [
        (
            root.join("examples/showcase/mission_twin/patrol.sd"),
            "/v1/analytics/mission-twin",
        ),
        (
            root.join("examples/showcase/certify/deployment_bundle/rover.sd"),
            "/v1/analytics/certification-pack",
        ),
        (
            root.join("examples/showcase/human_robot/approval_escalation.sd"),
            "/v1/analytics/human-teaming",
        ),
        (
            root.join("examples/showcase/policy/warehouse.sd"),
            "/v1/analytics/governance",
        ),
    ];
    for (program, path) in cases {
        assert!(program.exists(), "missing {}", program.display());
        let mut state = ControlCenterState::new();
        state.program_path = Some(program);
        let response = handle_get(&mut state, path);
        assert_eq!(response.status, 200, "{path}: {}", response.body);
        let json: serde_json::Value = serde_json::from_str(&response.body).expect(path);
        assert_eq!(json["version"], "v1");
    }
}

#[test]
fn later_time_travel_endpoint_inspects_program_trace() {
    let root = repo_root();
    let program = root.join("examples/showcase/differentiation/decision_trail/main.sd");
    let trace = root.join("examples/showcase/differentiation/decision_trail/main.trace");
    assert!(program.exists());
    assert!(trace.exists());
    let mut state = ControlCenterState::new();
    state.program_path = Some(program);
    let response = handle_get(
        &mut state,
        "/v1/analytics/time-travel?at=T%2B00%3A01&inspect=decisions",
    );
    assert_eq!(response.status, 200, "{}", response.body);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert_eq!(json["version"], "v1");
    assert!(json.get("time_travel").is_some());
}
