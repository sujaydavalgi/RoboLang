//! Certification evidence pack CLI formatting and bundle I/O.

use std::fs;
use std::path::Path;

pub use spanda_assurance::{build_certification_pack, CertificationPack};

/// Persist a certification pack to a JSON file or directory bundle path.
pub fn write_certification_bundle(pack: &CertificationPack, path: &Path) -> std::io::Result<()> {
    if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
        let content = serde_json::to_string_pretty(pack)?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        return fs::write(path, content);
    }
    fs::create_dir_all(path)?;
    fs::write(
        path.join("certification-pack.json"),
        serde_json::to_string_pretty(pack)?,
    )?;
    for item in &pack.evidence {
        fs::write(
            path.join(format!("{}.json", item.kind)),
            serde_json::to_string_pretty(&item.payload)?,
        )?;
    }
    Ok(())
}

/// Format certification pack summary for CLI output.
pub fn format_certification_pack(pack: &CertificationPack, json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(pack).unwrap_or_else(|e| e.to_string());
    }
    let mut lines = vec![
        format!("Certification pack: {}", pack.program),
        format!("Passed: {}", pack.passed),
        format!("Evidence items: {}", pack.evidence.len()),
    ];
    for item in &pack.evidence {
        lines.push(format!("  {} passed={}", item.kind, item.passed));
    }
    lines.join("\n")
}
