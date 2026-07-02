//! CLI-injected telemetry sink wiring telemetry store into the interpreter.
//!
use std::sync::Arc;

use spanda_runtime::device_telemetry_sink::set_device_telemetry_sink;
use spanda_runtime::telemetry_sink::SharedTelemetrySink;
use spanda_telemetry_store::{TelemetryStoreDeviceSink, TelemetryStoreSink};

/// Shared telemetry sink for default `spanda` CLI runs.
pub fn default_telemetry_sink() -> SharedTelemetrySink {
    set_device_telemetry_sink(Arc::new(TelemetryStoreDeviceSink));
    crate::plugin_runtime::register_platform_event_runtime();
    Arc::new(TelemetryStoreSink)
}
