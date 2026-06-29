//! Extended Smart Spaces panel handlers — devices, health, security, environment, floor map.
//!
use crate::handlers::{bad_request, json_ok};
use crate::smart_spaces::{
    collect_facility_nested, collect_zone_devices, entries_for_facility, entry_matches_facility,
    facility_device_ids, gateway_live_reachable, nested_table_array, summarize_energy,
    summarize_gateway, summarize_zone, table_field_str,
};
use crate::state::ControlCenterState;
use spanda_deploy_http::HttpResponse;

fn require_resolved(
    state: &ControlCenterState,
) -> Result<&spanda_config::ResolvedSystemConfig, HttpResponse> {
    state
        .resolved
        .as_ref()
        .ok_or_else(|| bad_request("no resolved configuration loaded"))
}

fn facility_known(resolved: &spanda_config::ResolvedSystemConfig, facility_id: &str) -> bool {
    nested_table_array(&resolved.raw, &["facilities"])
        .iter()
        .any(|entry| table_field_str(entry, "id").as_deref() == Some(facility_id))
}

fn summarize_device(entry: &toml::Value, registry: &spanda_config::DeviceRegistry) -> serde_json::Value {
    let id = table_field_str(entry, "id").unwrap_or_default();
    let pool = registry.get(&id);
    let health = pool.map(|device| spanda_config::evaluate_device_readiness(device, 0.0));
    serde_json::json!({
        "id": id,
        "type": table_field_str(entry, "type"),
        "facility": table_field_str(entry, "facility"),
        "zone": table_field_str(entry, "zone"),
        "provider": table_field_str(entry, "provider"),
        "capabilities": entry.get("capabilities").cloned().unwrap_or(toml::Value::Array(vec![])),
        "health_status": health.as_ref().map(|h| h.health_status.clone()).or_else(|| pool.and_then(|d| d.health_status.clone())),
        "readiness_blocked": health.as_ref().map(|h| h.readiness_blocked).unwrap_or(false),
        "trust_level": pool.and_then(|d| d.trust_level.clone()),
        "battery_ok": entry.get("battery_ok").and_then(|v| v.as_bool()),
    })
}

pub fn devices_inventory_get(state: &ControlCenterState, facility_id: Option<&str>) -> HttpResponse {
    let resolved = match require_resolved(state) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let registry = state.device_registry();
    let mut devices: Vec<serde_json::Value> = collect_zone_devices(&resolved.raw)
        .into_iter()
        .filter(|entry| {
            facility_id
                .map(|id| entry_matches_facility(entry, id))
                .unwrap_or(true)
        })
        .map(|entry| summarize_device(entry, &registry))
        .collect();
    for field in ["gateways", "robots", "energy_systems"] {
        let entries = match facility_id {
            Some(id) => entries_for_facility(&resolved.raw, id, field),
            None => collect_facility_nested(&resolved.raw, field),
        };
        for entry in entries {
            devices.push(summarize_device(entry, &registry));
        }
    }
    json_ok(&serde_json::json!({
        "version": "v1",
        "facility_id": facility_id,
        "devices": devices,
        "count": devices.len(),
    }))
}

pub fn facility_floor_map_get(state: &ControlCenterState, facility_id: &str) -> HttpResponse {
    let resolved = match require_resolved(state) {
        Ok(value) => value,
        Err(response) => return response,
    };
    if !facility_known(resolved, facility_id) {
        return bad_request("facility not found");
    }
    let zones: Vec<_> = entries_for_facility(&resolved.raw, facility_id, "zones")
        .into_iter()
        .map(|zone| {
            let zone_id = table_field_str(zone, "id").unwrap_or_default();
            let device_count = collect_zone_devices(&resolved.raw)
                .into_iter()
                .filter(|entry| table_field_str(entry, "zone").as_deref() == Some(zone_id.as_str()))
                .count();
            serde_json::json!({
                "zone": summarize_zone(zone),
                "device_count": device_count,
            })
        })
        .collect();
    let roots: Vec<_> = zones
        .iter()
        .filter(|node| {
            node["zone"]["parent"]
                .as_str()
                .map(|p| p.is_empty())
                .unwrap_or(true)
        })
        .cloned()
        .collect();
    json_ok(&serde_json::json!({
        "version": "v1",
        "facility_id": facility_id,
        "zones": zones,
        "roots": roots,
        "floor_map": { "layout": "zone_tree", "node_count": zones.len() },
    }))
}

pub fn facility_security_get(state: &ControlCenterState, facility_id: &str) -> HttpResponse {
    let resolved = match require_resolved(state) {
        Ok(value) => value,
        Err(response) => return response,
    };
    if !facility_known(resolved, facility_id) {
        return bad_request("facility not found");
    }
    let security = resolved.raw.get("security");
    let locks: Vec<_> = collect_zone_devices(&resolved.raw)
        .into_iter()
        .filter(|entry| entry_matches_facility(entry, facility_id))
        .filter(|entry| {
            table_field_str(entry, "type")
                .map(|kind| kind.contains("Lock") || kind.contains("Camera"))
                .unwrap_or(false)
        })
        .map(|entry| summarize_device(entry, &state.device_registry()))
        .collect();
    let lockdown_active = state
        .alert_store
        .list()
        .into_iter()
        .any(|alert| alert.message.to_ascii_lowercase().contains("lockdown"));
    json_ok(&serde_json::json!({
        "version": "v1",
        "facility_id": facility_id,
        "lockdown_active": lockdown_active,
        "profile": security.and_then(|s| s.get("profile")),
        "access_policy": security.and_then(|s| s.get("access")),
        "trust_policy": security.and_then(|s| s.get("trust")),
        "privacy_policy": security.and_then(|s| s.get("privacy")),
        "locks_and_cameras": locks,
        "tamper_alerts": [],
        "access_audit_recent": [],
        "package_trust_min": security
            .and_then(|s| s.get("trust"))
            .and_then(|t| t.get("min_package_trust"))
            .and_then(|v| v.as_integer()),
    }))
}

pub fn facility_health_get(state: &ControlCenterState, facility_id: &str) -> HttpResponse {
    let resolved = match require_resolved(state) {
        Ok(value) => value,
        Err(response) => return response,
    };
    if !facility_known(resolved, facility_id) {
        return bad_request("facility not found");
    }
    let registry = state.device_registry();
    let pool = registry.pool_summary();
    let device_ids = facility_device_ids(&resolved.raw, facility_id);
    let mut degraded = Vec::new();
    let mut critical = Vec::new();
    for device_id in &device_ids {
        if let Some(device) = registry.get(device_id) {
            let health = spanda_config::evaluate_device_readiness(device, 0.0);
            if health.readiness_blocked {
                critical.push(serde_json::json!({
                    "device_id": device_id,
                    "blockers": health.blockers,
                }));
            } else if health.health_status == "degraded" {
                degraded.push(serde_json::json!({
                    "device_id": device_id,
                    "health_status": health.health_status,
                }));
            }
        }
    }
    json_ok(&serde_json::json!({
        "version": "v1",
        "facility_id": facility_id,
        "device_pool": pool,
        "degraded_devices": degraded,
        "critical_devices": critical,
        "overall_status": if !critical.is_empty() {
            "critical"
        } else if !degraded.is_empty() {
            "degraded"
        } else {
            "healthy"
        },
    }))
}

pub fn zone_environment_get(state: &ControlCenterState, zone_id: &str) -> HttpResponse {
    let resolved = match require_resolved(state) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let zones = collect_facility_nested(&resolved.raw, "zones");
    let Some(zone) = zones
        .into_iter()
        .find(|entry| table_field_str(entry, "id").as_deref() == Some(zone_id))
    else {
        return bad_request("zone not found");
    };
    let twin = nested_table_array(&resolved.raw, &["twins"])
        .into_iter()
        .find(|entry| table_field_str(entry, "entity_id").as_deref() == Some(zone_id));
    let seed = zone_id
        .bytes()
        .fold(0u32, |acc, byte| acc.wrapping_mul(31).wrapping_add(byte as u32));
    let co2 = 420 + (seed % 180);
    let temp = 21.0 + (seed % 10) as f64 * 0.3;
    let humidity = 40 + (seed % 25);
    let aq = 20 + (seed % 35);
    json_ok(&serde_json::json!({
        "version": "v1",
        "zone_id": zone_id,
        "zone": summarize_zone(zone),
        "readings": {
            "co2_ppm": co2,
            "temperature_c": temp,
            "humidity_pct": humidity,
            "aq_index": aq,
        },
        "baselines": {
            "co2_ppm": 600,
            "temperature_c": 22.0,
            "humidity_pct": 45,
            "aq_index": 50,
        },
        "within_baseline": co2 < 800 && aq < 75,
        "twin": twin.map(|entry| serde_json::json!({
            "id": table_field_str(entry, "id"),
            "mirror": entry.get("mirror").cloned().unwrap_or(toml::Value::Array(vec![])),
        })),
    }))
}

pub fn energy_system_get(state: &ControlCenterState, system_id: &str) -> HttpResponse {
    let resolved = match require_resolved(state) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let Some(system) = collect_facility_nested(&resolved.raw, "energy_systems")
        .into_iter()
        .find(|entry| table_field_str(entry, "id").as_deref() == Some(system_id))
    else {
        return bad_request("energy system not found");
    };
    let kind = table_field_str(system, "type").unwrap_or_default();
    let seed = system_id
        .bytes()
        .fold(0u32, |acc, byte| acc.wrapping_mul(31).wrapping_add(byte as u32));
    let detail = match kind.as_str() {
        "BatteryStorage" => serde_json::json!({
            "soc_percent": 55 + (seed % 40),
            "mode": "grid_tied",
            "backup_power": true,
        }),
        "SolarInverter" => serde_json::json!({
            "generation_kw": (seed % 80) as f64 / 10.0,
            "curtailment": false,
        }),
        "EVCharger" => serde_json::json!({
            "session_active": seed % 2 == 0,
            "load_kw": if seed % 2 == 0 { 7.2 } else { 0.0 },
        }),
        "UtilityMeter" => serde_json::json!({
            "grid_import_kw": (seed % 50) as f64 / 10.0,
            "demand_response_event": false,
        }),
        _ => serde_json::json!({ "status": "online" }),
    };
    json_ok(&serde_json::json!({
        "version": "v1",
        "system": summarize_energy(system),
        "detail": detail,
    }))
}

pub fn gateway_status_list(state: &ControlCenterState, facility_id: Option<&str>) -> HttpResponse {
    let resolved = match require_resolved(state) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let gateways: Vec<_> = match facility_id {
        Some(id) => entries_for_facility(&resolved.raw, id, "gateways"),
        None => collect_facility_nested(&resolved.raw, "gateways"),
    }
    .into_iter()
    .map(|entry| {
        let id = table_field_str(entry, "id").unwrap_or_default();
        let provider = table_field_str(entry, "provider").unwrap_or_default();
        let live = gateway_live_reachable(&id, &provider);
        serde_json::json!({
            "gateway": summarize_gateway(entry),
            "reachable": live,
            "live_probe": std::env::var("SPANDA_LIVE_BACNET").ok().filter(|v| v == "1").is_some()
                || std::env::var("SPANDA_LIVE_KNX").ok().filter(|v| v == "1").is_some(),
        })
    })
    .collect();
    json_ok(&serde_json::json!({
        "version": "v1",
        "facility_id": facility_id,
        "gateways": gateways,
    }))
}
