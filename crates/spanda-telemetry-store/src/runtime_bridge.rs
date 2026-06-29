//! Telemetry-store-backed implementation of the runtime telemetry boundary.
//!
use spanda_runtime::telemetry::RuntimeTelemetry;
use spanda_runtime::telemetry_sink::TelemetrySink;
use spanda_runtime::value::RuntimeValue;

/// Full telemetry persistence delegating to `spanda-telemetry-store`.
#[derive(Debug, Default, Clone, Copy)]
pub struct TelemetryStoreSink;

impl TelemetrySink for TelemetryStoreSink {
    fn configure_session_persist(&self, enabled: bool) {
        crate::configure_session_persist(enabled);
    }

    fn begin_run_session(&self, source: Option<&str>) {
        let _ = crate::begin_run_session(source);
    }

    fn end_run_session(
        &self,
        mission_trace_path: Option<&str>,
        metrics: Option<&RuntimeTelemetry>,
        timestamp_ms: f64,
    ) {
        let _ = crate::end_run_session(mission_trace_path, metrics, timestamp_ms);
    }

    fn record_sensor_reading(
        &self,
        sensor_id: &str,
        sensor_type: &str,
        value: &RuntimeValue,
        timestamp_ms: f64,
        robot_id: Option<&str>,
    ) {
        let _ = crate::record_sensor_reading(sensor_id, sensor_type, value, timestamp_ms, robot_id);
    }

    fn record_health_event(&self, target: &str, status: &str, timestamp_ms: f64) {
        let _ = crate::record_health_event(target, status, timestamp_ms);
    }

    fn record_platform_event(
        &self,
        event_type: &str,
        source: &str,
        entity_id: Option<&str>,
        payload: serde_json::Value,
        timestamp_ms: f64,
    ) {
        use chrono::{TimeZone, Utc};
        let mut event =
            spanda_audit::PlatformEvent::new(event_type, source, payload);
        if let Some(id) = entity_id {
            event = event.with_entity_id(id);
        }
        if let Some(ts) = Utc.timestamp_millis_opt(timestamp_ms as i64).single() {
            event = event.with_timestamp(ts);
        }
        let _ = crate::record_platform_event(&event);
    }

    fn record_task_heartbeat(
        &self,
        task_name: &str,
        timestamp_ms: f64,
        robot_id: Option<&str>,
        history_interval_ms: f64,
    ) {
        let _ = crate::record_task_heartbeat(
            task_name,
            timestamp_ms,
            robot_id,
            history_interval_ms,
        );
    }

    fn record_topic_publish(
        &self,
        robot_id: Option<&str>,
        topic_path: &str,
        value: &RuntimeValue,
        timestamp_ms: f64,
    ) {
        let _ = crate::record_topic_publish(robot_id, topic_path, value, timestamp_ms);
    }
}
