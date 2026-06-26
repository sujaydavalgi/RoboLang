//! REST v1 route handlers for Spanda Control Center.
//!
use crate::state::ControlCenterState;
use serde::Serialize;
use spanda_config::DeviceLifecycleState;
use spanda_deploy_http::{HttpRequest, HttpResponse};
use spanda_fleet::remote::{default_fleet_agents_path, load_fleet_agent_registry};
use spanda_ops::{Alert, AlertSeverity, AlertType};
use spanda_security::{ApiKeyStore, RbacAction, RbacContext};

const API_VERSION: &str = "v1";

#[derive(Serialize)]
struct HealthPayload {
    ok: bool,
    version: &'static str,
    service: &'static str,
}

#[derive(Serialize)]
struct DashboardPayload {
    version: &'static str,
    device_pool: spanda_config::DevicePoolSummary,
    fleet_agent_count: usize,
    alert_count: usize,
    rbac_roles: usize,
}

pub fn handle_request(state: &mut ControlCenterState, request: &HttpRequest) -> HttpResponse {
    let path = request.path.split('?').next().unwrap_or(&request.path);
    if request.method == "OPTIONS" {
        return cors_preflight();
    }
    if path == "/health" || path == "/v1/health" || path == "/healthz" {
        return json_ok(&HealthPayload {
            ok: true,
            version: API_VERSION,
            service: "spanda-control-center",
        });
    }
    if path == "/" || path == "/control-center" {
        return html_ok(include_str!("static/control-center.html"));
    }
    if !path.starts_with("/v1/") {
        return not_found();
    }
    let ctx = state
        .api_keys
        .authenticate(request.authorization.as_deref());
    match path {
        "/v1/dashboard" => dashboard(state),
        "/v1/devices" if request.method == "GET" => devices_list(state),
        "/v1/fleet/agents" => fleet_agents(),
        "/v1/alerts" if request.method == "GET" => alerts_list(state),
        "/v1/alerts/test" if request.method == "POST" => alerts_test(state, ctx.as_ref()),
        "/v1/secrets" if request.method == "GET" => secrets_list(state, ctx.as_ref()),
        "/v1/rbac/matrix" => rbac_matrix(),
        p if p.starts_with("/v1/devices/") && request.method == "PATCH" => {
            let id = p.trim_start_matches("/v1/devices/");
            device_patch(state, id, &request.body, ctx.as_ref())
        }
        _ => not_found(),
    }
}

fn dashboard(state: &ControlCenterState) -> HttpResponse {
    let registry = state.device_registry();
    let fleet = load_fleet_agent_registry(&default_fleet_agents_path());
    json_ok(&DashboardPayload {
        version: API_VERSION,
        device_pool: registry.pool_summary(),
        fleet_agent_count: fleet.agents.len(),
        alert_count: state.alert_store.list().len(),
        rbac_roles: state.api_keys.keys.len(),
    })
}

fn devices_list(state: &ControlCenterState) -> HttpResponse {
    let registry = state.device_registry();
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "devices": registry.pool_entries(),
    }))
}

fn device_patch(
    state: &mut ControlCenterState,
    device_id: &str,
    body: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Provision) {
        return unauthorized();
    }
    let payload: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return bad_request(&e.to_string()),
    };
    let Some(state_str) = payload.get("lifecycle_state").and_then(|v| v.as_str()) else {
        return bad_request("missing lifecycle_state");
    };
    let lifecycle = DeviceLifecycleState::parse(state_str);
    let mut registry = state.device_registry();
    if let Err(e) = registry.set_lifecycle(device_id, lifecycle) {
        return bad_request(&e);
    }
    if let Some(resolved) = state.resolved.as_mut() {
        resolved.device_registry = registry;
    }
    json_ok(&serde_json::json!({
        "ok": true,
        "device_id": device_id,
        "lifecycle_state": lifecycle.as_str(),
    }))
}

fn fleet_agents() -> HttpResponse {
    let fleet = load_fleet_agent_registry(&default_fleet_agents_path());
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "agents": fleet.agents,
    }))
}

fn alerts_list(state: &ControlCenterState) -> HttpResponse {
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "alerts": state.alert_store.list_owned(),
    }))
}

fn alerts_test(state: &mut ControlCenterState, ctx: Option<&RbacContext>) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Operate) {
        return unauthorized();
    }
    let mut alert = Alert {
        id: format!("test-{}", now_ms()),
        alert_type: AlertType::Custom,
        severity: AlertSeverity::Info,
        message: "Control Center alert test".into(),
        source: "control-center".into(),
        timestamp_ms: now_ms(),
        delivered_via: vec![],
    };
    state.alert_dispatcher.dispatch(&mut alert);
    state.alert_store.push(alert.clone());
    json_ok(&serde_json::json!({
        "ok": true,
        "alert": alert,
    }))
}

fn secrets_list(state: &ControlCenterState, ctx: Option<&RbacContext>) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Deploy) {
        return unauthorized();
    }
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "secrets": state.secret_vault.list_metadata(),
    }))
}

fn rbac_matrix() -> HttpResponse {
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "matrix": spanda_security::permission_matrix(),
    }))
}

fn json_ok<T: Serialize>(value: &T) -> HttpResponse {
    let body = serde_json::to_string(value).unwrap_or_else(|_| "{}".into());
    HttpResponse {
        status: 200,
        body,
    }
}

fn html_ok(html: &str) -> HttpResponse {
    HttpResponse {
        status: 200,
        body: html.to_string(),
    }
}

fn bad_request(message: &str) -> HttpResponse {
    HttpResponse {
        status: 400,
        body: serde_json::json!({ "ok": false, "error": message }).to_string(),
    }
}

fn unauthorized() -> HttpResponse {
    HttpResponse {
        status: 401,
        body: serde_json::json!({ "ok": false, "error": "unauthorized" }).to_string(),
    }
}

fn not_found() -> HttpResponse {
    HttpResponse {
        status: 404,
        body: serde_json::json!({ "ok": false, "error": "not found" }).to_string(),
    }
}

fn cors_preflight() -> HttpResponse {
    HttpResponse {
        status: 204,
        body: String::new(),
    }
}

fn now_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64() * 1000.0)
        .unwrap_or(0.0)
}

pub fn encode_response(response: &HttpResponse, content_type: &str) -> String {
    if response.body.starts_with("HTTP/1.1") {
        return response.body.clone();
    }
    let status_text = match response.status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        _ => "Error",
    };
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: Authorization, Content-Type\r\n\r\n{}",
        response.status,
        status_text,
        content_type,
        response.body.len(),
        response.body
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_endpoint_ok() {
        let mut state = ControlCenterState::new();
        let response = handle_request(
            &mut state,
            &HttpRequest {
                method: "GET".into(),
                path: "/v1/health".into(),
                body: String::new(),
                authorization: None,
            },
        );
        assert_eq!(response.status, 200);
        assert!(response.body.contains("spanda-control-center"));
    }
}
