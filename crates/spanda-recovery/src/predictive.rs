//! Predictive recovery — early failure detection from telemetry indicators.
//!
use crate::types::{OrchestratorStrategy, PredictiveIndicator};
use serde_json::Value;
use spanda_config::entity::EntityRegistry;

/// Known predictive indicator patterns and their recommended actions.
const INDICATOR_PATTERNS: &[(&str, &str, &str, OrchestratorStrategy)] = &[
    (
        "memory_leak",
        "warning",
        "restart_component",
        OrchestratorStrategy::RestartComponent,
    ),
    (
        "increasing_latency",
        "warning",
        "graceful_degradation",
        OrchestratorStrategy::GracefulDegradation,
    ),
    (
        "cpu_spike",
        "warning",
        "restart_component",
        OrchestratorStrategy::RestartComponent,
    ),
    (
        "battery_degradation",
        "warning",
        "graceful_degradation",
        OrchestratorStrategy::GracefulDegradation,
    ),
    (
        "temperature_increase",
        "critical",
        "safe_shutdown",
        OrchestratorStrategy::SafeShutdown,
    ),
    (
        "connectivity_instability",
        "warning",
        "reconnect",
        OrchestratorStrategy::Reconnect,
    ),
    (
        "packet_loss",
        "warning",
        "switch_network",
        OrchestratorStrategy::SwitchNetwork,
    ),
    (
        "sensor_drift",
        "warning",
        "reinitialize",
        OrchestratorStrategy::Reinitialize,
    ),
    (
        "repeated_retries",
        "warning",
        "restart_component",
        OrchestratorStrategy::RestartComponent,
    ),
    (
        "crash_frequency",
        "critical",
        "restart_robot",
        OrchestratorStrategy::RestartRobot,
    ),
];

/// Scan telemetry for predictive recovery indicators.
pub fn scan_predictive_indicators(
    registry: &EntityRegistry,
    telemetry: Option<&Value>,
) -> Vec<PredictiveIndicator> {
    // Scan telemetry for predictive recovery indicators.
    //
    // Parameters:
    // - `registry` — entity registry
    // - `telemetry` — optional telemetry JSON snapshot
    //
    // Returns:
    // Predictive indicators requiring preventative recovery.
    //
    // Options:
    // None.
    //
    // Example:
    // let indicators = scan_predictive_indicators(&registry, telemetry.as_ref());

    let mut indicators = Vec::new();

    if let Some(data) = telemetry {
        scan_telemetry_object(data, "", &mut indicators);
    }

    // Heuristic scan from entity health degradation.
    for entity in registry.list() {
        let health = format!("{:?}", entity.health_status);
        if health.contains("Degraded") || health.contains("Warning") {
            indicators.push(PredictiveIndicator {
                indicator: "health_degradation".into(),
                entity_id: entity.id.clone(),
                severity: "warning".into(),
                confidence: 0.7,
                recommended_action: OrchestratorStrategy::GracefulDegradation,
                preventative: true,
            });
        }
        if health.contains("Critical") {
            indicators.push(PredictiveIndicator {
                indicator: "health_critical".into(),
                entity_id: entity.id.clone(),
                severity: "critical".into(),
                confidence: 0.9,
                recommended_action: OrchestratorStrategy::RestartRobot,
                preventative: false,
            });
        }
    }

    indicators.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    indicators
}

/// Determine if preventative recovery should be triggered.
pub fn should_trigger_preventative(indicators: &[PredictiveIndicator]) -> bool {
    indicators
        .iter()
        .any(|i| i.preventative && i.confidence >= 0.75)
}

fn scan_telemetry_object(value: &Value, prefix: &str, out: &mut Vec<PredictiveIndicator>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                check_metric_pattern(&path, val, out);
                scan_telemetry_object(val, &path, out);
            }
        }
        _ => {}
    }
}

fn check_metric_pattern(path: &str, value: &Value, out: &mut Vec<PredictiveIndicator>) {
    let path_lower = path.to_ascii_lowercase();
    for (pattern, severity, _action_name, strategy) in INDICATOR_PATTERNS {
        if path_lower.contains(pattern) {
            let confidence = metric_confidence(value);
            if confidence >= 0.5 {
                let entity_id = extract_entity_id(path);
                out.push(PredictiveIndicator {
                    indicator: (*pattern).into(),
                    entity_id,
                    severity: (*severity).into(),
                    confidence,
                    recommended_action: strategy.clone(),
                    preventative: confidence < 0.85,
                });
            }
        }
    }
}

fn metric_confidence(value: &Value) -> f64 {
    match value {
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if f > 90.0 {
                    0.9
                } else if f > 70.0 {
                    0.75
                } else {
                    0.6
                }
            } else {
                0.5
            }
        }
        Value::Bool(b) => {
            if *b {
                0.8
            } else {
                0.3
            }
        }
        Value::String(s) => {
            let lower = s.to_ascii_lowercase();
            if lower.contains("critical") || lower.contains("fail") {
                0.85
            } else if lower.contains("warn") || lower.contains("degrad") {
                0.7
            } else {
                0.5
            }
        }
        _ => 0.5,
    }
}

fn extract_entity_id(path: &str) -> String {
    path.split('.')
        .find(|seg| {
            seg.contains("robot")
                || seg.contains("device")
                || seg.contains("sensor")
                || seg.contains("fleet")
        })
        .unwrap_or("unknown")
        .to_string()
}
