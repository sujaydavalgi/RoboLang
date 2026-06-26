//! Third-party integration webhooks (PagerDuty inbound sync).
//!
use crate::handlers::{bad_request, json_ok, now_ms, unauthorized};
use crate::state::ControlCenterState;
use serde::Deserialize;
use spanda_deploy_http::HttpResponse;
use spanda_ops::pagerduty;
use spanda_security::{ApiKeyStore, RbacAction, RbacContext};

#[derive(Debug, Deserialize)]
struct PagerDutyWebhookEnvelope {
    #[serde(default)]
    event: Option<String>,
    #[serde(default)]
    incident_id: Option<String>,
    #[serde(default)]
    dedup_key: Option<String>,
    #[serde(default)]
    assignee: Option<String>,
    #[serde(default)]
    messages: Vec<PagerDutyWebhookMessage>,
}

#[derive(Debug, Deserialize)]
struct PagerDutyWebhookMessage {
    #[serde(default)]
    event: Option<String>,
    #[serde(default)]
    incident: Option<PagerDutyWebhookIncident>,
}

#[derive(Debug, Deserialize)]
struct PagerDutyWebhookIncident {
    #[serde(default)]
    incident_key: Option<String>,
    #[serde(default)]
    _id: Option<String>,
}

fn webhook_secret_ok(headers: &[(String, String)]) -> bool {
    let expected = std::env::var("SPANDA_PAGERDUTY_WEBHOOK_SECRET").unwrap_or_default();
    if expected.trim().is_empty() {
        return false;
    }
    let provided = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("x-spanda-webhook-secret"))
        .map(|(_, value)| value.as_str());
    provided == Some(expected.as_str())
}

fn normalize_event(body: &PagerDutyWebhookEnvelope) -> (Option<String>, Option<String>, Option<String>) {
    if let Some(event) = body.event.as_deref() {
        return (
            Some(event.to_string()),
            body.incident_id.clone(),
            body.assignee.clone(),
        );
    }
    if let Some(message) = body.messages.first() {
        let event = message.event.clone();
        let dedup = message
            .incident
            .as_ref()
            .and_then(|incident| incident.incident_key.clone());
        return (event, dedup, None);
    }
    (None, body.incident_id.clone(), body.assignee.clone())
}

fn pagerduty_webhook_authorized(headers: &[(String, String)], ctx: Option<&RbacContext>) -> bool {
    webhook_secret_ok(headers) || ApiKeyStore::check(ctx, RbacAction::Operate)
}

pub fn pagerduty_webhook(
    state: &mut ControlCenterState,
    body: &str,
    headers: &[(String, String)],
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !pagerduty_webhook_authorized(headers, ctx) {
        return unauthorized();
    }
    let payload: PagerDutyWebhookEnvelope = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(error) => return bad_request(&error.to_string()),
    };
    let (event, incident_ref, assignee) = normalize_event(&payload);
    let Some(event_name) = event else {
        return bad_request("missing event");
    };
    let incident_id = incident_ref.or(payload.incident_id).or(payload.dedup_key);
    let Some(incident_id) = incident_id.filter(|value| !value.trim().is_empty()) else {
        return bad_request("missing incident_id or dedup_key");
    };
    let synced = if event_name.contains("ack") {
        state
            .incident_store
            .acknowledge_by_ref(&incident_id, assignee)
    } else if event_name.contains("resolve") {
        state.incident_store.resolve_by_ref(&incident_id)
    } else {
        return bad_request("unsupported event; use acknowledge or resolve");
    };
    let Some(incident) = synced else {
        return bad_request("incident not found or already resolved");
    };
    let _ = crate::persistence::persist_runtime_state(state);
    json_ok(&serde_json::json!({
        "version": "v1",
        "ok": true,
        "event": event_name,
        "incident": incident,
        "synced_at_ms": now_ms(),
        "pagerduty_outbound": pagerduty::sync_incident_status_to_pagerduty(&incident, &event_name),
    }))
}
