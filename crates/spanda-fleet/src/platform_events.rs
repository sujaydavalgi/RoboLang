//! Platform event emission for fleet orchestration.
//!
use spanda_audit::platform_event::names;
use spanda_audit::PlatformEvent;
use spanda_runtime::publish_platform_event;
use serde_json::json;

/// Record `FleetMemberJoined` when a robot joins an orchestrated fleet group.
pub fn record_fleet_member_joined(fleet_id: &str, member_id: &str) {
    let event = PlatformEvent::new(
        names::FLEET_MEMBER_JOINED,
        "spanda-fleet",
        json!({
            "fleet_id": fleet_id,
            "member_id": member_id,
        }),
    )
    .with_entity_id(format!("fleet/{fleet_id}"));
    publish_platform_event(None, &event);
}

/// Record `FleetMemberLeft` when a robot leaves a fleet group or registry.
pub fn record_fleet_member_left(fleet_id: &str, member_id: &str) {
    let event = PlatformEvent::new(
        names::FLEET_MEMBER_LEFT,
        "spanda-fleet",
        json!({
            "fleet_id": fleet_id,
            "member_id": member_id,
        }),
    )
    .with_entity_id(format!("fleet/{fleet_id}"));
    publish_platform_event(None, &event);
}
