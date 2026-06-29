//! Assurance-backed implementation of the runtime assurance boundary.
//!
use spanda_runtime::assurance_runtime::AssuranceRuntime;
use spanda_runtime::continuity_primitives::{
    default_checkpoint_store_path, extract_continuity_policies, issue_to_continuity_trigger,
    load_checkpoint, load_checkpoint_store, parse_trigger, program_has_continuity_for_trigger,
    record_checkpoint, save_checkpoint_store,
};
use spanda_runtime::continuity_types::{
    ContinuityCheckpointStore, ContinuityContext, ContinuityPolicySpec, ContinuityTrigger,
    MissionStateSnapshot, TakeoverReport,
};
use spanda_runtime::recovery_primitives::{
    classify_failure, default_knowledge_store_path, extract_recovery_policies,
    issue_to_recovery_issue, load_recovery_knowledge_store, program_has_recovery_for_issue,
    record_recovery_outcome, save_recovery_knowledge_store,
};
use spanda_runtime::recovery_types::{
    FailureClassification, RecoveryContext, RecoveryKnowledgeBase, RecoveryPlan, RecoveryResult,
    SafeRecoveryAction,
};
use spanda_ast::nodes::Program;
use std::path::{Path, PathBuf};

use crate::continuity::plan_takeover;
use crate::recovery::{
    execute_recovery_plan, merge_recovery_knowledge, validate_recovery_plan, RecoveryPlanner,
};

/// Full assurance runtime delegating to `spanda-assurance` planners and validators.
#[derive(Debug, Default, Clone, Copy)]
pub struct AssuranceBackedRuntime;

impl AssuranceRuntime for AssuranceBackedRuntime {
    fn classify_failure(&self, issue: &str) -> FailureClassification {
        classify_failure(issue)
    }

    fn default_knowledge_store_path(&self) -> PathBuf {
        default_knowledge_store_path()
    }

    fn load_recovery_knowledge_store(&self, path: &Path) -> RecoveryKnowledgeBase {
        load_recovery_knowledge_store(path)
    }

    fn save_recovery_knowledge_store(
        &self,
        path: &Path,
        kb: &RecoveryKnowledgeBase,
    ) -> std::io::Result<()> {
        save_recovery_knowledge_store(path, kb)
    }

    fn merge_recovery_knowledge(
        &self,
        program: &Program,
        persisted: &RecoveryKnowledgeBase,
    ) -> RecoveryKnowledgeBase {
        merge_recovery_knowledge(program, persisted)
    }

    fn record_recovery_outcome(&self, kb: &mut RecoveryKnowledgeBase, result: &RecoveryResult) {
        record_recovery_outcome(kb, result);
    }

    fn extract_recovery_policies(&self, program: &Program) -> Vec<spanda_runtime::recovery_types::RecoveryPolicySpec> {
        extract_recovery_policies(program)
    }

    fn issue_to_recovery_issue(&self, event: &str) -> Option<String> {
        issue_to_recovery_issue(event)
    }

    fn program_has_recovery_for_issue(&self, program: &Program, issue: &str) -> bool {
        program_has_recovery_for_issue(program, issue)
    }

    fn plan_recovery(&self, program: &Program, context: &RecoveryContext) -> RecoveryPlan {
        RecoveryPlanner::plan(program, context)
    }

    fn validate_recovery_plan(
        &self,
        program: &Program,
        plan: &RecoveryPlan,
    ) -> Vec<SafeRecoveryAction> {
        validate_recovery_plan(program, plan)
    }

    fn build_recovery_result_from_plan(
        &self,
        program: &Program,
        plan: &RecoveryPlan,
    ) -> RecoveryResult {
        execute_recovery_plan(program, plan)
    }

    fn recovery_allowed(&self, program: &Program, issue: &str) -> bool {
        let context = RecoveryContext {
            issue: issue.into(),
            diagnosis: None,
            classification: Some(classify_failure(issue)),
            level: spanda_runtime::recovery_types::RecoveryLevel::Level3AutomaticWithValidation,
        };
        let plan = RecoveryPlanner::plan(program, &context);
        let result = execute_recovery_plan(program, &plan);
        !matches!(
            result.status,
            spanda_runtime::recovery_types::RecoveryStatus::Unsafe
                | spanda_runtime::recovery_types::RecoveryStatus::Failed
        )
    }

    fn extract_continuity_policies(&self, program: &Program) -> Vec<ContinuityPolicySpec> {
        extract_continuity_policies(program)
    }

    fn issue_to_continuity_trigger(&self, issue: &str) -> Option<ContinuityTrigger> {
        issue_to_continuity_trigger(issue)
    }

    fn program_has_continuity_for_trigger(
        &self,
        program: &Program,
        trigger: ContinuityTrigger,
    ) -> bool {
        program_has_continuity_for_trigger(program, trigger)
    }

    fn plan_takeover(
        &self,
        program: &Program,
        context: &ContinuityContext,
        successor: Option<&str>,
    ) -> TakeoverReport {
        plan_takeover(program, context, successor)
    }

    fn default_checkpoint_store_path(&self) -> PathBuf {
        default_checkpoint_store_path()
    }

    fn load_checkpoint_store(&self, path: &Path) -> ContinuityCheckpointStore {
        load_checkpoint_store(path)
    }

    fn save_checkpoint_store(
        &self,
        path: &Path,
        store: &ContinuityCheckpointStore,
    ) -> std::io::Result<()> {
        save_checkpoint_store(path, store)
    }

    fn record_checkpoint(
        &self,
        store: &mut ContinuityCheckpointStore,
        mission: &str,
        robot: &str,
        snapshot: MissionStateSnapshot,
    ) {
        record_checkpoint(store, mission, robot, snapshot);
    }

    fn load_checkpoint(
        &self,
        store: &ContinuityCheckpointStore,
        mission: &str,
        robot: &str,
    ) -> Option<MissionStateSnapshot> {
        load_checkpoint(store, mission, robot)
    }

    fn parse_trigger(&self, s: &str) -> ContinuityTrigger {
        parse_trigger(s)
    }

    fn evaluate_recovery_program(
        &self,
        program: &Program,
    ) -> spanda_runtime::recovery_types::RecoveryReport {
        // Delegate to the full assurance recovery evaluation with no external context.
        crate::recovery::evaluate_recovery(program, None, None)
    }
}

/// Register the real assurance runtime with the global OnceLock.
///
/// Parameters:
/// None.
///
/// Returns:
/// Unit; idempotent (subsequent calls are silently ignored).
///
/// Options:
/// None.
///
/// Example:
/// spanda_assurance::runtime_bridge::register();
pub fn register() {
    // Inject the real assurance engine into the global platform runtime slot.
    spanda_runtime::assurance_runtime::set_platform_assurance_runtime(
        std::sync::Arc::new(AssuranceBackedRuntime),
    );
}
