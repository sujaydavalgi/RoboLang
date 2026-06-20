use crate::dependency::LockedDependency;
use crate::error::{PackageError, PackageResult};
use crate::manifest::PackageManifest;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

pub const LOCKFILE_FILENAME: &str = "spanda.lock";

/// Resolved dependency graph written to `spanda.lock`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lockfile {
    pub version: u32,
    pub package: LockPackageInfo,
    pub dependencies: BTreeMap<String, LockedDependency>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockPackageInfo {
    pub name: String,
    pub version: String,
}

impl Lockfile {
    pub fn new(manifest: &PackageManifest, deps: BTreeMap<String, LockedDependency>) -> Self {
        Self {
            version: 1,
            package: LockPackageInfo {
                name: manifest.package.name.clone(),
                version: manifest.package.version.clone(),
            },
            dependencies: deps,
        }
    }

    pub fn parse_str(content: &str) -> PackageResult<Self> {
        serde_json::from_str(content).map_err(|e| PackageError::Lockfile(e.to_string()))
    }

    pub fn load(path: &Path) -> PackageResult<Self> {
        let content = std::fs::read_to_string(path).map_err(PackageError::from)?;
        Self::parse_str(&content)
    }

    pub fn load_from_dir(dir: &Path) -> PackageResult<Self> {
        Self::load(&dir.join(LOCKFILE_FILENAME))
    }

    pub fn save(&self, path: &Path) -> PackageResult<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| PackageError::Lockfile(e.to_string()))?;
        std::fs::write(path, content).map_err(PackageError::from)?;
        Ok(())
    }

    pub fn save_to_dir(&self, dir: &Path) -> PackageResult<()> {
        self.save(&dir.join(LOCKFILE_FILENAME))
    }
}
