//! Static dependency graph construction and rendering for Spanda programs.
//!
pub mod build;
pub mod digital_thread;
pub mod format;

pub use build::{build_dependency_graph, DependencyGraph, GraphEdge, GraphNode, GraphNodeKind};
pub use digital_thread::{
    query_digital_thread, DigitalThreadDeviceLink, DigitalThreadQuery, DigitalThreadReport,
    LifecyclePhase, LifecycleRow,
};
pub use format::{format_dependency_graph, GraphFormat};
