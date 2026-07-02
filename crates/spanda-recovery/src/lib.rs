//! Recovery Orchestrator — platform-wide recovery planning, coordination, validation, and audit.
//!
//! Central intelligence for recovery across the Spanda autonomous ecosystem.
//! Integrates with assurance, readiness, trust, entity model, diagnosis,
//! mission continuity, and fleet recovery without replacing existing APIs.
//!
pub mod decision;
pub mod evidence;
pub mod format;
pub mod graph;
pub mod learning;
pub mod orchestrator;
pub mod playbook;
pub mod plugin;
pub mod policy;
pub mod predictive;
pub mod simulation;
pub mod types;
pub mod validation;

pub use decision::decide_recovery;
pub use format::{
    format_decision, format_graph, format_history, format_metrics, format_orchestrator_report,
    format_playbooks,
};
pub use graph::{
    analyze_impact, build_recovery_graph, enrich_plan_with_impact, list_recoverable_entities,
    RecoveryGraph, RecoveryGraphEdge, RecoveryGraphNode,
};
pub use learning::{compute_metrics, load_knowledge, recommend_strategy, RecoveryHistoryStore};
pub use orchestrator::RecoveryOrchestrator;
pub use playbook::{default_playbooks, find_playbook, load_playbooks, match_playbooks};
pub use plugin::{
    RecoveryPluginRegistry, PLUGIN_KIND_DASHBOARD, PLUGIN_KIND_INTEGRATION, PLUGIN_KIND_PLAYBOOK,
    PLUGIN_KIND_POLICY, PLUGIN_KIND_REPORT, PLUGIN_KIND_SIMULATOR, PLUGIN_KIND_STRATEGY,
    PLUGIN_KIND_VALIDATOR,
};
pub use policy::{evaluate_policy, load_recovery_policies, policy_for_entity};
pub use predictive::{scan_predictive_indicators, should_trigger_preventative};
pub use simulation::run_simulation;
pub use types::*;
pub use validation::validate_recovery;
