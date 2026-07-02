//! Decision audit trail parsing and reporting for Spanda mission traces.
//!
//! Extended with distributed decision architecture (brain / spinal cord / reflex model).

mod authority;
mod conflict;
mod consensus;
mod diagnostics;
mod engine;
mod escalation;
mod offline;
mod policy_cache;
mod emit;
mod report;
mod runtime_bridge;
mod security;
mod simulate;
mod trace;
mod trees;
mod types;

pub use authority::{
    default_safety_boundaries, entity_may_decide_locally, extract_decision_authorities,
    validate_against_policy,
};
pub use conflict::{resolve_conflict, CompetingDecision, ConflictResolution};
pub use consensus::{resolve_consensus, ConsensusResult, ConsensusStrategy, ConsensusVote};
pub use diagnostics::collect_decision_diagnostics;
pub use engine::{
    evaluate_distributed_decisions, format_distributed_report, DecisionContext,
    DistributedDecisionReport,
};
pub use emit::{decision_trace_enabled, v3_decision_payload};
pub use escalation::{approve_escalation, build_escalation_chain, EscalationReason};
pub use offline::{
    extract_offline_policies, offline_to_decision_policy, validate_offline_action,
    OfflinePolicySpec,
};
pub use policy_cache::{build_policy_cache, LocalPolicyCache};
pub use runtime_bridge::DecisionBackedRuntime;
pub use report::{
    format_decision_audit, format_decision_explanations, DecisionAuditReport, DecisionChain,
    DecisionEvidence, DecisionRecord, DecisionTimeline,
};
pub use security::{
    security_audit, simulate_attack, threat_model_summary, validate_security_envelope,
    AttackScenario, SecurityAuditFinding,
};
pub use simulate::{
    format_simulation_report, simulate_distributed_decisions, SimulationOptions, SimulationReport,
};
pub use trace::{audit_decisions_from_trace, explain_decisions_from_trace};
pub use trees::{
    evaluate_tree, extract_decision_trees, tree_hash, DecisionTreeResult, DecisionTreeSpec,
};
pub use types::{
    DecisionAuthority, DecisionBoundary, DecisionDelegation, DecisionEscalation, DecisionLayer,
    DecisionPolicy, DecisionScope, DecisionSecurityEnvelope, DecisionType,
    DistributedDecisionRecord, CONFLICT_PRECEDENCE,
};
