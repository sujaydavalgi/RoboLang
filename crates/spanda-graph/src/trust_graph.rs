//! Trust-weighted dependency graph — mission → capability → hardware → package → provider.

use crate::build::{build_dependency_graph, DependencyGraph, GraphNodeKind};
use crate::format::GraphFormat;
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use spanda_config::ResolvedSystemConfig;
use spanda_trust::{evaluate_composite_trust, CompositeTrustOptions, CompositeTrustReport};
use std::collections::{HashMap, VecDeque};

/// Trust-weighted edge between graph nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustDependency {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub trust_score: u32,
}

/// Trust path from mission stack to a dependency leaf.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustPath {
    pub start: String,
    pub end: String,
    pub nodes: Vec<String>,
    pub min_trust: u32,
}

/// Node in a trust graph with per-node score annotation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustGraphNode {
    pub id: String,
    pub label: String,
    pub kind: GraphNodeKind,
    pub trust_score: u32,
}

/// Full trust graph for a Spanda program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustGraph {
    pub source: String,
    pub composite_score: u32,
    pub composite_tier: String,
    pub nodes: Vec<TrustGraphNode>,
    pub dependencies: Vec<TrustDependency>,
    pub paths: Vec<TrustPath>,
}

/// Build a trust graph by composing dependency structure and composite trust scores.
pub fn build_trust_graph(
    program: &Program,
    source: &str,
    source_label: &str,
    config: Option<&ResolvedSystemConfig>,
) -> TrustGraph {
    let dependency = build_dependency_graph(program, source_label, config);
    let trust = evaluate_composite_trust(
        program,
        source,
        source_label,
        &CompositeTrustOptions::default(),
    );
    enrich_trust_graph(dependency, &trust)
}

/// Format a trust graph for CLI or CI output.
pub fn format_trust_graph(graph: &TrustGraph, format: GraphFormat) -> String {
    match format {
        GraphFormat::Json => serde_json::to_string_pretty(graph).unwrap_or_else(|_| "{}".into()),
        GraphFormat::Mermaid => format_trust_mermaid(graph),
        GraphFormat::Dot => format_trust_dot(graph),
        GraphFormat::Text => format_trust_text(graph),
    }
}

fn enrich_trust_graph(dependency: DependencyGraph, trust: &CompositeTrustReport) -> TrustGraph {
    let category_scores = category_score_map(trust);
    let nodes: Vec<TrustGraphNode> = dependency
        .nodes
        .iter()
        .map(|node| TrustGraphNode {
            id: node.id.clone(),
            label: node.label.clone(),
            kind: node.kind,
            trust_score: node_trust_score(node.kind, &category_scores, trust.score),
        })
        .collect();
    let node_scores: HashMap<String, u32> = nodes
        .iter()
        .map(|node| (node.id.clone(), node.trust_score))
        .collect();
    let dependencies: Vec<TrustDependency> = dependency
        .edges
        .iter()
        .map(|edge| TrustDependency {
            from: edge.from.clone(),
            to: edge.to.clone(),
            relation: edge.relation.clone(),
            trust_score: *node_scores.get(&edge.to).unwrap_or(&trust.score),
        })
        .collect();
    let paths = enumerate_trust_paths(&dependency, &node_scores);
    TrustGraph {
        source: dependency.source,
        composite_score: trust.score,
        composite_tier: trust.tier.clone(),
        nodes,
        dependencies,
        paths,
    }
}

fn category_score_map(trust: &CompositeTrustReport) -> HashMap<String, u32> {
    trust
        .categories
        .iter()
        .map(|category| (category.name.clone(), category.score))
        .collect()
}

fn node_trust_score(kind: GraphNodeKind, categories: &HashMap<String, u32>, composite: u32) -> u32 {
    let key = match kind {
        GraphNodeKind::Package => "package_trust",
        GraphNodeKind::Provider => "package_trust",
        GraphNodeKind::Hardware => "device_integrity",
        GraphNodeKind::Capability => "safety_integrity",
        GraphNodeKind::Safety => "safety_integrity",
        GraphNodeKind::Mission
        | GraphNodeKind::Robot
        | GraphNodeKind::Sensor
        | GraphNodeKind::Actuator => {
            return composite;
        }
    };
    *categories.get(key).unwrap_or(&composite)
}

fn enumerate_trust_paths(
    dependency: &DependencyGraph,
    node_scores: &HashMap<String, u32>,
) -> Vec<TrustPath> {
    let mission_ids: Vec<String> = dependency
        .nodes
        .iter()
        .filter(|node| node.kind == GraphNodeKind::Mission)
        .map(|node| node.id.clone())
        .collect();
    let leaf_ids: Vec<String> = dependency
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, GraphNodeKind::Package | GraphNodeKind::Provider))
        .map(|node| node.id.clone())
        .collect();
    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
    for edge in &dependency.edges {
        adjacency
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
    }

    let mut paths = Vec::new();
    for start in mission_ids {
        for end in &leaf_ids {
            if let Some(path) = bfs_path(&start, end, &adjacency) {
                let min_trust = path
                    .iter()
                    .filter_map(|id| node_scores.get(id))
                    .copied()
                    .min()
                    .unwrap_or(0);
                paths.push(TrustPath {
                    start: start.clone(),
                    end: end.clone(),
                    nodes: path,
                    min_trust,
                });
            }
        }
    }
    paths
}

fn bfs_path(
    start: &str,
    end: &str,
    adjacency: &HashMap<String, Vec<String>>,
) -> Option<Vec<String>> {
    let mut queue = VecDeque::from([(start.to_string(), vec![start.to_string()])]);
    while let Some((current, path)) = queue.pop_front() {
        if current == end {
            return Some(path);
        }
        if let Some(next_nodes) = adjacency.get(&current) {
            for next in next_nodes {
                if path.contains(next) {
                    continue;
                }
                let mut extended = path.clone();
                extended.push(next.clone());
                queue.push_back((next.clone(), extended));
            }
        }
    }
    None
}

fn format_trust_text(graph: &TrustGraph) -> String {
    let mut lines = vec![
        format!("Trust graph: {}", graph.source),
        format!(
            "Composite trust: {}/100 ({})",
            graph.composite_score, graph.composite_tier
        ),
        String::new(),
    ];
    for path in &graph.paths {
        lines.push(format!(
            "Path {} → {} (min trust {})",
            path.start, path.end, path.min_trust
        ));
    }
    if !graph.paths.is_empty() {
        lines.push(String::new());
    }
    for dependency in &graph.dependencies {
        lines.push(format!(
            "  {} --[{}/100]--> {} ({})",
            dependency.from, dependency.trust_score, dependency.to, dependency.relation
        ));
    }
    lines.join("\n")
}

fn format_trust_mermaid(graph: &TrustGraph) -> String {
    let mut out = String::from("flowchart TD\n");
    for node in &graph.nodes {
        out.push_str(&format!(
            "  {}[\"{} ({}/100)\"]\n",
            mermaid_id(&node.id),
            escape_label(&node.label),
            node.trust_score
        ));
    }
    for edge in &graph.dependencies {
        out.push_str(&format!(
            "  {} -->|{} {}| {}\n",
            mermaid_id(&edge.from),
            escape_label(&edge.relation),
            edge.trust_score,
            mermaid_id(&edge.to)
        ));
    }
    out
}

fn format_trust_dot(graph: &TrustGraph) -> String {
    let mut out = String::from("digraph trust {\n  rankdir=TB;\n");
    for node in &graph.nodes {
        out.push_str(&format!(
            "  \"{}\" [label=\"{}\\n{}/100\"];\n",
            dot_id(&node.id),
            escape_label(&node.label),
            node.trust_score
        ));
    }
    for edge in &graph.dependencies {
        out.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{} {}\"];\n",
            dot_id(&edge.from),
            dot_id(&edge.to),
            escape_label(&edge.relation),
            edge.trust_score
        ));
    }
    out.push_str("}\n");
    out
}

fn mermaid_id(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn dot_id(id: &str) -> String {
    id.replace('"', "\\\"")
}

fn escape_label(label: &str) -> String {
    label.replace('"', "'")
}
