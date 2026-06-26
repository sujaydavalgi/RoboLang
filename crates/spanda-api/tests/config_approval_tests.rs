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
    let state_dir =
        std::env::temp_dir().join(format!("spanda-approval-publish-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&state_dir);
    std::fs::create_dir_all(&state_dir).unwrap();
    std::env::set_var(
        "SPANDA_CONTROL_CENTER_STATE_DIR",
        state_dir.to_string_lossy().to_string(),
    );
    std::env::set_var("SPANDA_API_KEY", "approval-publish-key");

    let example = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/packages/basic_project/spanda.toml");
    let mut state = ControlCenterState::new().with_config_path(example.clone());
    state.api_keys = spanda_security::ApiKeyStore::from_env();
    state.reload_config().expect("reload example config");

    let resolved = state.resolved.as_ref().expect("resolved config").clone();
    let snapshot_dir = default_snapshots_dir();
    let _ = std::fs::remove_dir_all(&snapshot_dir);
    let meta = save_config_snapshot(&resolved, &snapshot_dir, Some("baseline".into()), None)
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

#[test]
fn two_approver_quorum_publishes_after_second_vote() {
    let _guard = ENV_TEST_LOCK.lock().unwrap();
    let state_dir =
        std::env::temp_dir().join(format!("spanda-approval-quorum-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&state_dir);
    std::fs::create_dir_all(&state_dir).unwrap();
    std::env::set_var(
        "SPANDA_CONTROL_CENTER_STATE_DIR",
        state_dir.to_string_lossy().to_string(),
    );

    let keys_path = state_dir.join("api-keys.json");
    std::fs::write(
        &keys_path,
        r#"[
          {"key_id":"submitter","token":"submit-key","role":"administrator","tenant_id":"default"},
          {"key_id":"approver-a","token":"approve-a-key","role":"administrator","tenant_id":"default"},
          {"key_id":"approver-b","token":"approve-b-key","role":"administrator","tenant_id":"default"}
        ]"#,
    )
    .unwrap();
    std::env::set_var(
        "SPANDA_API_KEYS_FILE",
        keys_path.to_string_lossy().to_string(),
    );
    std::env::remove_var("SPANDA_API_KEY");

    let example = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/packages/basic_project/spanda.toml");
    let mut state = ControlCenterState::new().with_config_path(example);
    state.api_keys = spanda_security::ApiKeyStore::from_env_and_file();
    state.reload_config().expect("reload example config");

    let resolved = state.resolved.as_ref().expect("resolved config").clone();
    let snapshot_dir = default_snapshots_dir();
    let _ = std::fs::remove_dir_all(&snapshot_dir);
    let meta = save_config_snapshot(&resolved, &snapshot_dir, Some("baseline".into()), None)
        .expect("save snapshot");

    let (submit, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "POST".into(),
            path: "/v1/config/approvals".into(),
            body: format!(r#"{{"snapshot_id":"{}","required_approvals":2}}"#, meta.id),
            authorization: Some("submit-key".into()),
        },
        "",
    );
    assert_eq!(submit.status, 200, "submit body: {}", submit.body);
    let approval_id: String = serde_json::from_str::<serde_json::Value>(&submit.body)
        .expect("parse submit")
        .pointer("/approval/id")
        .and_then(|value| value.as_str())
        .expect("approval id")
        .to_string();

    state.resolved = None;
    let (first, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "POST".into(),
            path: format!("/v1/config/approvals/{approval_id}/approve"),
            body: String::new(),
            authorization: Some("approve-a-key".into()),
        },
        "",
    );
    assert_eq!(first.status, 200, "first approve body: {}", first.body);
    assert!(first.body.contains("\"met\":false"));
    assert!(state.resolved.is_none());

    let (second, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "POST".into(),
            path: format!("/v1/config/approvals/{approval_id}/approve"),
            body: String::new(),
            authorization: Some("approve-b-key".into()),
        },
        "",
    );
    assert_eq!(second.status, 200, "second approve body: {}", second.body);
    assert!(second.body.contains("\"met\":true"));
    assert!(second.body.contains("\"reloaded_from_disk\""));
    assert!(state.resolved.is_some());
}
