//! Static dependency graph construction and rendering for Spanda programs.
//!
pub mod build;
pub mod digital_thread;
pub mod entity_alignment;
pub mod format;
pub mod trust_graph;

pub use build::{build_dependency_graph, DependencyGraph, GraphEdge, GraphNode, GraphNodeKind};
pub use digital_thread::{
    query_digital_thread, DigitalThreadDeviceLink, DigitalThreadQuery, DigitalThreadReport,
    LifecyclePhase, LifecycleRow,
};
pub use entity_alignment::{
    annotate_entity_ids, build_alignment_context, graph_node_id, program_graph_entity_edges,
    resolve_entity_id, EntityAlignmentContext, ProgramGraphEntityEdge,
};
pub use format::{format_dependency_graph, GraphFormat};
pub use trust_graph::{
    build_trust_graph, format_trust_graph, TrustDependency, TrustGraph, TrustGraphNode, TrustPath,
};
