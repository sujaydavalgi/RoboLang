//! Decision audit trail parsing and reporting for Spanda mission traces.
//!
//! Extended with distributed decision architecture (brain / spinal cord / reflex model).

mod authority;
mod conflict;
mod enforcement;
mod consensus;
mod diagnostics;
mod emit;
mod engine;
mod escalation;
mod escalation_store;
mod nonce_cache;
mod offline;
mod policy_cache;
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
pub use emit::{decision_trace_enabled, v3_decision_payload};
pub use engine::{
    evaluate_distributed_decisions, format_distributed_report, DecisionContext,
    DistributedDecisionReport,
};
pub use escalation::{approve_escalation, build_escalation_chain, EscalationReason};
pub use escalation_store::{
    approve_escalation_persisted, default_escalation_store_path, escalation_is_approved,
    load_escalation_store, register_pending_escalation, save_escalation_store, EscalationGrant,
    PendingEscalation, PersistedEscalationStore,
};
pub use nonce_cache::{
    clear_persisted_nonce_registry, default_nonce_registry_path, load_nonce_registry,
    register_persisted_nonce, save_nonce_registry, PersistedNonceRegistry,
};
pub use offline::{
    extract_offline_policies, offline_policy_signing_payload, offline_to_decision_policy,
    resolve_offline_policies, sign_offline_policy, validate_offline_action,
    validate_offline_policy_trust, verify_offline_policy_signature, OfflinePolicySpec,
};
pub use policy_cache::{
    build_policy_cache, default_policy_cache_path, load_persisted_policy_cache,
    merge_decision_trees_with_cache, merge_offline_policies_with_cache,
    save_persisted_policy_cache, LocalPolicyCache, PersistedPolicyCache,
};
pub use report::{
    format_decision_audit, format_decision_explanations, DecisionAuditReport, DecisionChain,
    DecisionEvidence, DecisionRecord, DecisionTimeline,
};
pub use runtime_bridge::{register_platform_runtime, DecisionBackedRuntime};
pub use enforcement::{
    cached_policy_must_be_signed, clear_nonce_registry, detect_policy_tampering,
    high_risk_requires_central_approval, local_action_respects_kill_switch,
    local_action_respects_safety_boundaries, offline_decision_expired, policy_hash,
    reflex_may_act_without_central, register_decision_nonce, resolve_split_brain,
    tamper_policy_for_test, untrusted_entity_may_not_takeover, validate_authority_scope,
    validate_decision_timestamp, validate_decision_trace_payload, verify_decision_tree_hash,
    NonceRegistry, TraceValidationResult, TRACE_REQUIRED_FIELDS,
};
pub use security::{
    detect_tampered_decision_trace, run_attack_simulation, security_audit, simulate_attack,
    threat_model_summary, validate_security_envelope, AttackScenario, AttackSimulationResult,
    SecurityAuditFinding,
};
pub use simulate::{
    format_simulation_report, simulate_distributed_decisions, SimulationOptions, SimulationReport,
};
pub use trace::{audit_decisions_from_trace, explain_decisions_from_trace};
pub use trees::{
    decision_tree_signing_payload, evaluate_tree, extract_decision_trees, layer_precedence_key,
    layer_str_precedence_key, resolve_decision_trees, sign_decision_tree, tree_hash,
    validate_decision_tree_trust, verify_decision_tree_signature, DecisionTreeResult,
    DecisionTreeSpec,
};
pub use types::{
    DecisionAuthority, DecisionBoundary, DecisionDelegation, DecisionEscalation, DecisionLayer,
    DecisionPolicy, DecisionScope, DecisionSecurityEnvelope, DecisionType,
    DistributedDecisionRecord, CONFLICT_PRECEDENCE,
};
