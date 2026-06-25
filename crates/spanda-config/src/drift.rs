//! Configuration drift detection between resolved system baselines.
//!
use crate::device_identity::DeviceIdentityRecord;
use crate::mapping::{ActuatorMapping, LogicalPhysicalMap, SensorMapping};
use crate::resolver::diff_configs;
use crate::resolved::ResolvedSystemConfig;
use serde::{Deserialize, Serialize};
use spanda_ast::foundations::DeployDecl;
use spanda_ast::nodes::{Program, RobotDecl};
use std::collections::{BTreeSet, HashMap};

/// Severity tier for a drift finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Configuration area where drift was observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftDimension {
    Configuration,
    Fleet,
    Device,
    Provider,
    Package,
    Mapping,
    Program,
    Hardware,
    Firmware,
}

/// Single drift delta between baseline and current configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriftFinding {
    pub dimension: DriftDimension,
    pub severity: DriftSeverity,
    pub message: String,
    pub path: Option<String>,
}

/// Structured drift report for baseline vs current resolved configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigDriftReport {
    pub findings: Vec<DriftFinding>,
    pub passed: bool,
    pub baseline_project: String,
    pub current_project: String,
}

impl ConfigDriftReport {
    pub fn push(&mut self, finding: DriftFinding) {
        if finding.severity >= DriftSeverity::Medium {
            self.passed = false;
        }
        self.findings.push(finding);
    }
}

/// Compare two resolved configurations and emit structured drift findings.
pub fn detect_config_drift(
    baseline: &ResolvedSystemConfig,
    current: &ResolvedSystemConfig,
) -> ConfigDriftReport {
    // Compare baseline and current resolved configs for operational drift.
    //
    // Parameters:
    // - `baseline` — approved or expected configuration
    // - `current` — configuration under inspection
    //
    // Returns:
    // Structured drift report with severity-tagged findings.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = detect_config_drift(&approved, &live);

    let mut report = ConfigDriftReport {
        findings: Vec::new(),
        passed: true,
        baseline_project: baseline.project_name().to_string(),
        current_project: current.project_name().to_string(),
    };

    // Flag raw TOML key deltas under the configuration dimension.
    for line in diff_configs(&baseline.raw, &current.raw) {
        report.push(DriftFinding {
            dimension: DriftDimension::Configuration,
            severity: DriftSeverity::Medium,
            message: line,
            path: None,
        });
    }

    // Compare fleet identifiers when both sides declare a fleet.
    if baseline.fleet_id() != current.fleet_id() {
        report.push(DriftFinding {
            dimension: DriftDimension::Fleet,
            severity: DriftSeverity::High,
            message: format!(
                "fleet.id changed: {:?} -> {:?}",
                baseline.fleet_id(),
                current.fleet_id()
            ),
            path: Some("fleet.id".into()),
        });
    }

    // Detect robots added or removed from the fleet tree.
    diff_string_sets(
        &mut report,
        DriftDimension::Fleet,
        "robot",
        &baseline.robot_ids().into_iter().map(str::to_string).collect::<Vec<_>>(),
        &current.robot_ids().into_iter().map(str::to_string).collect::<Vec<_>>(),
    );

    // Compare provider and package manifests.
    diff_string_sets(
        &mut report,
        DriftDimension::Provider,
        "provider",
        &baseline.providers,
        &current.providers,
    );
    diff_string_sets(
        &mut report,
        DriftDimension::Package,
        "package",
        &baseline.packages,
        &current.packages,
    );

    // Compare device identity records field-by-field.
    diff_device_registry(&mut report, baseline, current);

    // Compare logical-to-physical mappings derived from each config.
    diff_logical_map(&mut report, &baseline.logical_map, &current.logical_map);

    report
}

/// Append program alignment findings against the current resolved configuration.
pub fn append_program_drift(
    report: &mut ConfigDriftReport,
    program: &Program,
    current: &ResolvedSystemConfig,
) {
    // Add program-vs-config mapping drift to an existing report.
    //
    // Parameters:
    // - `report` — drift report to extend in place
    // - `program` — parsed `.sd` program
    // - `current` — live resolved configuration
    //
    // Returns:
    // Nothing; findings are appended to `report`.
    //
    // Options:
    // None.
    //
    // Example:
    // append_program_drift(&mut report, &program, &current);

    for issue in current
        .logical_map
        .verify_against_program(program, &current.device_registry)
    {
        report.push(DriftFinding {
            dimension: DriftDimension::Program,
            severity: DriftSeverity::High,
            message: issue,
            path: None,
        });
    }
}

/// Render drift findings as human-readable lines (legacy text consumers).
pub fn format_drift_lines(report: &ConfigDriftReport) -> Vec<String> {
    report
        .findings
        .iter()
        .map(|finding| {
            let path = finding
                .path
                .as_deref()
                .map(|p| format!(" @ {p}"))
                .unwrap_or_default();
            format!(
                "[{:?}/{:?}] {}{}",
                finding.dimension, finding.severity, finding.message, path
            )
        })
        .collect()
}

fn diff_string_sets(
    report: &mut ConfigDriftReport,
    dimension: DriftDimension,
    label: &str,
    baseline: &[String],
    current: &[String],
) {
    let base: BTreeSet<&str> = baseline.iter().map(String::as_str).collect();
    let live: BTreeSet<&str> = current.iter().map(String::as_str).collect();
    for item in base.difference(&live) {
        report.push(DriftFinding {
            dimension,
            severity: DriftSeverity::High,
            message: format!("removed {label} '{item}'"),
            path: Some(item.to_string()),
        });
    }
    for item in live.difference(&base) {
        report.push(DriftFinding {
            dimension,
            severity: DriftSeverity::Medium,
            message: format!("added {label} '{item}'"),
            path: Some(item.to_string()),
        });
    }
}

fn diff_device_registry(
    report: &mut ConfigDriftReport,
    baseline: &ResolvedSystemConfig,
    current: &ResolvedSystemConfig,
) {
    let base: HashMap<&str, &DeviceIdentityRecord> = baseline
        .device_registry
        .devices
        .iter()
        .map(|d| (d.id.as_str(), d))
        .collect();
    let live: HashMap<&str, &DeviceIdentityRecord> = current
        .device_registry
        .devices
        .iter()
        .map(|d| (d.id.as_str(), d))
        .collect();

    for id in base.keys() {
        if !live.contains_key(id) {
            report.push(DriftFinding {
                dimension: DriftDimension::Device,
                severity: DriftSeverity::Critical,
                message: format!("device '{id}' removed"),
                path: Some(format!("devices.{id}")),
            });
        }
    }
    for id in live.keys() {
        if !base.contains_key(id) {
            report.push(DriftFinding {
                dimension: DriftDimension::Device,
                severity: DriftSeverity::High,
                message: format!("device '{id}' added"),
                path: Some(format!("devices.{id}")),
            });
        }
    }
    for (id, base_device) in &base {
        let Some(live_device) = live.get(id) else {
            continue;
        };
        diff_optional_field(report, id, "ip", &base_device.ip_address, &live_device.ip_address);
        diff_optional_field(
            report,
            id,
            "endpoint",
            &base_device.endpoint_url,
            &live_device.endpoint_url,
        );
        diff_optional_field(
            report,
            id,
            "provider",
            &base_device.provider,
            &live_device.provider,
        );
        diff_optional_field(
            report,
            id,
            "firmware",
            &base_device.firmware_version,
            &live_device.firmware_version,
        );
        diff_optional_field(
            report,
            id,
            "trust_level",
            &base_device.trust_level,
            &live_device.trust_level,
        );
        diff_optional_field(
            report,
            id,
            "security_identity",
            &base_device.security_identity,
            &live_device.security_identity,
        );
        if base_device.capabilities != live_device.capabilities {
            report.push(DriftFinding {
                dimension: DriftDimension::Device,
                severity: DriftSeverity::Medium,
                message: format!(
                    "device '{id}' capabilities changed: {:?} -> {:?}",
                    base_device.capabilities, live_device.capabilities
                ),
                path: Some(format!("devices.{id}.capabilities")),
            });
        }
    }
}

fn diff_optional_field(
    report: &mut ConfigDriftReport,
    device_id: &str,
    field: &str,
    baseline: &Option<String>,
    current: &Option<String>,
) {
    if baseline == current {
        return;
    }
    report.push(DriftFinding {
        dimension: DriftDimension::Device,
        severity: if field == "security_identity" || field == "trust_level" {
            DriftSeverity::High
        } else {
            DriftSeverity::Medium
        },
        message: format!(
            "device '{device_id}' {field}: {:?} -> {:?}",
            baseline, current
        ),
        path: Some(format!("devices.{device_id}.{field}")),
    });
}

fn diff_logical_map(
    report: &mut ConfigDriftReport,
    baseline: &LogicalPhysicalMap,
    current: &LogicalPhysicalMap,
) {
    diff_sensor_map(report, baseline, current);
    diff_actuator_map(report, baseline, current);
}

fn diff_sensor_map(
    report: &mut ConfigDriftReport,
    baseline: &LogicalPhysicalMap,
    current: &LogicalPhysicalMap,
) {
    let base_keys: BTreeSet<&str> = baseline.sensors.keys().map(String::as_str).collect();
    let live_keys: BTreeSet<&str> = current.sensors.keys().map(String::as_str).collect();
    for key in base_keys.difference(&live_keys) {
        report.push(DriftFinding {
            dimension: DriftDimension::Mapping,
            severity: DriftSeverity::High,
            message: format!("sensor mapping '{key}' removed"),
            path: Some(format!("mapping.sensors.{key}")),
        });
    }
    for key in live_keys.difference(&base_keys) {
        report.push(DriftFinding {
            dimension: DriftDimension::Mapping,
            severity: DriftSeverity::Medium,
            message: format!("sensor mapping '{key}' added"),
            path: Some(format!("mapping.sensors.{key}")),
        });
    }
    for key in base_keys.intersection(&live_keys) {
        diff_sensor_fields(report, key, baseline.sensors.get(*key).unwrap(), current.sensors.get(*key).unwrap());
    }
}

fn diff_actuator_map(
    report: &mut ConfigDriftReport,
    baseline: &LogicalPhysicalMap,
    current: &LogicalPhysicalMap,
) {
    let base_keys: BTreeSet<&str> = baseline.actuators.keys().map(String::as_str).collect();
    let live_keys: BTreeSet<&str> = current.actuators.keys().map(String::as_str).collect();
    for key in base_keys.difference(&live_keys) {
        report.push(DriftFinding {
            dimension: DriftDimension::Mapping,
            severity: DriftSeverity::High,
            message: format!("actuator mapping '{key}' removed"),
            path: Some(format!("mapping.actuators.{key}")),
        });
    }
    for key in live_keys.difference(&base_keys) {
        report.push(DriftFinding {
            dimension: DriftDimension::Mapping,
            severity: DriftSeverity::Medium,
            message: format!("actuator mapping '{key}' added"),
            path: Some(format!("mapping.actuators.{key}")),
        });
    }
    for key in base_keys.intersection(&live_keys) {
        diff_actuator_fields(
            report,
            key,
            baseline.actuators.get(*key).unwrap(),
            current.actuators.get(*key).unwrap(),
        );
    }
}

fn diff_sensor_fields(
    report: &mut ConfigDriftReport,
    key: &str,
    baseline: &SensorMapping,
    current: &SensorMapping,
) {
    if baseline.physical_device_id != current.physical_device_id {
        report.push(DriftFinding {
            dimension: DriftDimension::Mapping,
            severity: DriftSeverity::High,
            message: format!(
                "sensor '{key}' physical device: {} -> {}",
                baseline.physical_device_id, current.physical_device_id
            ),
            path: Some(format!("mapping.sensors.{key}.physical_device_id")),
        });
    }
    diff_optional_field(report, key, "ip", &baseline.ip_address, &current.ip_address);
    diff_optional_field(report, key, "endpoint", &baseline.endpoint_url, &current.endpoint_url);
}

fn diff_actuator_fields(
    report: &mut ConfigDriftReport,
    key: &str,
    baseline: &ActuatorMapping,
    current: &ActuatorMapping,
) {
    if baseline.physical_device_id != current.physical_device_id {
        report.push(DriftFinding {
            dimension: DriftDimension::Mapping,
            severity: DriftSeverity::High,
            message: format!(
                "actuator '{key}' physical device: {} -> {}",
                baseline.physical_device_id, current.physical_device_id
            ),
            path: Some(format!("mapping.actuators.{key}.physical_device_id")),
        });
    }
    if baseline.has_emergency_stop != current.has_emergency_stop {
        report.push(DriftFinding {
            dimension: DriftDimension::Mapping,
            severity: DriftSeverity::Critical,
            message: format!(
                "actuator '{key}' emergency_stop capability: {} -> {}",
                baseline.has_emergency_stop, current.has_emergency_stop
            ),
            path: Some(format!("mapping.actuators.{key}.emergency_stop")),
        });
    }
    diff_optional_field(report, key, "ip", &baseline.ip_address, &current.ip_address);
    diff_optional_field(report, key, "endpoint", &baseline.endpoint_url, &current.endpoint_url);
}

/// Live agent status used for expected-vs-actual drift checks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AgentDriftSnapshot {
    pub agent_id: String,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub robot_name: Option<String>,
    #[serde(default)]
    pub hardware_profile: Option<String>,
    #[serde(default)]
    pub firmware_version: Option<String>,
    #[serde(default)]
    pub program_hash: Option<String>,
    #[serde(default)]
    pub current_version: Option<String>,
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub healthy: bool,
    #[serde(default)]
    pub attestation_contract: Option<String>,
    #[serde(default)]
    pub attestation_verified: Option<bool>,
    #[serde(default)]
    pub boot_state: Option<String>,
}

/// Expected deploy state derived from a program and resolved configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpectedAgentState {
    pub target_key: String,
    pub robot_name: String,
    pub hardware_profile: Option<String>,
    pub program_hash: Option<String>,
    pub firmware_by_device: HashMap<String, String>,
    pub packages: Vec<String>,
}

/// Build expected agent states from deploy declarations and configuration.
pub fn expected_agent_states(
    program: &Program,
    config: Option<&ResolvedSystemConfig>,
    program_hash: Option<&str>,
) -> Vec<ExpectedAgentState> {
    // Derive per-target expected deploy posture from program and config.
    //
    // Parameters:
    // - `program` — parsed `.sd` source
    // - `config` — optional resolved system configuration
    // - `program_hash` — optional SHA-256 hex digest of the program file
    //
    // Returns:
    // Expected states keyed by deploy target (`Robot@Hardware`).
    //
    // Options:
    // None.
    //
    // Example:
    // let expected = expected_agent_states(&program, Some(&cfg), Some("abc…"));

    let Program::Program {
        deployments,
        robots,
        ..
    } = program;
    let robot_names: Vec<String> = robots
        .iter()
        .map(|robot| {
            let RobotDecl::RobotDecl { name, .. } = robot;
            name.clone()
        })
        .collect();
    let packages = config.map(|cfg| cfg.packages.clone()).unwrap_or_default();
    let hash = program_hash.map(str::to_string);
    let mut states = Vec::new();
    if deployments.is_empty() {
        if let Some(robot) = robot_names.first() {
            states.push(ExpectedAgentState {
                target_key: robot.clone(),
                robot_name: robot.clone(),
                hardware_profile: config
                    .and_then(|cfg| cfg.logical_map.robots.get(robot))
                    .and_then(|m| m.hardware_profile.clone()),
                program_hash: hash.clone(),
                firmware_by_device: firmware_for_robot(config, robot),
                packages: packages.clone(),
            });
        }
        return states;
    }
    for deploy in deployments {
        let DeployDecl::DeployDecl {
            robot_name,
            targets,
            ..
        } = deploy;
        for hardware in targets {
            states.push(ExpectedAgentState {
                target_key: deploy_target_key(robot_name, hardware),
                robot_name: robot_name.clone(),
                hardware_profile: Some(hardware.clone()),
                program_hash: hash.clone(),
                firmware_by_device: firmware_for_robot(config, robot_name),
                packages: packages.clone(),
            });
        }
    }
    states
}

/// Compare expected deploy posture against a live agent status snapshot.
pub fn detect_agent_drift(
    expected: &ExpectedAgentState,
    actual: &AgentDriftSnapshot,
) -> Vec<DriftFinding> {
    // Emit drift findings for one agent target.
    //
    // Parameters:
    // - `expected` — declared deploy posture
    // - `actual` — live agent `/v1/status` snapshot
    //
    // Returns:
    // Severity-tagged drift findings.
    //
    // Options:
    // None.
    //
    // Example:
    // let findings = detect_agent_drift(&expected, &snapshot);

    let mut findings = Vec::new();
    let agent = actual.agent_id.as_str();
    if !actual.healthy {
        findings.push(DriftFinding {
            dimension: DriftDimension::Hardware,
            severity: DriftSeverity::High,
            message: format!("agent '{agent}' reported unhealthy status"),
            path: Some(format!("agents.{agent}.healthy")),
        });
    }
    if let Some(ref expected_hw) = expected.hardware_profile {
        match &actual.hardware_profile {
            Some(live) if live == expected_hw => {}
            Some(live) => findings.push(DriftFinding {
                dimension: DriftDimension::Hardware,
                severity: DriftSeverity::High,
                message: format!(
                    "agent '{agent}' hardware profile: expected {expected_hw}, actual {live}"
                ),
                path: Some(format!("agents.{agent}.hardware_profile")),
            }),
            None => findings.push(DriftFinding {
                dimension: DriftDimension::Hardware,
                severity: DriftSeverity::Medium,
                message: format!(
                    "agent '{agent}' missing hardware profile (expected {expected_hw})"
                ),
                path: Some(format!("agents.{agent}.hardware_profile")),
            }),
        }
    }
    match (&expected.program_hash, &actual.program_hash) {
        (Some(expected_hash), Some(live)) if expected_hash != live => {
            findings.push(DriftFinding {
                dimension: DriftDimension::Program,
                severity: DriftSeverity::Critical,
                message: format!(
                    "agent '{agent}' program hash mismatch: expected {expected_hash}, actual {live}"
                ),
                path: Some(format!("agents.{agent}.program_hash")),
            });
        }
        (Some(expected_hash), None) => findings.push(DriftFinding {
            dimension: DriftDimension::Program,
            severity: DriftSeverity::High,
            message: format!(
                "agent '{agent}' missing program hash (expected {expected_hash})"
            ),
            path: Some(format!("agents.{agent}.program_hash")),
        }),
        _ => {}
    }
    if !expected.firmware_by_device.is_empty() {
        let expected_versions: BTreeSet<&str> = expected
            .firmware_by_device
            .values()
            .map(String::as_str)
            .collect();
        match &actual.firmware_version {
            Some(live) if expected_versions.contains(live.as_str()) => {}
            Some(live) => findings.push(DriftFinding {
                dimension: DriftDimension::Firmware,
                severity: DriftSeverity::High,
                message: format!(
                    "agent '{agent}' firmware '{live}' not in expected set {:?}",
                    expected_versions
                ),
                path: Some(format!("agents.{agent}.firmware_version")),
            }),
            None => findings.push(DriftFinding {
                dimension: DriftDimension::Firmware,
                severity: DriftSeverity::Medium,
                message: format!(
                    "agent '{agent}' missing firmware report (expected {:?})",
                    expected_versions
                ),
                path: Some(format!("agents.{agent}.firmware_version")),
            }),
        }
    }
    diff_string_findings(
        &mut findings,
        DriftDimension::Package,
        "package",
        agent,
        &expected.packages,
        &actual.packages,
    );
    findings
}

/// Append agent drift findings onto an existing configuration drift report.
pub fn append_agent_drift(report: &mut ConfigDriftReport, findings: Vec<DriftFinding>) {
    for finding in findings {
        report.push(finding);
    }
}

fn deploy_target_key(robot: &str, hardware: &str) -> String {
    format!("{robot}@{hardware}")
}

fn firmware_for_robot(
    config: Option<&ResolvedSystemConfig>,
    robot: &str,
) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Some(cfg) = config else {
        return map;
    };
    for device in &cfg.device_registry.devices {
        let owns = device
            .robot_id
            .as_deref()
            .is_none_or(|id| id == robot);
        if !owns {
            continue;
        }
        if let Some(fw) = &device.firmware_version {
            map.insert(device.id.clone(), fw.clone());
        }
    }
    map
}

fn diff_string_findings(
    findings: &mut Vec<DriftFinding>,
    dimension: DriftDimension,
    label: &str,
    agent: &str,
    baseline: &[String],
    current: &[String],
) {
    if baseline.is_empty() {
        return;
    }
    let base: BTreeSet<&str> = baseline.iter().map(String::as_str).collect();
    let live: BTreeSet<&str> = current.iter().map(String::as_str).collect();
    for item in base.difference(&live) {
        findings.push(DriftFinding {
            dimension,
            severity: DriftSeverity::High,
            message: format!("agent '{agent}' missing {label} '{item}'"),
            path: Some(format!("agents.{agent}.{label}.{item}")),
        });
    }
    for item in live.difference(&base) {
        findings.push(DriftFinding {
            dimension,
            severity: DriftSeverity::Medium,
            message: format!("agent '{agent}' unexpected {label} '{item}'"),
            path: Some(format!("agents.{agent}.{label}.{item}")),
        });
    }
}
