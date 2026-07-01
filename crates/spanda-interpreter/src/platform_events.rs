//! Platform event emission for interpreter lifecycle hooks.
//!
use spanda_ast::nodes::{Program, RobotDecl};
use spanda_audit::platform_event::names;
use spanda_audit::{AuditRuntime, PlatformEvent};
use spanda_runtime::publish_platform_event;
use spanda_runtime::RecoveryStatus;
use spanda_runtime::telemetry_sink::TelemetrySink;
use serde_json::json;

/// Record a mission lifecycle platform event when audit runtime is configured.
pub(crate) fn emit_mission_platform_event(
    audit: Option<&mut AuditRuntime>,
    telemetry: &dyn TelemetrySink,
    event_type: &str,
    program: &Program,
    trace_source: Option<&str>,
    success: bool,
) {
    let mission_key = trace_source
        .map(str::to_string)
        .or_else(|| first_robot_name(program))
        .unwrap_or_else(|| "program".into());
    let event = PlatformEvent::new(
        event_type,
        "spanda-interpreter",
        json!({
            "mission": mission_key,
            "success": success,
            "robot_count": robot_count(program),
        }),
    )
    .with_entity_id(format!("mission/{mission_key}"));
    if let Some(rt) = audit {
        let _ = rt.record_platform_event(&event);
    }
    telemetry.record_platform_event(
        event.event_type.as_str(),
        &event.source,
        event.entity_id.as_deref(),
        event.payload.clone(),
        event.timestamp.timestamp_millis() as f64,
    );
}

pub(crate) fn emit_mission_started(
    audit: Option<&mut AuditRuntime>,
    telemetry: &dyn TelemetrySink,
    program: &Program,
    trace_source: Option<&str>,
) {
    emit_mission_platform_event(
        audit,
        telemetry,
        names::MISSION_STARTED,
        program,
        trace_source,
        true,
    );
}

pub(crate) fn emit_mission_aborted(
    audit: Option<&mut AuditRuntime>,
    telemetry: &dyn TelemetrySink,
    program: &Program,
    trace_source: Option<&str>,
    reason: Option<&str>,
) {
    let mission_key = trace_source
        .map(str::to_string)
        .or_else(|| first_robot_name(program))
        .unwrap_or_else(|| "program".into());
    let event = PlatformEvent::new(
        names::MISSION_ABORTED,
        "spanda-interpreter",
        json!({
            "mission": mission_key,
            "reason": reason.unwrap_or("runtime_error"),
            "robot_count": robot_count(program),
        }),
    )
    .with_entity_id(format!("mission/{mission_key}"));
    if let Some(rt) = audit {
        let _ = rt.record_platform_event(&event);
    }
    telemetry.record_platform_event(
        event.event_type.as_str(),
        &event.source,
        event.entity_id.as_deref(),
        event.payload.clone(),
        event.timestamp.timestamp_millis() as f64,
    );
}

pub(crate) fn emit_mission_completed(
    audit: Option<&mut AuditRuntime>,
    telemetry: &dyn TelemetrySink,
    program: &Program,
    trace_source: Option<&str>,
    success: bool,
) {
    emit_mission_platform_event(
        audit,
        telemetry,
        names::MISSION_COMPLETED,
        program,
        trace_source,
        success,
    );
}

/// Record recovery lifecycle platform events for runtime recovery execution.
pub(crate) fn emit_recovery_triggered(issue: &str, plan: &str) {
    let event = PlatformEvent::new(
        names::RECOVERY_TRIGGERED,
        "spanda-interpreter",
        json!({
            "plan": plan,
            "fault": issue,
        }),
    )
    .with_entity_id(format!("runtime/{issue}"));
    publish_platform_event(None, &event);
}

/// Record recovery completion or failure after plan execution.
pub(crate) fn emit_recovery_outcome(issue: &str, plan: &str, status: RecoveryStatus) {
    let (event_type, label) = match status {
        RecoveryStatus::Success | RecoveryStatus::PartialSuccess => {
            (names::RECOVERY_COMPLETED, "success")
        }
        _ => (names::RECOVERY_FAILED, "failed"),
    };
    let event = PlatformEvent::new(
        event_type,
        "spanda-interpreter",
        json!({
            "plan": plan,
            "fault": issue,
            "status": label,
            "recovery_status": format!("{status:?}"),
        }),
    )
    .with_entity_id(format!("runtime/{issue}"));
    publish_platform_event(None, &event);
}

/// Record mission pause during recovery or reliability actions.
pub(crate) fn emit_mission_paused(reason: &str) {
    let event = PlatformEvent::new(
        names::MISSION_PAUSED,
        "spanda-interpreter",
        json!({ "reason": reason }),
    )
    .with_entity_id("mission/active");
    publish_platform_event(None, &event);
}

/// Record operating mode transition into a degraded posture.
pub(crate) fn emit_degraded_mode_entered(mode: &str, trigger: &str, entity_id: &str) {
    let event = PlatformEvent::new(
        names::DEGRADED_MODE_ENTERED,
        "spanda-interpreter",
        json!({
            "mode": mode,
            "trigger": trigger,
        }),
    )
    .with_entity_id(entity_id);
    publish_platform_event(None, &event);
}

fn robot_count(program: &Program) -> usize {
    let Program::Program { robots, .. } = program;
    robots.len()
}

fn first_robot_name(program: &Program) -> Option<String> {
    let Program::Program { robots, .. } = program;
    robots.first().map(|robot| match robot {
        RobotDecl::RobotDecl { name, .. } => name.clone(),
    })
}
