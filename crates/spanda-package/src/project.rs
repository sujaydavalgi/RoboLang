use crate::error::{PackageError, PackageResult};
use crate::manifest::{PackageManifest, MANIFEST_FILENAME};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_MAIN: &str = r#"module main;

import std.robotics;
import std.sensors;

robot WarehouseBot {
    sensor front_lidar: LidarScan;
    actuator drive: MotionCommand;

    behavior navigate {
        // TODO: implement navigation
    }
}
"#;

const DEFAULT_README: &str = r#"# Spanda Package

Created with `spanda init`.

## Commands

- `spanda check` — type-check sources
- `spanda build` — compile the project
- `spanda test` — run tests
- `spanda install` — resolve dependencies and write spanda.lock
"#;

/// Initialize a new Spanda package in `dir`.
pub fn init_package(
    dir: &Path,
    name: Option<&str>,
    description: Option<&str>,
) -> PackageResult<PathBuf> {
    let pkg_name = name
        .map(str::to_string)
        .or_else(|| dir.file_name().and_then(|n| n.to_str()).map(str::to_string))
        .unwrap_or_else(|| "my_robot".into());

    fs::create_dir_all(dir).map_err(PackageError::from)?;
    fs::create_dir_all(dir.join("src")).map_err(PackageError::from)?;
    fs::create_dir_all(dir.join("tests")).map_err(PackageError::from)?;

    let manifest = PackageManifest {
        package: crate::manifest::PackageSection {
            name: pkg_name.clone(),
            version: "0.1.0".into(),
            description: description.map(str::to_string),
            license: Some("Apache-2.0".into()),
            authors: vec![],
        },
        dependencies: Default::default(),
        dev_dependencies: Default::default(),
        hardware: Default::default(),
        capabilities: Default::default(),
        requires_hardware: Default::default(),
        safety: Default::default(),
        adapter: Default::default(),
        categories: vec![],
        license_compat: vec![],
    };

    manifest.save(&dir.join(MANIFEST_FILENAME))?;
    fs::write(dir.join("src/main.sd"), DEFAULT_MAIN).map_err(PackageError::from)?;
    fs::write(dir.join("README.md"), DEFAULT_README).map_err(PackageError::from)?;

    Ok(dir.to_path_buf())
}

/// Collect `.sd` source files from a project (src/ and tests/).
pub fn collect_source_files(project_root: &Path) -> PackageResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    for sub in ["src", "tests"] {
        let dir = project_root.join(sub);
        if dir.is_dir() {
            collect_sd_files(&dir, &mut files)?;
        }
    }
    if files.is_empty() {
        if let Ok(entries) = fs::read_dir(project_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "sd") {
                    files.push(path);
                }
            }
        }
    }
    Ok(files)
}

fn collect_sd_files(dir: &Path, out: &mut Vec<PathBuf>) -> PackageResult<()> {
    for entry in fs::read_dir(dir).map_err(PackageError::from)? {
        let entry = entry.map_err(PackageError::from)?;
        let path = entry.path();
        if path.is_dir() {
            collect_sd_files(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "sd") {
            out.push(path);
        }
    }
    Ok(())
}

/// Add a dependency to the manifest and save.
pub fn add_dependency(
    project_root: &Path,
    name: &str,
    spec: crate::dependency::DependencySpec,
) -> PackageResult<()> {
    let manifest_path = project_root.join(MANIFEST_FILENAME);
    let mut manifest = PackageManifest::load(&manifest_path)?;
    manifest.dependencies.insert(name.to_string(), spec);
    manifest.save(&manifest_path)
}

/// Remove a dependency from the manifest and save.
pub fn remove_dependency(project_root: &Path, name: &str) -> PackageResult<bool> {
    let manifest_path = project_root.join(MANIFEST_FILENAME);
    let mut manifest = PackageManifest::load(&manifest_path)?;
    let removed = manifest.dependencies.remove(name).is_some();
    if removed {
        manifest.save(&manifest_path)?;
    }
    Ok(removed)
}
