//! Recovery graph — dependency, impact, and recovery relationship analysis.
//!
use crate::types::{OrchestratedRecoveryPlan, RecoveryEntityTarget};
use serde::{Deserialize, Serialize};
use spanda_config::entity::{
    EntityKind, EntityRegistry, EntityRelationship, EntityRelationshipKind,
};
use std::collections::{HashSet, VecDeque};

/// Node in the recovery graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryGraphNode {
    pub id: String,
    pub kind: EntityKind,
    pub display_name: String,
    pub recoverable: bool,
    pub health_status: String,
    pub depth: u32,
}

/// Edge in the recovery graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryGraphEdge {
    pub from: String,
    pub to: String,
    pub relationship: String,
    pub critical: bool,
}

/// Full recovery graph with dependency and impact views.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryGraph {
    pub nodes: Vec<RecoveryGraphNode>,
    pub dependency_edges: Vec<RecoveryGraphEdge>,
    pub impact_edges: Vec<RecoveryGraphEdge>,
    pub recovery_edges: Vec<RecoveryGraphEdge>,
    pub root_entity_id: Option<String>,
}

/// Build recovery graph from entity registry.
pub fn build_recovery_graph(registry: &EntityRegistry, root_id: Option<&str>) -> RecoveryGraph {
    // Build recovery graph from entity registry.
    //
    // Parameters:
    // - `registry` — entity registry projection
    // - `root_id` — optional root entity for subgraph focus
    //
    // Returns:
    // Recovery graph with dependency, impact, and recovery edges.
    //
    // Options:
    // None.
    //
    // Example:
    // let graph = build_recovery_graph(&registry, Some("robot-1"));

    let root = root_id.map(str::to_owned);
    let entity_ids: HashSet<String> = if let Some(ref rid) = root {
        subgraph_ids(registry, rid)
    } else {
        registry.list().into_iter().map(|e| e.id.clone()).collect()
    };

    let mut nodes = Vec::new();
    for id in &entity_ids {
        if let Some(entity) = registry.get(id) {
            let depth = if let Some(ref rid) = root {
                depth_from_root(registry, rid, id)
            } else {
                0
            };
            nodes.push(RecoveryGraphNode {
                id: entity.id.clone(),
                kind: entity.entity_type.clone(),
                display_name: entity
                    .display_name
                    .clone()
                    .or_else(|| entity.name.clone())
                    .unwrap_or_else(|| entity.id.clone()),
                recoverable: is_recoverable_kind(&entity.entity_type),
                health_status: format!("{:?}", entity.health_status),
                depth,
            });
        }
    }

    let mut dependency_edges = Vec::new();
    let mut impact_edges = Vec::new();
    let mut recovery_edges = Vec::new();

    for rel in &registry.relationships {
        if !entity_ids.contains(&rel.from_id) || !entity_ids.contains(&rel.to_id) {
            continue;
        }
        let edge = relationship_to_edge(rel);
        dependency_edges.push(edge.clone());
        impact_edges.push(RecoveryGraphEdge {
            from: edge.to.clone(),
            to: edge.from.clone(),
            relationship: format!("impacts_{}", edge.relationship),
            critical: edge.critical,
        });
        if matches!(
            rel.kind,
            EntityRelationshipKind::DependsOn
                | EntityRelationshipKind::Consumes
                | EntityRelationshipKind::Controls
        ) {
            recovery_edges.push(edge);
        }
    }

    for id in &entity_ids {
        if let Some(entity) = registry.get(id) {
            if let Some(ref parent) = entity.parent_id {
                if entity_ids.contains(parent) {
                    dependency_edges.push(RecoveryGraphEdge {
                        from: id.clone(),
                        to: parent.clone(),
                        relationship: "child_of".into(),
                        critical: false,
                    });
                    impact_edges.push(RecoveryGraphEdge {
                        from: parent.clone(),
                        to: id.clone(),
                        relationship: "parent_impact".into(),
                        critical: true,
                    });
                }
            }
        }
    }

    nodes.sort_by(|a, b| a.depth.cmp(&b.depth).then_with(|| a.id.cmp(&b.id)));

    RecoveryGraph {
        nodes,
        dependency_edges,
        impact_edges,
        recovery_edges,
        root_entity_id: root,
    }
}

/// Analyze upstream and downstream impact for a failed entity.
pub fn analyze_impact(graph: &RecoveryGraph, failed_entity_id: &str) -> (Vec<String>, Vec<String>) {
    // Analyze upstream and downstream impact for a failed entity.
    //
    // Parameters:
    // - `graph` — recovery graph
    // - `failed_entity_id` — entity that failed
    //
    // Returns:
    // Tuple of (upstream entities, downstream entities).
    //
    // Options:
    // None.
    //
    // Example:
    // let (up, down) = analyze_impact(&graph, "camera-1");

    let mut upstream = HashSet::new();
    let mut downstream = HashSet::new();

    let mut queue = VecDeque::from([(failed_entity_id.to_string(), true)]);
    let mut visited = HashSet::new();
    while let Some((id, _)) = queue.pop_front() {
        if !visited.insert(id.clone()) {
            continue;
        }
        if id != failed_entity_id {
            upstream.insert(id.clone());
        }
        for edge in &graph.dependency_edges {
            if edge.from == id {
                queue.push_back((edge.to.clone(), true));
            }
        }
    }

    queue = VecDeque::from([(failed_entity_id.to_string(), false)]);
    visited.clear();
    while let Some((id, _)) = queue.pop_front() {
        if !visited.insert(id.clone()) {
            continue;
        }
        if id != failed_entity_id {
            downstream.insert(id.clone());
        }
        for edge in &graph.dependency_edges {
            if edge.to == id {
                queue.push_back((edge.from.clone(), false));
            }
        }
    }

    let mut up: Vec<String> = upstream.into_iter().collect();
    let mut down: Vec<String> = downstream.into_iter().collect();
    up.sort();
    down.sort();
    (up, down)
}

/// Enrich orchestrated plan with graph-derived impact analysis.
pub fn enrich_plan_with_impact(plan: &mut OrchestratedRecoveryPlan, graph: &RecoveryGraph) {
    let (up, down) = analyze_impact(graph, &plan.entity_id);
    plan.upstream_impact = up;
    plan.downstream_impact = down;
}

/// List all recoverable entity targets from registry.
pub fn list_recoverable_entities(registry: &EntityRegistry) -> Vec<RecoveryEntityTarget> {
    registry
        .list()
        .iter()
        .filter(|e| is_recoverable_kind(&e.entity_type))
        .map(|e| RecoveryEntityTarget {
            id: e.id.clone(),
            kind: e.entity_type.clone(),
            display_name: e
                .display_name
                .clone()
                .or_else(|| e.name.clone())
                .unwrap_or_else(|| e.id.clone()),
        })
        .collect()
}

/// Format recovery graph as text for CLI output.
pub fn format_recovery_graph_text(graph: &RecoveryGraph) -> String {
    let mut out = String::new();
    out.push_str("Recovery Graph\n");
    out.push_str("==============\n\n");
    if let Some(ref root) = graph.root_entity_id {
        out.push_str(&format!("Root: {root}\n\n"));
    }
    out.push_str(&format!("Nodes ({}):\n", graph.nodes.len()));
    for node in &graph.nodes {
        out.push_str(&format!(
            "  [{}] {} ({:?}) depth={} recoverable={}\n",
            node.id, node.display_name, node.kind, node.depth, node.recoverable
        ));
    }
    out.push_str(&format!(
        "\nDependency edges ({}):\n",
        graph.dependency_edges.len()
    ));
    for edge in &graph.dependency_edges {
        out.push_str(&format!(
            "  {} --[{}]--> {}{}\n",
            edge.from,
            edge.relationship,
            edge.to,
            if edge.critical { " (critical)" } else { "" }
        ));
    }
    out.push_str(&format!("\nImpact edges ({}):\n", graph.impact_edges.len()));
    for edge in &graph.impact_edges {
        out.push_str(&format!(
            "  {} --[{}]--> {}\n",
            edge.from, edge.relationship, edge.to
        ));
    }
    out
}

fn is_recoverable_kind(kind: &EntityKind) -> bool {
    !matches!(kind, EntityKind::Incident | EntityKind::Hazard)
}

fn subgraph_ids(registry: &EntityRegistry, root_id: &str) -> HashSet<String> {
    let mut ids = HashSet::new();
    ids.insert(root_id.to_string());
    let mut queue = VecDeque::from([root_id.to_string()]);
    while let Some(id) = queue.pop_front() {
        for rel in registry.relationships_for(&id) {
            let other = if rel.from_id == id {
                &rel.to_id
            } else {
                &rel.from_id
            };
            if ids.insert(other.clone()) {
                queue.push_back(other.clone());
            }
        }
        if let Some(entity) = registry.get(&id) {
            if let Some(ref parent) = entity.parent_id {
                if ids.insert(parent.clone()) {
                    queue.push_back(parent.clone());
                }
            }
            for child_id in &entity.children_ids {
                if ids.insert(child_id.clone()) {
                    queue.push_back(child_id.clone());
                }
            }
        }
    }
    ids
}

fn depth_from_root(registry: &EntityRegistry, root_id: &str, entity_id: &str) -> u32 {
    if root_id == entity_id {
        return 0;
    }
    let mut queue = VecDeque::from([(root_id.to_string(), 0u32)]);
    let mut visited = HashSet::new();
    while let Some((id, depth)) = queue.pop_front() {
        if !visited.insert(id.clone()) {
            continue;
        }
        if id == entity_id {
            return depth;
        }
        for rel in registry.relationships_for(&id) {
            let other = if rel.from_id == id {
                rel.to_id.clone()
            } else {
                rel.from_id.clone()
            };
            queue.push_back((other, depth + 1));
        }
    }
    u32::MAX
}

fn relationship_to_edge(rel: &EntityRelationship) -> RecoveryGraphEdge {
    let kind = rel.kind.as_str();
    RecoveryGraphEdge {
        from: rel.from_id.clone(),
        to: rel.to_id.clone(),
        relationship: kind.to_string(),
        critical: matches!(
            rel.kind,
            EntityRelationshipKind::DependsOn | EntityRelationshipKind::Controls
        ),
    }
}
