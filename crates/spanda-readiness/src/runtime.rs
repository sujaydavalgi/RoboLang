//! Runtime health context for live readiness evaluation.

use spanda_ast::nodes::{Program, RobotDecl, SensorDecl};
use spanda_hal::HardwareMonitor;

/// Live fault and event signals used during readiness evaluation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeReadinessContext {
    pub faults: Vec<String>,
    pub events: Vec<String>,
}

const DEFAULT_HEALTH_FAULTS: &[&str] = &["GPSDegraded", "CameraOffline", "RobotHealthCritical"];

/// Register robot sensors/actuators on a hardware monitor from the program AST.
pub fn seed_hardware_monitor(program: &Program, monitor: &mut HardwareMonitor) {
    let Program::Program { robots, .. } = program;
    for robot in robots {
        let RobotDecl::RobotDecl {
            sensors, actuators, ..
        } = robot;
        for sensor in sensors {
            let SensorDecl::SensorDecl {
                name, sensor_type, ..
            } = sensor;
            monitor.register_sensor(name, sensor_type);
        }
        for actuator in actuators {
            let spanda_ast::nodes::ActuatorDecl::ActuatorDecl {
                name,
                actuator_type,
                ..
            } = actuator;
            monitor.register_actuator(name, actuator_type);
        }
    }
}

/// Build runtime readiness context from a program and optional fault injection.
pub fn build_runtime_context(
    program: &Program,
    inject_health_faults: bool,
) -> RuntimeReadinessContext {
    let mut monitor = HardwareMonitor::default();
    seed_hardware_monitor(program, &mut monitor);
    if inject_health_faults {
        for fault in DEFAULT_HEALTH_FAULTS {
            monitor.inject_fault((*fault).to_string());
        }
    }
    RuntimeReadinessContext {
        faults: monitor.runtime_faults(),
        events: monitor.runtime_events(),
    }
}

impl RuntimeReadinessContext {
    /// Capture faults and events from an existing hardware monitor instance.
    pub fn from_monitor(monitor: &HardwareMonitor) -> Self {
        Self {
            faults: monitor.runtime_faults(),
            events: monitor.runtime_events(),
        }
    }
}
