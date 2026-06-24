//! Fleet remote relay, agents, and mesh coordination extracted from Spanda core.
//!
pub mod agent;
pub mod mesh;
pub mod orchestrator;
pub mod continuity_agent;
pub mod continuity_mesh;
pub mod recovery_agent;
pub mod recovery_mesh;
pub mod telemetry_mesh;
pub mod remote;
pub mod swarm_coordinator;
pub mod swarm_continuity;
mod types;

pub use agent::*;
pub use mesh::*;
pub use orchestrator::*;
pub use continuity_mesh::*;
pub use recovery_mesh::*;
pub use telemetry_mesh::*;
pub use remote::*;
pub use swarm_coordinator::*;
pub use swarm_continuity::*;
pub use types::PeerDelivery;
