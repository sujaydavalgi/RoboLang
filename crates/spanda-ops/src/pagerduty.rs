//! PagerDuty Events API v2 compatible alert payload formatting and incident sync.
//!
use crate::alerting::Alert;
use crate::incidents::{Incident, IncidentStatus};
use serde_json::{json, Value};

/// Format an alert as a PagerDuty Events API v2 JSON body.
pub fn pagerduty_events_payload(
    alert: &Alert,
    routing_key: &str,
    incident_id: Option<&str>,
) -> String {
    let severity = match alert.severity {
        crate::alerting::AlertSeverity::Critical => "critical",
        crate::alerting::AlertSeverity::Warning => "warning",
        _ => "info",
    };
    let mut custom_details = json!({
        "alert_id": alert.id,
        "alert_type": format!("{:?}", alert.alert_type),
    });
    if let Some(incident_id) = incident_id {
        custom_details["incident_id"] = json!(incident_id);
    }
    json!({
        "routing_key": routing_key,
        "event_action": "trigger",
        "dedup_key": alert.id,
        "payload": {
            "summary": alert.message,
            "severity": severity,
            "source": alert.source,
            "custom_details": custom_details,
        }
    })
    .to_string()
}

/// Build a PagerDuty acknowledge or resolve event for an incident.
pub fn pagerduty_incident_event_payload(
    incident: &Incident,
    routing_key: &str,
    event_action: &str,
) -> Option<String> {
    let dedup_key = incident
        .pagerduty_dedup_key
        .as_deref()
        .or(incident.source_alert_id.as_deref())?;
    Some(
        json!({
            "routing_key": routing_key,
            "event_action": event_action,
            "dedup_key": dedup_key,
            "payload": {
                "summary": incident.title,
                "severity": format!("{:?}", incident.severity).to_ascii_lowercase(),
                "source": "spanda-control-center",
                "custom_details": {
                    "incident_id": incident.id,
                }
            }
        })
        .to_string(),
    )
}

/// Push incident status changes to PagerDuty when outbound env vars are configured.
pub fn sync_incident_status_to_pagerduty(incident: &Incident, event_hint: &str) -> Value {
    let url = match std::env::var("SPANDA_ALERT_PAGERDUTY_URL") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            return json!({ "skipped": true, "reason": "SPANDA_ALERT_PAGERDUTY_URL unset" });
        }
    };
    let routing_key =
        std::env::var("SPANDA_ALERT_PAGERDUTY_ROUTING_KEY").unwrap_or_else(|_| "spanda".into());
    let event_action =
        if event_hint.contains("resolve") || incident.status == IncidentStatus::Resolved {
            "resolve"
        } else {
            "acknowledge"
        };
    let Some(body) = pagerduty_incident_event_payload(incident, &routing_key, event_action) else {
        return json!({ "skipped": true, "reason": "no pagerduty dedup key on incident" });
    };
    match crate::alerting::send_webhook_body(&url, &body) {
        Ok(()) => json!({
            "ok": true,
            "event_action": event_action,
            "url": url,
        }),
        Err(error) => json!({
            "ok": false,
            "event_action": event_action,
            "error": error,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerting::{AlertSeverity, AlertType};
    use crate::incidents::{IncidentSeverity, IncidentStatus};

    #[test]
    fn pagerduty_payload_contains_routing_key() {
        let alert = Alert {
            id: "a1".into(),
            alert_type: AlertType::Crash,
            severity: AlertSeverity::Critical,
            message: "process crash".into(),
            source: "runtime".into(),
            timestamp_ms: 1.0,
            delivered_via: vec![],
        };
        let body = pagerduty_events_payload(&alert, "test-routing-key", Some("incident-1"));
        assert!(body.contains("test-routing-key"));
        assert!(body.contains("process crash"));
        assert!(body.contains("incident-1"));
    }

    #[test]
    fn pagerduty_ack_payload_uses_dedup_key() {
        let incident = Incident {
            id: "incident-1".into(),
            title: "test".into(),
            description: "desc".into(),
            severity: IncidentSeverity::Critical,
            status: IncidentStatus::Acknowledged,
            source_alert_id: Some("alert-1".into()),
            pagerduty_dedup_key: Some("alert-1".into()),
            created_at_ms: 1.0,
            acknowledged_at_ms: Some(2.0),
            resolved_at_ms: None,
            assignee: None,
        };
        let body = pagerduty_incident_event_payload(&incident, "rk", "acknowledge").expect("body");
        assert!(body.contains("acknowledge"));
        assert!(body.contains("alert-1"));
    }
}
