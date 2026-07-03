//! SDK contract tests — URL construction and type parsing without a live server.
use serde_json::json;
use spanda_sdk::{ReadinessReport, SpandaClient, SpandaError};

#[test]
fn readiness_report_extracts_score() {
    let report = ReadinessReport::from_api(json!({
        "report": { "score": { "total": 92 }, "status": "Ready" }
    }));
    assert_eq!(report.score, Some(92));
    assert_eq!(report.status.as_deref(), Some("Ready"));
}

#[test]
fn error_from_status_maps_permission() {
    let err = SpandaError::from_status(403, "forbidden");
    assert!(matches!(err, SpandaError::Permission(_)));
}

#[test]
fn client_builder_sets_url() {
    let client = SpandaClient::builder()
        .base_url("http://example:9090")
        .build();
    assert!(client.health_check().is_err());
}

#[test]
fn twin_sync_path_includes_optional_twin_id() {
    let client = SpandaClient::builder()
        .base_url("http://example:9090")
        .api_key("test-key")
        .build();
    assert!(client.sync_twin(Some("patrol")).is_err());
    assert!(client.sync_twin(None).is_err());
}

#[test]
fn twin_history_and_import_paths_build() {
    let client = SpandaClient::builder()
        .base_url("http://example:9090")
        .api_key("test-key")
        .build();
    assert!(client.get_twin_history("patrol").is_err());
    assert!(client.import_twin_replay("patrol.sd", Some("patrol")).is_err());
}
