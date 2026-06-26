//! PagerDuty inbound webhook acknowledges and resolves incidents by reference.

use spanda_api::integrations::pagerduty_webhook;
use spanda_api::ControlCenterState;
use spanda_ops::IncidentSeverity;
use spanda_security::{default_tenant_id, RbacContext, Role};

#[test]
fn pagerduty_webhook_acknowledges_by_incident_id() {
    let mut state = ControlCenterState::new();
    let incident = state.incident_store.create(
        "fleet offline".into(),
        "agent unreachable".into(),
        IncidentSeverity::Critical,
        Some("alert-42".into()),
    );
    if let Some(stored) = state.incident_store.get_mut(&incident.id) {
        stored.pagerduty_dedup_key = Some("alert-42".into());
    }
    let ctx = RbacContext {
        key_id: "test".into(),
        role: Role::Administrator,
        tenant_id: default_tenant_id(),
    };
    let body = format!(
        r#"{{"event":"incident.acknowledged","incident_id":"{}"}}"#,
        incident.id
    );
    let response = pagerduty_webhook(&mut state, &body, &[], Some(&ctx));
    assert_eq!(response.status, 200, "{}", response.body);
    assert!(response.body.contains("acknowledged"));
}

#[test]
fn pagerduty_webhook_requires_auth_without_secret() {
    let mut state = ControlCenterState::new();
    let body = r#"{"event":"incident.resolve","incident_id":"missing"}"#;
    let response = pagerduty_webhook(&mut state, body, &[], None);
    assert_eq!(response.status, 401);
    let ctx = RbacContext {
        key_id: "test".into(),
        role: Role::Administrator,
        tenant_id: default_tenant_id(),
    };
    let response = pagerduty_webhook(&mut state, body, &[], Some(&ctx));
    assert_eq!(response.status, 400);
}
