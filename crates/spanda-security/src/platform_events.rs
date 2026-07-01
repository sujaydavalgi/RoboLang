//! Platform event emission for security lifecycle hooks.
//!
use spanda_audit::platform_event::names;
use spanda_audit::PlatformEvent;
use spanda_runtime::publish_platform_event;
use serde_json::json;

/// Record `AuthFailed` when authentication is rejected.
pub fn record_auth_failed(reason: &str, principal: Option<&str>) {
    let event = PlatformEvent::new(
        names::AUTH_FAILED,
        "spanda-security",
        json!({
            "reason": reason,
            "principal": principal,
        }),
    )
    .with_entity_id(
        principal
            .map(|id| format!("principal/{id}"))
            .unwrap_or_else(|| "principal/unknown".into()),
    );
    publish_platform_event(None, &event);
}

/// Record `SecretRotated` after a managed secret value is rotated.
pub fn record_secret_rotated(name: &str, rotation_count: u32) {
    let event = PlatformEvent::new(
        names::SECRET_ROTATED,
        "spanda-security",
        json!({
            "name": name,
            "rotation_count": rotation_count,
        }),
    )
    .with_entity_id(format!("secret/{name}"));
    publish_platform_event(None, &event);
}
