//! Map dependency-graph nodes to unified entity IDs and program traceability edges.
//!
use crate::build::{DependencyGraph, GraphEdge, GraphNode, GraphNodeKind};
use serde::{Deserialize, Serialize};
use spanda_config::{mission_entity_id, ResolvedSystemConfig};
use std::collections::{HashMap, HashSet};

/// Context for resolving graph node keys to entity registry IDs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityAlignmentContext {
    pub device_ids: HashSet<String>,
    pub robot_names: HashSet<String>,
    pub mission_robot: HashMap<String, String>,
}

/// Stable graph node id (`{kind}:{key}` lowercase) used by [`DependencyGraph`].
pub fn graph_node_id(kind: GraphNodeKind, key: &str) -> String {
    format!("{kind:?}:{key}").to_ascii_lowercase()
}

/// Build alignment context from a dependency graph and optional resolved configuration.
pub fn build_alignment_context(
    graph: &DependencyGraph,
    config: Option<&ResolvedSystemConfig>,
) -> EntityAlignmentContext {
    let mut ctx = EntityAlignmentContext::default();
    if let Some(resolved) = config {
        for device in &resolved.device_registry.devices {
            ctx.device_ids.insert(device.id.clone());
        }
        if let Some(tree) = resolved.device_tree.fleet.as_ref() {
            for robot in &tree.robots {
                ctx.robot_names.insert(robot.id.clone());
            }
        }
    }
    for edge in &graph.edges {
        if edge.relation != "runs" {
            continue;
        }
        let Some(robot_node) = graph.nodes.iter().find(|node| node.id == edge.from) else {
            continue;
        };
        if robot_node.kind != GraphNodeKind::Robot {
            continue;
        }
        if ctx.robot_names.is_empty() || ctx.robot_names.contains(&robot_node.label) {
            ctx.mission_robot
                .insert(edge.to.clone(), robot_node.label.clone());
        }
    }
    for node in &graph.nodes {
        if node.kind == GraphNodeKind::Robot {
            ctx.robot_names.insert(node.label.clone());
        }
    }
    ctx
}

/// Annotate each graph node with `entity_id` metadata when resolvable.
pub fn annotate_entity_ids(graph: &mut DependencyGraph, ctx: &EntityAlignmentContext) {
    for node in &mut graph.nodes {
        if let Some(entity_id) = resolve_entity_id(node, ctx) {
            node.metadata.insert("entity_id".into(), entity_id);
        }
    }
}

/// Resolve a graph node to a unified entity registry id, when possible.
pub fn resolve_entity_id(node: &GraphNode, ctx: &EntityAlignmentContext) -> Option<String> {
    let key = graph_node_key(&node.id)?;
    match node.kind {
        GraphNodeKind::Robot => Some(node.label.clone()),
        GraphNodeKind::Mission => {
            let robot = ctx.mission_robot.get(&node.id)?;
            Some(mission_entity_id(robot, &node.label))
        }
        GraphNodeKind::Provider | GraphNodeKind::Package => Some(key.to_string()),
        GraphNodeKind::Hardware | GraphNodeKind::Sensor | GraphNodeKind::Actuator => {
            if ctx.device_ids.contains(key) {
                Some(key.to_string())
            } else if ctx.device_ids.contains(&node.label) {
                Some(node.label.clone())
            } else {
                None
            }
        }
        GraphNodeKind::Capability | GraphNodeKind::Safety => None,
    }
}

/// Program-graph edge projected to entity ids when both endpoints resolve.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgramGraphEntityEdge {
    pub from_entity_id: String,
    pub to_entity_id: String,
    pub relation: String,
}

/// Collect entity-aligned edges from an annotated dependency graph.
pub fn program_graph_entity_edges(
    graph: &DependencyGraph,
    ctx: &EntityAlignmentContext,
) -> Vec<ProgramGraphEntityEdge> {
    let entity_for = |node_id: &str| {
        graph
            .nodes
            .iter()
            .find(|node| node.id == node_id)
            .and_then(|node| resolve_entity_id(node, ctx))
    };
    graph
        .edges
        .iter()
        .filter_map(|edge: &GraphEdge| {
            let from = entity_for(&edge.from)?;
            let to = entity_for(&edge.to)?;
            Some(ProgramGraphEntityEdge {
                from_entity_id: from,
                to_entity_id: to,
                relation: edge.relation.clone(),
            })
        })
        .collect()
}

fn graph_node_key(node_id: &str) -> Option<&str> {
    node_id.split_once(':').map(|(_, key)| key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::build_dependency_graph;
    use spanda_lexer::tokenize;
    use spanda_parser::parse;

    fn parse_source(source: &str) -> spanda_ast::nodes::Program {
        let tokens = tokenize(source).expect("tokenize");
        parse(tokens).expect("parse")
    }

    #[test]
    fn resolves_robot_and_mission_entity_ids() {
        let source = r#"
robot Rover {
  exposes capabilities [ gps_navigation ];
  mission Patrol { requires capabilities [ gps_navigation ]; patrol; }
  behavior patrol() {}
}
"#;
        let program = parse_source(source);
        let mut graph = build_dependency_graph(&program, "rover.sd", None);
        let ctx = build_alignment_context(&graph, None);
        annotate_entity_ids(&mut graph, &ctx);
        let robot = graph
            .nodes
            .iter()
            .find(|node| node.kind == GraphNodeKind::Robot)
            .expect("robot node");
        assert_eq!(
            robot.metadata.get("entity_id").map(String::as_str),
            Some("Rover")
        );
        let mission = graph
            .nodes
            .iter()
            .find(|node| node.kind == GraphNodeKind::Mission)
            .expect("mission node");
        assert_eq!(
            mission.metadata.get("entity_id").map(String::as_str),
            Some("mission:Rover:Patrol")
        );
    }
}
