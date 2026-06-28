//! Deployment gate evaluation before rollout.
//!
use crate::engine::evaluate_readiness;
use crate::types::{ReadinessOptions, ReadinessSeverity};
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use spanda_capability::capability_traceability;

/// Named deployment gate with pass/fail outcome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeploymentGate {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

/// Deployment gate evaluation report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeploymentGateReport {
    pub passed: bool,
    pub gates: Vec<DeploymentGate>,
}

/// Threshold policy for deployment gates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeploymentGatePolicy {
    pub minimum_readiness_score: u32,
    pub require_safety_audit: bool,
    pub require_capability_traceability: bool,
    pub require_official_provenance: bool,
    pub require_registry_signatures: bool,
}

impl Default for DeploymentGatePolicy {
    fn default() -> Self {
        Self {
            minimum_readiness_score: 80,
            require_safety_audit: true,
            require_capability_traceability: true,
            require_official_provenance: false,
            require_registry_signatures: false,
        }
    }
}

impl DeploymentGatePolicy {
    pub fn production() -> Self {
        Self {
            minimum_readiness_score: 90,
            require_safety_audit: true,
            require_capability_traceability: true,
            require_official_provenance: true,
            require_registry_signatures: true,
        }
    }
}

/// Evaluate deployment gates for a program before rollout.
pub fn evaluate_deployment_gates(
    program: &Program,
    source: &str,
    options: &ReadinessOptions,
    policy: &DeploymentGatePolicy,
) -> DeploymentGateReport {
    // Run readiness, safety, and capability gates for deploy blocking.
    //
    // Parameters:
    // - `program` — parsed `.sd` program
    // - `source` — program source for safety audit
    // - `options` — readiness evaluation options
    // - `policy` — gate thresholds and required checks
    //
    // Returns:
    // Deployment gate report with per-gate pass/fail.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = evaluate_deployment_gates(&program, source, &options, &policy);

    let mut gates = Vec::new();
    let readiness = evaluate_readiness(program, options);
    gates.push(DeploymentGate {
        name: "readiness".into(),
        passed: readiness.mission_ready && readiness.score.total >= policy.minimum_readiness_score,
        message: format!(
            "score {} (min {}), mission_ready={}",
            readiness.score.total, policy.minimum_readiness_score, readiness.mission_ready
        ),
    });
    if policy.require_safety_audit {
        let audit = crate::auditor::audit_program(program, source);
        let passed = audit.critical_count == 0 && audit.high_count == 0;
        gates.push(DeploymentGate {
            name: "safety".into(),
            passed,
            message: format!(
                "critical={}, high={}, medium={}, low={}",
                audit.critical_count, audit.high_count, audit.medium_count, audit.low_count
            ),
        });
    }
    if policy.require_capability_traceability {
        let trace = capability_traceability(program);
        let failed = trace
            .capability_rows
            .iter()
            .filter(|row| row.status == "FAIL")
            .count();
        let passed = trace.errors.is_empty() && failed == 0;
        gates.push(DeploymentGate {
            name: "capability".into(),
            passed,
            message: if passed {
                "capability traceability passed".into()
            } else {
                format!(
                    "{failed} capability rows failed, {} trace errors",
                    trace.errors.len()
                )
            },
        });
    }
    if let Some(cfg) = options.system_config.as_deref() {
        let mut low_trust = Vec::new();
        for package in &cfg.packages {
            let trust =
                spanda_package::evaluate_package_trust(package, None, Some(&cfg.project_root));
            if !trust.passed {
                low_trust.push(format!("{package} ({}/100)", trust.score));
            }
        }
        gates.push(DeploymentGate {
            name: "package_trust".into(),
            passed: low_trust.is_empty(),
            message: if low_trust.is_empty() {
                "all configured packages meet trust threshold".into()
            } else {
                format!("low trust packages: {}", low_trust.join(", "))
            },
        });
    }
    if policy.require_official_provenance || policy.require_registry_signatures {
        if let Some(root) = gate_project_root(options) {
            let provenance = spanda_package::evaluate_project_provenance_gate(&root);
            if policy.require_official_provenance {
                gates.push(DeploymentGate {
                    name: "official_provenance".into(),
                    passed: provenance.passed_official_provenance(),
                    message: if provenance.official_overrides.is_empty() {
                        "no official package name overrides detected".into()
                    } else {
                        format!(
                            "official name overrides without registry provenance: {}",
                            provenance.official_overrides.join(", ")
                        )
                    },
                });
            }
            if policy.require_registry_signatures {
                gates.push(DeploymentGate {
                    name: "registry_signatures".into(),
                    passed: provenance.passed_registry_signatures(),
                    message: if provenance.passed_registry_signatures() {
                        "registry signature policy satisfied".into()
                    } else {
                        format!(
                            "registry signature failures: {}",
                            provenance.registry_signature_failures.join("; ")
                        )
                    },
                });
            }
        } else {
            if policy.require_official_provenance {
                gates.push(DeploymentGate {
                    name: "official_provenance".into(),
                    passed: false,
                    message: "project root with spanda.toml required for provenance audit".into(),
                });
            }
            if policy.require_registry_signatures {
                gates.push(DeploymentGate {
                    name: "registry_signatures".into(),
                    passed: false,
                    message: "project root with spanda.lock required for registry signature audit"
                        .into(),
                });
            }
        }
    }
    let health_issues = readiness
        .issues
        .iter()
        .filter(|issue| issue.factor == "Health" && issue.severity >= ReadinessSeverity::High)
        .count();
    gates.push(DeploymentGate {
        name: "health".into(),
        passed: health_issues == 0,
        message: if health_issues == 0 {
            "no high-severity health issues".into()
        } else {
            format!("{health_issues} high-severity health issues")
        },
    });
    let passed = gates.iter().all(|gate| gate.passed);
    DeploymentGateReport { passed, gates }
}

fn gate_project_root(options: &ReadinessOptions) -> Option<std::path::PathBuf> {
    // Resolve the project root for provenance gates from config or source path.

    if let Some(root) = &options.project_root {
        return Some(root.clone());
    }
    if let Some(cfg) = options.system_config.as_ref() {
        return Some(cfg.project_root.clone());
    }
    let source = options.source_path.as_deref()?;
    let start = if source.is_dir() {
        source.to_path_buf()
    } else {
        source.parent()?.to_path_buf()
    };
    spanda_package::find_project_root(&start)
}
