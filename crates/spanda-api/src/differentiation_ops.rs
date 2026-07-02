//! NEXT differentiation analytics REST handlers — what-if, risk, forecast, trust graph.

use crate::program::parse_program_file;
use crate::state::ControlCenterState;
use spanda_deploy_http::HttpResponse;
use spanda_graph::{build_trust_graph, format_trust_graph, GraphFormat};
use spanda_readiness::{
    default_deploy_target, evaluate_readiness_forecast, parse_forecast_horizon,
    ReadinessForecastOptions,
};
use spanda_risk::evaluate_mission_risk;
use spanda_whatif::{run_what_if_analysis, WhatIfOptions};

use crate::handlers::{bad_request, json_ok, parse_query};

fn load_program(
    state: &ControlCenterState,
) -> Result<(spanda_ast::nodes::Program, String, String), String> {
    let Some(path) = state.program_path.as_ref() else {
        return Err("no program loaded; use control-center serve --program <file.sd>".into());
    };
    parse_program_file(path)
}

/// `GET /v1/analytics/what-if?scenario=gps_failure&all=1`
pub fn analytics_what_if(state: &ControlCenterState, query: &str) -> HttpResponse {
    let params = parse_query(query);
    let (program, _source, label) = match load_program(state) {
        Ok(value) => value,
        Err(message) => return bad_request(&message),
    };
    let all = params.get("all").is_some_and(|v| v == "1" || v == "true");
    let scenario = params.get("scenario").cloned();
    let options = WhatIfOptions {
        all,
        scenarios: scenario.map(|s| vec![s]).unwrap_or_default(),
    };
    let report = run_what_if_analysis(&program, &label, &options);
    json_ok(&serde_json::json!({
        "version": "v1",
        "program": label,
        "what_if": report,
    }))
}

/// `GET /v1/analytics/mission-risk`
pub fn analytics_mission_risk(state: &ControlCenterState) -> HttpResponse {
    let (program, source, label) = match load_program(state) {
        Ok(value) => value,
        Err(message) => return bad_request(&message),
    };
    let report = evaluate_mission_risk(&program, &source, &label);
    json_ok(&serde_json::json!({
        "version": "v1",
        "program": label,
        "mission_risk": report,
    }))
}

/// `GET /v1/analytics/readiness-forecast?horizon=7d&all=1`
pub fn analytics_readiness_forecast(state: &ControlCenterState, query: &str) -> HttpResponse {
    let params = parse_query(query);
    let (program, _source, label) = match load_program(state) {
        Ok(value) => value,
        Err(message) => return bad_request(&message),
    };
    let all = params.get("all").is_some_and(|v| v == "1" || v == "true");
    let horizons = if all {
        vec![7, 14, 30]
    } else if let Some(raw) = params.get("horizon") {
        parse_forecast_horizon(raw)
            .map(|days| vec![days])
            .unwrap_or_else(|| vec![7])
    } else {
        vec![7]
    };
    let target = default_deploy_target(&program);
    let report = evaluate_readiness_forecast(
        &program,
        &label,
        &ReadinessForecastOptions {
            horizons_days: horizons,
            minimum_score: 80,
            history_path: None,
            target,
        },
    );
    json_ok(&serde_json::json!({
        "version": "v1",
        "program": label,
        "readiness_forecast": report,
    }))
}

/// `GET /v1/analytics/trust-graph?format=json`
pub fn analytics_trust_graph(state: &ControlCenterState, query: &str) -> HttpResponse {
    let params = parse_query(query);
    let (program, source, label) = match load_program(state) {
        Ok(value) => value,
        Err(message) => return bad_request(&message),
    };
    let format = params
        .get("format")
        .map(|value| GraphFormat::parse(value))
        .unwrap_or(GraphFormat::Json);
    let graph = build_trust_graph(&program, &source, &label, state.resolved.as_ref());
    if format == GraphFormat::Json {
        return json_ok(&serde_json::json!({
            "version": "v1",
            "program": label,
            "trust_graph": graph,
        }));
    }
    json_ok(&serde_json::json!({
        "version": "v1",
        "program": label,
        "format": match format {
            GraphFormat::Mermaid => "mermaid",
            GraphFormat::Dot => "dot",
            GraphFormat::Text => "text",
            GraphFormat::Json => "json",
        },
        "body": format_trust_graph(&graph, format),
    }))
}
