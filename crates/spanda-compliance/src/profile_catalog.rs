//! Signed compliance profile catalog for tamper-evident template distribution.
//!
use crate::profiles::ComplianceProfile;
use serde::{Deserialize, Serialize};
use spanda_audit::crypto::{public_key_from_material, sha256, verify_signature};

const CATALOG_SIGNING_MATERIAL: &str = "spanda-official-catalog-signing-v1";

/// One signed entry in the profile catalog manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignedProfileEntry {
    pub name: String,
    pub version: String,
    pub content_sha256: String,
    pub signature: String,
    pub template_path: String,
}

/// Catalog manifest listing signed profile templates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileCatalogManifest {
    pub publisher_pubkey: String,
    pub entries: Vec<SignedProfileEntry>,
}

/// Resolved signed profile ready for evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignedProfileTemplate {
    pub name: String,
    pub version: String,
    pub verified: bool,
    pub profile: ComplianceProfile,
    pub content_sha256: String,
}

fn catalog_manifest_json() -> &'static str {
    include_str!("../templates/catalog.json")
}

fn template_json(path: &str) -> Option<&'static str> {
    match path {
        "templates/defense.json" => Some(include_str!("../templates/defense.json")),
        "templates/medical.json" => Some(include_str!("../templates/medical.json")),
        "templates/iso26262.json" => Some(include_str!("../templates/iso26262.json")),
        _ => None,
    }
}

/// Load and verify the built-in signed profile catalog.
pub fn load_signed_profile_catalog() -> Result<Vec<SignedProfileTemplate>, String> {
    let manifest: ProfileCatalogManifest =
        serde_json::from_str(catalog_manifest_json()).map_err(|error| error.to_string())?;
    let expected_pubkey = public_key_from_material(CATALOG_SIGNING_MATERIAL);
    if manifest.publisher_pubkey != expected_pubkey {
        return Err("catalog publisher pubkey mismatch".into());
    }
    let mut templates = Vec::new();
    for entry in manifest.entries {
        let Some(raw) = template_json(&entry.template_path) else {
            return Err(format!("missing template {}", entry.template_path));
        };
        let hash = sha256(raw);
        if hash.0 != entry.content_sha256 {
            return Err(format!("catalog hash mismatch for profile {}", entry.name));
        }
        let verified = verify_signature(
            &entry.content_sha256,
            &entry.signature,
            &manifest.publisher_pubkey,
        );
        let mut profile: ComplianceProfile =
            serde_json::from_str(raw).map_err(|error| error.to_string())?;
        profile.template_notice = crate::profiles::template_notice();
        templates.push(SignedProfileTemplate {
            name: entry.name,
            version: entry.version,
            verified,
            profile,
            content_sha256: entry.content_sha256,
        });
    }
    Ok(templates)
}

/// Resolve a signed profile by name when signature verification succeeded.
pub fn signed_profile_by_name(name: &str) -> Option<ComplianceProfile> {
    load_signed_profile_catalog()
        .ok()?
        .into_iter()
        .find(|entry| entry.name.eq_ignore_ascii_case(name) && entry.verified)
        .map(|entry| entry.profile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_audit::crypto::sign;

    #[test]
    fn catalog_entries_verify() {
        let templates = load_signed_profile_catalog().expect("catalog");
        assert!(templates
            .iter()
            .any(|entry| entry.name == "defense" && entry.verified));
        assert!(templates
            .iter()
            .any(|entry| entry.name == "medical" && entry.verified));
        assert!(templates
            .iter()
            .any(|entry| entry.name == "iso26262" && entry.verified));
    }

    #[test]
    #[ignore = "run manually to regenerate catalog signatures"]
    fn print_catalog_signatures() {
        let pubkey = public_key_from_material(CATALOG_SIGNING_MATERIAL);
        println!("publisher_pubkey={pubkey}");
        for (name, path) in [
            ("defense", "templates/defense.json"),
            ("medical", "templates/medical.json"),
            ("iso26262", "templates/iso26262.json"),
        ] {
            let raw = template_json(path).expect("template");
            let hash = sha256(raw);
            let signature = sign(&hash.0, CATALOG_SIGNING_MATERIAL);
            println!("{name} sha256={} signature={signature}", hash.0);
        }
    }
}
