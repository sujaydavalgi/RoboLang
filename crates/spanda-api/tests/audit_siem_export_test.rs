//! Mutation audit SIEM export formats.

use spanda_api::audit_log::{export_mutation_audit_cef, export_mutation_audit_jsonl};
use std::fs::OpenOptions;
use std::io::Write;

#[test]
fn mutation_audit_exports_cef_and_jsonl() {
    let path = std::env::temp_dir().join(format!("spanda-audit-export-{}", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let line = serde_json::json!({
        "id": "rec-1",
        "event_type": "control_center.api.mutation",
        "payload": r#"{"method":"POST","path":"/v1/ota/execute","actor_key_id":"ops","correlation_id":"c-1"}"#,
        "timestamp_ms": 1.0,
    });
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .expect("open");
    writeln!(file, "{}", line).expect("write");
    let cef = export_mutation_audit_cef(&path).expect("cef");
    assert!(cef.contains("CEF:0|Spanda|ControlCenter"));
    assert!(cef.contains("/v1/ota/execute"));
    let jsonl = export_mutation_audit_jsonl(&path).expect("jsonl");
    assert!(jsonl.contains("control_center.api.mutation"));
    let _ = std::fs::remove_file(path);
}
