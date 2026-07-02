//! Injectable distributed decision runtime boundary for the interpreter.

use spanda_ast::nodes::Program;
use std::collections::HashMap;
use std::sync::Arc;

/// Result of evaluating a local decision tree at runtime.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DecisionTreeEvalResult {
    pub tree_name: String,
    pub layer: String,
    pub condition_matched: String,
    pub actions: Vec<String>,
    pub tree_hash: String,
}

/// Fleet consensus evaluation result for mesh coordination traces.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FleetConsensusEvalResult {
    pub strategy: String,
    pub selected_action: String,
    pub quorum_met: bool,
    pub vote_count: usize,
}

/// Extension points for live distributed decision evaluation at runtime.
pub trait DecisionRuntime: Send + Sync {
    /// Evaluate decision trees against current signal values.
    fn evaluate_trees(
        &self,
        program: &Program,
        signals: &HashMap<String, bool>,
    ) -> Vec<DecisionTreeEvalResult>;

    /// Resolve fleet consensus from member votes (trust-weighted by default).
    fn resolve_fleet_consensus(
        &self,
        votes: &[(String, String, f64)],
        quorum_fraction: f64,
    ) -> FleetConsensusEvalResult;
}

/// Built-in no-op decision runtime when spanda-decision bridge is not injected.
#[derive(Debug, Default, Clone, Copy)]
pub struct BuiltinDecisionRuntime;

impl DecisionRuntime for BuiltinDecisionRuntime {
    fn evaluate_trees(
        &self,
        _program: &Program,
        _signals: &HashMap<String, bool>,
    ) -> Vec<DecisionTreeEvalResult> {
        Vec::new()
    }

    fn resolve_fleet_consensus(
        &self,
        votes: &[(String, String, f64)],
        quorum_fraction: f64,
    ) -> FleetConsensusEvalResult {
        let selected = votes
            .first()
            .map(|(_, action, _)| action.clone())
            .unwrap_or_else(|| "no_action".into());
        FleetConsensusEvalResult {
            strategy: "coordinator_decision".into(),
            selected_action: selected,
            quorum_met: !votes.is_empty() && quorum_fraction <= 1.0,
            vote_count: votes.len(),
        }
    }
}

static PLATFORM_DECISION_RUNTIME: std::sync::OnceLock<SharedDecisionRuntime> =
    std::sync::OnceLock::new();

/// Inject a real decision runtime from spanda-decision bridge.
pub fn set_platform_decision_runtime(runtime: SharedDecisionRuntime) {
    let _ = PLATFORM_DECISION_RUNTIME.set(runtime);
}

/// Active platform decision runtime, or built-in default.
pub fn platform_decision_runtime() -> SharedDecisionRuntime {
    PLATFORM_DECISION_RUNTIME
        .get()
        .cloned()
        .unwrap_or_else(default_decision_runtime)
}

/// Shared decision runtime handle passed through run options.
pub type SharedDecisionRuntime = Arc<dyn DecisionRuntime>;

/// Default built-in decision runtime.
pub fn default_decision_runtime() -> SharedDecisionRuntime {
    Arc::new(BuiltinDecisionRuntime)
}
