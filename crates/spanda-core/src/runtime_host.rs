//! Core-backed [`RuntimeHost`] hooks for the extracted runtime kernel.
//!
use crate::nav2_adapter;
use crate::slam_adapter;
use spanda_runtime::RuntimeHost;

/// Default host wiring domain adapters from `spanda-core` into `spanda-runtime`.
pub struct CoreRuntimeHost;

impl RuntimeHost for CoreRuntimeHost {
    fn slam_import_known(&self, path: &str) -> bool {
        slam_adapter::slam_import_paths().contains(&path)
    }

    fn navigation_import_known(&self, path: &str) -> bool {
        nav2_adapter::nav2_import_paths().contains(&path)
    }
}

/// Shared core runtime host instance for interpreter wiring.
pub fn core_runtime_host() -> &'static CoreRuntimeHost {
    &CoreRuntimeHost
}
