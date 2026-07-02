//! Distributed decision API operations — CLI parity endpoints.

use crate::handlers::{bad_request, json_ok};
use crate::program::parse_program_file;
use crate::state::ControlCenterState;
use serde::Deserialize;
use spanda_decision::{
    approve_escalation, evaluate_distributed_decisions, extract_decision_authorities,
    extract_decision_trees, extract_offline_policies, load_persisted_policy_cache,
    simulate_distributed_decisions, DecisionContext, DecisionLayer, SimulationOptions,
};
use spanda_deploy_http::HttpResponse;
use std::collections::HashMap;

const API_VERSION: &str = "v1";

fn entity_not_found(message: &str) -> HttpResponse {
    HttpResponse {
        status: 404,
        body: serde_json::json!({ "ok": false, "error": message }).to_string(),
    }
}

fn load_program(
    state: &ControlCenterState,
    file: Option<&str>,
) -> Result<spanda_ast::nodes::Program, HttpResponse> {
    let path = if let Some(path_str) = file {
        if let Some(root) = state.project_root() {
            root.join(path_str)
        } else {
            std::path::PathBuf::from(path_str)
        }
    } else if let Some(p) = state.program_path.clone() {
        p
    } else {
        return Err(bad_request("no program file specified"));
    };
    if !path.exists() {
        return Err(entity_not_found(&format!(
            "program not found: {}",
            path.display()
        )));
    }
    let (program, _, _) = parse_program_file(&path).map_err(|e| bad_request(&e))?;
    Ok(program)
}

#[derive(Debug, Deserialize, Default)]
pub struct DecisionSimulateRequest {
    pub file: Option<String>,
    pub entity_id: Option<String>,
    pub mission: Option<String>,
    #[serde(default)]
    pub offline: bool,
    #[serde(default)]
    pub network_partition: bool,
    #[serde(default)]
    pub fleet_coordinator_failure: bool,
    #[serde(default)]
    pub signals: HashMap<String, bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DecisionEscalateRequest {
    pub escalation_id: String,
    pub approver: String,
    pub entity_id: Option<String>,
}

/// GET /v1/decisions — list decision architecture for loaded program.
pub fn list_decisions(state: &ControlCenterState, query: &str) -> HttpResponse {
    let file = parse_query_param(query, "file");
    let program = match load_program(state, file.as_deref()) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let authorities = extract_decision_authorities(&program);
    let trees = extract_decision_trees(&program);
    let offline = extract_offline_policies(&program);
    json_ok(&serde_json::json!({
        "api_version": API_VERSION,
        "authorities": authorities,
        "decision_trees": trees,
        "offline_policies": offline,
    }))
}

/// GET /v1/entities/{id}/decisions — entity-scoped decision evaluation.
pub fn entity_decisions(state: &ControlCenterState, entity_id: &str, query: &str) -> HttpResponse {
    let file = parse_query_param(query, "file");
    let program = match load_program(state, file.as_deref()) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let action = parse_query_param(query, "action").unwrap_or_else(|| "continue_mission".into());
    let ctx = DecisionContext {
        entity_id: entity_id.into(),
        mission: parse_query_param(query, "mission"),
        layer: DecisionLayer::LocalEntity,
        action,
        signals: HashMap::new(),
        offline_minutes: parse_query_param(query, "offline_minutes")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        policy_version: parse_query_param(query, "policy_version")
            .unwrap_or_else(|| "1.0.0".into()),
    };
    let report = evaluate_distributed_decisions(&program, &ctx);
    json_ok(&serde_json::json!({
        "api_version": API_VERSION,
        "entity_id": entity_id,
        "report": report,
    }))
}

/// POST /v1/decisions/simulate — simulate distributed decisions.
pub fn simulate_decisions(state: &ControlCenterState, body: &str) -> HttpResponse {
    let req: DecisionSimulateRequest = serde_json::from_str(body).unwrap_or_default();
    let program = match load_program(state, req.file.as_deref()) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let options = SimulationOptions {
        offline: req.offline,
        network_partition: req.network_partition,
        fleet_coordinator_failure: req.fleet_coordinator_failure,
        entity_id: req.entity_id.unwrap_or_else(|| "Rover".into()),
        mission: req.mission,
        signals: req.signals,
    };
    let sim = simulate_distributed_decisions(&program, options);
    json_ok(&serde_json::json!({
        "api_version": API_VERSION,
        "simulation": sim,
    }))
}

/// POST /v1/decisions/escalate — approve a pending escalation.
pub fn escalate_decision(_state: &ControlCenterState, body: &str) -> HttpResponse {
    let req: DecisionEscalateRequest = match serde_json::from_str(body) {
        Ok(req) => req,
        Err(e) => return bad_request(&format!("invalid body: {e}")),
    };
    let mut escalation = spanda_decision::DecisionEscalation {
        from_layer: DecisionLayer::LocalEntity,
        to_layer: DecisionLayer::ControlCenter,
        reason: "pending approval".into(),
        entity_id: req.entity_id.unwrap_or_else(|| "unknown".into()),
        pending_approval: true,
        escalation_id: req.escalation_id.clone(),
    };
    match approve_escalation(&mut escalation, &req.approver) {
        Ok(()) => json_ok(&serde_json::json!({
            "api_version": API_VERSION,
            "ok": true,
            "escalation": escalation,
        })),
        Err(e) => bad_request(&e),
    }
}

/// GET /v1/decision-policies — list decision policies from program.
pub fn list_decision_policies(state: &ControlCenterState, query: &str) -> HttpResponse {
    let file = parse_query_param(query, "file");
    let program = match load_program(state, file.as_deref()) {
        Ok(p) => p,
        Err(e) => return e,
    };
    json_ok(&serde_json::json!({
        "api_version": API_VERSION,
        "offline_policies": extract_offline_policies(&program),
        "decision_trees": extract_decision_trees(&program),
    }))
}

/// GET /v1/decisions/traces — list v3 decision frames from a mission trace file.
pub fn list_decision_traces(state: &ControlCenterState, query: &str) -> HttpResponse {
    let trace_path = if let Some(path) = parse_query_param(query, "trace") {
        std::path::PathBuf::from(path)
    } else if let Some(program_file) = parse_query_param(query, "file") {
        let root = state
            .project_root()
            .map(std::path::PathBuf::from)
            .unwrap_or_default();
        let path = if root.as_os_str().is_empty() {
            std::path::PathBuf::from(&program_file)
        } else {
            root.join(&program_file)
        };
        let trace = path.with_extension("trace");
        if trace.exists() {
            trace
        } else {
            path.file_stem()
                .map(|stem| path.with_file_name(format!("{}.trace", stem.to_string_lossy())))
                .unwrap_or(trace)
        }
    } else if let Some(p) = state.program_path.clone() {
        p.with_extension("trace")
    } else {
        return bad_request("no trace file specified");
    };
    if !trace_path.exists() {
        return entity_not_found(&format!("trace not found: {}", trace_path.display()));
    }
    let trace = match spanda_runtime::replay::MissionTrace::load(&trace_path) {
        Ok(trace) => trace,
        Err(e) => return bad_request(&e.to_string()),
    };
    let frames: Vec<_> = trace
        .frames
        .iter()
        .filter(|f| f.payload.get("version").and_then(|v| v.as_u64()) == Some(3))
        .map(|f| {
            serde_json::json!({
                "sim_time_ms": f.sim_time_ms,
                "event": f.event,
                "payload": f.payload,
            })
        })
        .collect();
    json_ok(&serde_json::json!({
        "api_version": API_VERSION,
        "trace": trace_path.display().to_string(),
        "count": frames.len(),
        "frames": frames,
    }))
}

/// GET /v1/decision-policy-cache — persisted signed offline policy cache on disk.
pub fn list_decision_policy_cache(_state: &ControlCenterState, query: &str) -> HttpResponse {
    let cache_path = parse_query_param(query, "cache").map(std::path::PathBuf::from);
    let cache = load_persisted_policy_cache(cache_path.as_deref());
    let path = cache_path
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| {
            spanda_decision::default_policy_cache_path()
                .display()
                .to_string()
        });
    json_ok(&serde_json::json!({
        "api_version": API_VERSION,
        "cache_path": path,
        "policy_count": cache.policies.len(),
        "updated_at_ms": cache.updated_at_ms,
        "policies": cache.policies,
    }))
}

fn parse_query_param(query: &str, key: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        if k == key {
            Some(v.to_string())
        } else {
            None
        }
    })
}

/// JSON string helper for gRPC parity.
pub fn list_decisions_json(state: &ControlCenterState, query: &str) -> String {
    list_decisions(state, query).body
}

/// JSON string helper for gRPC parity.
pub fn entity_decisions_json(state: &ControlCenterState, entity_id: &str, query: &str) -> String {
    entity_decisions(state, entity_id, query).body
}

/// JSON string helper for gRPC parity.
pub fn simulate_decisions_json(state: &ControlCenterState, body: &str) -> String {
    simulate_decisions(state, body).body
}

/// JSON string helper for gRPC parity.
pub fn list_decision_traces_json(state: &ControlCenterState, query: &str) -> String {
    list_decision_traces(state, query).body
}

/// JSON string helper for gRPC parity.
pub fn list_decision_policy_cache_json(state: &ControlCenterState, query: &str) -> String {
    list_decision_policy_cache(state, query).body
}

/// JSON string helper for gRPC parity.
pub fn list_decision_policies_json(state: &ControlCenterState, query: &str) -> String {
    list_decision_policies(state, query).body
}
