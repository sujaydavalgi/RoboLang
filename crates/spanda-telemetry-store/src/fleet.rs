//! Fleet OTLP aggregation across multiple robot telemetry shards.

use crate::error::{TelemetryStoreError, TelemetryStoreResult};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// One robot OTLP/JSON snapshot ingested by the fleet mesh.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FleetTelemetryShard {
    pub robot_id: String,
    pub otlp_json: String,
}

/// Merge per-robot OTLP/JSON bodies into a single `resourceMetrics` export.
pub fn merge_fleet_otlp_json(shards: &[FleetTelemetryShard]) -> TelemetryStoreResult<String> {
    let mut resource_metrics: Vec<Value> = Vec::new();
    for shard in shards {
        let parsed: Value = serde_json::from_str(&shard.otlp_json).map_err(|error| {
            TelemetryStoreError::Serialization(format!(
                "invalid OTLP JSON for robot {}: {error}",
                shard.robot_id
            ))
        })?;
        let Some(entries) = parsed.get("resourceMetrics").and_then(Value::as_array) else {
            continue;
        };
        for entry in entries {
            let mut resource_entry = entry.clone();
            if let Some(resource) = resource_entry.get_mut("resource") {
                let mut attributes = resource
                    .get("attributes")
                    .and_then(|value| value.as_array())
                    .cloned()
                    .unwrap_or_default();
                attributes.push(json!({
                    "key": "spanda.robot.id",
                    "value": { "stringValue": shard.robot_id }
                }));
                resource["attributes"] = Value::Array(attributes);
            }
            resource_metrics.push(resource_entry);
        }
    }
    serde_json::to_string_pretty(&json!({ "resourceMetrics": resource_metrics }))
        .map_err(|error| TelemetryStoreError::Serialization(error.to_string()))
}

/// Build fleet shards from a robot-id → OTLP JSON map.
pub fn shards_from_map(shards: &BTreeMap<String, String>) -> Vec<FleetTelemetryShard> {
    shards
        .iter()
        .map(|(robot_id, otlp_json)| FleetTelemetryShard {
            robot_id: robot_id.clone(),
            otlp_json: otlp_json.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_fleet_otlp_adds_robot_attributes() {
        let body = r#"{"resourceMetrics":[{"resource":{"attributes":[]},"scopeMetrics":[]}]}"#;
        let merged = merge_fleet_otlp_json(&[FleetTelemetryShard {
            robot_id: "rover-a".into(),
            otlp_json: body.into(),
        }])
        .unwrap();
        assert!(merged.contains("rover-a"));
        assert!(merged.contains("spanda.robot.id"));
    }
}
