//! Bridge implementing `FleetTamperRuntime` for spanda-tamper real correlation engine.

use spanda_runtime::fleet_tamper_runtime::{set_fleet_tamper_runtime, FleetTamperRuntime};
use std::collections::HashMap;
use std::sync::Arc;

/// Concrete implementation of FleetTamperRuntime backed by the real spanda-tamper engine.
#[derive(Debug, Default, Clone, Copy)]
pub struct TamperBackedFleetRuntime;

impl FleetTamperRuntime for TamperBackedFleetRuntime {
    fn correlate_fleet_tamper_traces_json(
        &self,
        fleet_name: &str,
        shards: &HashMap<String, String>,
    ) -> Result<String, String> {
        // Deserialise each shard, correlate via real engine, and return serialised report.
        let mut traces = Vec::new();
        for (robot_id, trace_json) in shards {
        let trace: crate::runtime::MissionTrace = serde_json::from_str(trace_json)
            .map_err(|e| format!("parse {robot_id}: {e}"))?;
        let label = format!("{robot_id}.trace");
        traces.push((robot_id.clone(), trace, label));
    }
    let report = crate::fleet::correlate_fleet_tamper_traces(fleet_name, &traces);
    serde_json::to_string(&report).map_err(|e| e.to_string())
}

    fn format_fleet_tamper_report_json(&self, report_json: &str) -> String {
        // Deserialise the report and format it as human-readable text.
        match serde_json::from_str::<crate::fleet::FleetTamperReport>(report_json) {
            Ok(report) => crate::fleet::format_fleet_tamper_report(&report, crate::detect::TamperFormat::Text),
            Err(e) => format!("Fleet tamper report parse error: {e}"),
        }
    }
}

/// Register the real tamper runtime with the global OnceLock.
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
/// spanda_tamper::runtime_bridge::register();
pub fn register() {
    // Inject the real tamper engine into the global runtime slot.
    set_fleet_tamper_runtime(Arc::new(TamperBackedFleetRuntime));
}
