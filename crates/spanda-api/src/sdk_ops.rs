//! Program-level SDK operations — CLI parity endpoints delegating to domain crates.
//!
use crate::handlers::{bad_request, json_ok};

const API_VERSION: &str = "v1";
use crate::program::parse_program_file;
use crate::state::ControlCenterState;
use serde::Deserialize;
use spanda_assurance::{
    assure_program_with_config, diagnose_from_trace, diagnose_program_with_config,
    evaluate_recovery, MissionAssuranceSummary,
};
use spanda_capability::{capability_traceability, evaluate_health_checks, infer_robot_capabilities};
use spanda_config::verify_with_system_config;
use spanda_deploy_http::HttpResponse;
use spanda_hardware::VerifyOptions;
use spanda_readiness::{
    evaluate_readiness_with_runtime, verify_mission, ReadinessOptions, ReadinessReport,
};
use spanda_trust::{evaluate_composite_trust, CompositeTrustOptions};
use std::path::{Path, PathBuf};

fn entity_not_found(message: &str) -> HttpResponse {
    HttpResponse {
        status: 404,
        body: serde_json::json!({ "ok": false, "error": message }).to_string(),
    }
}

/// Shared request body for program-scoped SDK operations.
#[derive(Debug, Deserialize, Default)]
pub struct ProgramRequest {
    /// Path to a `.sd` program or `.trace` file (relative to project root or absolute).
    pub file: Option<String>,
    pub target: Option<String>,
    #[serde(default)]
    pub include_runtime: bool,
    #[serde(default)]
    pub inject_health_faults: bool,
    #[serde(default)]
    pub traceability: bool,
    #[serde(default)]
    pub capabilities: bool,
    /// When true, `program_simulation` runs the driver (CLI `spanda sim` parity).
    #[serde(default)]
    pub execute: bool,
    /// When true, `program_replay` runs deterministic verification against a trace.
    #[serde(default)]
    pub deterministic: bool,
    /// When true, `program_replay` applies trace frames via playback (no re-run).
    #[serde(default)]
    pub playback: bool,
}

fn resolve_program_path(state: &ControlCenterState, file: Option<&str>) -> Result<PathBuf, String> {
    if let Some(path_str) = file {
        let path = PathBuf::from(path_str);
        if path.is_absolute() {
            return Ok(path);
        }
        if let Some(root) = state.project_root() {
            return Ok(root.join(path_str));
        }
        return Ok(path);
    }
    state
        .program_path
        .clone()
        .ok_or_else(|| "no program file specified (set file in body or --program)".to_string())
}

fn load_program(state: &ControlCenterState, file: Option<&str>) -> Result<(spanda_ast::nodes::Program, PathBuf, String), HttpResponse> {
    let path = resolve_program_path(state, file).map_err(|msg| bad_request(&msg))?;
    if !path.exists() {
        return Err(entity_not_found(&format!("program not found: {}", path.display())));
    }
    let (program, _source, label) = parse_program_file(&path).map_err(|e| bad_request(&e))?;
    Ok((program, path, label))
}

fn system_config_ref(state: &ControlCenterState) -> Option<&spanda_config::ResolvedSystemConfig> {
    state.resolved.as_ref()
}

/// POST /v1/programs/readiness — full program readiness (CLI parity).
pub fn program_readiness(state: &ControlCenterState, body: &str) -> HttpResponse {
    let req: ProgramRequest = serde_json::from_str(body).unwrap_or_default();
    let (program, path, _label) = match load_program(state, req.file.as_deref()) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let mut options = ReadinessOptions {
        target: req.target,
        include_runtime: req.include_runtime,
        inject_health_faults: req.inject_health_faults,
        source_path: Some(path.clone()),
        ..ReadinessOptions::default()
    };
    if let Some(cfg) = system_config_ref(state) {
        options.system_config = Some(std::sync::Arc::new(cfg.clone()));
    }
    let report: ReadinessReport =
        evaluate_readiness_with_runtime(&program, &options, None);
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "file": path.display().to_string(),
        "report": report,
    }))
}

/// POST /v1/programs/assure — mission assurance (CLI parity).
pub fn program_assure(state: &ControlCenterState, body: &str) -> HttpResponse {
    let req: ProgramRequest = serde_json::from_str(body).unwrap_or_default();
    let (program, path, _label) = match load_program(state, req.file.as_deref()) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let summary: MissionAssuranceSummary = assure_program_with_config(
        &program,
        path.to_str().unwrap_or("program.sd"),
        system_config_ref(state),
    );
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "file": path.display().to_string(),
        "report": summary,
    }))
}

/// POST /v1/programs/diagnose — diagnosis from program or trace (CLI parity).
pub fn program_diagnose(state: &ControlCenterState, body: &str) -> HttpResponse {
    let req: ProgramRequest = serde_json::from_str(body).unwrap_or_default();
    let path = match resolve_program_path(state, req.file.as_deref()) {
        Ok(p) => p,
        Err(msg) => return bad_request(&msg),
    };
    if !path.exists() {
        return entity_not_found(&format!("file not found: {}", path.display()));
    }
    let report = if path.extension().and_then(|e| e.to_str()) == Some("trace") {
        match diagnose_from_trace(&path) {
            Ok(r) => r,
            Err(e) => return bad_request(&e.to_string()),
        }
    } else {
        let (program, _, _) = match parse_program_file(&path) {
            Ok(v) => v,
            Err(e) => return bad_request(&e),
        };
        diagnose_program_with_config(&program, system_config_ref(state))
    };
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "file": path.display().to_string(),
        "report": report,
    }))
}

/// POST /v1/programs/recovery/heal — recovery evaluation (CLI parity).
pub fn program_heal(state: &ControlCenterState, body: &str) -> HttpResponse {
    let req: ProgramRequest = serde_json::from_str(body).unwrap_or_default();
    let (program, path, _label) = match load_program(state, req.file.as_deref()) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let registry = state.device_registry();
    let report = evaluate_recovery(&program, None, Some(&registry));
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "file": path.display().to_string(),
        "report": report,
    }))
}

/// POST /v1/programs/verify/hardware — hardware compatibility (CLI parity).
pub fn program_verify_hardware(state: &ControlCenterState, body: &str) -> HttpResponse {
    let req: ProgramRequest = serde_json::from_str(body).unwrap_or_default();
    let (program, path, _label) = match load_program(state, req.file.as_deref()) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let options = VerifyOptions {
        target: req.target,
        ..VerifyOptions::default()
    };
    let report = verify_with_system_config(&program, system_config_ref(state), options);
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "file": path.display().to_string(),
        "report": report,
    }))
}

/// POST /v1/programs/verify/capabilities — capability verification (CLI parity).
pub fn program_verify_capabilities(state: &ControlCenterState, body: &str) -> HttpResponse {
    let req: ProgramRequest = serde_json::from_str(body).unwrap_or_default();
    let (program, path, _label) = match load_program(state, req.file.as_deref()) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let capabilities = infer_robot_capabilities(&program);
    let health = evaluate_health_checks(&program);
    let trace = if req.traceability {
        Some(capability_traceability(&program))
    } else {
        None
    };
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "file": path.display().to_string(),
        "capabilities": capabilities,
        "health": health,
        "traceability": trace,
    }))
}

/// POST /v1/programs/verify/mission — mission verification (CLI parity).
pub fn program_verify_mission(state: &ControlCenterState, body: &str) -> HttpResponse {
    let req: ProgramRequest = serde_json::from_str(body).unwrap_or_default();
    let (program, path, _label) = match load_program(state, req.file.as_deref()) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let report = verify_mission(&program, None);
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "file": path.display().to_string(),
        "report": report,
    }))
}

/// GET /v1/trust/program — composite program trust (CLI parity).
pub fn trust_program(state: &ControlCenterState, query: &str) -> HttpResponse {
    let params = crate::handlers::parse_query(query);
    let file = params.get("file").map(String::as_str);
    let path = match resolve_program_path(state, file) {
        Ok(p) => p,
        Err(msg) => return bad_request(&msg),
    };
    if !path.exists() {
        return entity_not_found(&format!("program not found: {}", path.display()));
    }
    let source = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return bad_request(&format!("read {} failed: {e}", path.display())),
    };
    let (program, _, label) = match parse_program_file(&path) {
        Ok(v) => v,
        Err(e) => return bad_request(&e),
    };
    let report = evaluate_composite_trust(
        &program,
        &source,
        &label,
        &CompositeTrustOptions::default(),
    );
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "file": path.display().to_string(),
        "report": report,
    }))
}

/// GET /v1/entities — unified entity inventory (humans + devices).
pub fn list_entities(state: &ControlCenterState) -> HttpResponse {
    let mut entities = Vec::new();
    if let Some(resolved) = state.resolved.as_ref() {
        for human in &resolved.human_registry.humans {
            entities.push(serde_json::json!({
                "id": human.id,
                "kind": "human",
                "display_name": human.display_name,
                "role": human.role,
                "trust_level": human.trust_level,
            }));
        }
    }
    for entry in state.device_registry().pool_entries() {
        entities.push(serde_json::json!({
            "id": entry.id,
            "kind": "device",
            "display_name": entry.logical_name,
            "lifecycle_state": entry.lifecycle_state,
            "trust_level": entry.trust_level,
        }));
    }
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "entities": entities,
        "count": entities.len(),
    }))
}

/// GET /v1/entities/{id} — entity lookup by id.
pub fn get_entity(state: &ControlCenterState, entity_id: &str) -> HttpResponse {
    if let Some(resolved) = state.resolved.as_ref() {
        if let Some(human) = resolved.human_registry.humans.iter().find(|h| h.id == entity_id) {
            return json_ok(&serde_json::json!({
                "version": API_VERSION,
                "entity": {
                    "id": human.id,
                    "kind": "human",
                    "display_name": human.display_name,
                    "role": human.role,
                    "capabilities": human.capabilities,
                    "trust_level": human.trust_level,
                },
            }));
        }
    }
    let registry = state.device_registry();
    if let Some(device) = registry.get(entity_id) {
        return json_ok(&serde_json::json!({
            "version": API_VERSION,
            "entity": {
                "id": device.id,
                "kind": "device",
                "display_name": device.logical_name,
                "lifecycle_state": device.lifecycle_state,
                "trust_level": device.trust_level,
                "health": device.health_status,
            },
        }));
    }
    entity_not_found(&format!("entity '{entity_id}' not found"))
}

/// GET /v1/health/entity/{id} — health for a device entity.
pub fn entity_health(state: &ControlCenterState, entity_id: &str) -> HttpResponse {
    let registry = state.device_registry();
    let Some(device) = registry.get(entity_id) else {
        return entity_not_found(&format!("device entity '{entity_id}' not found"));
    };
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "entity_id": entity_id,
        "health": device.health_status,
        "lifecycle_state": device.lifecycle_state,
    }))
}

/// GET /v1/trust/entity/{id} — trust metadata for an entity.
pub fn entity_trust(state: &ControlCenterState, entity_id: &str) -> HttpResponse {
    let registry = state.device_registry();
    if let Some(device) = registry.get(entity_id) {
        return json_ok(&serde_json::json!({
            "version": API_VERSION,
            "entity_id": entity_id,
            "kind": "device",
            "trust_level": device.trust_level,
            "lifecycle_state": device.lifecycle_state,
        }));
    }
    if let Some(resolved) = state.resolved.as_ref() {
        if let Some(human) = resolved.human_registry.humans.iter().find(|h| h.id == entity_id) {
            return json_ok(&serde_json::json!({
                "version": API_VERSION,
                "entity_id": entity_id,
                "kind": "human",
                "trust_level": human.trust_level,
            }));
        }
    }
    entity_not_found(&format!("entity '{entity_id}' not found"))
}

/// POST /v1/programs/replay — replay or inspect a mission trace (CLI parity).
pub fn program_replay(state: &ControlCenterState, body: &str) -> HttpResponse {
    let req: ProgramRequest = serde_json::from_str(body).unwrap_or_default();
    let path = match resolve_program_path(state, req.file.as_deref()) {
        Ok(p) => p,
        Err(msg) => return bad_request(&msg),
    };
    if !path.exists() {
        return entity_not_found(&format!("trace not found: {}", path.display()));
    }
    let trace = match spanda_runtime::replay::MissionTrace::load(&path) {
        Ok(t) => t,
        Err(e) => return bad_request(&e.to_string()),
    };
    if req.playback {
        let options = spanda_interpreter::RunOptions {
            max_loop_iterations: 20,
            replay_from_ms: Some(0.0),
            ..Default::default()
        };
        return match spanda_driver::playback_mission(
            path.to_str().unwrap_or("mission.trace"),
            options,
        ) {
            Ok((report, robot_state)) => json_ok(&serde_json::json!({
                "version": API_VERSION,
                "file": path.display().to_string(),
                "replay": {
                    "mode": "playback",
                    "frames_applied": report.frames_applied,
                    "states_applied": report.states_applied,
                    "final_pose": robot_state.pose,
                    "loaded": true,
                },
            })),
            Err(e) => bad_request(&e.to_string()),
        };
    }
    if req.deterministic {
        let trace_path = path.to_str().unwrap_or("mission.trace");
        let source_path = resolve_trace_source(trace_path, &trace.source);
        let source = match std::fs::read_to_string(&source_path) {
            Ok(s) => s,
            Err(e) => return bad_request(&format!("read {} failed: {e}", source_path)),
        };
        let options = spanda_interpreter::RunOptions {
            max_loop_iterations: 20,
            record_trace: true,
            replay_deterministic: true,
            trace_source: Some(trace.source.clone()),
            ..Default::default()
        };
        return match spanda_driver::replay_mission(&source, trace_path, options) {
            Ok((_result, verification)) => json_ok(&serde_json::json!({
                "version": API_VERSION,
                "file": path.display().to_string(),
                "replay": {
                    "mode": "deterministic",
                    "source": trace.source,
                    "frame_count": trace.frames.len(),
                    "verification": verification,
                    "loaded": true,
                },
            })),
            Err(e) => bad_request(&e.to_string()),
        };
    }
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "file": path.display().to_string(),
        "replay": {
            "source": trace.source,
            "frame_count": trace.frames.len(),
            "deterministic": trace.deterministic,
            "loaded": true,
        },
    }))
}

fn resolve_trace_source(trace_file: &str, source: &str) -> String {
    if Path::new(source).is_file() {
        return source.to_string();
    }
    if let Some(parent) = Path::new(trace_file).parent() {
        let candidate = parent.join(source);
        if candidate.is_file() {
            return candidate.to_string_lossy().into_owned();
        }
    }
    source.to_string()
}

/// POST /v1/programs/simulation — run simulation or return planning metadata.
pub fn program_simulation(state: &ControlCenterState, body: &str) -> HttpResponse {
    let req: ProgramRequest = serde_json::from_str(body).unwrap_or_default();
    let path = match resolve_program_path(state, req.file.as_deref()) {
        Ok(p) => p,
        Err(msg) => return bad_request(&msg),
    };
    if !path.exists() {
        return entity_not_found(&format!("program not found: {}", path.display()));
    }
    if !req.execute {
        let (program, _, _) = match parse_program_file(&path) {
            Ok(v) => v,
            Err(e) => return bad_request(&e),
        };
        let robot_count = program.robots().len();
        return json_ok(&serde_json::json!({
            "version": API_VERSION,
            "file": path.display().to_string(),
            "simulation": {
                "robot_count": robot_count,
                "dry_run": true,
                "status": "planned",
            },
        }));
    }
    let source = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return bad_request(&format!("read {} failed: {e}", path.display())),
    };
    let options = spanda_interpreter::RunOptions {
        max_loop_iterations: 20,
        trace_source: Some(path.to_string_lossy().into_owned()),
        ..Default::default()
    };
    match spanda_driver::run(&source, options) {
        Ok(result) => json_ok(&serde_json::json!({
            "version": API_VERSION,
            "file": path.display().to_string(),
            "simulation": {
                "dry_run": false,
                "status": "completed",
                "event_count": result.events.len(),
                "log_count": result.logs.len(),
                "has_trace": result.mission_trace.is_some(),
                "result": result,
            },
        })),
        Err(e) => bad_request(&e.to_string()),
    }
}

/// JSON body for gRPC `EvaluateProgramReadiness` (parity with `POST /v1/programs/readiness`).
pub fn program_readiness_json(state: &ControlCenterState, body: &str) -> String {
    program_readiness(state, body).body
}

/// JSON body for gRPC `EvaluateProgramAssure`.
pub fn program_assure_json(state: &ControlCenterState, body: &str) -> String {
    program_assure(state, body).body
}

/// JSON body for gRPC `EvaluateProgramDiagnose`.
pub fn program_diagnose_json(state: &ControlCenterState, body: &str) -> String {
    program_diagnose(state, body).body
}

/// JSON body for gRPC `EvaluateProgramHeal`.
pub fn program_heal_json(state: &ControlCenterState, body: &str) -> String {
    program_heal(state, body).body
}

/// JSON body for gRPC `VerifyProgramHardware`.
pub fn program_verify_hardware_json(state: &ControlCenterState, body: &str) -> String {
    program_verify_hardware(state, body).body
}

/// JSON body for gRPC `VerifyProgramCapabilities`.
pub fn program_verify_capabilities_json(state: &ControlCenterState, body: &str) -> String {
    program_verify_capabilities(state, body).body
}

/// JSON body for gRPC `VerifyProgramMission`.
pub fn program_verify_mission_json(state: &ControlCenterState, body: &str) -> String {
    program_verify_mission(state, body).body
}

/// JSON body for gRPC `RunProgramSimulation`.
pub fn program_simulation_json(state: &ControlCenterState, body: &str) -> String {
    program_simulation(state, body).body
}

/// JSON body for gRPC `ReplayProgram`.
pub fn program_replay_json(state: &ControlCenterState, body: &str) -> String {
    program_replay(state, body).body
}

/// JSON body for gRPC `GetTrustProgram`.
pub fn trust_program_json(state: &ControlCenterState, query: &str) -> String {
    trust_program(state, query).body
}

/// JSON body for gRPC `ListEntities`.
pub fn list_entities_json(state: &ControlCenterState) -> String {
    list_entities(state).body
}

/// JSON body for gRPC `GetEntity`.
pub fn get_entity_json(state: &ControlCenterState, entity_id: &str) -> String {
    get_entity(state, entity_id).body
}

/// JSON body for gRPC `GetEntityHealth`.
pub fn entity_health_json(state: &ControlCenterState, entity_id: &str) -> String {
    entity_health(state, entity_id).body
}

/// JSON body for gRPC `GetEntityTrust`.
pub fn entity_trust_json(state: &ControlCenterState, entity_id: &str) -> String {
    entity_trust(state, entity_id).body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_request_defaults() {
        let req: ProgramRequest = serde_json::from_str("{}").unwrap();
        assert!(!req.include_runtime);
        assert!(req.file.is_none());
    }
}
