//! Deployment certification evidence pack composition.

use crate::recovery_coverage::evaluate_recovery_coverage;
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use spanda_certify::build_certification_proof;
use spanda_certify::CertificationProofReport;
use spanda_readiness::{evaluate_readiness, evaluate_safety_coverage, ReadinessOptions};
use spanda_trust::{evaluate_composite_trust, CompositeTrustOptions};

/// One evidence artifact in a certification pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CertificationEvidence {
    pub kind: String,
    pub passed: bool,
    pub payload: serde_json::Value,
}

/// Deployment-ready certification evidence bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CertificationPack {
    pub program: String,
    pub passed: bool,
    pub proof: CertificationProofReport,
    pub evidence: Vec<CertificationEvidence>,
}

/// Build a certification pack composing proof, readiness, safety, recovery, and trust evidence.
pub fn build_certification_pack(
    program: &Program,
    source: &str,
    source_label: &str,
    strict: bool,
) -> CertificationPack {
    let proof = build_certification_proof(program, source_label, strict);
    let readiness = evaluate_readiness(
        program,
        &ReadinessOptions {
            simulate: true,
            include_runtime: true,
            ..ReadinessOptions::default()
        },
    );
    let safety = evaluate_safety_coverage(program, source_label);
    let recovery = evaluate_recovery_coverage(program, source_label);
    let trust = evaluate_composite_trust(
        program,
        source,
        source_label,
        &CompositeTrustOptions::default(),
    );
    let evidence = vec![
        evidence_row("verification", proof.passed, &proof),
        evidence_row("readiness", readiness.mission_ready, &readiness),
        evidence_row("safety", safety.overall_coverage_pct >= 80, &safety),
        evidence_row("recovery", recovery.coverage_pct >= 80, &recovery),
        evidence_row("trust", trust.score >= 70, &trust),
    ];
    let passed = proof.passed && evidence.iter().all(|item| item.passed);
    CertificationPack {
        program: source_label.into(),
        passed,
        proof,
        evidence,
    }
}

fn evidence_row<T: Serialize>(kind: &str, passed: bool, payload: &T) -> CertificationEvidence {
    CertificationEvidence {
        kind: kind.into(),
        passed,
        payload: serde_json::to_value(payload).unwrap_or(serde_json::Value::Null),
    }
}
