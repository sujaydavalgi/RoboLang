//! Package integration tests for audit/blockchain library metadata.

use spanda_package::{
    high_risk_capabilities, is_high_risk_capability, resolve_package_import, search_registry,
    validate_package, ApplicationPermissions, PackageManifest,
};

#[test]
fn registry_includes_blockchain_packages() {
    let ledger = search_registry("ledger");
    assert!(ledger.iter().any(|e| e.name == "spanda-ledger"));
    let provenance = search_registry("provenance");
    assert!(provenance.iter().any(|e| e.name == "spanda-provenance"));
    assert!(resolve_package_import("ledger.mock"));
    assert!(resolve_package_import("provenance.core"));
    assert!(resolve_package_import("identity.core"));
}

#[test]
fn high_risk_capability_detection() {
    assert!(is_high_risk_capability("ledger.anchor"));
    assert!(is_high_risk_capability("identity.sign"));
    assert!(!is_high_risk_capability("camera.read"));
    assert!(high_risk_capabilities().contains(&"audit.write"));
}

#[test]
fn audit_package_manifest_validation() {
    let manifest = PackageManifest::parse_str(
        r#"
[package]
name = "spanda-ledger-ethereum"
version = "0.1.0"
license = "Apache-2.0"

[capabilities]
uses = ["ledger.anchor", "network.outbound", "audit.write"]

[safety]
level = "experimental"
"#,
    )
    .unwrap();
    let mut perms = ApplicationPermissions::default();
    perms.capabilities.insert("camera.read".into());
    perms
        .allowed_safety_levels
        .insert(spanda_package::SafetyLevel::Experimental);
    let report = validate_package(&manifest, &perms).unwrap();
    assert!(report.warnings.iter().any(|w| w.contains("ledger.anchor")));
}
