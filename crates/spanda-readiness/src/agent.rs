//! Agent and service readiness evaluation from deployed program source.

use crate::engine::evaluate_readiness_with_runtime;
use crate::runtime::build_runtime_context;
use crate::target::readiness_options_from_flags;
use crate::types::ReadinessReport;
use spanda_lexer::tokenize;
use spanda_parser::parse;

/// Evaluate readiness for an on-device agent from program source text.
pub fn evaluate_agent_readiness(
    source: &str,
    target: Option<&str>,
    include_runtime: bool,
    inject_health_faults: bool,
) -> Result<ReadinessReport, String> {
    let tokens = tokenize(source).map_err(|e| e.to_string())?;
    let program = parse(tokens).map_err(|e| e.to_string())?;
    let options = readiness_options_from_flags(
        &program,
        target.map(String::from),
        include_runtime,
        inject_health_faults,
        false,
        false,
    );
    let runtime = options
        .include_runtime
        .then(|| build_runtime_context(&program, options.inject_health_faults));
    Ok(evaluate_readiness_with_runtime(
        &program,
        &options,
        runtime.as_ref(),
    ))
}

/// JSON payload for `GET /v1/readiness` agent endpoints.
pub fn evaluate_agent_readiness_json(
    source: &str,
    target: Option<&str>,
    include_runtime: bool,
    inject_health_faults: bool,
) -> Result<String, String> {
    let report = evaluate_agent_readiness(source, target, include_runtime, inject_health_faults)?;
    serde_json::to_string(&serde_json::json!({
        "ok": true,
        "mission_ready": report.mission_ready,
        "readiness": report,
    }))
    .map_err(|e| e.to_string())
}
