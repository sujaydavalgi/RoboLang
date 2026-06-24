//! Persistent append-only telemetry storage for devices, sensors, and heartbeats.
//!
//! Events are written to `.spanda/telemetry-store.jsonl` with a heartbeat index
//! sidecar at `.spanda/telemetry-heartbeats.json`. Enable with `--persist-telemetry`
//! or `SPANDA_TELEMETRY_STORE=1`.

pub mod error;
pub mod prometheus;
pub mod record;
pub mod store;

pub use error::{TelemetryStoreError, TelemetryStoreResult};
pub use record::{HeartbeatIndex, TelemetryEvent};
pub use prometheus::render_prometheus;
pub use store::{
    append_event, begin_run_session, configure_session_persist, default_heartbeat_index_path,
    default_store_path, end_run_session, env_persist_enabled, global_store, is_heartbeat_metric,
    persist_enabled, record_device_heartbeat, record_device_telemetry, record_health_event,
    record_sensor_reading, record_task_heartbeat, record_topic_publish, resolve_heartbeat_index_path,
    resolve_store_path, wall_timestamp_ms, PersistentTelemetryStore, TelemetryQuery, TelemetryStats,
};
