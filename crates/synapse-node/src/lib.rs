#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use synapse_core::{check, run, RunOptions, SynapseError};

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
pub struct RunResultJs {
    pub state: RobotStateJs,
    pub events: Vec<String>,
    pub logs: Vec<String>,
}

#[napi(object)]
pub struct RunOptionsJs {
    pub entry_behavior: Option<String>,
    #[napi(ts_type = "number")]
    pub max_loop_iterations: Option<u32>,
}

fn map_diagnostics(err: &SynapseError) -> Vec<DiagnosticJs> {
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
    let opts = options.unwrap_or(RunOptionsJs {
        entry_behavior: None,
        max_loop_iterations: None,
    });
    let result = run(
        &source,
        RunOptions {
            entry_behavior: opts.entry_behavior,
            max_loop_iterations: opts.max_loop_iterations.unwrap_or(10) as usize,
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
    })
}

#[napi]
pub fn core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
