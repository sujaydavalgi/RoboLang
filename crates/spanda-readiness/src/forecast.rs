//! Readiness forecasting — current score, history trends, and time-based degradation.

use crate::engine::evaluate_readiness;
use crate::safety_coverage::evaluate_safety_coverage;
use crate::trends::{analyze_readiness_trends, load_readiness_history, ReadinessForecast};
use crate::types::{ReadinessOptions, ReadinessReport};
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use std::path::Path;

/// Predicted readiness at a future horizon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessPrediction {
    pub horizon_days: u32,
    pub current_score: u32,
    pub predicted_score: u32,
    pub degradation_per_day: f64,
    pub projected_risks: Vec<String>,
    pub risk_warning: bool,
}

/// Full readiness forecast report for a program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessForecastReport {
    pub program: String,
    pub mission_ready: bool,
    pub current_score: u32,
    pub maximum_score: u32,
    pub history_samples: usize,
    pub degradation_per_day: f64,
    pub predictions: Vec<ReadinessPrediction>,
    pub warnings: Vec<String>,
}

/// Options for readiness forecasting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessForecastOptions {
    pub horizons_days: Vec<u32>,
    pub minimum_score: u32,
    pub history_path: Option<std::path::PathBuf>,
    pub target: Option<String>,
}

impl Default for ReadinessForecastOptions {
    fn default() -> Self {
        Self {
            horizons_days: vec![7, 14, 30],
            minimum_score: 80,
            history_path: None,
            target: None,
        }
    }
}

/// Evaluate current readiness and project future scores with degradation heuristics.
pub fn evaluate_readiness_forecast(
    program: &Program,
    source_label: &str,
    options: &ReadinessForecastOptions,
) -> ReadinessForecastReport {
    let readiness = evaluate_readiness(
        program,
        &ReadinessOptions {
            simulate: true,
            include_runtime: true,
            target: options.target.clone(),
            ..ReadinessOptions::default()
        },
    );
    let safety = evaluate_safety_coverage(program, source_label);
    let history_path = options
        .history_path
        .clone()
        .unwrap_or_else(crate::trends::default_readiness_history_path);
    let history = load_readiness_history(&history_path);
    let trend_report = analyze_readiness_trends(
        &history,
        source_label,
        options.horizons_days.first().copied(),
        options.minimum_score,
    );

    let mut warnings = trend_report.warnings.clone();
    let history_slope = trend_report
        .overall_trend
        .as_ref()
        .map(|t| t.slope_per_day)
        .unwrap_or(0.0);
    let projected_risks = projected_risks(&readiness, safety.overall_coverage_pct);
    let current_score = readiness.score.total;
    let heuristic_decay = heuristic_degradation_per_day(&readiness, safety.overall_coverage_pct);
    let degradation_per_day = if readiness.mission_ready && current_score >= options.minimum_score {
        if history_slope < 0.0 {
            history_slope
        } else {
            -heuristic_decay.min(0.5)
        }
    } else if history_slope < 0.0 {
        history_slope.min(-heuristic_decay)
    } else {
        -heuristic_decay
    };

    let predictions: Vec<ReadinessPrediction> = options
        .horizons_days
        .iter()
        .map(|days| {
            let history_forecast = analyze_readiness_trends(
                &history,
                source_label,
                Some(*days),
                options.minimum_score,
            )
            .forecast;
            let predicted_score = history_forecast
                .as_ref()
                .map(|f| f.predicted_score)
                .unwrap_or_else(|| project_score(current_score, degradation_per_day, *days));
            let risk_warning = predicted_score < options.minimum_score;
            let mut horizon_risks = projected_risks.clone();
            if let Some(ReadinessForecast { message, .. }) = history_forecast {
                if !message.is_empty() {
                    horizon_risks.push(message);
                }
            }
            if risk_warning {
                horizon_risks.push(format!(
                    "predicted score {predicted_score} below minimum {} at {days}d",
                    options.minimum_score
                ));
            }
            ReadinessPrediction {
                horizon_days: *days,
                current_score,
                predicted_score,
                degradation_per_day,
                projected_risks: horizon_risks,
                risk_warning,
            }
        })
        .collect();

    if trend_report.sample_count == 0 {
        warnings.push(
            "no readiness history; forecast uses heuristic degradation only — run `spanda readiness --record`"
                .into(),
        );
    }

    ReadinessForecastReport {
        program: source_label.into(),
        mission_ready: readiness.mission_ready,
        current_score,
        maximum_score: readiness.score.maximum,
        history_samples: trend_report.sample_count,
        degradation_per_day,
        predictions,
        warnings,
    }
}

/// Format a readiness forecast report for CLI output.
pub fn format_readiness_forecast(report: &ReadinessForecastReport, json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(report).unwrap_or_else(|e| e.to_string());
    }

    let mut lines = vec![
        format!("Readiness forecast: {}", report.program),
        format!(
            "Current: {}/{} mission_ready={}",
            report.current_score, report.maximum_score, report.mission_ready
        ),
        format!(
            "Degradation: {:.2}/day (history samples: {})",
            report.degradation_per_day, report.history_samples
        ),
        String::new(),
    ];

    for prediction in &report.predictions {
        lines.push(format!(
            "  {}d: {} → {} (risk={})",
            prediction.horizon_days,
            prediction.current_score,
            prediction.predicted_score,
            prediction.risk_warning
        ));
        for risk in &prediction.projected_risks {
            lines.push(format!("    - {risk}"));
        }
    }

    for warning in &report.warnings {
        lines.push(format!("Warning: {warning}"));
    }

    lines.join("\n")
}

fn heuristic_degradation_per_day(readiness: &ReadinessReport, safety_coverage_pct: u32) -> f64 {
    let mut decay = 0.0;
    if !readiness.mission_ready {
        decay += 1.5;
    }
    for issue in &readiness.issues {
        match issue.severity {
            crate::types::ReadinessSeverity::Critical => decay += 2.0,
            crate::types::ReadinessSeverity::High => decay += 1.0,
            crate::types::ReadinessSeverity::Medium => decay += 0.5,
            _ => decay += 0.1,
        }
    }
    if safety_coverage_pct < 80 {
        decay += (80 - safety_coverage_pct.min(80)) as f64 / 20.0;
    }
    for factor in &readiness.score.factors {
        if factor.score < 70 {
            decay += 0.3;
        }
    }
    decay.clamp(0.0, 5.0)
}

fn projected_risks(readiness: &ReadinessReport, safety_coverage_pct: u32) -> Vec<String> {
    let mut risks = Vec::new();
    if !readiness.mission_ready {
        risks.push("mission not ready at current evaluation".into());
    }
    if safety_coverage_pct < 100 {
        risks.push(format!(
            "safety coverage {safety_coverage_pct}% may degrade under field stress"
        ));
    }
    for issue in readiness.issues.iter().take(3) {
        risks.push(format!(
            "{}: {}",
            format!("{:?}", issue.severity),
            issue.message
        ));
    }
    risks
}

fn project_score(current: u32, degradation_per_day: f64, days: u32) -> u32 {
    (current as f64 + degradation_per_day * days as f64)
        .round()
        .clamp(0.0, 100.0) as u32
}

/// Resolve history path from optional override.
pub fn forecast_history_path(override_path: Option<&Path>) -> std::path::PathBuf {
    override_path
        .map(Path::to_path_buf)
        .unwrap_or_else(crate::trends::default_readiness_history_path)
}
