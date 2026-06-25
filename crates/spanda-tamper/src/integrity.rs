//! Verify-time integrity hashing and baseline comparison for program artifacts.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use spanda_ast::foundations::{
    DeployDecl, HardwareDecl, HealthPolicyDecl, KillSwitchDecl, MissionDecl,
};
use spanda_ast::nodes::{Program, RobotDecl};
use spanda_ast::policy_decl::OperationalPolicyDecl;

/// Output format for integrity reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IntegrityFormat {
    #[default]
    Text,
    Json,
}

/// Per-artifact integrity posture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactIntegrityStatus {
    Trusted,
    Modified,
    Unknown,
}

/// One hashed program artifact with optional baseline comparison.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrityArtifact {
    pub artifact_type: String,
    pub name: String,
    pub hash: String,
    pub status: ArtifactIntegrityStatus,
    pub baseline_hash: Option<String>,
}

/// Full integrity verification report for a program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub program: String,
    pub baseline: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    pub artifacts: Vec<IntegrityArtifact>,
    #[serde(default)]
    pub agent_artifacts: Vec<IntegrityArtifact>,
    pub passed: bool,
}

/// Expected deploy posture for agent integrity comparison.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentIntegrityExpected {
    pub program_hash: Option<String>,
    pub hardware_profile: Option<String>,
}

/// Live agent status snapshot for integrity comparison.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentIntegrityActual {
    pub agent_id: String,
    pub program_hash: Option<String>,
    pub hardware_profile: Option<String>,
    pub healthy: bool,
}

/// Hash a serializable artifact into a SHA-256 hex digest.
fn hash_artifact<T: Serialize>(value: &T) -> String {
    let json = serde_json::to_string(value).unwrap_or_default();
    hex::encode(Sha256::digest(json.as_bytes()))
}

/// Collect integrity artifacts from a parsed program.
fn collect_artifacts(program: &Program) -> Vec<(String, String, String)> {
    let mut artifacts = Vec::new();
    let Program::Program {
        hardware_profiles,
        operational_policies,
        deployments,
        kill_switches,
        health_policies,
        imports,
        robots,
        ..
    } = program;

    for hardware in hardware_profiles {
        let HardwareDecl::HardwareDecl { name, .. } = hardware;
        artifacts.push((
            "hardware".into(),
            name.clone(),
            hash_artifact(hardware),
        ));
    }

    for policy in operational_policies {
        let OperationalPolicyDecl::OperationalPolicyDecl { name, .. } = policy;
        artifacts.push((
            "policy".into(),
            name.clone(),
            hash_artifact(policy),
        ));
    }

    for ks in kill_switches {
        let KillSwitchDecl::KillSwitchDecl { name, .. } = ks;
        artifacts.push((
            "kill_switch".into(),
            name.clone(),
            hash_artifact(ks),
        ));
    }

    for health in health_policies {
        let HealthPolicyDecl::HealthPolicyDecl { name, .. } = health;
        artifacts.push((
            "health_policy".into(),
            name.clone(),
            hash_artifact(health),
        ));
    }

    for deployment in deployments {
        let DeployDecl::DeployDecl { robot_name, .. } = deployment;
        artifacts.push((
            "deploy".into(),
            robot_name.clone(),
            hash_artifact(deployment),
        ));
    }

    for import in imports {
        let spanda_ast::nodes::ImportDecl::ImportDecl { path, .. } = import;
        artifacts.push(("package".into(), path.clone(), hash_artifact(import)));
    }

    for robot in robots {
        collect_robot_artifacts(robot, &mut artifacts);
    }

    artifacts
}

fn collect_robot_artifacts(robot: &RobotDecl, artifacts: &mut Vec<(String, String, String)>) {
    let RobotDecl::RobotDecl {
        name,
        mission,
        safety,
        exposes_capabilities,
        ..
    } = robot;

    if let Some(mission_decl) = mission {
        let MissionDecl::MissionDecl { name: mission_name, .. } = mission_decl;
        let mission_label = mission_name.as_deref().unwrap_or("default");
        artifacts.push((
            "mission".into(),
            format!("{name}/{mission_label}"),
            hash_artifact(mission_decl),
        ));
    }

    if let Some(safety_block) = safety {
        artifacts.push((
            "safety".into(),
            name.clone(),
            hash_artifact(safety_block),
        ));
    }

    if !exposes_capabilities.is_empty() {
        artifacts.push((
            "capabilities".into(),
            name.clone(),
            hash_artifact(exposes_capabilities),
        ));
    }
}

/// Generate an integrity report for a program with optional baseline comparison.
pub fn generate_integrity_report(
    program: &Program,
    source_label: &str,
    baseline_program: Option<&Program>,
    baseline_label: Option<&str>,
) -> IntegrityReport {
    // Hash declared artifacts and compare against an optional approved baseline program.
    //
    // Parameters:
    // - `program` — parsed candidate program
    // - `source_label` — file label for the candidate
    // - `baseline_program` — optional approved baseline AST
    // - `baseline_label` — optional baseline file label
    //
    // Returns:
    // Integrity report with per-artifact status and pass/fail rollup.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = generate_integrity_report(&program, "rover.sd", None, None);

    let current = collect_artifacts(program);
    let baseline_map = baseline_program.map(|baseline| {
        collect_artifacts(baseline)
            .into_iter()
            .map(|(kind, name, hash)| ((kind, name), hash))
            .collect::<std::collections::BTreeMap<(String, String), String>>()
    });

    let artifacts = current
        .into_iter()
        .map(|(artifact_type, name, hash)| {
            let (status, baseline_hash) = match baseline_map.as_ref() {
                None => (ArtifactIntegrityStatus::Unknown, None),
                Some(map) => match map.get(&(artifact_type.clone(), name.clone())) {
                    None => (ArtifactIntegrityStatus::Modified, None),
                    Some(base_hash) if base_hash == &hash => {
                        (ArtifactIntegrityStatus::Trusted, Some(base_hash.clone()))
                    }
                    Some(base_hash) => (
                        ArtifactIntegrityStatus::Modified,
                        Some(base_hash.clone()),
                    ),
                },
            };
            IntegrityArtifact {
                artifact_type,
                name,
                hash,
                status,
                baseline_hash,
            }
        })
        .collect::<Vec<_>>();

    let passed = if baseline_map.is_none() {
        true
    } else {
        artifacts
            .iter()
            .all(|artifact| artifact.status == ArtifactIntegrityStatus::Trusted)
    };

    IntegrityReport {
        program: source_label.into(),
        baseline: baseline_label.map(str::to_string),
        agent: None,
        artifacts,
        agent_artifacts: Vec::new(),
        passed,
    }
}

/// Compare expected deploy posture against a live agent snapshot.
pub fn compare_agent_integrity(
    expected: &AgentIntegrityExpected,
    actual: &AgentIntegrityActual,
) -> Vec<IntegrityArtifact> {
    // Map agent status fields to integrity artifacts with Trusted/Modified status.
    //
    // Parameters:
    // - `expected` — declared program hash and hardware profile
    // - `actual` — live agent `/v1/status` fields
    //
    // Returns:
    // Per-field integrity artifacts for the agent target.
    //
    // Options:
    // None.
    //
    // Example:
    // let checks = compare_agent_integrity(&expected, &actual);

    let mut artifacts = Vec::new();
    let health_status = if actual.healthy {
        ArtifactIntegrityStatus::Trusted
    } else {
        ArtifactIntegrityStatus::Modified
    };
    artifacts.push(IntegrityArtifact {
        artifact_type: "agent".into(),
        name: format!("{}/health", actual.agent_id),
        hash: if actual.healthy {
            "healthy".into()
        } else {
            "unhealthy".into()
        },
        status: health_status,
        baseline_hash: Some("healthy".into()),
    });
    if let Some(expected_hw) = &expected.hardware_profile {
        let (status, hash, baseline) = match &actual.hardware_profile {
            Some(live) if live == expected_hw => (
                ArtifactIntegrityStatus::Trusted,
                live.clone(),
                Some(expected_hw.clone()),
            ),
            Some(live) => (
                ArtifactIntegrityStatus::Modified,
                live.clone(),
                Some(expected_hw.clone()),
            ),
            None => (
                ArtifactIntegrityStatus::Modified,
                "missing".into(),
                Some(expected_hw.clone()),
            ),
        };
        artifacts.push(IntegrityArtifact {
            artifact_type: "agent".into(),
            name: format!("{}/hardware_profile", actual.agent_id),
            hash,
            status,
            baseline_hash: baseline,
        });
    }
    if let Some(expected_hash) = &expected.program_hash {
        let (status, hash, baseline) = match &actual.program_hash {
            Some(live) if live == expected_hash => (
                ArtifactIntegrityStatus::Trusted,
                live.clone(),
                Some(expected_hash.clone()),
            ),
            Some(live) => (
                ArtifactIntegrityStatus::Modified,
                live.clone(),
                Some(expected_hash.clone()),
            ),
            None => (
                ArtifactIntegrityStatus::Modified,
                "missing".into(),
                Some(expected_hash.clone()),
            ),
        };
        artifacts.push(IntegrityArtifact {
            artifact_type: "agent".into(),
            name: format!("{}/program_hash", actual.agent_id),
            hash,
            status,
            baseline_hash: baseline,
        });
    }
    artifacts
}

/// Attach agent integrity artifacts to a report and update pass/fail rollup.
pub fn apply_agent_integrity(
    report: &mut IntegrityReport,
    agent_id: &str,
    agent_artifacts: Vec<IntegrityArtifact>,
) {
    // Merge agent comparison artifacts into an integrity report.
    //
    // Parameters:
    // - `report` — integrity report to update in place
    // - `agent_id` — agent target identifier
    // - `agent_artifacts` — per-field agent integrity checks
    //
    // Returns:
    // None.
    //
    // Options:
    // None.
    //
    // Example:
    // apply_agent_integrity(&mut report, "Rover@RoverV1", checks);

    report.agent = Some(agent_id.into());
    report.agent_artifacts = agent_artifacts;
    if report
        .agent_artifacts
        .iter()
        .any(|artifact| artifact.status == ArtifactIntegrityStatus::Modified)
    {
        report.passed = false;
    }
}

/// Format an integrity report for CLI output.
pub fn format_integrity_report(report: &IntegrityReport, format: IntegrityFormat) -> String {
    // Render integrity report as JSON or human-readable text.
    //
    // Parameters:
    // - `report` — integrity verification report
    // - `format` — text or JSON output
    //
    // Returns:
    // Formatted report string.
    //
    // Options:
    // None.
    //
    // Example:
    // let text = format_integrity_report(&report, IntegrityFormat::Text);

    if format == IntegrityFormat::Json {
        return serde_json::to_string_pretty(report).unwrap_or_else(|e| e.to_string());
    }

    let mut lines = vec![
        format!("Integrity check: {}", report.program),
        if let Some(baseline) = &report.baseline {
            format!("Baseline: {baseline}")
        } else {
            "Baseline: none (hashes only)".into()
        },
        if report.passed {
            "Result: PASS".into()
        } else {
            "Result: FAIL".into()
        },
    ];
    if let Some(agent) = &report.agent {
        lines.insert(2, format!("Agent: {agent}"));
    }
    if report.artifacts.is_empty() {
        lines.push("No artifacts found.".into());
    } else {
        lines.push("Artifacts:".into());
        for artifact in &report.artifacts {
            lines.push(format!(
                "  [{:?}] {}:{} — {}",
                artifact.status, artifact.artifact_type, artifact.name, artifact.hash
            ));
        }
    }
    if !report.agent_artifacts.is_empty() {
        lines.push("Agent artifacts:".into());
        for artifact in &report.agent_artifacts {
            lines.push(format!(
                "  [{:?}] {}:{} — {}",
                artifact.status, artifact.artifact_type, artifact.name, artifact.hash
            ));
        }
    }
    lines.join("\n")
}
