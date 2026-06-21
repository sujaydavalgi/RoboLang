//! Remote fleet peer relay via HTTP fleet agents.

use crate::deploy_http::{http_request, parse_http_url, HttpResponse};
use crate::fleet_orchestrator::PeerDelivery;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetAgentEntry {
    pub robot_name: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetAgentRegistry {
    pub agents: Vec<FleetAgentEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerRelayRequest {
    pub from_robot: String,
    pub to_robot: String,
    pub topic: String,
    pub step: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerRelayResponse {
    pub ok: bool,
    pub to_robot: String,
    pub topic: String,
    pub step: String,
    #[serde(default)]
    pub error: Option<String>,
}

pub fn default_fleet_agents_path() -> PathBuf {
    PathBuf::from(".spanda/fleet-agents.json")
}

pub fn load_fleet_agent_registry(path: &Path) -> FleetAgentRegistry {
    if !path.exists() {
        return FleetAgentRegistry::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_fleet_agent_registry(path: &Path, registry: &FleetAgentRegistry) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(registry).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

pub fn register_fleet_agent(
    registry: &mut FleetAgentRegistry,
    robot_name: String,
    url: String,
    token: Option<String>,
) -> Result<(), String> {
    parse_http_url(&url)?;
    registry.agents.retain(|entry| entry.robot_name != robot_name);
    registry.agents.push(FleetAgentEntry {
        robot_name,
        url,
        token,
    });
    registry
        .agents
        .sort_by(|a, b| a.robot_name.cmp(&b.robot_name));
    Ok(())
}

pub fn lookup_fleet_agent<'a>(
    registry: &'a FleetAgentRegistry,
    robot_name: &str,
) -> Option<&'a FleetAgentEntry> {
    registry
        .agents
        .iter()
        .find(|entry| entry.robot_name == robot_name)
}

fn agent_endpoint(base_url: &str, path: &str) -> Result<String, String> {
    let parsed = parse_http_url(base_url)?;
    Ok(format!(
        "{}://{}:{}{}",
        parsed.scheme, parsed.host, parsed.port, path
    ))
}

fn decode_response<T: for<'de> Deserialize<'de>>(response: HttpResponse) -> Result<T, String> {
    if response.status >= 400 {
        return Err(format!("fleet agent HTTP {}: {}", response.status, response.body));
    }
    serde_json::from_str(&response.body).map_err(|e| format!("invalid fleet agent JSON: {e}"))
}

pub fn agent_health(entry: &FleetAgentEntry) -> Result<bool, String> {
    let url = agent_endpoint(&entry.url, "/v1/health")?;
    let response = http_request("GET", &url, None, entry.token.as_deref())?;
    let body: serde_json::Value = decode_response(response)?;
    Ok(body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false))
}

pub fn relay_peer_delivery(
    entry: &FleetAgentEntry,
    delivery: &PeerDelivery,
) -> Result<PeerRelayResponse, String> {
    let url = agent_endpoint(&entry.url, "/v1/peer")?;
    let payload = serde_json::to_string(&PeerRelayRequest {
        from_robot: delivery.from_robot.clone(),
        to_robot: delivery.to_robot.clone(),
        topic: delivery.topic.clone(),
        step: delivery.step.clone(),
    })
    .map_err(|e| e.to_string())?;
    let response = http_request("POST", &url, Some(&payload), entry.token.as_deref())?;
    decode_response(response)
}

pub fn relay_peer_deliveries(
    deliveries: &[PeerDelivery],
    registry: &FleetAgentRegistry,
) -> (u32, u32) {
    // Push peer mission steps to registered remote fleet agents.
    let mut relayed = 0u32;
    let mut failed = 0u32;
    for delivery in deliveries {
        let Some(agent) = lookup_fleet_agent(registry, &delivery.to_robot) else {
            failed += 1;
            continue;
        };
        match relay_peer_delivery(agent, delivery) {
            Ok(resp) if resp.ok => relayed += 1,
            _ => failed += 1,
        }
    }
    (relayed, failed)
}

pub fn registry_by_robot(registry: &FleetAgentRegistry) -> HashMap<String, FleetAgentEntry> {
    registry
        .agents
        .iter()
        .cloned()
        .map(|entry| (entry.robot_name.clone(), entry))
        .collect()
}
