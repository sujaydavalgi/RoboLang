//! Persistent append-only telemetry storage for devices, sensors, and heartbeats.
//!
//! Events are written to `.spanda/telemetry-store.jsonl` (default) or
//! `.spanda/telemetry-store.db` when `SPANDA_TELEMETRY_BACKEND=sqlite`, with a
//! heartbeat index sidecar or SQLite table. Enable with `--persist-telemetry` or
//! `SPANDA_TELEMETRY_STORE=1`.

pub mod error;
pub mod fleet;
#[cfg(feature = "push")]
pub mod fleet_ingest;
pub mod memory;
pub mod otlp;
pub mod prometheus;
#[cfg(feature = "push")]
pub mod push;
pub mod record;
pub mod serve;
#[cfg(feature = "sqlite")]
pub mod sqlite;
pub mod device_runtime_bridge;
pub mod fleet_runtime_bridge;
pub mod runtime_bridge;
pub mod store;

pub use error::{TelemetryStoreError, TelemetryStoreResult};
pub use fleet::{merge_fleet_otlp_json, shards_from_map, FleetTelemetryShard};
#[cfg(feature = "push")]
pub use fleet_ingest::{
    env_fleet_auto_ingest_enabled, env_fleet_mesh_token, env_fleet_mesh_url, env_robot_id,
    ingest_global_store_to_fleet_mesh, maybe_auto_ingest_fleet_after_session,
};
pub use memory::{
    memory_append_json_line, memory_append_runtime_metrics, memory_clear, memory_render_otlp_json,
    memory_render_prometheus, memory_stats, MemoryTelemetryStore,
};
pub use otlp::{render_otlp_from_events, render_otlp_json};
pub use prometheus::{render_prometheus, render_prometheus_from_events};
#[cfg(feature = "push")]
pub use push::{
    env_auto_push_enabled, env_otlp_endpoint, env_otlp_token, env_push_interval_ms,
    maybe_auto_push_after_session, push_global_store, push_otlp_json, run_otlp_push_loop,
    OtlpPushOptions,
};
pub use record::{HeartbeatIndex, TelemetryEvent};
pub use serve::{run_telemetry_server, TelemetryServeOptions};
#[cfg(feature = "sqlite")]
pub use sqlite::{default_sqlite_store_path, env_backend_sqlite, resolve_sqlite_path};
pub use device_runtime_bridge::TelemetryStoreDeviceSink;
pub use runtime_bridge::TelemetryStoreSink;
pub use store::{
    append_event, begin_run_session, configure_session_persist, default_heartbeat_index_path,
    default_store_path, end_run_session, env_persist_enabled, global_store, is_heartbeat_metric,
    persist_enabled, record_device_heartbeat, record_device_telemetry, record_health_event,
    record_sensor_reading, record_task_heartbeat, record_topic_publish, record_platform_event,
    resolve_heartbeat_index_path, resolve_store_path, stats_from_events, wall_timestamp_ms,
    PersistentTelemetryStore, TelemetryQuery, TelemetrySessionSummary, TelemetryStats,
    TelemetryStoreInfo,
};
