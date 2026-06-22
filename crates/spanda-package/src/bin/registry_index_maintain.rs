//! Refresh hosted registry checksums and Ed25519 signatures in `registry/index.json`.
//!
//! Usage:
//!   cargo run -p spanda-package --bin registry-index-maintain
//!   cargo run -p spanda-package --bin registry-index-maintain -- --verify
//!
//! Signing material: `SPANDA_REGISTRY_SIGN_KEY`, or the hosted default
//! `spanda-hosted-registry-v1` (public key in `registry/TRUST_KEY`).

use spanda_package::registry_sign::{sign_registry_tarball, verify_registry_signature};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const HOSTED_SIGN_MATERIAL: &str = "spanda-hosted-registry-v1";

#[derive(serde::Deserialize, serde::Serialize)]
struct IndexEntry {
    name: String,
    #[serde(default)]
    description: String,
    versions: Vec<String>,
    #[serde(default)]
    category: String,
    #[serde(default)]
    license: String,
    #[serde(default)]
    import_paths: Vec<String>,
    #[serde(default)]
    version_checksums: BTreeMap<String, String>,
    #[serde(default)]
    version_signatures: BTreeMap<String, spanda_package::registry_sign::RegistryVersionSignature>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sha256_file(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn signing_material() -> String {
    env::var("SPANDA_REGISTRY_SIGN_KEY")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| HOSTED_SIGN_MATERIAL.to_string())
}

fn maintain(verify_only: bool) -> Result<(), String> {
    let root = repo_root();
    let index_path = root.join("registry/index.json");
    let packages_dir = root.join("registry/packages");
    let body = fs::read_to_string(&index_path)
        .map_err(|err| format!("read {}: {err}", index_path.display()))?;
    let mut entries: Vec<IndexEntry> =
        serde_json::from_str(&body).map_err(|err| format!("parse index.json: {err}"))?;
    let sign_key = signing_material();
    let mut checksum_updates = 0usize;
    let mut signature_updates = 0usize;

    for entry in &mut entries {
        let mut checksums = BTreeMap::new();
        let mut signatures = BTreeMap::new();
        for version in &entry.versions {
            let tarball = packages_dir.join(&entry.name).join(version);
            if !tarball.is_file() {
                return Err(format!("missing tarball: {}", tarball.display()));
            }
            let digest = sha256_file(&tarball)?;
            let signed = sign_registry_tarball(&entry.name, version, &digest, &sign_key);
            if verify_only {
                let expected_checksum = entry
                    .version_checksums
                    .get(version)
                    .ok_or_else(|| format!("missing checksum for {}/{}", entry.name, version))?;
                if expected_checksum != &digest {
                    return Err(format!(
                        "checksum mismatch for {}/{}: expected {expected_checksum}, got {digest}",
                        entry.name, version
                    ));
                }
                let expected_sig = entry.version_signatures.get(version).ok_or_else(|| {
                    format!("missing signature for {}/{}", entry.name, version)
                })?;
                if !verify_registry_signature(
                    &entry.name,
                    version,
                    &digest,
                    expected_sig,
                    &expected_sig.public_key,
                ) {
                    return Err(format!(
                        "invalid signature for {}/{}",
                        entry.name, version
                    ));
                }
            } else {
                if entry.version_checksums.get(version) != Some(&digest) {
                    checksum_updates += 1;
                }
                if entry.version_signatures.get(version) != Some(&signed) {
                    signature_updates += 1;
                }
                let sidecar = tarball.parent().unwrap().join(format!("{version}.sha256"));
                fs::write(&sidecar, format!("{digest}\n"))
                    .map_err(|err| format!("write {}: {err}", sidecar.display()))?;
            }
            checksums.insert(version.clone(), digest);
            signatures.insert(version.clone(), signed);
        }
        if !verify_only {
            entry.version_checksums = checksums;
            entry.version_signatures = signatures;
        }
    }

    if verify_only {
        println!("✓ registry index checksums and signatures verified");
        return Ok(());
    }

    fs::write(
        &index_path,
        serde_json::to_string_pretty(&entries).map_err(|err| err.to_string())? + "\n",
    )
    .map_err(|err| format!("write {}: {err}", index_path.display()))?;
    println!(
        "✓ updated {checksum_updates} checksum(s) and {signature_updates} signature(s) in {}",
        index_path.strip_prefix(&root).unwrap_or(&index_path).display()
    );
    Ok(())
}

fn main() {
    let verify_only = env::args().any(|arg| arg == "--verify");
    if let Err(err) = maintain(verify_only) {
        eprintln!("registry-index-maintain: {err}");
        std::process::exit(1);
    }
}
