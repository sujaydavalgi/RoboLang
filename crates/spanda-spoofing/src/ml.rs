//! Optional ML backend for trace spoofing analysis via HTTP endpoint or stubs.

use crate::trace::{MissionTrace, SpoofingAlert, SpoofingSeverity};
use serde::Deserialize;

/// Merge optional ML spoofing alerts into an existing alert list.
pub fn merge_ml_spoofing_alerts(trace: &MissionTrace, alerts: &mut Vec<SpoofingAlert>) {
    // Fetch ML alerts from HTTP or stub backends and append after confidence filtering.
    //
    // Parameters:
    // - `trace` — mission trace under analysis
    // - `alerts` — heuristic alerts to extend in place
    //
    // Returns:
    // None (appends ML alerts when a backend responds successfully).
    //
    // Options:
    // `SPANDA_SPOOFING_ML_ENDPOINT` — HTTP URL accepting trace JSON.
    // `SPANDA_SPOOFING_ML_BACKEND` — `mock`, `file`, or `script` fallback.
    // `SPANDA_SPOOFING_ML_ALERTS_PATH` — JSON alerts file for `file` backend.
    // `SPANDA_SPOOFING_ML_SCRIPT` — shell command for `script` backend (stdout JSON).
    // `SPANDA_SPOOFING_ML_MIN_CONFIDENCE` — minimum confidence (0.0–1.0) to keep ML alerts.
    //
    // Example:
    // merge_ml_spoofing_alerts(&trace, &mut alerts);

    let mut merged = fetch_ml_spoofing_alerts(trace);
    apply_min_confidence_filter(&mut merged);
    alerts.extend(merged);
}

/// Fetch ML spoofing alerts without merging into an existing list.
pub fn fetch_ml_spoofing_alerts(trace: &MissionTrace) -> Vec<SpoofingAlert> {
    if let Some(alerts) = query_http_ml_alerts(trace).filter(|alerts| !alerts.is_empty()) {
        return alerts.into_iter().map(normalize_ml_alert).collect();
    }
    query_stub_ml_alerts(trace)
        .unwrap_or_default()
        .into_iter()
        .map(normalize_ml_alert)
        .collect()
}

fn query_http_ml_alerts(trace: &MissionTrace) -> Option<Vec<SpoofingAlert>> {
    #[cfg(not(feature = "http"))]
    {
        let _ = trace;
        return None;
    }
    #[cfg(feature = "http")]
    {
        let endpoint = std::env::var("SPANDA_SPOOFING_ML_ENDPOINT")
            .ok()
            .filter(|value| !value.trim().is_empty())?;
        let body = serde_json::to_string(trace).ok()?;
        let response =
            spanda_deploy_http::http_request("POST", &endpoint, Some(&body), None).ok()?;
        if !(200..300).contains(&response.status) {
            return None;
        }
        let payload = serde_json::from_str::<MlSpoofingResponse>(&response.body).ok()?;
        Some(payload.alerts)
    }
}

fn query_stub_ml_alerts(trace: &MissionTrace) -> Option<Vec<SpoofingAlert>> {
    let backend = std::env::var("SPANDA_SPOOFING_ML_BACKEND")
        .ok()
        .filter(|value| !value.trim().is_empty())?;
    match backend.trim().to_ascii_lowercase().as_str() {
        "mock" => Some(mock_ml_alerts(trace)),
        "file" => read_file_ml_alerts(),
        "script" => run_script_ml_alerts(trace),
        _ => None,
    }
}

fn mock_ml_alerts(trace: &MissionTrace) -> Vec<SpoofingAlert> {
    let spoof_frame = trace
        .frames
        .iter()
        .find(|frame| frame.event.to_ascii_lowercase().contains("spoof"));
    if spoof_frame.is_none() {
        return Vec::new();
    }
    vec![SpoofingAlert {
        sensor: "gps".into(),
        severity: SpoofingSeverity::High,
        confidence: 0.92,
        message: "mock ML model flagged GPS spoof pattern".into(),
        evidence: "ml:mock-backend".into(),
        sim_time_ms: spoof_frame.map(|frame| frame.sim_time_ms),
    }]
}

fn read_file_ml_alerts() -> Option<Vec<SpoofingAlert>> {
    let path = std::env::var("SPANDA_SPOOFING_ML_ALERTS_PATH")
        .ok()
        .filter(|value| !value.trim().is_empty())?;
    let text = std::fs::read_to_string(&path).ok()?;
    let payload = serde_json::from_str::<MlSpoofingResponse>(&text).ok()?;
    Some(payload.alerts)
}

fn run_script_ml_alerts(trace: &MissionTrace) -> Option<Vec<SpoofingAlert>> {
    let script = std::env::var("SPANDA_SPOOFING_ML_SCRIPT")
        .ok()
        .filter(|value| !value.trim().is_empty())?;
    let body = serde_json::to_string(trace).ok()?;
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(&script)
        .env("SPANDA_SPOOFING_TRACE_JSON", body)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let payload = serde_json::from_slice::<MlSpoofingResponse>(&output.stdout).ok()?;
    Some(payload.alerts)
}

fn apply_min_confidence_filter(alerts: &mut Vec<SpoofingAlert>) {
    let min_confidence = std::env::var("SPANDA_SPOOFING_ML_MIN_CONFIDENCE")
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| (0.0..=1.0).contains(value))
        .unwrap_or(0.0);
    if min_confidence <= 0.0 {
        return;
    }
    alerts.retain(|alert| alert.confidence >= min_confidence);
}

fn normalize_ml_alert(mut alert: SpoofingAlert) -> SpoofingAlert {
    if !alert.evidence.starts_with("ml:") {
        alert.evidence = format!("ml:{}", alert.evidence);
    }
    alert
}

#[derive(Debug, Deserialize)]
struct MlSpoofingResponse {
    #[serde(default)]
    alerts: Vec<SpoofingAlert>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::TraceFrame;

    #[test]
    fn ml_merge_is_noop_without_backend() {
        let _guard = crate::testing::env_lock();
        let trace = sample_trace();
        let mut alerts = Vec::new();
        std::env::remove_var("SPANDA_SPOOFING_ML_ENDPOINT");
        std::env::remove_var("SPANDA_SPOOFING_ML_BACKEND");
        merge_ml_spoofing_alerts(&trace, &mut alerts);
        assert!(alerts.is_empty());
    }

    #[test]
    fn mock_backend_flags_spoof_trace() {
        let _guard = crate::testing::env_lock();
        std::env::remove_var("SPANDA_SPOOFING_ML_ENDPOINT");
        std::env::remove_var("SPANDA_SPOOFING_ML_MIN_CONFIDENCE");
        std::env::set_var("SPANDA_SPOOFING_ML_BACKEND", "mock");
        let trace = sample_trace();
        let alerts = fetch_ml_spoofing_alerts(&trace);
        assert_eq!(alerts.len(), 1);
        assert!(alerts[0].evidence.starts_with("ml:"));
        std::env::remove_var("SPANDA_SPOOFING_ML_BACKEND");
    }

    #[test]
    fn min_confidence_filters_low_score_alerts() {
        let _guard = crate::testing::env_lock();
        std::env::remove_var("SPANDA_SPOOFING_ML_ENDPOINT");
        std::env::set_var("SPANDA_SPOOFING_ML_BACKEND", "mock");
        std::env::set_var("SPANDA_SPOOFING_ML_MIN_CONFIDENCE", "0.95");
        let trace = sample_trace();
        let mut alerts = fetch_ml_spoofing_alerts(&trace);
        apply_min_confidence_filter(&mut alerts);
        assert!(alerts.is_empty());
        std::env::remove_var("SPANDA_SPOOFING_ML_BACKEND");
        std::env::remove_var("SPANDA_SPOOFING_ML_MIN_CONFIDENCE");
    }

    #[test]
    fn ml_response_deserializes_alerts() {
        let json = r#"{"alerts":[{"sensor":"gps","severity":"high","confidence":0.9,"message":"ml","evidence":"model"}]}"#;
        let payload: MlSpoofingResponse = serde_json::from_str(json).unwrap();
        assert_eq!(payload.alerts.len(), 1);
        assert_eq!(payload.alerts[0].severity, SpoofingSeverity::High);
    }

    fn sample_trace() -> MissionTrace {
        MissionTrace {
            version: 1,
            source: "spoof.trace".into(),
            deterministic: true,
            frames: vec![
                TraceFrame {
                    sim_time_ms: 0.0,
                    event: "gps_reading".into(),
                    payload: serde_json::json!({}),
                },
                TraceFrame {
                    sim_time_ms: 500.0,
                    event: "emit gps.spoofed".into(),
                    payload: serde_json::json!({}),
                },
            ],
        }
    }
}
