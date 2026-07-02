//! REST API tests for NEXT differentiation analytics endpoints.

use spanda_api::handlers::handle_request;
use spanda_api::state::ControlCenterState;
use spanda_deploy_http::HttpRequest;
use std::path::PathBuf;

fn forecast_program() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/showcase/forecast/degradation.sd")
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
fn differentiation_analytics_endpoints_return_reports() {
    let program = forecast_program();
    assert!(program.exists(), "missing {}", program.display());
    let mut state = ControlCenterState::new();
    state.program_path = Some(program);

    for path in [
        "/v1/analytics/what-if?all=1",
        "/v1/analytics/mission-risk",
        "/v1/analytics/readiness-forecast?all=1",
        "/v1/analytics/trust-graph",
    ] {
        let response = handle_get(&mut state, path);
        assert_eq!(response.status, 200, "{path}: {}", response.body);
        let json: serde_json::Value = serde_json::from_str(&response.body).expect(path);
        assert_eq!(json["version"], "v1");
    }
}

#[test]
fn differentiation_analytics_requires_program() {
    let mut state = ControlCenterState::new();
    let response = handle_get(&mut state, "/v1/analytics/mission-risk");
    assert_eq!(response.status, 400);
}
