//! Distributed fleet orchestration beyond in-process `spanda fleet run`.
//!
//! Builds coordination plans from program-level `fleet` declarations and peer-robot wiring,
//! then executes a round-robin mission coordination pass across fleet members.

use crate::ast::{Program, RobotDecl};
use crate::comm::{CommBus, InMemoryCommBus, PeerRobotDecl, TransportKind};
use crate::foundations::MissionDecl;
use crate::robotics_platform::{FleetDecl, FleetRegistry, MissionRuntime};
use crate::runtime::RuntimeValue;
use serde::{Deserialize, Serialize};
pub use spanda_fleet::PeerDelivery;

/// Per-member coordination state during orchestration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetMemberState {
    pub robot_name: String,
    pub mission_name: Option<String>,
    pub mission_state: String,
    pub current_step: String,
    pub has_peer_link: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub peer_handoffs: Vec<String>,
}

/// Orchestration report for one fleet group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetOrchestrationReport {
    pub fleet_name: String,
    pub members: Vec<FleetMemberState>,
    pub coordination_mode: String,
    pub steps_advanced: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub peer_messages: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub peer_deliveries: Vec<PeerDelivery>,
    #[serde(default)]
    pub remote_relayed: u32,
    #[serde(default)]
    pub remote_failed: u32,
}

/// Full orchestration result for a program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetOrchestrationResult {
    pub program: String,
    pub fleets: Vec<FleetOrchestrationReport>,
    pub success: bool,
}

fn robot_by_name<'a>(robots: &'a [RobotDecl], name: &str) -> Option<&'a RobotDecl> {
    robots.iter().find(|r| r.name() == name)
}

fn mission_for_robot(robot: &RobotDecl) -> Option<MissionRuntime> {
    let RobotDecl::RobotDecl { mission, .. } = robot;
    let mission = mission.as_ref()?;
    let MissionDecl::MissionDecl {
        name,
        duration_hours,
        steps,
        ..
    } = mission;
    Some(MissionRuntime::new(
        name.clone(),
        steps.clone(),
        *duration_hours,
    ))
}

pub fn peer_handoffs(member_name: &str, step: &str, peer_robots: &[PeerRobotDecl]) -> Vec<String> {
    if step.is_empty() || peer_robots.is_empty() {
        return Vec::new();
    }
    peer_robots
        .iter()
        .map(|peer| {
            let PeerRobotDecl::PeerRobotDecl { name, .. } = peer;
            format!("{member_name}->{name}:step={step}")
        })
        .collect()
}

pub fn deliver_peer_steps(
    mesh_bus: &mut InMemoryCommBus,
    from_robot: &str,
    step: &str,
    peer_robots: &[PeerRobotDecl],
) -> Vec<PeerDelivery> {
    if step.is_empty() || peer_robots.is_empty() {
        return Vec::new();
    }
    let mut deliveries = Vec::new();
    for peer in peer_robots {
        let PeerRobotDecl::PeerRobotDecl { name, .. } = peer;
        mesh_bus.register_robot(name);
        mesh_bus.publish_peer(
            name,
            "mission_step",
            RuntimeValue::String {
                value: step.to_string(),
            },
            TransportKind::Sim,
            Some(from_robot),
        );
        let path = format!("/{name}/mission_step");
        let delivered = mesh_bus.published_messages().iter().any(|message| {
            message.topic_path == path && message.source_id.as_deref() == Some(from_robot)
        });
        deliveries.push(PeerDelivery {
            from_robot: from_robot.to_string(),
            to_robot: name.clone(),
            topic: "mission_step".into(),
            step: step.to_string(),
            delivered,
        });
    }
    deliveries
}

/// Build fleet registry from program declarations.
pub fn fleet_registry_from_program(program: &Program) -> FleetRegistry {
    let Program::Program { fleets, .. } = program;
    let mut registry = FleetRegistry::default();
    for fleet in fleets {
        let FleetDecl::FleetDecl { name, members, .. } = fleet;
        registry.register(name, members.clone());
    }
    registry
}

/// Orchestrate fleet members by advancing missions in round-robin order.
pub fn orchestrate_fleets(program: &Program, program_path: &str) -> FleetOrchestrationResult {
    // Coordinate declared fleet groups using each member robot's mission controller.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `program_path` — source path for reporting
    //
    // Returns:
    // Orchestration report with per-member mission states.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = orchestrate_fleets(&program, "fleet.sd");

    let Program::Program { fleets, robots, .. } = program;
    let mut reports = Vec::new();

    for fleet in fleets {
        let FleetDecl::FleetDecl { name, members, .. } = fleet;
        let mut member_states = Vec::new();
        let mut steps_advanced = 0u32;
        let mut peer_messages = Vec::new();
        let mut peer_deliveries = Vec::new();
        let mut mesh_bus = InMemoryCommBus::new();

        for member_name in members {
            mesh_bus.register_robot(member_name);
        }

        for member_name in members {
            let Some(robot) = robot_by_name(robots, member_name) else {
                member_states.push(FleetMemberState {
                    robot_name: member_name.clone(),
                    mission_name: None,
                    mission_state: "MissingRobot".into(),
                    current_step: String::new(),
                    has_peer_link: false,
                    peer_handoffs: Vec::new(),
                });
                continue;
            };
            let RobotDecl::RobotDecl {
                peer_robots, mission, ..
            } = robot;
            let mut runtime = mission_for_robot(robot);
            let (mission_name, mission_state, current_step) = if let Some(ref mut m) = runtime {
                m.start();
                let step = m.advance().unwrap_or_default();
                if !step.is_empty() {
                    steps_advanced += 1;
                }
                (
                    m.name.clone(),
                    m.state.as_str().to_string(),
                    step,
                )
            } else {
                (None, "NoMission".into(), String::new())
            };
            let handoffs = peer_handoffs(member_name, &current_step, peer_robots);
            peer_messages.extend(handoffs.clone());
            peer_deliveries.extend(deliver_peer_steps(
                &mut mesh_bus,
                member_name,
                &current_step,
                peer_robots,
            ));
            let _ = mission;
            member_states.push(FleetMemberState {
                robot_name: member_name.clone(),
                mission_name,
                mission_state,
                current_step,
                has_peer_link: !peer_robots.is_empty(),
                peer_handoffs: handoffs,
            });
        }

        let has_peer_link = member_states.iter().any(|m| m.has_peer_link);
        let coordination_mode = if !peer_deliveries.is_empty() {
            "peer_mesh_mission".into()
        } else if has_peer_link {
            "peer_round_robin_mission".into()
        } else {
            "round_robin_mission".into()
        };
        reports.push(FleetOrchestrationReport {
            fleet_name: name.clone(),
            members: member_states,
            coordination_mode,
            steps_advanced,
            peer_messages,
            peer_deliveries,
            remote_relayed: 0,
            remote_failed: 0,
        });
    }

    let success = reports.iter().all(|r| {
        r.members.iter().all(|m| m.mission_state != "MissingRobot")
    });

    FleetOrchestrationResult {
        program: program_path.to_string(),
        fleets: reports,
        success,
    }
}

/// Orchestrate fleets and relay peer deliveries to registered remote fleet agents.
pub fn orchestrate_fleets_remote(
    program: &Program,
    program_path: &str,
    registry: &spanda_fleet::FleetAgentRegistry,
) -> FleetOrchestrationResult {
    // Coordinate locally, then push peer mission steps to remote fleet agents.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `program_path` — source path for reporting
    // - `registry` — registered remote fleet agents by robot name
    //
    // Returns:
    // Orchestration report with remote relay counters.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = orchestrate_fleets_remote(&program, "fleet.sd", &registry);

    let mut result = orchestrate_fleets(program, program_path);
    let mut success = result.success;
    for fleet in &mut result.fleets {
        let (relayed, failed) =
            spanda_fleet::relay_peer_deliveries(&fleet.peer_deliveries, registry);
        fleet.remote_relayed = relayed;
        fleet.remote_failed = failed;
        if relayed > 0 {
            fleet.coordination_mode = "distributed_peer_mesh".into();
        }
        if failed > 0 {
            success = false;
        }
    }
    result.success = success;
    result
}

/// Orchestrate fleets and relay peer deliveries through a mesh coordinator.
pub fn orchestrate_fleets_mesh(
    program: &Program,
    program_path: &str,
    mesh_url: &str,
    token: Option<&str>,
) -> FleetOrchestrationResult {
    // Coordinate locally, then push peer mission steps to a fleet mesh coordinator.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `program_path` — source path for reporting
    // - `mesh_url` — mesh coordinator base URL
    // - `token` — optional bearer token for the mesh coordinator
    //
    // Returns:
    // Orchestration report with remote relay counters.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = orchestrate_fleets_mesh(&program, "fleet.sd", "http://mesh:8767", None);

    let mut result = orchestrate_fleets(program, program_path);
    let mut success = result.success;
    for fleet in &mut result.fleets {
        if fleet.peer_deliveries.is_empty() {
            continue;
        }
        match spanda_fleet::relay_deliveries_via_mesh(
            mesh_url,
            &fleet.peer_deliveries,
            token,
        ) {
            Ok(resp) => {
                fleet.remote_relayed = resp.relayed;
                fleet.remote_failed = resp.failed;
                if resp.relayed > 0 {
                    fleet.coordination_mode = "distributed_peer_mesh".into();
                }
                if resp.failed > 0 {
                    success = false;
                }
            }
            Err(_) => {
                fleet.remote_failed = fleet.peer_deliveries.len() as u32;
                success = false;
            }
        }
    }
    result.success = success;
    result
}
