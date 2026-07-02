//! LATER differentiation analytics REST handlers — mission twin, certification pack,
//! time travel, human teaming, autonomous governance.

use crate::program::parse_program_file;
use crate::state::ControlCenterState;
use spanda_assurance::build_certification_pack;
use spanda_deploy_http::HttpResponse;
use spanda_policy::{evaluate_policy, list_policies};
use spanda_readiness::{evaluate_human_teaming, evaluate_mission_twin};
use spanda_runtime::replay::MissionTrace;
use spanda_runtime::{
    inspect_mission_at, parse_inspect_facets, parse_time_travel_at, TimeTravelInspect,
};
use std::path::PathBuf;

use crate::handlers::{bad_request, json_ok, parse_query};

fn load_program(
    state: &ControlCenterState,
) -> Result<(spanda_ast::nodes::Program, String, String), String> {
    let Some(path) = state.program_path.as_ref() else {
        return Err("no program loaded; use control-center serve --program <file.sd>".into());
    };
    parse_program_file(path).map(|(program, source, label)| (program, source, label))
}

fn resolve_trace_path(
    state: &ControlCenterState,
    params: &std::collections::HashMap<String, String>,
) -> Result<PathBuf, String> {
    if let Some(raw) = params.get("trace") {
        let candidates = trace_candidates(state, raw);
        for candidate in candidates {
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
        return Err(format!("trace file not found: {raw}"));
    }
    if let Some(program_path) = state.program_path.as_ref() {
        let auto = program_path.with_extension("trace");
        if auto.is_file() {
            return Ok(auto);
        }
    }
    Err(
        "missing trace query parameter; pass trace=<path> or record a .trace beside the program"
            .into(),
    )
}

fn trace_candidates(state: &ControlCenterState, raw: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let direct = PathBuf::from(raw);
    out.push(direct.clone());
    if direct.is_absolute() {
        return out;
    }
    if let Some(program_path) = state.program_path.as_ref() {
        if let Some(parent) = program_path.parent() {
            out.push(parent.join(raw));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        out.push(cwd.join(raw));
    }
    out
}

/// `GET /v1/analytics/mission-twin`
pub fn analytics_mission_twin(state: &ControlCenterState) -> HttpResponse {
    let (program, _source, label) = match load_program(state) {
        Ok(value) => value,
        Err(message) => return bad_request(&message),
    };
    let report = evaluate_mission_twin(&program, &label);
    json_ok(&serde_json::json!({
        "version": "v1",
        "program": label,
        "mission_twin": report,
    }))
}

/// `GET /v1/analytics/certification-pack?strict=0`
pub fn analytics_certification_pack(state: &ControlCenterState, query: &str) -> HttpResponse {
    let params = parse_query(query);
    let strict = params
        .get("strict")
        .is_some_and(|v| v == "1" || v == "true");
    let (program, source, label) = match load_program(state) {
        Ok(value) => value,
        Err(message) => return bad_request(&message),
    };
    let pack = build_certification_pack(&program, &source, &label, strict);
    json_ok(&serde_json::json!({
        "version": "v1",
        "program": label,
        "certification_pack": pack,
    }))
}

/// `GET /v1/analytics/time-travel?at=T+00:01&inspect=decisions&trace=<path>`
pub fn analytics_time_travel(state: &ControlCenterState, query: &str) -> HttpResponse {
    let params = parse_query(query);
    let at_raw = match params.get("at") {
        Some(value) => value.as_str(),
        None => return bad_request("missing at query parameter"),
    };
    let trace_path = match resolve_trace_path(state, &params) {
        Ok(path) => path,
        Err(message) => return bad_request(&message),
    };
    let trace = match MissionTrace::load(trace_path.to_string_lossy().as_ref()) {
        Ok(value) => value,
        Err(error) => return bad_request(&error.to_string()),
    };
    let at_ms = match parse_time_travel_at(at_raw, &trace) {
        Ok(value) => value,
        Err(error) => return bad_request(&error.to_string()),
    };
    let facets = params
        .get("inspect")
        .map(|raw| parse_inspect_facets(raw))
        .unwrap_or_else(|| vec![TimeTravelInspect::Decisions]);
    let explorer = inspect_mission_at(&trace, at_ms, &facets);
    json_ok(&serde_json::json!({
        "version": "v1",
        "trace": trace_path.display().to_string(),
        "at": at_raw,
        "time_travel": explorer,
    }))
}

/// `GET /v1/analytics/human-teaming`
pub fn analytics_human_teaming(state: &ControlCenterState) -> HttpResponse {
    let (program, _source, label) = match load_program(state) {
        Ok(value) => value,
        Err(message) => return bad_request(&message),
    };
    let report = evaluate_human_teaming(&program, &label);
    json_ok(&serde_json::json!({
        "version": "v1",
        "program": label,
        "human_teaming": report,
    }))
}

/// `GET /v1/analytics/governance?policy=WarehousePolicy`
pub fn analytics_governance(state: &ControlCenterState, query: &str) -> HttpResponse {
    let params = parse_query(query);
    let (program, _source, label) = match load_program(state) {
        Ok(value) => value,
        Err(message) => return bad_request(&message),
    };
    let policies = list_policies(&program);
    if policies.is_empty() {
        return bad_request("no operational policy blocks declared in loaded program");
    }
    let policy_name = params
        .get("policy")
        .cloned()
        .unwrap_or_else(|| policies[0].clone());
    let report = match evaluate_policy(&program, &policy_name, &label) {
        Ok(value) => value,
        Err(error) => return bad_request(&error.to_string()),
    };
    json_ok(&serde_json::json!({
        "version": "v1",
        "program": label,
        "policy": policy_name,
        "governance": report,
    }))
}
