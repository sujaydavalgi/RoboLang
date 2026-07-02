//! src crate public API and re-exports.
//!
#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use spanda_driver::{check, lower_to_sir, run, RunOptions};
use spanda_core::verify_compatibility;
use spanda_error::SpandaError;
use spanda_format::format_source;
use spanda_hardware::VerifyOptions;

#[napi(object)]
pub struct DiagnosticJs {
    pub message: String,
    pub line: u32,
    pub column: u32,
}

#[napi(object)]
pub struct CheckResultJs {
    pub ok: bool,
    pub diagnostics: Vec<DiagnosticJs>,
}

#[napi(object)]
pub struct PoseStateJs {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
    pub z: Option<f64>,
}

#[napi(object)]
pub struct VelocityStateJs {
    pub linear: f64,
    pub angular: f64,
}

#[napi(object)]
pub struct RobotStateJs {
    pub pose: PoseStateJs,
    pub velocity: VelocityStateJs,
    pub emergency_stop: bool,
}

#[napi(object)]
pub struct TaskMetricsJs {
    pub name: String,
    pub priority: String,
    pub interval_ms: f64,
    #[napi(ts_type = "number")]
    pub ticks: f64,
    #[napi(ts_type = "number")]
    pub skipped: f64,
    #[napi(ts_type = "number")]
    pub missed_deadlines: f64,
    #[napi(ts_type = "number")]
    pub budget_violations: f64,
    pub last_duration_ms: f64,
    pub max_duration_ms: f64,
}

#[napi(object)]
pub struct SchedulerMetricsJs {
    #[napi(ts_type = "number")]
    pub multiplexed_tasks: f64,
    #[napi(ts_type = "number")]
    pub scheduler_ticks: f64,
    pub base_tick_ms: f64,
    #[napi(ts_type = "number")]
    pub emergency_stops: f64,
}

#[napi(object)]
pub struct ExecutionMetricsJs {
    #[napi(ts_type = "number")]
    pub spawns: f64,
    #[napi(ts_type = "number")]
    pub joins: f64,
    #[napi(ts_type = "number")]
    pub parallel_blocks: f64,
    #[napi(ts_type = "number")]
    pub fire_and_forget_spawns: f64,
}

#[napi(object)]
pub struct RuntimeTelemetryJs {
    pub tasks: Vec<TaskMetricsJs>,
    pub scheduler: SchedulerMetricsJs,
    pub execution: ExecutionMetricsJs,
    #[napi(ts_type = "number")]
    pub replay_frames: f64,
}

#[napi(object)]
pub struct RunResultJs {
    pub state: RobotStateJs,
    pub events: Vec<String>,
    pub logs: Vec<String>,
    pub metrics: RuntimeTelemetryJs,
}

#[napi(object)]
pub struct RunOptionsJs {
    pub entry_behavior: Option<String>,
    #[napi(ts_type = "number")]
    pub max_loop_iterations: Option<u32>,
    pub trace_scheduler: Option<bool>,
    pub trace_tasks: Option<bool>,
    pub replay_trace: Option<bool>,
}

fn map_diagnostics(err: &SpandaError) -> Vec<DiagnosticJs> {
    // Description:
    //     Map diagnostics.
    //
    // Inputs:
    //     err: &SpandaError
    //         Caller-supplied err.
    //
    // Outputs:
    //     result: Vec<DiagnosticJs>
    //         Return value from `map_diagnostics`.
    //
    // Example:
    //     let result = spanda_node::map_diagnostics(err);

    // Produce diagnostics as the result.
    err.diagnostics()
        .into_iter()
        .map(|d| DiagnosticJs {
            message: d.message,
            line: d.line,
            column: d.column,
        })
        .collect()
}

#[napi]
pub fn check_source(source: String) -> CheckResultJs {
    // Description:
    //     Check source.
    //
    // Inputs:
    //     source: String
    //         Caller-supplied source.
    //
    // Outputs:
    //     result: CheckResultJs
    //         Return value from `check_source`.
    //
    // Example:
    //     let result = spanda_node::check_source(source);

    // Match on check and handle each case.
    match check(&source) {
        Ok(()) => CheckResultJs {
            ok: true,
            diagnostics: vec![],
        },
        Err(e) => CheckResultJs {
            ok: false,
            diagnostics: map_diagnostics(&e),
        },
    }
}

#[napi]
pub fn run_source(source: String, options: Option<RunOptionsJs>) -> Result<RunResultJs> {
    // Description:
    //     Run source.
    //
    // Inputs:
    //     source: String
    //         Caller-supplied source.
    //     options: Option<RunOptionsJs>
    //         Caller-supplied options.
    //
    // Outputs:
    //     result: Result<RunResultJs>
    //         Return value from `run_source`.
    //
    // Example:
    //     let result = spanda_node::run_source(source, options);

    // Compute opts for the following logic.
    let opts = options.unwrap_or(RunOptionsJs {
        entry_behavior: None,
        max_loop_iterations: None,
        trace_scheduler: None,
        trace_tasks: None,
        replay_trace: None,
    });
    let result = run(
        &source,
        RunOptions {
            entry_behavior: opts.entry_behavior,
            max_loop_iterations: opts.max_loop_iterations.unwrap_or(10) as usize,
            trace_scheduler: opts.trace_scheduler.unwrap_or(false),
            trace_tasks: opts.trace_tasks.unwrap_or(false),
            replay_trace: opts.replay_trace.unwrap_or(false),
            ..Default::default()
        },
    )
    .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(RunResultJs {
        state: RobotStateJs {
            pose: PoseStateJs {
                x: result.state.pose.x,
                y: result.state.pose.y,
                theta: result.state.pose.theta,
                z: result.state.pose.z,
            },
            velocity: VelocityStateJs {
                linear: result.state.velocity.linear,
                angular: result.state.velocity.angular,
            },
            emergency_stop: result.state.emergency_stop,
        },
        events: result.events,
        logs: result.logs,
        metrics: RuntimeTelemetryJs {
            tasks: result
                .metrics
                .tasks
                .into_values()
                .map(|t| TaskMetricsJs {
                    name: t.name,
                    priority: t.priority,
                    interval_ms: t.interval_ms,
                    ticks: t.ticks as f64,
                    skipped: t.skipped as f64,
                    missed_deadlines: t.missed_deadlines as f64,
                    budget_violations: t.budget_violations as f64,
                    last_duration_ms: t.last_duration_ms,
                    max_duration_ms: t.max_duration_ms,
                })
                .collect(),
            scheduler: SchedulerMetricsJs {
                multiplexed_tasks: result.metrics.scheduler.multiplexed_tasks as f64,
                scheduler_ticks: result.metrics.scheduler.scheduler_ticks as f64,
                base_tick_ms: result.metrics.scheduler.base_tick_ms,
                emergency_stops: result.metrics.scheduler.emergency_stops as f64,
            },
            execution: ExecutionMetricsJs {
                spawns: result.metrics.execution.spawns as f64,
                joins: result.metrics.execution.joins as f64,
                parallel_blocks: result.metrics.execution.parallel_blocks as f64,
                fire_and_forget_spawns: result.metrics.execution.fire_and_forget_spawns as f64,
            },
            replay_frames: result.metrics.replay_frames as f64,
        },
    })
}

#[napi]
pub fn core_version() -> String {
    // Description:
    //     Core version.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: String
    //         Return value from `core_version`.
    //
    // Example:
    //     let result = spanda_node::core_version();

    // Produce to string as the result.
    env!("CARGO_PKG_VERSION").to_string()
}

#[napi(object)]
pub struct CompatItemJs {
    pub category: String,
    pub message: String,
    pub severity: String,
    pub line: u32,
    pub column: u32,
}

#[napi(object)]
pub struct VerifyResultJs {
    pub ok: bool,
    pub compatible: bool,
    pub items: Vec<CompatItemJs>,
}

#[napi]
pub fn verify_source(source: String) -> VerifyResultJs {
    // Description:
    //     Verify source.
    //
    // Inputs:
    //     source: String
    //         Caller-supplied source.
    //
    // Outputs:
    //     result: VerifyResultJs
    //         Return value from `verify_source`.
    //
    // Example:
    //     let result = spanda_node::verify_source(source);

    // Match on default and handle each case.
    match verify_compatibility(&source, &VerifyOptions::default()) {
        Ok(report) => VerifyResultJs {
            ok: report.compatible,
            compatible: report.compatible,
            items: report
                .items
                .into_iter()
                .map(|i| CompatItemJs {
                    category: i.category,
                    message: i.message,
                    severity: format!("{:?}", i.severity).to_lowercase(),
                    line: i.line,
                    column: i.column,
                })
                .collect(),
        },
        Err(e) => VerifyResultJs {
            ok: false,
            compatible: false,
            items: map_diagnostics(&e)
                .into_iter()
                .map(|d| CompatItemJs {
                    category: "error".into(),
                    message: d.message,
                    severity: "error".into(),
                    line: d.line,
                    column: d.column,
                })
                .collect(),
        },
    }
}

#[napi]
pub fn sir_source(source: String) -> Result<String> {
    // Description:
    //     Sir source.
    //
    // Inputs:
    //     source: String
    //         Caller-supplied source.
    //
    // Outputs:
    //     result: Result<String>
    //         Return value from `sir_source`.
    //
    // Example:
    //     let result = spanda_node::sir_source(source);

    // Produce lower to sir as the result.
    lower_to_sir(&source)
        .map(|sir| serde_json::to_string_pretty(&sir).unwrap_or_default())
        .map_err(|e| Error::from_reason(e.to_string()))
}

#[napi]
pub fn fmt_source(source: String) -> String {
    // Description:
    //     Fmt source.
    //
    // Inputs:
    //     source: String
    //         Caller-supplied source.
    //
    // Outputs:
    //     result: String
    //         Return value from `fmt_source`.
    //
    // Example:
    //     let result = spanda_node::fmt_source(source);

    // Produce format source as the result.
    format_source(&source)
}
