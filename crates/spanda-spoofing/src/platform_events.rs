//! Platform event emission when spoof-check finds actionable alerts.
//!
use spanda_audit::platform_event::names;
use spanda_audit::PlatformEvent;
use spanda_runtime::publish_platform_event;
use serde_json::json;

use crate::detect::SpoofingReport;
use crate::trace::SpoofingSeverity;

/// Record `SpoofingDetected` when spoof-check reports high-severity alerts.
pub fn record_spoofing_platform_events(report: &SpoofingReport) {
    let actionable = report.alerts.iter().any(|alert| {
        matches!(
            alert.severity,
            SpoofingSeverity::High | SpoofingSeverity::Critical
        )
    });
    if !actionable && report.passed {
        return;
    }
    let top_alert = report.alerts.first().map(|alert| {
        json!({
            "sensor": alert.sensor,
            "severity": format!("{:?}", alert.severity),
            "confidence": alert.confidence,
            "message": alert.message,
        })
    });
    let event = PlatformEvent::new(
        names::SPOOFING_DETECTED,
        "spanda-spoofing",
        json!({
            "source": report.source,
            "kind": format!("{:?}", report.kind),
            "passed": report.passed,
            "alert_count": report.alerts.len(),
            "requires_operator_confirmation": report.requires_operator_confirmation,
            "top_alert": top_alert,
        }),
    )
    .with_entity_id(format!("spoof-check/{}", report.source));
    publish_platform_event(None, &event);
}
