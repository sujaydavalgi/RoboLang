//! Digital mission twin — live mission state mirror composing readiness and mission verification.

use crate::engine::evaluate_readiness;
use crate::forecast::{
    evaluate_readiness_forecast, ReadinessForecastOptions, ReadinessForecastReport,
};
use crate::mission::verify_mission;
use crate::types::{ReadinessOptions, ReadinessReport};
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;

/// Mission progress and checkpoint model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionStateModel {
    pub mission_count: usize,
    pub mission_plan_steps: usize,
    pub robots: Vec<String>,
    pub missions: Vec<String>,
    pub verification_passed: bool,
}

/// Active risk signals for the mission twin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionRiskModel {
    pub open_issues: usize,
    pub critical_issues: usize,
    pub readiness_score: u32,
    pub mission_ready: bool,
}

/// Full digital mission twin report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionTwinReport {
    pub program: String,
    pub state: MissionStateModel,
    pub readiness: ReadinessReport,
    pub risks: MissionRiskModel,
    pub forecast: ReadinessForecastReport,
}

/// Build a digital mission twin report for a program.
pub fn evaluate_mission_twin(program: &Program, source_label: &str) -> MissionTwinReport {
    let readiness = evaluate_readiness(
        program,
        &ReadinessOptions {
            simulate: true,
            include_runtime: true,
            ..ReadinessOptions::default()
        },
    );
    let mission_reports = verify_mission(program, None);
    let verification_passed = mission_reports.iter().all(|report| report.achievable);
    let forecast = evaluate_readiness_forecast(
        program,
        source_label,
        &ReadinessForecastOptions {
            horizons_days: vec![7, 14],
            ..ReadinessForecastOptions::default()
        },
    );
    let Program::Program {
        robots,
        mission_plans,
        ..
    } = program;
    let mission_names: Vec<String> = robots
        .iter()
        .filter_map(|robot| {
            let spanda_ast::nodes::RobotDecl::RobotDecl { mission, name, .. } = robot;
            mission.as_ref().map(|_| name.clone())
        })
        .collect();
    let plan_steps = mission_plans
        .iter()
        .map(|plan| {
            let spanda_ast::assurance_decl::MissionPlanDecl::MissionPlanDecl { steps, .. } = plan;
            steps.len()
        })
        .sum();
    let critical = readiness
        .issues
        .iter()
        .filter(|issue| {
            matches!(
                issue.severity,
                crate::types::ReadinessSeverity::Critical | crate::types::ReadinessSeverity::High
            )
        })
        .count();
    MissionTwinReport {
        program: source_label.into(),
        state: MissionStateModel {
            mission_count: mission_names.len(),
            mission_plan_steps: plan_steps,
            robots: robots
                .iter()
                .map(|robot| {
                    let spanda_ast::nodes::RobotDecl::RobotDecl { name, .. } = robot;
                    name.clone()
                })
                .collect(),
            missions: mission_names,
            verification_passed,
        },
        risks: MissionRiskModel {
            open_issues: readiness.issues.len(),
            critical_issues: critical,
            readiness_score: readiness.score.total,
            mission_ready: readiness.mission_ready,
        },
        readiness,
        forecast,
    }
}

/// Format mission twin report for CLI output.
pub fn format_mission_twin(report: &MissionTwinReport, json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(report).unwrap_or_else(|e| e.to_string());
    }
    format!(
        "Mission twin: {}\nMissions: {} robots, {} plan steps\nReadiness: {}/{} mission_ready={}\nOpen issues: {} (critical {})\nForecast 7d: {}",
        report.program,
        report.state.mission_count,
        report.state.mission_plan_steps,
        report.readiness.score.total,
        report.readiness.score.maximum,
        report.readiness.mission_ready,
        report.risks.open_issues,
        report.risks.critical_issues,
        report
            .forecast
            .predictions
            .first()
            .map(|p| p.predicted_score.to_string())
            .unwrap_or_else(|| "n/a".into())
    )
}
