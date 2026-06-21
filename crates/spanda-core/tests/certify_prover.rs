//! Certification proof artifact tests.

use spanda_core::{build_certification_proof, compile, CompatSeverity};

#[test]
fn certified_example_proof_passes() {
    let source = include_str!("../../../examples/robotics/certified_deployment.sd");
    let program = compile(source).expect("compile").program;
    let proof = build_certification_proof(&program, "certified_deployment.sd", true);
    assert!(proof.passed);
    assert!(!proof.certifications.is_empty());
    assert!(!proof.deploy_targets.is_empty());
}

#[test]
fn ota_example_proof_fails_under_strict() {
    let source = include_str!("../../../examples/robotics/ota_deployment.sd");
    let program = compile(source).expect("compile").program;
    let proof = build_certification_proof(&program, "ota_deployment.sd", true);
    assert!(!proof.passed);
    assert!(
        proof
            .checklist
            .iter()
            .any(|i| i.severity == CompatSeverity::Error),
    );
}

#[test]
fn proof_includes_program_hash_for_existing_file() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/robotics/certified_deployment.sd"
    );
    let source = std::fs::read_to_string(path).expect("read example");
    let program = compile(&source).expect("compile").program;
    let proof = build_certification_proof(&program, path, false);
    assert!(proof.program_hash.is_some());
}
