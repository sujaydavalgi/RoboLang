//! Bridge implementing `FleetTelemetryRuntime` for spanda-telemetry-store merge engine.

use spanda_runtime::fleet_telemetry_runtime::{set_fleet_telemetry_runtime, FleetTelemetryRuntime};
use std::collections::HashMap;
use std::sync::Arc;

/// Concrete implementation of FleetTelemetryRuntime backed by the real telemetry-store engine.
#[derive(Debug, Default, Clone, Copy)]
pub struct TelemetryStoreFleetRuntime;

impl FleetTelemetryRuntime for TelemetryStoreFleetRuntime {
    fn merge_fleet_otlp_json(&self, shards: &HashMap<String, String>) -> Result<String, String> {
        // Convert the map of robot_id → otlp_json into FleetTelemetryShard slices and merge.
        let shard_vec: Vec<crate::fleet::FleetTelemetryShard> = shards
            .iter()
            .map(|(robot_id, otlp_json)| crate::fleet::FleetTelemetryShard {
                robot_id: robot_id.clone(),
                otlp_json: otlp_json.clone(),
            })
            .collect();
        crate::fleet::merge_fleet_otlp_json(&shard_vec).map_err(|e| e.to_string())
    }
}

/// Register the real fleet telemetry runtime with the global OnceLock.
///
/// Parameters:
/// None.
///
/// Returns:
/// Unit; idempotent (subsequent calls are silently ignored).
///
/// Options:
/// None.
///
/// Example:
/// spanda_telemetry_store::fleet_runtime_bridge::register();
pub fn register() {
    // Inject the real telemetry engine into the global fleet runtime slot.
    set_fleet_telemetry_runtime(Arc::new(TelemetryStoreFleetRuntime));
}
