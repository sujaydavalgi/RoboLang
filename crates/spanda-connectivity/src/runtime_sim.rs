//! Runtime-facing connectivity simulation helpers without AST/runtime dependencies.
//!
use std::collections::HashSet;

/// Runtime geofence circle loaded from a program declaration.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GeofenceRuntime {
    pub name: String,
    pub center_lat: f64,
    pub center_lon: f64,
    pub radius_m: f64,
}

/// Haversine distance in meters between two WGS84 points.
pub fn haversine_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6_371_000.0;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
    2.0 * R * a.sqrt().asin()
}

/// Return true when `(lat, lon)` is inside the geofence circle.
pub fn geofence_contains(fence: &GeofenceRuntime, lat: f64, lon: f64) -> bool {
    haversine_m(lat, lon, fence.center_lat, fence.center_lon) <= fence.radius_m
}

/// Map hardware monitor events to connectivity trigger `(domain, event)` pairs.
pub fn hardware_event_to_connectivity(event: &str) -> Option<(&'static str, &'static str)> {
    match event {
        "GpsFailure" => Some(("gps", "lost")),
        _ => None,
    }
}

/// Map comm bus fault names to connectivity trigger pairs.
pub fn fault_to_connectivity(fault: &str) -> Option<(&'static str, &'static str)> {
    match fault {
        "NetworkOutage" | "LteOutage" | "SatelliteOutage" | "WeakWifi" => {
            Some(("network", "disconnected"))
        }
        "BluetoothDisconnect" => Some(("bluetooth", "device_disconnected")),
        "FiveGHandoff" => Some(("cellular", "roaming")),
        "GpsSpoofing" => Some(("gps", "spoofed")),
        "GpsDrift" => Some(("gps", "drift")),
        _ => None,
    }
}

/// Apply GPS drift/spoofing simulation to WGS84 coordinates.
pub fn apply_gps_position_faults(
    faults: &HashSet<String>,
    true_lat: f64,
    true_lon: f64,
    sim_time_ms: f64,
) -> (f64, f64, f64) {
    if faults.contains("GpsSpoofing") {
        return (true_lat + 0.009, true_lon + 0.012, 0.3);
    }
    if faults.contains("GpsDrift") {
        let drift_m = (sim_time_ms / 1000.0) * 0.05;
        let d_deg = drift_m / 111_000.0;
        return (true_lat + d_deg, true_lon + d_deg * 0.5, 0.8);
    }
    (true_lat, true_lon, 1.0)
}

/// Return true when simulation faults disable the given connectivity link.
pub fn is_link_impaired(link: &str, faults: &HashSet<String>) -> bool {
    let link = link.to_ascii_lowercase();
    for fault in faults {
        match fault.as_str() {
            "NetworkOutage" => {
                if super::is_satellite_link(&link) || link == "bluetooth" || link == "ble" {
                    continue;
                }
                if super::is_wifi_link(&link)
                    || super::is_cellular_link(&link)
                    || link == "network"
                    || link == "ethernet"
                {
                    return true;
                }
            }
            "WeakWifi" if super::is_wifi_link(&link) || link == "network" => return true,
            "LteOutage" if super::is_cellular_link(&link) => return true,
            "SatelliteOutage" if super::is_satellite_link(&link) => return true,
            _ => {}
        }
    }
    false
}

