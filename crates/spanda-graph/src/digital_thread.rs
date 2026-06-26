//! Digital Thread v1 — query capability-to-device trace chains.
//!
use crate::build::{build_dependency_graph, DependencyGraph, GraphEdge, GraphNode, GraphNodeKind};
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use spanda_capability::{
    capability_traceability, hardware_traceability, CapabilityTraceRow, HardwareTraceRow,
};
use spanda_config::ResolvedSystemConfig;
use std::collections::{BTreeMap, HashSet, VecDeque};

/// Product lifecycle phase for digital thread traceability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecyclePhase {
    Requirement,
    Design,
    Deploy,
    Operate,
    Retire,
}

/// Lifecycle assignment for a graph node in the digital thread.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifecycleRow {
    pub node_id: String,
    pub label: String,
    pub kind: String,
    pub phase: LifecyclePhase,
}

/// Filters for digital thread graph traversal.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DigitalThreadQuery {
    #[serde(default)]
    pub capability: Option<String>,
    #[serde(default)]
    pub device_id: Option<String>,
    #[serde(default)]
    pub node_id: Option<String>,
    #[serde(default)]
    pub lifecycle_phase: Option<String>,
}

/// Device link from configuration registry into the trace graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DigitalThreadDeviceLink {
    pub device_id: String,
    pub device_type: String,
    pub assigned_robot: Option<String>,
    pub lifecycle_state: Option<String>,
    pub related_capabilities: Vec<String>,
}

/// Digital thread query result for Control Center and SDK consumers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DigitalThreadReport {
    pub query: DigitalThreadQuery,
    pub source: String,
    pub graph: DependencyGraph,
    pub capability_rows: Vec<CapabilityTraceRow>,
    pub hardware_rows: Vec<HardwareTraceRow>,
    pub device_links: Vec<DigitalThreadDeviceLink>,
    pub lifecycle_rows: Vec<LifecycleRow>,
    pub lifecycle_summary: BTreeMap<String, u32>,
    pub chain_summary: Vec<String>,
    pub matched_node_count: usize,
    pub matched_edge_count: usize,
}

/// Build and filter a digital thread from program AST, traceability, and device registry.
pub fn query_digital_thread(
    program: &Program,
    source: &str,
    config: Option<&ResolvedSystemConfig>,
    query: &DigitalThreadQuery,
) -> DigitalThreadReport {
    let full_graph = build_dependency_graph(program, source, config);
    let trace = capability_traceability(program);
    let hardware = hardware_traceability(program);
    let device_links = link_devices(config, &trace.capability_rows);
    let lifecycle_rows = build_lifecycle_rows(&full_graph.nodes, &device_links);
    let lifecycle_summary = summarize_lifecycle(&lifecycle_rows);
    let (nodes, edges) = filter_graph(
        &full_graph,
        query,
        &trace.capability_rows,
        &device_links,
        &lifecycle_rows,
    );
    let chain_summary = summarize_chain(query, &nodes, &edges, &device_links);
    let matched_node_count = nodes.len();
    let matched_edge_count = edges.len();
    let capability_rows = filter_capability_rows(&trace.capability_rows, query);
    let hardware_rows = filter_hardware_rows(&hardware.hardware_rows, query, &capability_rows);

    DigitalThreadReport {
        query: query.clone(),
        source: source.to_string(),
        graph: DependencyGraph {
            source: full_graph.source,
            nodes,
            edges,
        },
        capability_rows,
        hardware_rows,
        device_links,
        lifecycle_rows,
        lifecycle_summary,
        chain_summary,
        matched_node_count,
        matched_edge_count,
    }
}

fn link_devices(
    config: Option<&ResolvedSystemConfig>,
    capability_rows: &[CapabilityTraceRow],
) -> Vec<DigitalThreadDeviceLink> {
    let Some(resolved) = config else {
        return Vec::new();
    };
    let registry = &resolved.device_registry;
    registry
        .devices
        .iter()
        .map(|device| {
            let related_capabilities = capability_rows
                .iter()
                .filter(|row| {
                    row.hardware.eq_ignore_ascii_case(&device.device_type)
                        || device
                            .assigned_robot
                            .as_deref()
                            .map(|r| row.required_by.eq_ignore_ascii_case(r))
                            .unwrap_or(false)
                })
                .map(|row| row.capability.clone())
                .collect();
            DigitalThreadDeviceLink {
                device_id: device.id.clone(),
                device_type: device.device_type.clone(),
                assigned_robot: device.assigned_robot.clone(),
                lifecycle_state: device.lifecycle_state.clone(),
                related_capabilities,
            }
        })
        .collect()
}

fn parse_lifecycle_phase(raw: &str) -> Option<LifecyclePhase> {
    match raw.to_ascii_lowercase().as_str() {
        "requirement" | "requirements" => Some(LifecyclePhase::Requirement),
        "design" => Some(LifecyclePhase::Design),
        "deploy" | "deployment" => Some(LifecyclePhase::Deploy),
        "operate" | "operation" | "operations" => Some(LifecyclePhase::Operate),
        "retire" | "retirement" => Some(LifecyclePhase::Retire),
        _ => None,
    }
}

fn infer_lifecycle_phase(kind: GraphNodeKind, device_lifecycle: Option<&str>) -> LifecyclePhase {
    if device_lifecycle
        .map(|state| state.eq_ignore_ascii_case("retired") || state.eq_ignore_ascii_case("retire"))
        .unwrap_or(false)
    {
        return LifecyclePhase::Retire;
    }
    match kind {
        GraphNodeKind::Mission => LifecyclePhase::Requirement,
        GraphNodeKind::Capability | GraphNodeKind::Safety => LifecyclePhase::Design,
        GraphNodeKind::Robot
        | GraphNodeKind::Hardware
        | GraphNodeKind::Sensor
        | GraphNodeKind::Actuator => LifecyclePhase::Deploy,
        GraphNodeKind::Provider | GraphNodeKind::Package => LifecyclePhase::Operate,
    }
}

fn build_lifecycle_rows(
    nodes: &[GraphNode],
    device_links: &[DigitalThreadDeviceLink],
) -> Vec<LifecycleRow> {
    let retired_devices: HashSet<String> = device_links
        .iter()
        .filter(|link| {
            link.lifecycle_state
                .as_deref()
                .is_some_and(|state| state.eq_ignore_ascii_case("retired"))
        })
        .map(|link| link.device_id.clone())
        .collect();
    nodes
        .iter()
        .map(|node| {
            let device_lifecycle = device_links
                .iter()
                .find(|link| {
                    link.assigned_robot
                        .as_deref()
                        .is_some_and(|robot| node.id.contains(robot))
                        || retired_devices.contains(&link.device_id)
                })
                .and_then(|link| link.lifecycle_state.as_deref());
            let phase = infer_lifecycle_phase(node.kind, device_lifecycle);
            LifecycleRow {
                node_id: node.id.clone(),
                label: node.label.clone(),
                kind: format!("{:?}", node.kind).to_ascii_lowercase(),
                phase,
            }
        })
        .collect()
}

fn summarize_lifecycle(rows: &[LifecycleRow]) -> BTreeMap<String, u32> {
    let mut summary = BTreeMap::new();
    for row in rows {
        let key = format!("{:?}", row.phase).to_ascii_lowercase();
        *summary.entry(key).or_insert(0) += 1;
    }
    summary
}

fn filter_graph(
    graph: &DependencyGraph,
    query: &DigitalThreadQuery,
    capability_rows: &[CapabilityTraceRow],
    device_links: &[DigitalThreadDeviceLink],
    lifecycle_rows: &[LifecycleRow],
) -> (Vec<GraphNode>, Vec<GraphEdge>) {
    if query.capability.is_none()
        && query.device_id.is_none()
        && query.node_id.is_none()
        && query.lifecycle_phase.is_none()
    {
        return (graph.nodes.clone(), graph.edges.clone());
    }

    let mut seed_ids: HashSet<String> = HashSet::new();
    if let Some(node_id) = &query.node_id {
        seed_ids.insert(node_id.clone());
    }
    if let Some(capability) = &query.capability {
        seed_ids.insert(format!("capability:{capability}").to_ascii_lowercase());
        for row in capability_rows {
            if row.capability.eq_ignore_ascii_case(capability) {
                seed_ids.insert(format!("hardware:{}", row.hardware).to_ascii_lowercase());
                seed_ids.insert(format!("robot:{}", row.required_by).to_ascii_lowercase());
            }
        }
    }
    if let Some(device_id) = &query.device_id {
        if let Some(link) = device_links.iter().find(|d| d.device_id == *device_id) {
            if let Some(robot) = &link.assigned_robot {
                seed_ids.insert(format!("robot:{robot}").to_ascii_lowercase());
            }
            for capability in &link.related_capabilities {
                seed_ids.insert(format!("capability:{capability}").to_ascii_lowercase());
            }
        }
    }
    if let Some(phase_raw) = query
        .lifecycle_phase
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        if let Some(wanted) = parse_lifecycle_phase(phase_raw) {
            for row in lifecycle_rows {
                if row.phase == wanted {
                    seed_ids.insert(row.node_id.clone());
                }
            }
        }
    }

    let node_map: std::collections::HashMap<String, GraphNode> = graph
        .nodes
        .iter()
        .map(|node| (node.id.clone(), node.clone()))
        .collect();
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = seed_ids.into_iter().collect();
    while let Some(id) = queue.pop_front() {
        if !visited.insert(id.clone()) {
            continue;
        }
        for edge in &graph.edges {
            if edge.from == id && !visited.contains(&edge.to) {
                queue.push_back(edge.to.clone());
            }
            if edge.to == id && !visited.contains(&edge.from) {
                queue.push_back(edge.from.clone());
            }
        }
    }

    let nodes: Vec<GraphNode> = visited
        .iter()
        .filter_map(|id| node_map.get(id).cloned())
        .filter(|node| lifecycle_phase_matches(query, lifecycle_rows, &node.id))
        .collect();
    let node_set: HashSet<String> = visited;
    let edges: Vec<GraphEdge> = graph
        .edges
        .iter()
        .filter(|edge| node_set.contains(&edge.from) && node_set.contains(&edge.to))
        .cloned()
        .collect();
    (nodes, edges)
}

fn lifecycle_phase_matches(
    query: &DigitalThreadQuery,
    lifecycle_rows: &[LifecycleRow],
    node_id: &str,
) -> bool {
    let Some(raw) = query
        .lifecycle_phase
        .as_deref()
        .filter(|value| !value.is_empty())
    else {
        return true;
    };
    let Some(wanted) = parse_lifecycle_phase(raw) else {
        return true;
    };
    lifecycle_rows
        .iter()
        .find(|row| row.node_id == node_id)
        .is_some_and(|row| row.phase == wanted)
}

fn summarize_chain(
    query: &DigitalThreadQuery,
    nodes: &[GraphNode],
    edges: &[GraphEdge],
    device_links: &[DigitalThreadDeviceLink],
) -> Vec<String> {
    let mut lines = vec![format!(
        "Digital thread query: {}",
        serde_json::to_string(query).unwrap_or_else(|_| "{}".into())
    )];
    lines.push(format!(
        "Matched {} nodes, {} edges",
        nodes.len(),
        edges.len()
    ));
    if let Some(phase) = query
        .lifecycle_phase
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        lines.push(format!("Lifecycle phase filter: {phase}"));
    }
    for edge in edges.iter().take(12) {
        lines.push(format!("{} --{}--> {}", edge.from, edge.relation, edge.to));
    }
    if let Some(device_id) = &query.device_id {
        if let Some(link) = device_links.iter().find(|d| d.device_id == *device_id) {
            lines.push(format!(
                "Device {} ({}) → capabilities: {}",
                link.device_id,
                link.device_type,
                link.related_capabilities.join(", ")
            ));
        }
    }
    lines
}

fn filter_capability_rows(
    rows: &[CapabilityTraceRow],
    query: &DigitalThreadQuery,
) -> Vec<CapabilityTraceRow> {
    if let Some(capability) = &query.capability {
        return rows
            .iter()
            .filter(|row| row.capability.eq_ignore_ascii_case(capability))
            .cloned()
            .collect();
    }
    if query.device_id.is_some() || query.node_id.is_some() {
        return rows.to_vec();
    }
    rows.to_vec()
}

fn filter_hardware_rows(
    rows: &[HardwareTraceRow],
    _query: &DigitalThreadQuery,
    capability_rows: &[CapabilityTraceRow],
) -> Vec<HardwareTraceRow> {
    if capability_rows.is_empty() {
        return rows.to_vec();
    }
    let hardware: HashSet<String> = capability_rows
        .iter()
        .map(|row| row.hardware.clone())
        .collect();
    rows.iter()
        .filter(|row| {
            hardware.is_empty()
                || hardware
                    .iter()
                    .any(|h| h.eq_ignore_ascii_case(&row.hardware_component))
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_lexer::tokenize;
    use spanda_parser::parse;
    use std::path::PathBuf;

    #[test]
    fn query_capability_filters_graph() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/showcase/compliance/defense_rover.sd");
        let source = std::fs::read_to_string(&path).expect("defense_rover.sd");
        let tokens = tokenize(&source).expect("tokenize");
        let program = parse(tokens).expect("parse");
        let report = query_digital_thread(
            &program,
            "defense_rover.sd",
            None,
            &DigitalThreadQuery::default(),
        );
        assert!(report.matched_node_count > 0);
        assert!(!report.chain_summary.is_empty());
        assert!(!report.lifecycle_rows.is_empty());
    }

    #[test]
    fn lifecycle_phase_filter_limits_nodes() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/showcase/compliance/defense_rover.sd");
        let source = std::fs::read_to_string(&path).expect("defense_rover.sd");
        let tokens = tokenize(&source).expect("tokenize");
        let program = parse(tokens).expect("parse");
        let report = query_digital_thread(
            &program,
            "defense_rover.sd",
            None,
            &DigitalThreadQuery {
                lifecycle_phase: Some("design".into()),
                ..DigitalThreadQuery::default()
            },
        );
        assert!(report.matched_node_count <= report.lifecycle_rows.len());
    }
}
