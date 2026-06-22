//! Host hooks for domain-specific interpreter behavior in `spanda-core`.
//!

/// Domain-specific runtime services supplied by the embedding application.
pub trait RuntimeHost {
    /// Whether an import path enables SLAM adapter hooks.
    fn slam_import_known(&self, path: &str) -> bool;

    /// Whether an import path enables navigation adapter hooks.
    fn navigation_import_known(&self, path: &str) -> bool;
}

/// Return true when any import path enables SLAM adapter behavior.
pub fn imports_enable_slam(paths: &[&str], host: &dyn RuntimeHost) -> bool {
    paths.iter().any(|path| host.slam_import_known(path))
}

/// Return true when any import path enables navigation adapter behavior.
pub fn imports_enable_navigation(paths: &[&str], host: &dyn RuntimeHost) -> bool {
    paths.iter().any(|path| host.navigation_import_known(path))
}
