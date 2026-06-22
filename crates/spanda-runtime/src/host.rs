//! Host hooks for domain-specific interpreter behavior in `spanda-core`.
//!

use spanda_ast::comm_decl::TransportKind;
use std::collections::HashSet;

/// Domain-specific runtime services supplied by the embedding application.
pub trait RuntimeHost {
    /// Whether an import path enables SLAM adapter hooks.
    fn slam_import_known(&self, path: &str) -> bool;

    /// Whether an import path enables navigation adapter hooks.
    fn navigation_import_known(&self, path: &str) -> bool;

    /// Invoke an external Nav2 bridge when the host configures one.
    fn invoke_nav2_bridge(&self, goal: &str) -> Option<String> {
        let _ = goal;
        None
    }

    /// Invoke an external SLAM bridge when the host configures one.
    fn invoke_slam_bridge(&self, op: &str) -> Option<String> {
        let _ = op;
        None
    }

    /// Map an active connectivity link name to the default transport backend.
    fn connectivity_link_to_transport(&self, link: &str) -> TransportKind {
        let _ = link;
        TransportKind::Sim
    }

    /// Map a hardware event to a connectivity trigger `(domain, event)` pair.
    fn hardware_event_to_connectivity(&self, event: &str) -> Option<(&'static str, &'static str)> {
        let _ = event;
        None
    }

    /// Map a fault string to a connectivity trigger `(domain, event)` pair.
    fn fault_to_connectivity(&self, fault: &str) -> Option<(&'static str, &'static str)> {
        let _ = fault;
        None
    }

    /// Return true when the active link should be considered impaired by current faults.
    fn is_link_impaired(&self, link: &str, faults: &HashSet<String>) -> bool {
        let _ = (link, faults);
        false
    }

    /// Apply GPS drift/spoof faults to the true lat/lon at simulation time.
    fn apply_gps_position_faults(
        &self,
        faults: &HashSet<String>,
        true_lat: f64,
        true_lon: f64,
        sim_time_ms: f64,
    ) -> (f64, f64, f64) {
        let _ = (faults, sim_time_ms);
        (true_lat, true_lon, 1.0)
    }

    /// Return true when `(lat, lon)` is inside a geofence circle.
    fn geofence_contains(
        &self,
        center_lat: f64,
        center_lon: f64,
        radius_m: f64,
        lat: f64,
        lon: f64,
    ) -> bool {
        let _ = (center_lat, center_lon, radius_m, lat, lon);
        false
    }
}

/// Return true when any import path enables SLAM adapter behavior.
pub fn imports_enable_slam(paths: &[&str], host: &dyn RuntimeHost) -> bool {
    paths.iter().any(|path| host.slam_import_known(path))
}

/// Return true when any import path enables navigation adapter behavior.
pub fn imports_enable_navigation(paths: &[&str], host: &dyn RuntimeHost) -> bool {
    paths.iter().any(|path| host.navigation_import_known(path))
}
