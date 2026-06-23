//! Span lookup for readiness diagnostics in the IDE.

use spanda_ast::foundations::{DeployDecl, HealthCheckDecl};
use spanda_ast::nodes::Program;

/// Resolve a display line/column for a readiness issue factor.
pub fn line_column_for_factor(program: &Program, factor: &str) -> (u32, u32) {
    match factor {
        "Hardware" | "Battery" | "Connectivity" | "Storage" | "Compute" | "Packages"
        | "Providers" => deploy_span(program).unwrap_or_else(|| first_robot_span(program)),
        "Health" => first_health_check_span(program).unwrap_or_else(|| first_robot_span(program)),
        "Capabilities" | "Mission Requirements" => {
            mission_span(program).unwrap_or_else(|| first_robot_span(program))
        }
        "Safety" => first_robot_safety_span(program).unwrap_or_else(|| first_robot_span(program)),
        "Fleet" => first_fleet_span(program).unwrap_or((1, 1)),
        _ => (1, 1),
    }
}

fn deploy_span(program: &Program) -> Option<(u32, u32)> {
    let Program::Program { deployments, .. } = program;
    deployments.first().map(|deploy| {
        let DeployDecl::DeployDecl { span, .. } = deploy;
        (span.start.line, span.start.column)
    })
}

fn first_robot_span(program: &Program) -> (u32, u32) {
    let Program::Program { robots, .. } = program;
    robots
        .first()
        .map(|robot| {
            let spanda_ast::nodes::RobotDecl::RobotDecl { span, .. } = robot;
            (span.start.line, span.start.column)
        })
        .unwrap_or((1, 1))
}

fn first_health_check_span(program: &Program) -> Option<(u32, u32)> {
    let Program::Program { health_checks, .. } = program;
    health_checks.first().map(|hc| {
        let HealthCheckDecl::HealthCheckDecl { span, .. } = hc;
        (span.start.line, span.start.column)
    })
}

fn mission_span(program: &Program) -> Option<(u32, u32)> {
    let Program::Program { robots, .. } = program;
    for robot in robots {
        let spanda_ast::nodes::RobotDecl::RobotDecl { mission, .. } = robot;
        if let Some(mission) = mission {
            let spanda_ast::foundations::MissionDecl::MissionDecl { span, .. } = mission;
            return Some((span.start.line, span.start.column));
        }
    }
    None
}

fn first_robot_safety_span(program: &Program) -> Option<(u32, u32)> {
    let Program::Program { robots, .. } = program;
    for robot in robots {
        let spanda_ast::nodes::RobotDecl::RobotDecl { safety, .. } = robot;
        if let Some(spanda_ast::nodes::SafetyBlock::SafetyBlock { span, .. }) = safety {
            return Some((span.start.line, span.start.column));
        }
    }
    None
}

fn first_fleet_span(program: &Program) -> Option<(u32, u32)> {
    let Program::Program { fleets, .. } = program;
    fleets.first().map(|fleet| {
        let spanda_ast::robotics_decl::FleetDecl::FleetDecl { span, .. } = fleet;
        (span.start.line, span.start.column)
    })
}
