//! error support for Spanda.
//!
pub use spanda_error::{Diagnostic, SpandaError};

pub use spanda_driver::CompileResult;

pub use spanda_runtime::robot_state::{PoseState, RobotState, VelocityState};

pub use spanda_interpreter::{
    ObstacleConfig, RunOptions, RunResult,
};
