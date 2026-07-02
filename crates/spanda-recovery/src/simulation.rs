//! Recovery simulation — plan, simulate, dry-run, and validate without side effects.
//!
use crate::decision::decide_recovery;
use crate::evidence::generate_evidence;
use crate::graph::{build_recovery_graph, enrich_plan_with_impact};
use crate::learning::{compute_metrics, load_knowledge, RecoveryHistoryStore};
use crate::playbook::{find_playbook, load_playbooks, match_playbooks};
use crate::policy::load_recovery_policies;
use crate::predictive::scan_predictive_indicators;
use crate::types::{
    OrchestratedRecoveryPlan, OrchestratorRecoveryReport, RecoveryOrchestratorRequest,
    RecoveryPlaybook, RecoverySimulationMode,
};
use crate::validation::validate_recovery;
use spanda_assurance::recovery::{
    evaluate_recovery, simulate_failure_recovery, RecoveryContext, RecoveryPlanner,
};
use spanda_ast::nodes::Program;
use spanda_config::entity::EntityRegistry;
use spanda_config::resolved::ResolvedSystemConfig;
use spanda_runtime::recovery_primitives::classify_failure;
use spanda_runtime::recovery_types::RecoveryLevel;

/// Run recovery simulation in the requested mode.
pub fn run_simulation(
    program: &Program,
    registry: &EntityRegistry,
    resolved: Option<&ResolvedSystemConfig>,
    request: &RecoveryOrchestratorRequest,
    history: &RecoveryHistoryStore,
    telemetry: Option<&serde_json::Value>,
    plugins: Option<&crate::plugin::RecoveryPluginRegistry>,
) -> OrchestratorRecoveryReport {
    // Run recovery simulation in the requested mode.
    //
    // Parameters:
    // - `program` — Spanda program
    // - `registry` — entity registry
    // - `resolved` — optional resolved config
    // - `request` — orchestrator request
    // - `history` — recovery history for metrics
    // - `telemetry` — optional telemetry for predictive scan
    //
    // Returns:
    // Orchestrator recovery report.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = run_simulation(&program, &registry, resolved, &request, &history, None);

    let policies = resolved
        .map(|c| load_recovery_policies(c, registry))
        .unwrap_or_default();
    let mut playbooks = load_playbooks(resolved);
    if let Some(registry) = plugins {
        crate::playbook::merge_plugin_playbooks(&mut playbooks, registry);
    }
    let knowledge = load_knowledge(program);
    let graph = build_recovery_graph(registry, request.entity_id.as_deref());
    let predictive = scan_predictive_indicators(registry, telemetry);

    let failures: Vec<(String, String)> = if let Some(ref failure) = request.failure {
        let entity_id = request.entity_id.clone().unwrap_or_else(|| "system".into());
        vec![(entity_id, failure.clone())]
    } else if let Some(ref entity_id) = request.entity_id {
        vec![(entity_id.clone(), format!("{entity_id}_degraded"))]
    } else {
        registry
            .list()
            .iter()
            .take(5)
            .map(|e| (e.id.clone(), format!("{}_health_check", e.id)))
            .collect()
    };

    let mut plans = Vec::new();
    for (entity_id, failure) in failures {
        if let Some(entity) = registry.get(&entity_id) {
            let classification = classify_failure(&failure);
            let decision =
                decide_recovery(entity, &failure, &policies, registry, Some(classification));

            let matched_playbook: Option<RecoveryPlaybook> =
                if let Some(name) = request.playbook.as_ref() {
                    find_playbook(&playbooks, name).map(|p| p.clone())
                } else {
                    match_playbooks(&playbooks, &failure)
                        .first()
                        .map(|p| (*p).clone())
                };

            let strategies: Vec<_> = if let Some(ref pb) = matched_playbook {
                pb.steps.iter().map(|s| s.strategy.clone()).collect()
            } else {
                let recommended = plugins
                    .and_then(|registry| {
                        registry.resolve_strategy(decision.recommended_strategy.label())
                    })
                    .unwrap_or(decision.recommended_strategy.clone());
                vec![recommended]
            };

            let ctx = RecoveryContext {
                issue: failure.clone(),
                diagnosis: None,
                classification: Some(classification),
                level: RecoveryLevel::Level3AutomaticWithValidation,
            };
            let legacy_plan = RecoveryPlanner::plan(program, &ctx);

            let mut plan = OrchestratedRecoveryPlan {
                plan_id: format!("plan-{}-{}", entity_id, failure.replace(' ', "_")),
                entity_id: entity_id.clone(),
                entity_kind: entity.entity_type.clone(),
                failure: failure.clone(),
                classification,
                diagnosis: legacy_plan.diagnosis.clone(),
                strategies,
                escalation_level: decision.recommended_level,
                decision,
                legacy_plan: Some(legacy_plan.clone()),
                playbook: matched_playbook.as_ref().map(|p| p.name.clone()),
                upstream_impact: Vec::new(),
                downstream_impact: Vec::new(),
                estimated_duration_secs: legacy_plan.actions.len() as u64 * 30,
                risk: legacy_plan.risk.clone(),
            };
            enrich_plan_with_impact(&mut plan, &graph);
            plans.push(plan);
        }
    }

    let legacy_report = if let Some(ref failure) = request.failure {
        Some(simulate_failure_recovery(program, failure, None))
    } else {
        Some(evaluate_recovery(program, None, None))
    };

    let mut evidence = Vec::new();
    if request.mode != RecoverySimulationMode::Plan {
        for plan in &plans {
            if let Some(ref legacy) = plan.legacy_plan {
                let policy = policies.iter().find(|p| p.entity_id == plan.entity_id);
                let rules = policy
                    .map(|p| p.validation_rules.clone())
                    .unwrap_or_default();
                let validation =
                    validate_recovery(program, legacy, registry, &plan.entity_id, &rules);
                let duration = plan.estimated_duration_secs;
                evidence.push(generate_evidence(plan, &validation, None, duration));
            }
        }
    }

    let metrics = compute_metrics(history, &knowledge);
    let passed = plans
        .iter()
        .all(|p| p.decision.can_recover && p.decision.is_safe)
        && evidence
            .iter()
            .all(|e| e.status != spanda_runtime::recovery_types::RecoveryStatus::Failed);

    OrchestratorRecoveryReport {
        plans,
        evidence,
        metrics,
        predictive_indicators: predictive,
        legacy_report,
        simulation_mode: request.mode,
        passed,
    }
}
