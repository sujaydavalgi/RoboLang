//! Fleet mesh OTLP ingest after run sessions.

use crate::error::{TelemetryStoreError, TelemetryStoreResult};
use crate::global_store;
use crate::otlp::render_otlp_json;
use serde_json::json;

fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Return true when `SPANDA_FLEET_TELEMETRY_AUTO_INGEST=1` or `SPANDA_FLEET_TELEMETRY_INGEST=1`.
pub fn env_fleet_auto_ingest_enabled() -> bool {
    env_truthy("SPANDA_FLEET_TELEMETRY_AUTO_INGEST") || env_truthy("SPANDA_FLEET_TELEMETRY_INGEST")
}

/// Fleet mesh coordinator base URL from `SPANDA_FLEET_MESH_URL`.
pub fn env_fleet_mesh_url() -> Option<String> {
    std::env::var("SPANDA_FLEET_MESH_URL").ok()
}

/// Bearer token for fleet mesh calls (`SPANDA_FLEET_MESH_TOKEN`).
pub fn env_fleet_mesh_token() -> Option<String> {
    std::env::var("SPANDA_FLEET_MESH_TOKEN").ok()
}

/// Robot identity for fleet ingest (`SPANDA_ROBOT_ID`, then hostname env vars).
pub fn env_robot_id() -> String {
    std::env::var("SPANDA_ROBOT_ID")
        .or_else(|_| std::env::var("HOSTNAME"))
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "robot".into())
}

fn fleet_ingest_url(mesh_url: &str) -> String {
    if mesh_url.ends_with('/') {
        format!("{mesh_url}v1/fleet/telemetry/ingest")
    } else {
        format!("{mesh_url}/v1/fleet/telemetry/ingest")
    }
}

/// POST the global store OTLP snapshot to a fleet mesh coordinator.
#[cfg(feature = "push")]
pub fn ingest_global_store_to_fleet_mesh(
    mesh_url: &str,
    robot_id: &str,
    token: Option<&str>,
) -> TelemetryStoreResult<()> {
    let store = global_store()
        .lock()
        .map_err(|_| TelemetryStoreError::LockPoisoned)?;
    let otlp_json = render_otlp_json(&store)?;
    let payload = json!({
        "robot_id": robot_id,
        "otlp_json": otlp_json,
    })
    .to_string();
    let response = spanda_deploy_http::http_request(
        "POST",
        &fleet_ingest_url(mesh_url),
        Some(&payload),
        token,
    )
    .map_err(|error| TelemetryStoreError::Serialization(error))?;
    if (200..300).contains(&response.status) {
        return Ok(());
    }
    Err(TelemetryStoreError::Serialization(format!(
        "HTTP {} from {}",
        response.status,
        fleet_ingest_url(mesh_url)
    )))
}

/// Ingest to fleet mesh after a session ends when auto-ingest env vars are set.
#[cfg(feature = "push")]
pub fn maybe_auto_ingest_fleet_after_session() {
    if !env_fleet_auto_ingest_enabled() {
        return;
    }
    let Some(mesh_url) = env_fleet_mesh_url() else {
        eprintln!("SPANDA_FLEET_TELEMETRY_AUTO_INGEST set but SPANDA_FLEET_MESH_URL is missing");
        return;
    };
    let robot_id = env_robot_id();
    let token = env_fleet_mesh_token();
    match ingest_global_store_to_fleet_mesh(&mesh_url, &robot_id, token.as_deref()) {
        Ok(()) => eprintln!("Auto-ingested OTLP metrics for {robot_id} to {mesh_url}"),
        Err(error) => eprintln!("Fleet telemetry auto-ingest failed: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_fleet_auto_ingest_reads_flags() {
        std::env::set_var("SPANDA_FLEET_TELEMETRY_AUTO_INGEST", "1");
        assert!(env_fleet_auto_ingest_enabled());
        std::env::remove_var("SPANDA_FLEET_TELEMETRY_AUTO_INGEST");
        assert!(!env_fleet_auto_ingest_enabled());
    }

    #[test]
    fn env_robot_id_prefers_spanda_robot_id() {
        std::env::set_var("SPANDA_ROBOT_ID", "rover-alpha");
        assert_eq!(env_robot_id(), "rover-alpha");
        std::env::remove_var("SPANDA_ROBOT_ID");
    }
}
