//! Unified entity verification — routes all verification engines through [`EntityRegistry`].
//!
//! Every managed object (robot, device, fleet, mission, human, package, provider, …)
//! can be verified via a single `verify_entity` entry point. Domain-specific engines
//! (hardware, mission, fleet, device pool, quarantine, config validation) are invoked
//! based on the entity kind and optional program context.
//!
use crate::fleet_verify::{verify_fleet, FleetVerifyFinding};
use crate::mission::{verify_mission, MissionVerificationReport};
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use spanda_config::{
    evaluate_device_readiness, evaluate_quarantine_policy, verify_with_system_config,
    EntityHealthStatus, EntityKind, EntityReadinessStatus, EntityRecord, EntityRegistry,
    EntityRelationshipKind, EntityTrustStatus, ResolvedSystemConfig, ValidationSeverity,
};
use spanda_hardware::{CompatItem, CompatSeverity, VerifyOptions};

/// Options for entity-scoped verification.
#[derive(Debug, Clone, Default)]
pub struct EntityVerifyOptions {
    /// Optional `.sd` program for hardware, mission, and fleet checks.
    pub program: Option<Program>,
    /// Wall-clock milliseconds for device calibration expiry checks.
    pub now_ms: f64,
    /// When true, also verify dependency and relationship targets.
    pub include_dependencies: bool,
}

/// Single finding from entity verification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityVerifyFinding {
    pub category: String,
    pub severity: String,
    pub message: String,
    pub source: String,
}

/// Unified verification report for any entity kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityVerifyReport {
    pub entity_id: String,
    pub entity_type: String,
    pub compatible: bool,
    pub findings: Vec<EntityVerifyFinding>,
    pub capabilities: Vec<String>,
    pub relationships_checked: usize,
    pub dependencies_checked: usize,
    pub health_status: String,
    pub readiness_status: String,
    pub trust_status: String,
}

/// Verify any entity in the registry using kind-appropriate engines.
pub fn verify_entity(
    entity_id: &str,
    registry: &EntityRegistry,
    config: &ResolvedSystemConfig,
    options: &EntityVerifyOptions,
) -> Option<EntityVerifyReport> {
    // Run unified verification for one entity id against registry and resolved config.
    //
    // Parameters:
    // - `entity_id` — target entity identifier
    // - `registry` — unified entity registry projection
    // - `config` — resolved system configuration
    // - `options` — optional program and dependency scope
    //
    // Returns:
    // Verification report, or `None` when the entity id is unknown.
    //
    // Options:
    // `EntityVerifyOptions::include_dependencies` traverses `depends_on` edges.
    //
    // Example:
    // let report = verify_entity("rover-001", &registry, &cfg, &opts)?;

    let entity = registry.get(entity_id)?;
    let mut findings = Vec::new();
    let relationships = registry.relationships_for(entity_id);
    let relationships_checked = relationships.len();

    snapshot_findings(entity, &mut findings);
    verify_relationship_targets(registry, entity_id, &relationships, &mut findings);

    let dependencies_checked = if options.include_dependencies {
        verify_dependency_chain(registry, entity_id, &mut findings)
    } else {
        0
    };

    match entity.entity_type {
        EntityKind::Robot | EntityKind::Drone | EntityKind::Vehicle => {
            verify_robot_entity(entity, registry, config, options, &mut findings);
        }
        EntityKind::Fleet | EntityKind::Swarm => {
            verify_fleet_entity(entity, registry, config, options, &mut findings);
        }
        EntityKind::Mission => {
            verify_mission_entity(entity, registry, options, &mut findings);
        }
        EntityKind::Human | EntityKind::Team => {
            verify_human_entity(entity, config, &mut findings);
        }
        EntityKind::Package => verify_package_entity(entity, config, &mut findings),
        EntityKind::Provider => verify_provider_entity(entity, config, &mut findings),
        EntityKind::Facility | EntityKind::Building | EntityKind::Zone | EntityKind::Hazard => {
            verify_facility_entity(entity, registry, &mut findings);
        }
        EntityKind::Organization => verify_organization_entity(entity, registry, &mut findings),
        _ if is_device_kind(&entity.entity_type) => {
            verify_device_entity(entity, config, options.now_ms, &mut findings);
        }
        _ => {
            generic_entity_checks(entity, &mut findings);
        }
    }

    merge_config_validation_findings(config, entity_id, &mut findings);

    let compatible = !findings.iter().any(|f| f.severity == "error");
    Some(EntityVerifyReport {
        entity_id: entity.id.clone(),
        entity_type: entity.kind().to_string(),
        compatible,
        capabilities: entity.capabilities.clone(),
        relationships_checked,
        dependencies_checked,
        health_status: entity.health_status.as_str().to_string(),
        readiness_status: entity.readiness_status.as_str().to_string(),
        trust_status: entity.trust_status.as_str().to_string(),
        findings,
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

fn push_finding(
    findings: &mut Vec<EntityVerifyFinding>,
    category: &str,
    severity: &str,
    message: impl Into<String>,
    source: &str,
) {
    findings.push(EntityVerifyFinding {
        category: category.into(),
        severity: severity.into(),
        message: message.into(),
        source: source.into(),
    });
}

fn snapshot_findings(entity: &EntityRecord, findings: &mut Vec<EntityVerifyFinding>) {
    if matches!(
        entity.health_status,
        EntityHealthStatus::Degraded | EntityHealthStatus::Critical | EntityHealthStatus::Offline
    ) {
        push_finding(
            findings,
            "health",
            "warning",
            format!(
                "Entity health is {} — review before deployment",
                entity.health_status.as_str()
            ),
            "entity_snapshot",
        );
    }
    if entity.readiness_status == EntityReadinessStatus::NotReady {
        push_finding(
            findings,
            "readiness",
            "error",
            "Entity readiness is not_ready",
            "entity_snapshot",
        );
    }
    if matches!(
        entity.trust_status,
        EntityTrustStatus::Untrusted | EntityTrustStatus::Compromised
    ) {
        push_finding(
            findings,
            "trust",
            "error",
            format!("Entity trust is {}", entity.trust_status.as_str()),
            "entity_snapshot",
        );
    }
}

fn verify_relationship_targets(
    registry: &EntityRegistry,
    entity_id: &str,
    relationships: &[&spanda_config::EntityRelationship],
    findings: &mut Vec<EntityVerifyFinding>,
) {
    for edge in relationships {
        let other = if edge.from_id == entity_id {
            &edge.to_id
        } else {
            &edge.from_id
        };
        if registry.get(other).is_none() {
            push_finding(
                findings,
                "relationship",
                "error",
                format!(
                    "Relationship {:?} references missing entity '{other}'",
                    edge.kind
                ),
                "entity_graph",
            );
        }
    }
}

fn verify_dependency_chain(
    registry: &EntityRegistry,
    entity_id: &str,
    findings: &mut Vec<EntityVerifyFinding>,
) -> usize {
    let chain = registry.dependency_chain(entity_id);
    for dep_id in &chain {
        if registry.get(dep_id).is_none() {
            push_finding(
                findings,
                "dependency",
                "error",
                format!("Dependency chain references missing entity '{dep_id}'"),
                "entity_graph",
            );
        }
    }
    chain.len()
}

fn verify_robot_entity(
    entity: &EntityRecord,
    registry: &EntityRegistry,
    config: &ResolvedSystemConfig,
    options: &EntityVerifyOptions,
    findings: &mut Vec<EntityVerifyFinding>,
) {
    let robot_id = &entity.id;
    for device in devices_for_entity(registry, robot_id, config) {
        let health = evaluate_device_readiness(&device, options.now_ms);
        if health.readiness_blocked {
            push_finding(
                findings,
                "device",
                "error",
                format!(
                    "Device '{}' blocks readiness: {}",
                    device.id,
                    health.blockers.join(", ")
                ),
                "device_pool",
            );
        }
        let quarantine = evaluate_quarantine_policy(&device);
        if quarantine.quarantined {
            push_finding(
                findings,
                "quarantine",
                "error",
                format!(
                    "Device '{}' is quarantined: {}",
                    device.id,
                    quarantine.reasons.join(", ")
                ),
                "device_pool",
            );
        }
    }

    if let Some(ref program) = options.program {
        let target = robot_hardware_profile(config, robot_id);
        let hw = verify_with_system_config(
            program,
            Some(config),
            VerifyOptions {
                target: target.clone(),
                all_targets: target.is_none(),
                simulate: false,
                strict_certify: false,
            },
        );
        merge_hardware_findings(&hw.items, findings);
        for mission_report in verify_mission(program, target.as_deref()) {
            if mission_report.robot.as_deref() == Some(robot_id.as_str())
                && !mission_report.achievable
            {
                merge_mission_report(&mission_report, findings);
            }
        }
    }

    for mission in registry.linked_missions(robot_id) {
        if mission.readiness_status == EntityReadinessStatus::NotReady {
            push_finding(
                findings,
                "mission",
                "warning",
                format!("Linked mission '{}' is not ready", mission.id),
                "entity_registry",
            );
        }
    }
}

fn verify_fleet_entity(
    entity: &EntityRecord,
    registry: &EntityRegistry,
    config: &ResolvedSystemConfig,
    options: &EntityVerifyOptions,
    findings: &mut Vec<EntityVerifyFinding>,
) {
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

    if members.is_empty() {
        push_finding(
            findings,
            "fleet",
            "warning",
            "Fleet has no member entities in the relationship graph",
            "entity_graph",
        );
    }

    for member_id in &members {
        if registry.get(member_id).is_none() {
            push_finding(
                findings,
                "fleet",
                "error",
                format!("Fleet member '{member_id}' not found in entity registry"),
                "entity_graph",
            );
        }
    }

    if let Some(ref program) = options.program {
        let fleet_report = verify_fleet(program);
        merge_fleet_findings(&fleet_report.findings, findings);
    }

    for member_id in members {
        if let Some(member) = registry.get(&member_id) {
            if member.entity_type == EntityKind::Robot {
                verify_robot_entity(member, registry, config, options, findings);
            }
        }
    }
}

fn verify_mission_entity(
    entity: &EntityRecord,
    registry: &EntityRegistry,
    options: &EntityVerifyOptions,
    findings: &mut Vec<EntityVerifyFinding>,
) {
    if entity.readiness_status == EntityReadinessStatus::NotReady {
        push_finding(
            findings,
            "mission",
            "error",
            "Mission entity is not ready",
            "entity_snapshot",
        );
    }

    if let Some(ref program) = options.program {
        let mission_name = entity
            .name
            .as_deref()
            .or(entity.display_name.as_deref())
            .unwrap_or(&entity.id);
        for report in verify_mission(program, None) {
            let matches_mission = report
                .mission_name
                .as_deref()
                .is_some_and(|n| n == mission_name || n == entity.id);
            if matches_mission && !report.achievable {
                merge_mission_report(&report, findings);
            }
        }
    }

    let participants: Vec<_> = registry
        .relationships_for(&entity.id)
        .iter()
        .filter(|r| r.kind == EntityRelationshipKind::ParticipatesIn)
        .map(|r| {
            if r.to_id == entity.id {
                r.from_id.clone()
            } else {
                r.to_id.clone()
            }
        })
        .collect();
    for participant in participants {
        if registry.get(&participant).is_none() {
            push_finding(
                findings,
                "mission",
                "error",
                format!("Mission participant '{participant}' not found"),
                "entity_graph",
            );
        }
    }
}

fn verify_human_entity(
    entity: &EntityRecord,
    config: &ResolvedSystemConfig,
    findings: &mut Vec<EntityVerifyFinding>,
) {
    let human = config
        .human_registry
        .humans
        .iter()
        .find(|h| h.id == entity.id);
    if let Some(human) = human {
        if !human.is_available() {
            push_finding(
                findings,
                "human",
                "error",
                format!("Operator '{}' is not available", human.id),
                "human_registry",
            );
        }
        let today = chrono::Utc::now().date_naive().to_string();
        let certs_valid = human
            .certifications
            .iter()
            .all(|cert| cert_expires_on_or_after(cert.expires.as_ref(), &today));
        if !certs_valid {
            push_finding(
                findings,
                "human",
                "error",
                format!(
                    "Operator '{}' has expired or missing certifications",
                    human.id
                ),
                "human_registry",
            );
        }
    } else if entity.entity_type == EntityKind::Human {
        push_finding(
            findings,
            "human",
            "warning",
            format!(
                "Human entity '{}' not found in human registry TOML",
                entity.id
            ),
            "human_registry",
        );
    }
}

fn verify_package_entity(
    entity: &EntityRecord,
    config: &ResolvedSystemConfig,
    findings: &mut Vec<EntityVerifyFinding>,
) {
    let package_id = entity.package.as_deref().unwrap_or(entity.id.as_str());
    let known =
        config.providers.iter().any(|p| p == package_id) || entity.id.starts_with("package-");
    if !known && config.providers.is_empty() {
        push_finding(
            findings,
            "package",
            "warning",
            format!("Package '{package_id}' not listed in resolved providers"),
            "package_manifest",
        );
    }
}

fn verify_provider_entity(
    entity: &EntityRecord,
    config: &ResolvedSystemConfig,
    findings: &mut Vec<EntityVerifyFinding>,
) {
    let provider_id = entity.provider.as_deref().unwrap_or(entity.id.as_str());
    if !config.providers.iter().any(|p| p == provider_id) {
        push_finding(
            findings,
            "provider",
            "warning",
            format!("Provider '{provider_id}' not in resolved provider list"),
            "provider_registry",
        );
    }
}

fn verify_facility_entity(
    entity: &EntityRecord,
    registry: &EntityRegistry,
    findings: &mut Vec<EntityVerifyFinding>,
) {
    if entity.children_ids.is_empty() {
        push_finding(
            findings,
            "facility",
            "info",
            format!(
                "Facility entity '{}' has no child zones or assets",
                entity.id
            ),
            "entity_snapshot",
        );
    }
    for child_id in &entity.children_ids {
        if registry.get(child_id).is_none() {
            push_finding(
                findings,
                "facility",
                "error",
                format!("Child entity '{child_id}' not found for '{}'", entity.id),
                "entity_graph",
            );
        }
    }
}

fn verify_organization_entity(
    entity: &EntityRecord,
    registry: &EntityRegistry,
    findings: &mut Vec<EntityVerifyFinding>,
) {
    let owns_fleet = registry
        .relationships
        .iter()
        .any(|r| r.from_id == entity.id && r.kind == EntityRelationshipKind::Owns);
    if !owns_fleet && entity.children_ids.is_empty() {
        push_finding(
            findings,
            "organization",
            "info",
            format!(
                "Organization '{}' has no owned fleets or children",
                entity.id
            ),
            "entity_graph",
        );
    }
}

fn verify_device_entity(
    entity: &EntityRecord,
    config: &ResolvedSystemConfig,
    now_ms: f64,
    findings: &mut Vec<EntityVerifyFinding>,
) {
    if let Some(device) = config
        .device_registry
        .devices
        .iter()
        .find(|d| d.id == entity.id)
    {
        let health = evaluate_device_readiness(device, now_ms);
        if health.readiness_blocked {
            push_finding(
                findings,
                "device",
                "error",
                format!("Device blocks readiness: {}", health.blockers.join(", ")),
                "device_pool",
            );
        }
        let quarantine = evaluate_quarantine_policy(device);
        if quarantine.quarantined {
            push_finding(
                findings,
                "quarantine",
                "error",
                format!("Device quarantined: {}", quarantine.reasons.join(", ")),
                "device_pool",
            );
        }
    } else {
        push_finding(
            findings,
            "device",
            "warning",
            format!(
                "Device entity '{}' not found in device registry pool",
                entity.id
            ),
            "device_pool",
        );
    }
}

fn generic_entity_checks(entity: &EntityRecord, findings: &mut Vec<EntityVerifyFinding>) {
    if entity.capabilities.is_empty() {
        push_finding(
            findings,
            "capability",
            "info",
            format!("Entity '{}' has no declared capabilities", entity.id),
            "entity_snapshot",
        );
    }
}

fn merge_config_validation_findings(
    config: &ResolvedSystemConfig,
    entity_id: &str,
    findings: &mut Vec<EntityVerifyFinding>,
) {
    for finding in &config.validation.findings {
        if finding.message.contains(entity_id) || finding.code.contains(entity_id) {
            let severity = match finding.severity {
                ValidationSeverity::Error => "error",
                ValidationSeverity::Warning => "warning",
                ValidationSeverity::Info => "info",
            };
            push_finding(
                findings,
                &finding.code,
                severity,
                finding.message.clone(),
                "config_validation",
            );
        }
    }
}

fn merge_hardware_findings(items: &[CompatItem], findings: &mut Vec<EntityVerifyFinding>) {
    for item in items {
        if item.severity == CompatSeverity::Pass {
            continue;
        }
        let severity = match item.severity {
            CompatSeverity::Error => "error",
            CompatSeverity::Warning => "warning",
            CompatSeverity::Pass => "info",
        };
        push_finding(
            findings,
            &item.category,
            severity,
            item.message.clone(),
            "hardware_verify",
        );
    }
}

fn merge_mission_report(
    report: &MissionVerificationReport,
    findings: &mut Vec<EntityVerifyFinding>,
) {
    if !report.achievable {
        push_finding(
            findings,
            "mission",
            "error",
            format!(
                "Mission '{}' not achievable on robot {:?}: {}",
                report.mission_name.as_deref().unwrap_or("?"),
                report.robot,
                report.issues.join("; ")
            ),
            "mission_verify",
        );
    }
}

fn merge_fleet_findings(items: &[FleetVerifyFinding], findings: &mut Vec<EntityVerifyFinding>) {
    for item in items {
        push_finding(
            findings,
            &item.category,
            &item.severity,
            item.message.clone(),
            "fleet_verify",
        );
    }
}

fn devices_for_entity(
    registry: &EntityRegistry,
    entity_id: &str,
    config: &ResolvedSystemConfig,
) -> Vec<spanda_config::DeviceIdentityRecord> {
    let mut device_ids: Vec<String> = registry
        .relationships_for(entity_id)
        .iter()
        .filter(|r| {
            r.from_id == entity_id
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
            .and_then(|f| f.robots.iter().find(|r| r.id == entity_id))
        {
            collect_robot_device_ids(robot, &mut device_ids);
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

fn collect_robot_device_ids(robot: &spanda_config::RobotNode, out: &mut Vec<String>) {
    if let Some(compute) = robot.compute.as_ref() {
        for device in &compute.devices {
            out.push(device.id.clone());
        }
    }
}

fn cert_expires_on_or_after(expires: Option<&String>, today: &str) -> bool {
    match expires {
        Some(date) => date.as_str() >= today,
        None => true,
    }
}

fn robot_hardware_profile(config: &ResolvedSystemConfig, robot_id: &str) -> Option<String> {
    config
        .device_tree
        .fleet
        .as_ref()
        .and_then(|f| f.robots.iter().find(|r| r.id == robot_id))
        .and_then(|r| r.hardware_profile.clone())
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
    fn verify_robot_entity_returns_report() {
        let config = warehouse_config();
        let registry = build_entity_registry(&config);
        let report = verify_entity(
            "rover-001",
            &registry,
            &config,
            &EntityVerifyOptions {
                now_ms: 0.0,
                ..Default::default()
            },
        )
        .expect("rover-001 should exist");
        assert_eq!(report.entity_id, "rover-001");
        assert_eq!(report.entity_type, "robot");
    }

    #[test]
    fn verify_unknown_entity_returns_none() {
        let config = warehouse_config();
        let registry = build_entity_registry(&config);
        assert!(verify_entity(
            "missing-entity",
            &registry,
            &config,
            &EntityVerifyOptions::default()
        )
        .is_none());
    }

    #[test]
    fn verify_device_entity_checks_pool() {
        let config = warehouse_config();
        let registry = build_entity_registry(&config);
        let report = verify_entity(
            "gps-001",
            &registry,
            &config,
            &EntityVerifyOptions {
                now_ms: 0.0,
                ..Default::default()
            },
        )
        .expect("gps-001 should exist");
        assert_eq!(report.entity_type, "gps");
    }
}
