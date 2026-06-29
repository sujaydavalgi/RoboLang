//! Unified entity trust — routes trust engines through [`EntityRegistry`].
//!
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use spanda_config::{
    evaluate_quarantine_policy, EntityKind, EntityRecord, EntityRegistry, EntityTrustStatus,
    ResolvedSystemConfig,
};
use spanda_package::evaluate_package_trust;

/// Options for entity-scoped trust evaluation.
#[derive(Debug, Clone, Default)]
pub struct EntityTrustOptions {
    pub program: Option<Program>,
    pub program_source: Option<String>,
    pub program_label: Option<String>,
}

/// Trust category score for an entity evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityTrustCategory {
    pub name: String,
    pub score: u32,
    pub passed: bool,
    pub detail: String,
}

/// Unified trust report for any entity kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityTrustReport {
    pub entity_id: String,
    pub entity_type: String,
    pub trust_status: String,
    pub lifecycle_state: String,
    pub score: Option<u32>,
    pub passed: bool,
    pub categories: Vec<EntityTrustCategory>,
    pub certificates: Vec<String>,
    pub tamper_status: Option<String>,
    pub threat_status: Option<String>,
    pub sources: Vec<String>,
}

/// Evaluate trust for any entity in the registry.
pub fn evaluate_entity_trust(
    entity_id: &str,
    registry: &EntityRegistry,
    config: &ResolvedSystemConfig,
    options: &EntityTrustOptions,
) -> Option<EntityTrustReport> {
    // Evaluate trust posture for one entity using kind-appropriate engines.
    //
    // Parameters:
    // - `entity_id` — target entity identifier
    // - `registry` — unified entity registry projection
    // - `config` — resolved system configuration
    // - `options` — optional program for composite trust
    //
    // Returns:
    // Trust report, or `None` when the entity id is unknown.
    //
    // Options:
    // `EntityTrustOptions::program` enables composite trust when applicable.
    //
    // Example:
    // let report = evaluate_entity_trust("rover-001", &registry, &cfg, &opts)?;

    let entity = registry.get(entity_id)?;
    let mut categories = Vec::new();
    let mut sources = vec!["entity_snapshot".into()];
    let certificates = entity
        .security
        .as_ref()
        .map(|s| s.certificates.clone())
        .unwrap_or_default();
    let mut tamper_status = None;
    let threat_status = None;

    snapshot_trust_categories(entity, &mut categories);

    match &entity.entity_type {
        EntityKind::Package | EntityKind::Provider => {
            sources.push("package_trust".into());
            evaluate_package_entity_trust(entity, config, &mut categories);
        }
        kind if is_device_kind(kind) => {
            sources.push("device_pool".into());
            evaluate_device_entity_trust(entity, config, &mut categories);
        }
        EntityKind::Robot | EntityKind::Drone | EntityKind::Vehicle => {
            sources.push("device_pool".into());
            evaluate_robot_entity_trust(entity, registry, config, &mut categories);
        }
        _ => {}
    }

    if let (Some(program), Some(source)) = (&options.program, &options.program_source) {
        sources.push("composite_trust".into());
        let label = options.program_label.as_deref().unwrap_or("program.sd");
        let composite = crate::composite::evaluate_composite_trust(
            program,
            source,
            label,
            &crate::composite::CompositeTrustOptions {
                project_root: Some(config.project_root.clone()),
            },
        );
        categories.push(EntityTrustCategory {
            name: "composite".into(),
            score: composite.score,
            passed: composite.passed,
            detail: format!("Composite trust tier {}", composite.tier),
        });
        tamper_status = Some(composite.integrity_status.clone());
    }

    let passed = categories.iter().all(|c| c.passed)
        && !matches!(
            entity.trust_status,
            EntityTrustStatus::Untrusted | EntityTrustStatus::Compromised
        );
    let score = trust_score(&categories, entity);
    Some(EntityTrustReport {
        entity_id: entity.id.clone(),
        entity_type: entity.kind().to_string(),
        trust_status: entity.trust_status.as_str().to_string(),
        lifecycle_state: entity.lifecycle_state.as_str().to_string(),
        score: Some(score),
        passed,
        categories,
        certificates,
        tamper_status,
        threat_status,
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

fn snapshot_trust_categories(entity: &EntityRecord, categories: &mut Vec<EntityTrustCategory>) {
    let passed = !matches!(
        entity.trust_status,
        EntityTrustStatus::Untrusted | EntityTrustStatus::Compromised
    );
    categories.push(EntityTrustCategory {
        name: "entity_snapshot".into(),
        score: if passed { 100 } else { 20 },
        passed,
        detail: format!("Entity trust is {}", entity.trust_status.as_str()),
    });
    if let Some(identity) = entity.security.as_ref().and_then(|s| s.identity.as_deref()) {
        categories.push(EntityTrustCategory {
            name: "identity".into(),
            score: 90,
            passed: true,
            detail: format!("Identity id {identity}"),
        });
    }
}

fn evaluate_package_entity_trust(
    entity: &EntityRecord,
    config: &ResolvedSystemConfig,
    categories: &mut Vec<EntityTrustCategory>,
) {
    let name = entity
        .package
        .as_deref()
        .or(entity.provider.as_deref())
        .unwrap_or(entity.id.as_str())
        .trim_start_matches("package-")
        .trim_start_matches("provider-");
    let report = evaluate_package_trust(name, None, Some(&config.project_root));
    categories.push(EntityTrustCategory {
        name: "package".into(),
        score: report.score,
        passed: report.passed,
        detail: format!("Package trust tier {}", report.tier),
    });
}

fn evaluate_device_entity_trust(
    entity: &EntityRecord,
    config: &ResolvedSystemConfig,
    categories: &mut Vec<EntityTrustCategory>,
) {
    if let Some(device) = config
        .device_registry
        .devices
        .iter()
        .find(|d| d.id == entity.id)
    {
        let quarantine = evaluate_quarantine_policy(device);
        categories.push(EntityTrustCategory {
            name: "quarantine".into(),
            score: if quarantine.quarantined { 0 } else { 100 },
            passed: !quarantine.quarantined,
            detail: quarantine.reasons.join(", "),
        });
        categories.push(EntityTrustCategory {
            name: "device_trust".into(),
            score: if device.trust_level_enum().is_operational() {
                100
            } else {
                40
            },
            passed: device.trust_level_enum().is_operational(),
            detail: format!("Trust level {:?}", device.trust_level),
        });
    }
}

fn evaluate_robot_entity_trust(
    entity: &EntityRecord,
    registry: &EntityRegistry,
    config: &ResolvedSystemConfig,
    categories: &mut Vec<EntityTrustCategory>,
) {
    for device in devices_for_robot(registry, config, &entity.id) {
        let quarantine = evaluate_quarantine_policy(&device);
        if quarantine.quarantined {
            categories.push(EntityTrustCategory {
                name: format!("device:{}", device.id),
                score: 0,
                passed: false,
                detail: quarantine.reasons.join(", "),
            });
        }
    }
}

fn trust_score(categories: &[EntityTrustCategory], entity: &EntityRecord) -> u32 {
    if categories.is_empty() {
        return match entity.trust_status {
            EntityTrustStatus::Verified | EntityTrustStatus::Trusted => 100,
            EntityTrustStatus::Unknown => 50,
            EntityTrustStatus::Untrusted => 30,
            EntityTrustStatus::Compromised => 0,
        };
    }
    let total: u32 = categories.iter().map(|c| c.score).sum();
    total / categories.len() as u32
}

fn devices_for_robot(
    registry: &EntityRegistry,
    config: &ResolvedSystemConfig,
    robot_id: &str,
) -> Vec<spanda_config::DeviceIdentityRecord> {
    let mut device_ids: Vec<String> = registry
        .relationships_for(robot_id)
        .iter()
        .filter(|r| r.from_id == robot_id)
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
    fn evaluate_robot_trust_returns_report() {
        let config = warehouse_config();
        let registry = build_entity_registry(&config);
        let report = evaluate_entity_trust(
            "rover-001",
            &registry,
            &config,
            &EntityTrustOptions::default(),
        )
        .expect("rover-001");
        assert_eq!(report.entity_id, "rover-001");
        assert!(report.score.is_some());
    }
}
