//! Observability export — OTLP traces for Jaeger and trace previews.
//!
use crate::correlation::TraceRecord;
use crate::handlers::{bad_request, json_ok, parse_query, unauthorized};
use crate::state::ControlCenterState;
use spanda_deploy_http::HttpResponse;
use spanda_ops::{
    env_metrics_endpoint, env_otlp_token, env_traces_endpoint, push_otlp_metrics, push_otlp_traces,
    render_otlp_metrics_json, render_otlp_traces_json, ControlCenterMetrics, HttpTraceSpan,
};
use spanda_security::{ApiKeyStore, RbacAction, RbacContext};

fn spans_from_trace_log(records: &[TraceRecord]) -> Vec<HttpTraceSpan> {
    records
        .iter()
        .map(|record| HttpTraceSpan {
            correlation_id: record.correlation_id.clone(),
            method: record.method.clone(),
            path: record.path.clone(),
            status: record.status,
            timestamp_ms: record.timestamp_ms,
            duration_ms: record.duration_ms,
        })
        .collect()
}

fn control_center_metrics(state: &ControlCenterState) -> ControlCenterMetrics {
    let pool = state.device_registry().pool_summary();
    let alerts = state.alert_store.list_owned();
    let critical = alerts
        .iter()
        .filter(|alert| {
            format!("{:?}", alert.severity)
                .to_ascii_lowercase()
                .contains("critical")
        })
        .count();
    let traces = state.trace_log.list_owned();
    ControlCenterMetrics {
        devices_total: pool.total as u64,
        devices_healthy: pool.healthy as u64,
        alerts_total: alerts.len() as u64,
        alerts_critical: critical as u64,
        traces_recorded: traces.len() as u64,
        availability_percent: if pool.total == 0 {
            100.0
        } else {
            ((pool.healthy + pool.assigned) as f64 / pool.total as f64) * 100.0
        },
    }
}

pub fn otlp_metrics_preview(state: &ControlCenterState) -> HttpResponse {
    let metrics = control_center_metrics(state);
    let body = render_otlp_metrics_json(&metrics);
    json_ok(&serde_json::json!({
        "version": "v1",
        "metric_count": 6,
        "otlp": serde_json::from_str::<serde_json::Value>(&body).unwrap_or(serde_json::json!({})),
    }))
}

pub fn otlp_metrics_export(
    state: &ControlCenterState,
    query: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Operate) {
        return unauthorized();
    }
    let params = parse_query(query);
    let endpoint = params.get("endpoint").cloned().or_else(env_metrics_endpoint);
    let Some(endpoint) = endpoint else {
        return bad_request(
            "missing metrics endpoint; set SPANDA_OTLP_METRICS_ENDPOINT or pass ?endpoint=",
        );
    };
    let metrics = control_center_metrics(state);
    let body = render_otlp_metrics_json(&metrics);
    let token = env_otlp_token();
    match push_otlp_metrics(&endpoint, &body, token.as_deref()) {
        Ok(()) => json_ok(&serde_json::json!({
            "version": "v1",
            "ok": true,
            "endpoint": endpoint,
            "metric_count": 6,
        })),
        Err(message) => bad_request(&message),
    }
}

pub fn otlp_traces_preview(state: &ControlCenterState) -> HttpResponse {
    let spans = spans_from_trace_log(&state.trace_log.list_owned());
    let body = render_otlp_traces_json(&spans);
    json_ok(&serde_json::json!({
        "version": "v1",
        "span_count": spans.len(),
        "otlp": serde_json::from_str::<serde_json::Value>(&body).unwrap_or(serde_json::json!({})),
    }))
}

pub fn otlp_traces_export(
    state: &ControlCenterState,
    query: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Operate) {
        return unauthorized();
    }
    let params = parse_query(query);
    let endpoint = params.get("endpoint").cloned().or_else(env_traces_endpoint);
    let Some(endpoint) = endpoint else {
        return bad_request(
            "missing traces endpoint; set SPANDA_OTLP_TRACES_ENDPOINT or pass ?endpoint=",
        );
    };
    let spans = spans_from_trace_log(&state.trace_log.list_owned());
    let body = render_otlp_traces_json(&spans);
    let token = env_otlp_token();
    match push_otlp_traces(&endpoint, &body, token.as_deref()) {
        Ok(()) => json_ok(&serde_json::json!({
            "version": "v1",
            "ok": true,
            "endpoint": endpoint,
            "span_count": spans.len(),
        })),
        Err(message) => bad_request(&message),
    }
}

pub fn maybe_auto_push_latest_span(record: &TraceRecord) {
    if !spanda_ops::env_trace_auto_push_enabled() {
        return;
    }
    let Some(endpoint) = env_traces_endpoint() else {
        return;
    };
    let span = HttpTraceSpan {
        correlation_id: record.correlation_id.clone(),
        method: record.method.clone(),
        path: record.path.clone(),
        status: record.status,
        timestamp_ms: record.timestamp_ms,
        duration_ms: record.duration_ms,
    };
    let body = render_otlp_traces_json(&[span]);
    let token = env_otlp_token();
    if let Err(error) = push_otlp_traces(&endpoint, &body, token.as_deref()) {
        eprintln!("OTLP trace auto-push failed: {error}");
    }
}
