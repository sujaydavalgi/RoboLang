//! Resolve installed official lean-core packages from project manifests.
//!
use crate::adapter::framework_packages;
use crate::error::{PackageError, PackageResult};
use crate::lockfile::{Lockfile, LOCKFILE_FILENAME};
use crate::manifest::{PackageManifest, MANIFEST_FILENAME};
use std::collections::HashSet;
use std::path::Path;

/// Return dependency names that match known official framework packages.
pub fn installed_official_packages<'a>(
    dependency_names: impl IntoIterator<Item = &'a str>,
) -> Vec<&'static str> {
    // Collect installed official package names from a dependency list.
    //
    // Parameters:
    // - `dependency_names` — keys from `spanda.toml` `[dependencies]`
    //
    // Returns:
    // Sorted list of official package names present in the manifest.
    //
    // Options:
    // None.
    //
    // Example:
    // let names = installed_official_packages(["spanda-ros2", "my-local-lib"]);

    let official: HashSet<&str> = framework_packages().iter().map(|p| p.name).collect();
    let mut found: Vec<&str> = dependency_names
        .into_iter()
        .filter_map(|name| official.get(name).copied())
        .collect();
    found.sort_unstable();
    found.dedup();
    found
}

/// Whether a package name is a registered official framework package.
pub fn is_official_package(name: &str) -> bool {
    // Check if a package name is in the official framework catalog.
    //
    // Parameters:
    // - `name` — candidate package name
    //
    // Returns:
    // True when the name appears in `framework_packages()`.
    //
    // Options:
    // None.
    //
    // Example:
    // assert!(is_official_package("spanda-gps"));

    framework_packages().iter().any(|p| p.name == name)
}

/// Resolve official packages declared in a project manifest.
pub fn official_packages_from_manifest(manifest: &PackageManifest) -> Vec<String> {
    installed_official_packages(manifest.dependencies.keys().map(String::as_str))
        .into_iter()
        .map(str::to_string)
        .collect()
}

/// Resolve official packages from a resolved lockfile.
pub fn official_packages_from_lockfile(lockfile: &Lockfile) -> Vec<String> {
    installed_official_packages(lockfile.dependencies.keys().map(String::as_str))
        .into_iter()
        .map(str::to_string)
        .collect()
}

/// Load official package names for a project directory (prefers lockfile over manifest).
pub fn load_official_packages_for_project(root: &Path) -> PackageResult<Vec<String>> {
    // Load official package dependency names for a Spanda project root.
    //
    // Parameters:
    // - `root` — directory containing `spanda.toml`
    //
    // Returns:
    // Official package names from lockfile when present, otherwise manifest deps.
    //
    // Options:
    // None.
    //
    // Example:
    // let names = load_official_packages_for_project(project_root)?;

    let lock_path = root.join(LOCKFILE_FILENAME);
    if lock_path.is_file() {
        let lockfile = Lockfile::load(&lock_path)?;
        return Ok(official_packages_from_lockfile(&lockfile));
    }
    let manifest_path = root.join(MANIFEST_FILENAME);
    if manifest_path.is_file() {
        let manifest = PackageManifest::load_from_dir(root)?;
        return Ok(official_packages_from_manifest(&manifest));
    }
    Err(PackageError::Manifest(format!(
        "no {MANIFEST_FILENAME} or {LOCKFILE_FILENAME} in {}",
        root.display()
    )))
}

/// Resolve official packages for a source file by walking up to the project root.
pub fn load_official_packages_for_source(source: &Path) -> Vec<String> {
    // Resolve official packages for a `.sd` source path.
    //
    // Parameters:
    // - `source` — path to a Spanda source file or directory
    //
    // Returns:
    // Official package names when a project root is found; empty otherwise.
    //
    // Options:
    // None.
    //
    // Example:
    // let names = load_official_packages_for_source(Path::new("examples/packages/basic_project/src/main.sd"));

    let start = if source.is_dir() {
        source.to_path_buf()
    } else {
        source
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    };
    let Some(root) = crate::manifest::find_project_root(&start) else {
        return Vec::new();
    };
    load_official_packages_for_project(&root).unwrap_or_default()
}
