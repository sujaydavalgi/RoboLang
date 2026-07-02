//! REST API operations for Recovery Orchestrator (CLI/SDK parity).
//!
use serde::Deserialize;
use spanda_ast::nodes::Program;
use spanda_deploy_http::HttpResponse;
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_recovery::{
    OrchestratorContext, RecoveryOrchestrator, RecoveryOrchestratorRequest, RecoverySimulationMode,
};

use crate::handlers::{bad_request, json_ok};
use crate::state::ControlCenterState;

const API_VERSION: &str = "v1";

#[derive(Debug, Default, Deserialize, serde::Serialize)]
pub struct RecoveryRequest {
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub entity_id: Option<String>,
    #[serde(default)]
    pub failure: Option<String>,
    #[serde(default)]
    pub playbook: Option<String>,
    #[serde(default)]
    pub force_execute: bool,
}

fn load_program_from_state(
    state: &ControlCenterState,
    file: Option<&str>,
) -> Option<(Program, String)> {
    let path = if let Some(f) = file {
        std::path::PathBuf::from(f)
    } else {
        state.program_path.clone()?
    };
    let source = std::fs::read_to_string(&path).ok()?;
    let tokens = tokenize(&source).ok()?;
    let program = parse(tokens).ok()?;
    Some((program, path.display().to_string()))
}

fn orchestrator_request(
    body: &RecoveryRequest,
    mode: RecoverySimulationMode,
) -> RecoveryOrchestratorRequest {
    RecoveryOrchestratorRequest {
        entity_id: body.entity_id.clone(),
        failure: body.failure.clone(),
        mode,
        playbook: body.playbook.clone(),
        max_escalation_level: None,
        force_execute: body.force_execute,
    }
}

/// GET /v1/recovery/plans
pub fn list_recovery_plans(state: &ControlCenterState) -> HttpResponse {
    let registry = state.entity_registry();
    let orchestrator = RecoveryOrchestrator::new();
    let resolved = state.resolved.as_ref();
    let (program, file) = match load_program_from_state(state, None) {
        Some(v) => v,
        None => return bad_request("no program loaded"),
    };
    let request = RecoveryOrchestratorRequest::default();
    let report = orchestrator.plan_recovery(&program, &registry, resolved, &request);
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "file": file,
        "plans": report.plans,
        "passed": report.passed,
    }))
}

/// GET /v1/recovery/history
pub fn recovery_history(state: &ControlCenterState) -> HttpResponse {
    let orchestrator = RecoveryOrchestrator::new();
    let history = orchestrator.get_history(100);
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "history": history,
        "count": history.len(),
    }))
}

/// POST /v1/recovery/plan
pub fn recovery_plan(state: &ControlCenterState, body: &str) -> HttpResponse {
    let req: RecoveryRequest = serde_json::from_str(body).unwrap_or_default();
    let registry = state.entity_registry();
    let resolved = state.resolved.as_ref();
    let (program, file) = match load_program_from_state(state, req.file.as_deref()) {
        Some(v) => v,
        None => return bad_request("program not found"),
    };
    let orchestrator = RecoveryOrchestrator::new();
    let request = orchestrator_request(&req, RecoverySimulationMode::Plan);
    let report = orchestrator.plan_recovery(&program, &registry, resolved, &request);
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "file": file,
        "report": report,
    }))
}

/// POST /v1/recovery/simulate
pub fn recovery_simulate(state: &ControlCenterState, body: &str) -> HttpResponse {
    let req: RecoveryRequest = serde_json::from_str(body).unwrap_or_default();
    let registry = state.entity_registry();
    let resolved = state.resolved.as_ref();
    let (program, file) = match load_program_from_state(state, req.file.as_deref()) {
        Some(v) => v,
        None => return bad_request("program not found"),
    };
    let orchestrator = RecoveryOrchestrator::new();
    let request = orchestrator_request(&req, RecoverySimulationMode::Simulate);
    let report = orchestrator.simulate_recovery(&program, &registry, resolved, &request, None);
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "file": file,
        "report": report,
    }))
}

/// POST /v1/recovery/execute
pub fn recovery_execute(state: &ControlCenterState, body: &str) -> HttpResponse {
    let req: RecoveryRequest = serde_json::from_str(body).unwrap_or_default();
    let registry = state.entity_registry();
    let resolved = state.resolved.as_ref();
    let (program, file) = match load_program_from_state(state, req.file.as_deref()) {
        Some(v) => v,
        None => return bad_request("program not found"),
    };
    let mut orchestrator = RecoveryOrchestrator::new();
    let request = orchestrator_request(&req, RecoverySimulationMode::Validate);
    let ctx = OrchestratorContext {
        dry_run: false,
        skip_execution: !req.force_execute,
        ..Default::default()
    };
    let report = orchestrator.execute_recovery(&program, &registry, resolved, &request, &ctx);
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "file": file,
        "report": report,
    }))
}

/// POST /v1/recovery/validate
pub fn recovery_validate(state: &ControlCenterState, body: &str) -> HttpResponse {
    let req: RecoveryRequest = serde_json::from_str(body).unwrap_or_default();
    let registry = state.entity_registry();
    let resolved = state.resolved.as_ref();
    let (program, file) = match load_program_from_state(state, req.file.as_deref()) {
        Some(v) => v,
        None => return bad_request("program not found"),
    };
    let orchestrator = RecoveryOrchestrator::new();
    let request = orchestrator_request(&req, RecoverySimulationMode::Validate);
    let report = orchestrator.dry_run_recovery(&program, &registry, resolved, &request);
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "file": file,
        "report": report,
        "validation": report.evidence,
    }))
}

/// GET /v1/recovery/playbooks
pub fn recovery_playbooks(state: &ControlCenterState) -> HttpResponse {
    let orchestrator = RecoveryOrchestrator::new();
    let playbooks = orchestrator.list_playbooks(state.resolved.as_ref());
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "playbooks": playbooks,
    }))
}

/// GET /v1/recovery/metrics
pub fn recovery_metrics(state: &ControlCenterState) -> HttpResponse {
    let (program, file) = match load_program_from_state(state, None) {
        Some(v) => v,
        None => {
            return json_ok(&serde_json::json!({
                "version": API_VERSION,
                "metrics": spanda_recovery::RecoveryMetrics::default(),
            }))
        }
    };
    let orchestrator = RecoveryOrchestrator::new();
    let metrics = orchestrator.get_metrics(&program);
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "file": file,
        "metrics": metrics,
    }))
}

/// GET /v1/recovery/graph — recovery graph for entity subtree.
pub fn recovery_graph(state: &ControlCenterState, entity_id: Option<&str>) -> HttpResponse {
    let registry = state.entity_registry();
    let orchestrator = RecoveryOrchestrator::new();
    let graph = orchestrator.build_graph(&registry, entity_id);
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "graph": graph,
    }))
}

/// POST /v1/recovery/explain — decision explanations.
pub fn recovery_explain(state: &ControlCenterState, body: &str) -> HttpResponse {
    let req: RecoveryRequest = serde_json::from_str(body).unwrap_or_default();
    let registry = state.entity_registry();
    let entity_id = req.entity_id.unwrap_or_else(|| "system".into());
    let failure = req.failure.unwrap_or_else(|| "degraded".into());
    let orchestrator = RecoveryOrchestrator::new();
    let decision =
        orchestrator.explain_recovery(&registry, state.resolved.as_ref(), &entity_id, &failure);
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "entity_id": entity_id,
        "failure": failure,
        "decision": decision,
    }))
}

/// GET /v1/recovery/policies
pub fn recovery_policies(state: &ControlCenterState) -> HttpResponse {
    let registry = state.entity_registry();
    let orchestrator = RecoveryOrchestrator::new();
    let policies = state
        .resolved
        .as_ref()
        .map(|r| orchestrator.list_policies(&registry, r))
        .unwrap_or_default();
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "policies": policies,
    }))
}
