//! Scheduled drift scan API and alert dispatch tests.

use spanda_api::handlers::handle_request;
use spanda_api::state::ControlCenterState;
use spanda_config::{default_snapshots_dir, save_config_snapshot};
use spanda_deploy_http::HttpRequest;
use std::path::PathBuf;
use std::sync::Mutex;

static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

fn setup_state() -> (ControlCenterState, String) {
    let state_dir = std::env::temp_dir().join(format!(
        "spanda-drift-scan-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&state_dir);
    std::fs::create_dir_all(&state_dir).unwrap();
    std::env::set_var(
        "SPANDA_CONTROL_CENTER_STATE_DIR",
        state_dir.to_string_lossy().to_string(),
    );
    std::env::set_var("SPANDA_API_KEY", "drift-scan-test-key");

    let example =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/packages/basic_project/spanda.toml");
    let mut state = ControlCenterState::new().with_config_path(example);
    state.api_keys = spanda_security::ApiKeyStore::from_env();
    state.reload_config().expect("reload example config");

    let resolved = state.resolved.as_ref().expect("resolved config").clone();
    let snapshot_dir = default_snapshots_dir();
    let _ = std::fs::remove_dir_all(&snapshot_dir);
    let meta = save_config_snapshot(&resolved, &snapshot_dir, Some("baseline".into()), None)
        .expect("save snapshot");
    (state, meta.id)
}

#[test]
fn manual_drift_scan_records_history() {
    let _guard = ENV_TEST_LOCK.lock().unwrap();
    let (mut state, baseline_id) = setup_state();

    let (response, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "POST".into(),
            path: "/v1/drift/scan".into(),
            body: format!(r#"{{"baseline_id":"{baseline_id}"}}"#),
            authorization: Some("drift-scan-test-key".into()),
        },
        "",
    );
    assert_eq!(response.status, 200, "body: {}", response.body);
    assert!(response.body.contains("\"passed\""));

    let (list, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "GET".into(),
            path: "/v1/drift/scans".into(),
            body: String::new(),
            authorization: None,
        },
        "",
    );
    assert_eq!(list.status, 200);
    assert!(list.body.contains(&baseline_id));
    assert_eq!(state.drift_scan_store.list().len(), 1);
}

#[test]
fn drift_scan_lists_without_auth() {
    let _guard = ENV_TEST_LOCK.lock().unwrap();
    let (mut state, _) = setup_state();
    let (list, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "GET".into(),
            path: "/v1/drift/scans".into(),
            body: String::new(),
            authorization: None,
        },
        "",
    );
    assert_eq!(list.status, 200);
}
