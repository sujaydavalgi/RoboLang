//! Configuration approval publish workflow tests.

use spanda_api::handlers::handle_request;
use spanda_api::state::ControlCenterState;
use spanda_config::{default_snapshots_dir, save_config_snapshot};
use spanda_deploy_http::HttpRequest;
use std::path::PathBuf;
use std::sync::Mutex;

static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn approved_snapshot_publishes_to_runtime() {
    let _guard = ENV_TEST_LOCK.lock().unwrap();
    let state_dir = std::env::temp_dir().join(format!(
        "spanda-approval-publish-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&state_dir);
    std::fs::create_dir_all(&state_dir).unwrap();
    std::env::set_var(
        "SPANDA_CONTROL_CENTER_STATE_DIR",
        state_dir.to_string_lossy().to_string(),
    );
    std::env::set_var("SPANDA_API_KEY", "approval-publish-key");

    let example =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/packages/basic_project/spanda.toml");
    let mut state = ControlCenterState::new().with_config_path(example.clone());
    state.api_keys = spanda_security::ApiKeyStore::from_env();
    state.reload_config().expect("reload example config");

    let resolved = state.resolved.as_ref().expect("resolved config").clone();
    let snapshot_dir = default_snapshots_dir();
    let _ = std::fs::remove_dir_all(&snapshot_dir);
    let meta = save_config_snapshot(&resolved, &snapshot_dir, Some("baseline".into()))
        .expect("save snapshot");

    let (submit, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "POST".into(),
            path: "/v1/config/approvals".into(),
            body: format!(r#"{{"snapshot_id":"{}"}}"#, meta.id),
            authorization: Some("approval-publish-key".into()),
        },
        "",
    );
    assert_eq!(submit.status, 200);
    let approval_id: String = serde_json::from_str::<serde_json::Value>(&submit.body)
        .expect("parse submit")
        .pointer("/approval/id")
        .and_then(|value| value.as_str())
        .expect("approval id")
        .to_string();

    state.resolved = None;
    let (approved, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "POST".into(),
            path: format!("/v1/config/approvals/{approval_id}/approve"),
            body: String::new(),
            authorization: Some("approval-publish-key".into()),
        },
        "",
    );
    assert_eq!(approved.status, 200, "body: {}", approved.body);
    assert!(approved.body.contains("\"publish\""));
    assert!(state.resolved.is_some());
}
