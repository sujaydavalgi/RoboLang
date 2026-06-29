//! Unified entity health — routes health diagnostics through [`EntityRegistry`].
//!
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use spanda_capability::{evaluate_health_checks, evaluate_runtime_health};
use spanda_config::{
    evaluate_device_readiness, EntityHealthStatus, EntityKind, EntityRecord, EntityRegistry,
    EntityRelationshipKind, ResolvedSystemConfig,
};

/// Options for entity-scoped health evaluation.
#[derive(Debug, Clone, Default)]
pub struct EntityHealthOptions {
    pub program: Option<Program>,
    pub now_ms: f64,
}

/// Health diagnostic for an entity evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityHealthDiagnostic {
    pub category: String,
    pub severity: String,
    pub message: String,
}

/// Unified health report for any entity kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityHealthReport {
    pub entity_id: String,
    pub entity_type: String,
    pub health_status: String,
    pub lifecycle_state: String,
    pub diagnostics: Vec<EntityHealthDiagnostic>,
    pub metrics: EntityHealthMetrics,
    pub children_checked: usize,
    pub sources: Vec<String>,
}

/// Lightweight health metrics rollup for an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EntityHealthMetrics {
    pub blocked_devices: u32,
    pub total_devices: u32,
    pub health_checks_passed: u32,
    pub health_checks_failed: u32,
}

/// Evaluate health for any entity in the registry.
pub fn evaluate_entity_health(
    entity_id: &str,
    registry: &EntityRegistry,
    config: &ResolvedSystemConfig,
    options: &EntityHealthOptions,
) -> Option<EntityHealthReport> {
    // Evaluate health posture for one entity using kind-appropriate diagnostics.
    //
    // Parameters:
    // - `entity_id` — target entity identifier
    // - `registry` — unified entity registry projection
    // - `config` — resolved system configuration
    // - `options` — optional program for runtime health checks
    //
    // Returns:
    // Health report, or `None` when the entity id is unknown.
    //
    // Options:
    // `EntityHealthOptions::program` enables declared health_check evaluation.
    //
    // Example:
    // let report = evaluate_entity_health("rover-001", &registry, &cfg, &opts)?;

    let entity = registry.get(entity_id)?;
    let mut diagnostics = Vec::new();
    let mut metrics = EntityHealthMetrics::default();
    let mut sources = vec!["entity_snapshot".into()];

    snapshot_health_diagnostics(entity, &mut diagnostics);

    let children_checked = match &entity.entity_type {
        EntityKind::Robot | EntityKind::Drone | EntityKind::Vehicle => {
            sources.push("device_pool".into());
            evaluate_robot_health(
                entity,
                registry,
                config,
                options,
                &mut diagnostics,
                &mut metrics,
            )
        }
        EntityKind::Fleet | EntityKind::Swarm => rollup_member_health(
            entity,
            registry,
            config,
            options,
            &mut diagnostics,
            &mut metrics,
        ),
        kind if is_device_kind(kind) => {
            sources.push("device_pool".into());
            evaluate_device_health(entity, config, options.now_ms, &mut diagnostics);
            0
        }
        EntityKind::Human => {
            evaluate_human_health(entity, config, &mut diagnostics);
            0
        }
        EntityKind::Facility | EntityKind::Building | EntityKind::Zone => {
            rollup_child_health(entity, registry, &mut diagnostics)
        }
        _ => 0,
    };

    if let Some(ref program) = options.program {
        sources.push("health_checks".into());
        use spanda_capability::HealthStatus;
        let report = evaluate_health_checks(program);
        metrics.health_checks_passed = report
            .checks
            .iter()
            .filter(|c| c.status == HealthStatus::Healthy)
            .count() as u32;
        metrics.health_checks_failed = report
            .checks
            .iter()
            .filter(|c| c.status != HealthStatus::Healthy)
            .count() as u32;
        for check in &report.checks {
            if check.status != HealthStatus::Healthy {
                push_diagnostic(
                    &mut diagnostics,
                    "health_check",
                    "warning",
                    format!("Health check '{}' failed for {}", check.name, check.target),
                );
            }
        }
        let runtime = evaluate_runtime_health(&[], &[], program);
        if matches!(
            runtime.overall,
            HealthStatus::Degraded | HealthStatus::Critical | HealthStatus::Failed
        ) {
            push_diagnostic(
                &mut diagnostics,
                "runtime",
                "warning",
                "Runtime health evaluation reports degraded posture",
            );
        }
    }

    Some(EntityHealthReport {
        entity_id: entity.id.clone(),
        entity_type: entity.kind().to_string(),
        health_status: entity.health_status.as_str().to_string(),
        lifecycle_state: entity.lifecycle_state.as_str().to_string(),
        diagnostics,
        metrics,
        children_checked,
        sources,
    })
}

fn is_device_kind(kind: &EntityKind) -> bool {
    matches!(
        kind,
        EntityKind::Device
            | EntityKind::Sensor
            | EntityKind::Actuator
            | EntityKind::Gateway
            | EntityKind::Controller
            | EntityKind::Wearable
            | EntityKind::MedicalDevice
            | EntityKind::Camera
            | EntityKind::Gps
            | EntityKind::Plc
            | EntityKind::Compute
            | EntityKind::ArDevice
            | EntityKind::VrDevice
            | EntityKind::IotDevice
            | EntityKind::DigitalTwin
    )
}

fn push_diagnostic(
    diagnostics: &mut Vec<EntityHealthDiagnostic>,
    category: &str,
    severity: &str,
    message: impl Into<String>,
) {
    diagnostics.push(EntityHealthDiagnostic {
        category: category.into(),
        severity: severity.into(),
        message: message.into(),
    });
}

fn snapshot_health_diagnostics(
    entity: &EntityRecord,
    diagnostics: &mut Vec<EntityHealthDiagnostic>,
) {
    match entity.health_status {
        EntityHealthStatus::Degraded | EntityHealthStatus::Warning => {
            push_diagnostic(
                diagnostics,
                "health",
                "warning",
                format!("Entity health is {}", entity.health_status.as_str()),
            );
        }
        EntityHealthStatus::Critical | EntityHealthStatus::Offline => {
            push_diagnostic(
                diagnostics,
                "health",
                "critical",
                format!("Entity health is {}", entity.health_status.as_str()),
            );
        }
        _ => {}
    }
}

fn evaluate_robot_health(
    entity: &EntityRecord,
    registry: &EntityRegistry,
    config: &ResolvedSystemConfig,
    options: &EntityHealthOptions,
    diagnostics: &mut Vec<EntityHealthDiagnostic>,
    metrics: &mut EntityHealthMetrics,
) -> usize {
    let devices = devices_for_robot(registry, config, &entity.id);
    metrics.total_devices = devices.len() as u32;
    for device in &devices {
        let health = evaluate_device_readiness(device, options.now_ms);
        if health.readiness_blocked {
            metrics.blocked_devices += 1;
            push_diagnostic(
                diagnostics,
                "device",
                "error",
                format!(
                    "Device '{}' unhealthy: {}",
                    device.id,
                    health.blockers.join(", ")
                ),
            );
        }
    }
    devices.len()
}

fn rollup_member_health(
    entity: &EntityRecord,
    registry: &EntityRegistry,
    config: &ResolvedSystemConfig,
    options: &EntityHealthOptions,
    diagnostics: &mut Vec<EntityHealthDiagnostic>,
    metrics: &mut EntityHealthMetrics,
) -> usize {
    let members: Vec<String> = registry
        .relationships_for(&entity.id)
        .iter()
        .filter(|r| {
            r.from_id == entity.id
                && matches!(
                    r.kind,
                    EntityRelationshipKind::Contains | EntityRelationshipKind::Owns
                )
        })
        .map(|r| r.to_id.clone())
        .collect();
    let mut checked = members.len();
    for member_id in members {
        if let Some(member) = registry.get(&member_id) {
            if matches!(
                member.health_status,
                EntityHealthStatus::Degraded
                    | EntityHealthStatus::Critical
                    | EntityHealthStatus::Offline
            ) {
                push_diagnostic(
                    diagnostics,
                    "fleet_member",
                    "warning",
                    format!(
                        "Member '{}' health is {}",
                        member_id,
                        member.health_status.as_str()
                    ),
                );
            }
            if member.entity_type == EntityKind::Robot {
                checked +=
                    evaluate_robot_health(member, registry, config, options, diagnostics, metrics);
            }
        }
    }
    checked
}

fn evaluate_device_health(
    entity: &EntityRecord,
    config: &ResolvedSystemConfig,
    now_ms: f64,
    diagnostics: &mut Vec<EntityHealthDiagnostic>,
) {
    if let Some(device) = config
        .device_registry
        .devices
        .iter()
        .find(|d| d.id == entity.id)
    {
        let health = evaluate_device_readiness(device, now_ms);
        if health.readiness_blocked {
            push_diagnostic(
                diagnostics,
                "device",
                "error",
                format!("Device unhealthy: {}", health.blockers.join(", ")),
            );
        }
        if !health.firmware_supported {
            push_diagnostic(diagnostics, "firmware", "warning", "Firmware unsupported");
        }
        if health.calibration_expired {
            push_diagnostic(diagnostics, "calibration", "warning", "Calibration expired");
        }
    }
}

fn evaluate_human_health(
    entity: &EntityRecord,
    config: &ResolvedSystemConfig,
    diagnostics: &mut Vec<EntityHealthDiagnostic>,
) {
    if let Some(human) = config
        .human_registry
        .humans
        .iter()
        .find(|h| h.id == entity.id)
    {
        if let Some(status) = human.health_status.as_deref() {
            if status != "healthy" && status != "ok" {
                push_diagnostic(
                    diagnostics,
                    "human",
                    "warning",
                    format!("Operator health status is {status}"),
                );
            }
        }
    }
}

fn rollup_child_health(
    entity: &EntityRecord,
    registry: &EntityRegistry,
    diagnostics: &mut Vec<EntityHealthDiagnostic>,
) -> usize {
    for child_id in &entity.children_ids {
        if let Some(child) = registry.get(child_id) {
            if matches!(
                child.health_status,
                EntityHealthStatus::Degraded
                    | EntityHealthStatus::Critical
                    | EntityHealthStatus::Offline
            ) {
                push_diagnostic(
                    diagnostics,
                    "child",
                    "warning",
                    format!(
                        "Child '{child_id}' health is {}",
                        child.health_status.as_str()
                    ),
                );
            }
        }
    }
    entity.children_ids.len()
}

fn devices_for_robot(
    registry: &EntityRegistry,
    config: &ResolvedSystemConfig,
    robot_id: &str,
) -> Vec<spanda_config::DeviceIdentityRecord> {
    let mut device_ids: Vec<String> = registry
        .relationships_for(robot_id)
        .iter()
        .filter(|r| {
            r.from_id == robot_id
                && matches!(
                    r.kind,
                    EntityRelationshipKind::Contains
                        | EntityRelationshipKind::DependsOn
                        | EntityRelationshipKind::ConnectedTo
                )
        })
        .map(|r| r.to_id.clone())
        .collect();
    if device_ids.is_empty() {
        if let Some(robot) = config
            .device_tree
            .fleet
            .as_ref()
            .and_then(|f| f.robots.iter().find(|r| r.id == robot_id))
        {
            if let Some(compute) = robot.compute.as_ref() {
                device_ids.extend(compute.devices.iter().map(|d| d.id.clone()));
            }
        }
    }
    device_ids
        .into_iter()
        .filter_map(|id| {
            config
                .device_registry
                .devices
                .iter()
                .find(|d| d.id == id)
                .cloned()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_config::{build_entity_registry, ConfigResolver};
    use std::path::PathBuf;

    fn warehouse_config() -> ResolvedSystemConfig {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../spanda-config/tests/fixtures/warehouse");
        ConfigResolver::new()
            .resolve_from_dir(&root)
            .expect("warehouse fixture")
    }

    #[test]
    fn evaluate_robot_health_returns_report() {
        let config = warehouse_config();
        let registry = build_entity_registry(&config);
        let report = evaluate_entity_health(
            "rover-001",
            &registry,
            &config,
            &EntityHealthOptions {
                now_ms: 0.0,
                ..Default::default()
            },
        )
        .expect("rover-001");
        assert_eq!(report.entity_id, "rover-001");
    }
}
