//! SRE incident workflow API tests.

use spanda_api::handlers::handle_request;
use spanda_api::state::ControlCenterState;
use spanda_deploy_http::HttpRequest;
use std::sync::Mutex;

static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn incident_lifecycle_via_rest() {
    let _guard = ENV_TEST_LOCK.lock().unwrap();
    std::env::set_var("SPANDA_API_KEY", "incident-test-key");
    let mut state = ControlCenterState::new();
    state.api_keys = spanda_security::ApiKeyStore::from_env();

    let create_body = serde_json::json!({
        "title": "smoke incident",
        "description": "test incident workflow",
        "severity": "warning",
    });
    let (created, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "POST".into(),
            path: "/v1/sre/incidents".into(),
            body: create_body.to_string(),
            authorization: Some("incident-test-key".into()),
        },
        "",
    );
    assert_eq!(created.status, 200);
    let incident_id: String = serde_json::from_str::<serde_json::Value>(&created.body)
        .expect("parse create")
        .pointer("/incident/id")
        .and_then(|value| value.as_str())
        .expect("incident id")
        .to_string();

    let (ack, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "POST".into(),
            path: format!("/v1/sre/incidents/{incident_id}/ack"),
            body: r#"{"assignee":"oncall"}"#.into(),
            authorization: Some("incident-test-key".into()),
        },
        "",
    );
    assert_eq!(ack.status, 200);

    let (resolve, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "POST".into(),
            path: format!("/v1/sre/incidents/{incident_id}/resolve"),
            body: String::new(),
            authorization: Some("incident-test-key".into()),
        },
        "",
    );
    assert_eq!(resolve.status, 200);

    let (summary, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "GET".into(),
            path: "/v1/sre/summary".into(),
            body: String::new(),
            authorization: None,
        },
        "",
    );
    assert_eq!(summary.status, 200);
    assert!(summary.body.contains("mttr_hint_ms"));
    assert!(summary.body.contains("\"incidents_total\":1"));
}
