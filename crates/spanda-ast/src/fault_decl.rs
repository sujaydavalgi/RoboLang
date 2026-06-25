//! Runtime fault detection declarations (heartbeat, memory watch, resource watch, restart policy).
//!
use crate::nodes::Span;
use serde::{Deserialize, Serialize};

/// Heartbeat monitoring declaration for a runtime target.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum HeartbeatDecl {
    HeartbeatDecl {
        target: String,
        interval_ms: f64,
        timeout_ms: f64,
        on_missed_actions: Vec<String>,
        span: Span,
    },
}

/// Memory leak watch declaration with growth threshold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum MemoryWatchDecl {
    MemoryWatchDecl {
        target: String,
        growth_threshold: String,
        growth_window: String,
        actions: Vec<String>,
        span: Span,
    },
}

/// Single resource pressure condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceWatchCondition {
    pub resource: String,
    pub operator: String,
    pub threshold: String,
    #[serde(default)]
    pub duration: Option<String>,
    pub span: Span,
}

/// Resource pressure watch declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ResourceWatchDecl {
    ResourceWatchDecl {
        conditions: Vec<ResourceWatchCondition>,
        span: Span,
    },
}

/// Restart loop policy declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RestartPolicyDecl {
    RestartPolicyDecl {
        target: String,
        max_restarts: u32,
        window: String,
        on_exceeded_actions: Vec<String>,
        span: Span,
    },
}

/// Program-level runtime fault trigger (`on runtime crash { ... }`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RuntimeFaultTriggerDecl {
    RuntimeFaultTriggerDecl {
        event: String,
        body: Vec<String>,
        span: Span,
    },
}
