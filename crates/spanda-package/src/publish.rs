//! Package bundle creation and optional remote registry upload.

use crate::error::{PackageError, PackageResult};
use crate::manifest::{PackageManifest, MANIFEST_FILENAME};
use crate::project::collect_source_files;
use crate::registry_remote::registry_base_url;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishReport {
    pub bundle_path: PathBuf,
    pub uploaded: bool,
    pub upload_url: Option<String>,
}

/// Create a `.tar.gz` bundle containing manifest, lockfile, and source files.
pub fn bundle_package(root: &Path, manifest: &PackageManifest) -> PackageResult<PublishReport> {
    let sources = collect_source_files(root)?;
    if sources.is_empty() {
        return Err(PackageError::Validation(
            "no source files to publish".into(),
        ));
    }

    let dist = root.join("dist");
    fs::create_dir_all(&dist).map_err(PackageError::from)?;
    let bundle_name = format!("{}-{}.tar.gz", manifest.package.name, manifest.package.version);
    let bundle_path = dist.join(bundle_name);

    let mut paths = vec![root.join(MANIFEST_FILENAME)];
    let lock = root.join(crate::lockfile::LOCKFILE_FILENAME);
    if lock.exists() {
        paths.push(lock);
    }
    paths.extend(sources);

    create_tar_gz(&bundle_path, root, &paths)?;

    Ok(PublishReport {
        bundle_path,
        uploaded: false,
        upload_url: None,
    })
}

/// Bundle the package and optionally upload to `SPANDA_REGISTRY_URL`.
pub fn publish_package(root: &Path, manifest: &PackageManifest) -> PackageResult<PublishReport> {
    let mut report = bundle_package(root, manifest)?;
    if let Some(base) = registry_base_url() {
        let url = format!(
            "{base}/packages/{}/{}",
            manifest.package.name, manifest.package.version
        );
        match upload_bundle(&report.bundle_path, &url) {
            Ok(()) => {
                report.uploaded = true;
                report.upload_url = Some(url);
            }
            Err(err) => {
                eprintln!("Warning: registry upload failed: {err}");
                eprintln!("  Bundle written to {}", report.bundle_path.display());
            }
        }
    }
    Ok(report)
}

fn create_tar_gz(output: &Path, root: &Path, files: &[PathBuf]) -> PackageResult<()> {
    let rel_paths: Vec<String> = files
        .iter()
        .filter_map(|path| path.strip_prefix(root).ok())
        .map(|path| path.to_string_lossy().into_owned())
        .collect();

    let status = Command::new("tar")
        .arg("-czf")
        .arg(output)
        .arg("-C")
        .arg(root)
        .args(&rel_paths)
        .status()
        .map_err(PackageError::from)?;

    if status.success() {
        Ok(())
    } else {
        Err(PackageError::Validation(format!(
            "tar failed creating bundle (exit {status})"
        )))
    }
}

fn upload_bundle(bundle: &Path, url: &str) -> Result<(), String> {
    let output = Command::new("curl")
        .args([
            "-fsSL",
            "-X",
            "PUT",
            "-H",
            "Content-Type: application/gzip",
            "--data-binary",
            &format!("@{}", bundle.display()),
            url,
        ])
        .output()
        .map_err(|e| format!("curl not available: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "upload failed (exit {}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::PackageSection;
    use std::collections::HashMap;

    #[test]
    fn bundles_manifest_and_sources() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        fs::create_dir_all(root.join("src")).expect("src");
        fs::write(
            root.join(MANIFEST_FILENAME),
            "[package]\nname = \"demo-pkg\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(root.join("src/main.sd"), "robot R { behavior run() {} }").unwrap();
        let manifest = PackageManifest {
            package: PackageSection {
                name: "demo-pkg".into(),
                version: "0.1.0".into(),
                description: None,
                license: None,
                authors: vec![],
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            hardware: Default::default(),
            capabilities: Default::default(),
            requires_hardware: Default::default(),
            safety: Default::default(),
            adapter: Default::default(),
            categories: vec![],
            license_compat: vec![],
        };
        let report = bundle_package(root, &manifest).expect("bundle");
        assert!(report.bundle_path.exists());
        assert!(!report.uploaded);
    }
}
