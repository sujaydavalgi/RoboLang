//! Recovery Orchestrator — central platform recovery intelligence.
//!
use crate::decision::decide_recovery;
use crate::evidence::generate_evidence;
use crate::graph::{build_recovery_graph, enrich_plan_with_impact, list_recoverable_entities};
use crate::learning::{
    compute_metrics, load_knowledge, recommend_strategy, record_learning_outcome,
    RecoveryHistoryStore,
};
use crate::playbook::{load_playbooks, match_playbooks};
use crate::policy::{load_recovery_policies, policy_for_entity};
use crate::predictive::{scan_predictive_indicators, should_trigger_preventative};
use crate::simulation::run_simulation;
use crate::types::{
    EntityRecoveryPolicy, OrchestratedRecoveryPlan, OrchestratorContext,
    OrchestratorRecoveryEvidence, OrchestratorRecoveryReport, OrchestratorValidationResult,
    RecoveryOrchestratorRequest, RecoveryPlaybook, RecoverySimulationMode,
};
use crate::validation::validate_recovery;
use spanda_assurance::recovery::{
    evaluate_recovery, execute_recovery_plan, RecoveryContext, RecoveryPlanner,
};
use spanda_ast::nodes::Program;
use spanda_config::entity::EntityRegistry;
use spanda_config::resolved::ResolvedSystemConfig;
use spanda_runtime::recovery_primitives::classify_failure;
use spanda_runtime::recovery_types::{RecoveryLevel, RecoveryPlan};

/// Central Recovery Orchestrator service.
#[derive(Debug, Clone, Default)]
pub struct RecoveryOrchestrator {
    history: RecoveryHistoryStore,
    plugins: Option<crate::plugin::RecoveryPluginRegistry>,
}

impl RecoveryOrchestrator {
    /// Create a new orchestrator with empty history.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach plugin-contributed recovery extensions.
    pub fn with_plugins(mut self, plugins: crate::plugin::RecoveryPluginRegistry) -> Self {
        self.plugins = Some(plugins);
        self
    }

    fn plugin_registry(&self) -> Option<&crate::plugin::RecoveryPluginRegistry> {
        self.plugins.as_ref()
    }

    /// Plan recovery for an entity failure without execution.
    pub fn plan_recovery(
        &self,
        program: &Program,
        registry: &EntityRegistry,
        resolved: Option<&ResolvedSystemConfig>,
        request: &RecoveryOrchestratorRequest,
    ) -> OrchestratorRecoveryReport {
        // Plan recovery for an entity failure without execution.
        //
        // Parameters:
        // - `program` — Spanda program
        // - `registry` — entity registry
        // - `resolved` — optional resolved config
        // - `request` — orchestrator request
        //
        // Returns:
        // Recovery plan report.
        //
        // Options:
        // None.
        //
        // Example:
        // let report = orchestrator.plan_recovery(&program, &registry, resolved, &request);

        let mut req = request.clone();
        req.mode = RecoverySimulationMode::Plan;
        run_simulation(
            program,
            registry,
            resolved,
            &req,
            &self.history,
            None,
            self.plugin_registry(),
        )
    }

    /// Simulate recovery with expected timelines and mission impact.
    pub fn simulate_recovery(
        &self,
        program: &Program,
        registry: &EntityRegistry,
        resolved: Option<&ResolvedSystemConfig>,
        request: &RecoveryOrchestratorRequest,
        telemetry: Option<&serde_json::Value>,
    ) -> OrchestratorRecoveryReport {
        let mut req = request.clone();
        req.mode = RecoverySimulationMode::Simulate;
        run_simulation(
            program,
            registry,
            resolved,
            &req,
            &self.history,
            telemetry,
            self.plugin_registry(),
        )
    }

    /// Dry-run recovery — full pipeline without mutating runtime state.
    pub fn dry_run_recovery(
        &self,
        program: &Program,
        registry: &EntityRegistry,
        resolved: Option<&ResolvedSystemConfig>,
        request: &RecoveryOrchestratorRequest,
    ) -> OrchestratorRecoveryReport {
        let mut req = request.clone();
        req.mode = RecoverySimulationMode::DryRun;
        run_simulation(
            program,
            registry,
            resolved,
            &req,
            &self.history,
            None,
            self.plugin_registry(),
        )
    }

    /// Validate a recovery plan against all required gates.
    pub fn validate_recovery_plan(
        &self,
        program: &Program,
        registry: &EntityRegistry,
        resolved: Option<&ResolvedSystemConfig>,
        plan: &RecoveryPlan,
        entity_id: &str,
    ) -> OrchestratorValidationResult {
        let policies = resolved
            .map(|c| load_recovery_policies(c, registry))
            .unwrap_or_default();
        let rules = policy_for_entity(&policies, entity_id)
            .map(|p| p.validation_rules.clone())
            .unwrap_or_default();
        validate_recovery(program, plan, registry, entity_id, &rules)
    }

    /// Execute recovery — delegates to assurance layer, records evidence.
    pub fn execute_recovery(
        &mut self,
        program: &Program,
        registry: &EntityRegistry,
        resolved: Option<&ResolvedSystemConfig>,
        request: &RecoveryOrchestratorRequest,
        ctx: &OrchestratorContext,
    ) -> OrchestratorRecoveryReport {
        let plan_report = self.plan_recovery(program, registry, resolved, request);
        if ctx.dry_run || ctx.skip_execution {
            return plan_report;
        }

        let mut evidence = plan_report.evidence.clone();
        let mut results = Vec::new();

        for orch_plan in &plan_report.plans {
            if !orch_plan.decision.should_recover {
                continue;
            }
            if let Some(ref legacy) = orch_plan.legacy_plan {
                let result = execute_recovery_plan(program, legacy);
                results.push(result.clone());

                let policies = resolved
                    .map(|c| load_recovery_policies(c, registry))
                    .unwrap_or_default();
                let rules = policy_for_entity(&policies, &orch_plan.entity_id)
                    .map(|p| p.validation_rules.clone())
                    .unwrap_or_default();
                let validation =
                    validate_recovery(program, legacy, registry, &orch_plan.entity_id, &rules);
                let ev = generate_evidence(
                    orch_plan,
                    &validation,
                    Some(&result),
                    orch_plan.estimated_duration_secs,
                );
                evidence.push(ev.clone());
                record_learning_outcome(&mut self.history, ev);
            }
        }

        let knowledge = load_knowledge(program);
        let metrics = compute_metrics(&self.history, &knowledge);
        let passed = results
            .iter()
            .all(|r| r.status != spanda_runtime::recovery_types::RecoveryStatus::Failed)
            && plan_report.passed;

        OrchestratorRecoveryReport {
            plans: plan_report.plans,
            evidence,
            metrics,
            predictive_indicators: plan_report.predictive_indicators,
            legacy_report: Some(evaluate_recovery(
                program,
                request
                    .failure
                    .as_ref()
                    .map(|f| RecoveryContext {
                        issue: f.clone(),
                        diagnosis: None,
                        classification: None,
                        level: ctx.autonomy_level,
                    })
                    .as_ref(),
                None,
            )),
            simulation_mode: RecoverySimulationMode::Validate,
            passed,
        }
    }

    /// List recovery policies for all entities.
    pub fn list_policies(
        &self,
        registry: &EntityRegistry,
        resolved: &ResolvedSystemConfig,
    ) -> Vec<EntityRecoveryPolicy> {
        load_recovery_policies(resolved, registry)
    }

    /// List available recovery playbooks.
    pub fn list_playbooks(&self, resolved: Option<&ResolvedSystemConfig>) -> Vec<RecoveryPlaybook> {
        let mut playbooks = load_playbooks(resolved);
        if let Some(registry) = self.plugin_registry() {
            crate::playbook::merge_plugin_playbooks(&mut playbooks, registry);
        }
        playbooks
    }

    /// Get recovery history evidence records.
    pub fn get_history(&self, limit: usize) -> Vec<OrchestratorRecoveryEvidence> {
        self.history.recent(limit).into_iter().cloned().collect()
    }

    /// Get aggregated recovery metrics.
    pub fn get_metrics(&self, program: &Program) -> crate::types::RecoveryMetrics {
        let knowledge = load_knowledge(program);
        compute_metrics(&self.history, &knowledge)
    }

    /// Build recovery graph for visualization.
    pub fn build_graph(
        &self,
        registry: &EntityRegistry,
        root_id: Option<&str>,
    ) -> crate::graph::RecoveryGraph {
        build_recovery_graph(registry, root_id)
    }

    /// Explain recovery decision for an entity failure.
    pub fn explain_recovery(
        &self,
        registry: &EntityRegistry,
        resolved: Option<&ResolvedSystemConfig>,
        entity_id: &str,
        failure: &str,
    ) -> Option<crate::types::RecoveryDecision> {
        let policies = resolved
            .map(|c| load_recovery_policies(c, registry))
            .unwrap_or_default();
        let entity = registry.get(entity_id)?;
        Some(decide_recovery(entity, failure, &policies, registry, None))
    }

    /// List all recoverable entities.
    pub fn recoverable_entities(
        &self,
        registry: &EntityRegistry,
    ) -> Vec<crate::types::RecoveryEntityTarget> {
        list_recoverable_entities(registry)
    }

    /// Check predictive indicators and recommend preventative recovery.
    pub fn check_predictive(
        &self,
        registry: &EntityRegistry,
        telemetry: Option<&serde_json::Value>,
    ) -> (Vec<crate::types::PredictiveIndicator>, bool) {
        let indicators = scan_predictive_indicators(registry, telemetry);
        let trigger = should_trigger_preventative(&indicators);
        (indicators, trigger)
    }

    /// Recommend strategy from knowledge base.
    pub fn recommend_from_knowledge(
        &self,
        program: &Program,
        failure: &str,
    ) -> Option<spanda_runtime::recovery_types::RecoveryKnowledgeEntry> {
        recommend_strategy(&load_knowledge(program), failure)
    }

    /// Build orchestrated plan for a single entity (internal helper).
    pub fn build_orchestrated_plan(
        program: &Program,
        registry: &EntityRegistry,
        resolved: Option<&ResolvedSystemConfig>,
        entity_id: &str,
        failure: &str,
    ) -> Option<OrchestratedRecoveryPlan> {
        let entity = registry.get(entity_id)?;
        let policies = resolved
            .map(|c| load_recovery_policies(c, registry))
            .unwrap_or_default();
        let playbooks = load_playbooks(resolved);
        let classification = classify_failure(failure);
        let decision = decide_recovery(entity, failure, &policies, registry, Some(classification));
        let ctx = RecoveryContext {
            issue: failure.to_string(),
            diagnosis: None,
            classification: Some(classification),
            level: RecoveryLevel::Level3AutomaticWithValidation,
        };
        let legacy_plan = RecoveryPlanner::plan(program, &ctx);
        let matched = match_playbooks(&playbooks, failure)
            .first()
            .map(|p| (*p).clone());
        let strategies = if let Some(ref pb) = matched {
            pb.steps.iter().map(|s| s.strategy.clone()).collect()
        } else {
            vec![decision.recommended_strategy.clone()]
        };
        let graph = build_recovery_graph(registry, Some(entity_id));
        let mut plan = OrchestratedRecoveryPlan {
            plan_id: format!("plan-{entity_id}"),
            entity_id: entity_id.to_string(),
            entity_kind: entity.entity_type.clone(),
            failure: failure.to_string(),
            classification,
            diagnosis: legacy_plan.diagnosis.clone(),
            strategies,
            escalation_level: decision.recommended_level,
            decision,
            legacy_plan: Some(legacy_plan),
            playbook: matched.map(|p| p.name),
            upstream_impact: Vec::new(),
            downstream_impact: Vec::new(),
            estimated_duration_secs: 60,
            risk: "medium".into(),
        };
        enrich_plan_with_impact(&mut plan, &graph);
        Some(plan)
    }
}
