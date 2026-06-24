//! Runtime recovery action dispatch, operator approval, and fleet coordination.

use super::{Interpreter, RobotBackend};
use spanda_assurance::{
    classify_failure, default_knowledge_store_path, load_recovery_knowledge_store,
    merge_recovery_knowledge, record_recovery_outcome, save_recovery_knowledge_store,
    validate_recovery_plan, RecoveryContext, RecoveryLevel, RecoveryPlanner, RecoveryResult,
    RecoveryStatus,
};
use spanda_comm::CommBus;
use spanda_error::SpandaError;
use spanda_runtime::value::RuntimeValue;

fn parse_speed_cap(action: &str) -> Option<f64> {
    action
        .split_whitespace()
        .find_map(|part| part.parse::<f64>().ok())
}

fn normalize_mode_name(action: &str) -> Option<&'static str> {
    let lower = action.to_lowercase();
    if lower.contains("degraded") {
        Some("degraded")
    } else if lower.contains("safe") {
        Some("safe")
    } else if lower.contains("recovery") {
        Some("recovery")
    } else if lower.contains("emergency") {
        Some("emergency")
    } else if lower.contains("normal") {
        Some("normal")
    } else {
        None
    }
}

impl<B: RobotBackend> Interpreter<B> {
    /// Return true when operator approval is granted for a recovery action.
    pub(super) fn operator_approval_granted(&self, action: &str) -> bool {
        if self.granted_recovery_approvals.contains(action) {
            return true;
        }
        if let Ok(value) = std::env::var("SPANDA_OPERATOR_APPROVAL") {
            if value == "1" || value.eq_ignore_ascii_case("true") {
                return true;
            }
            if action.to_lowercase().contains(&value.to_lowercase()) {
                return true;
            }
        }
        if let Ok(value) = std::env::var("SPANDA_GRANT_RECOVERY_APPROVAL") {
            if action.to_lowercase().contains(&value.to_lowercase()) {
                return true;
            }
        }
        false
    }

    /// Record that operator approval is required before executing an action.
    pub(super) fn request_operator_approval(&mut self, action: &str) {
        self.pending_recovery_approvals.insert(action.to_string());
        self.log(format!(
            "recovery: operator approval required for '{action}'"
        ));
        self.record_mission_event(
            "recovery_approval_required",
            serde_json::json!({ "action": action }),
        );
    }

    /// Grant operator approval for a pending recovery action (comm topic / API hook).
    pub(super) fn grant_operator_approval(&mut self, action: &str) {
        self.pending_recovery_approvals.remove(action);
        self.granted_recovery_approvals.insert(action.to_string());
        self.log(format!(
            "recovery: operator approval granted for '{action}'"
        ));
        self.record_mission_event(
            "recovery_approval_granted",
            serde_json::json!({ "action": action }),
        );
    }

    /// Dispatch a single recovery action string at runtime.
    pub(super) fn dispatch_recovery_action(&mut self, action: &str) -> Result<bool, SpandaError> {
        let lower = action.to_lowercase();

        if let Some(mode) = normalize_mode_name(action) {
            if lower.contains("enter") || lower.contains("mode") || lower.contains(mode) {
                self.enter_mode(mode)?;
                return Ok(true);
            }
        }

        if lower.contains("reduce_speed") {
            if let Some(cap) = parse_speed_cap(action) {
                if let Some(monitor) = &mut self.safety_monitor {
                    monitor.apply_speed_cap(cap);
                }
                self.recovery_speed_cap = Some(cap);
                self.log(format!("recovery: speed cap set to {cap} m/s"));
                return Ok(true);
            }
        }

        if lower.contains("restart") && lower.contains("connect") {
            self.restart_active_connectivity()?;
            return Ok(true);
        }

        if lower.contains("pause") && lower.contains("mission") {
            self.pause_active_mission();
            return Ok(true);
        }

        if lower.contains("reassign")
            || lower.contains("redistribute")
            || lower.contains("promote")
            || lower.contains("replace")
        {
            self.coordinate_fleet_recovery(action)?;
            return Ok(true);
        }

        if lower.contains("halt") || lower.contains("emergency_stop") || lower.contains("stop") {
            if let Some(monitor) = &mut self.safety_monitor {
                monitor.set_emergency_stop(true);
            }
            self.backend.set_emergency_stop(true);
            return Ok(true);
        }

        self.log(format!("recovery: recorded action '{action}'"));
        Ok(true)
    }

    /// Poll Approval topics and environment for operator grants.
    pub(super) fn poll_recovery_approvals(&mut self) {
        for (path, text) in &self.options.inbound_comm_messages {
            self.comm_bus.push_inbound(
                path,
                RuntimeValue::String {
                    value: text.clone(),
                },
                None,
            );
        }

        let approval_topics: Vec<String> = self
            .topic_path_to_message_type
            .iter()
            .filter(|(_, message_type)| message_type.as_str() == "Approval")
            .map(|(path, _)| path.clone())
            .collect();

        for path in approval_topics {
            while let Some(envelope) = self.comm_bus.receive_envelope(&path) {
                if let RuntimeValue::String { value } = envelope.value {
                    self.grant_operator_approval(&value);
                } else {
                    for pending in self.pending_recovery_approvals.clone() {
                        self.grant_operator_approval(&pending);
                    }
                }
            }
        }

        for pending in self.pending_recovery_approvals.clone() {
            if self.operator_approval_granted(&pending) {
                self.grant_operator_approval(&pending);
            }
        }
    }

    /// Execute a validated recovery plan at runtime for the given issue.
    pub(super) fn execute_recovery_runtime(
        &mut self,
        issue: &str,
    ) -> Result<RecoveryResult, SpandaError> {
        self.poll_recovery_approvals();
        let Some(program) = self.health_program.clone() else {
            return Ok(RecoveryResult {
                plan: "none".into(),
                status: RecoveryStatus::Failed,
                executed_actions: vec![],
                failed_actions: vec![issue.into()],
                verification_outcome: "No recovery program cached".into(),
                evidence: spanda_assurance::RecoveryEvidence {
                    failure: issue.into(),
                    diagnosis: issue.into(),
                    plan: "none".into(),
                    safety_validation: "SKIP".into(),
                    recovery_actions: vec![],
                    outcome: "Failed".into(),
                    operator_approval: None,
                    verification: "No program".into(),
                },
            });
        };

        let context = RecoveryContext {
            issue: issue.into(),
            diagnosis: None,
            classification: Some(classify_failure(issue)),
            level: RecoveryLevel::Level3AutomaticWithValidation,
        };
        let plan = RecoveryPlanner::plan(&program, &context);
        let safe_actions = validate_recovery_plan(&program, &plan);
        let mut executed = Vec::new();
        let mut failed = Vec::new();
        let mut operator_approval = None;

        for safe in &safe_actions {
            let gates_ok = safe.safety_validation.passed
                && safe.hardware_verification.passed
                && safe.capability_verification.passed
                && safe.readiness_validation.passed;
            if !gates_ok {
                failed.push(safe.action.description.clone());
                continue;
            }
            if safe.action.requires_approval
                && !self.operator_approval_granted(&safe.action.description)
            {
                self.request_operator_approval(&safe.action.description);
                failed.push(format!("{} (approval required)", safe.action.description));
                operator_approval = Some("Operator".into());
                continue;
            }
            self.dispatch_recovery_action(&safe.action.description)?;
            executed.push(safe.action.description.clone());
        }

        let status = if failed.is_empty() && !executed.is_empty() {
            RecoveryStatus::Success
        } else if !executed.is_empty() {
            RecoveryStatus::PartialSuccess
        } else if safe_actions.iter().any(|a| !a.safety_validation.passed) {
            RecoveryStatus::Unsafe
        } else {
            RecoveryStatus::Failed
        };

        let evidence = spanda_assurance::RecoveryEvidence {
            failure: plan.failure.clone(),
            diagnosis: plan.diagnosis.clone(),
            plan: plan.name.clone(),
            safety_validation: if safe_actions.iter().all(|a| a.safety_validation.passed) {
                "PASS".into()
            } else {
                "FAIL".into()
            },
            recovery_actions: executed.clone(),
            outcome: format!("{status:?}"),
            operator_approval: operator_approval.clone(),
            verification: if status == RecoveryStatus::Success {
                "Recovery verified".into()
            } else {
                "Recovery incomplete".into()
            },
        };

        let result = RecoveryResult {
            plan: plan.name.clone(),
            status,
            executed_actions: executed,
            failed_actions: failed,
            verification_outcome: evidence.verification.clone(),
            evidence,
        };

        self.record_mission_event(
            "recovery_executed",
            serde_json::json!({
                "issue": issue,
                "status": format!("{:?}", result.status),
                "actions": result.executed_actions,
            }),
        );

        let persisted = load_recovery_knowledge_store(&self.recovery_knowledge_path);
        let mut knowledge = merge_recovery_knowledge(&program, &persisted);
        record_recovery_outcome(&mut knowledge, &result);
        let _ = save_recovery_knowledge_store(&self.recovery_knowledge_path, &knowledge);

        Ok(result)
    }

    fn restart_active_connectivity(&mut self) -> Result<(), SpandaError> {
        let link = self.active_connectivity_link.clone();
        self.default_transport = self.host.connectivity_link_to_transport(&link);
        self.comm_bus.reconnect_transport(self.default_transport);
        self.log(format!("recovery: restarted connectivity on '{link}'"));
        self.record_mission_event(
            "recovery_connectivity_restart",
            serde_json::json!({ "link": link }),
        );
        Ok(())
    }

    fn pause_active_mission(&mut self) {
        let Some(RuntimeValue::MissionControl { mut runtime }) = self.env.get("mission").cloned()
        else {
            return;
        };
        runtime.pause();
        self.env
            .define("mission", RuntimeValue::MissionControl { runtime });
        self.log("recovery: mission paused".into());
        self.record_mission_event("recovery_mission_paused", serde_json::json!({}));
    }

    fn coordinate_fleet_recovery(&mut self, action: &str) -> Result<(), SpandaError> {
        let fleet_names: Vec<String> = self.fleets.names().cloned().collect();
        let source = self.publish_source_id();
        self.comm_bus.publish(
            "/fleet/recovery",
            "Command",
            RuntimeValue::String {
                value: action.to_string(),
            },
            self.default_transport,
            Some(&source),
        );
        for fleet_name in fleet_names {
            if let Some(members) = self.fleets.members(&fleet_name) {
                self.log(format!(
                    "fleet_recovery: {action} for fleet {fleet_name} members={members:?}"
                ));
                self.record_mission_event(
                    "fleet_recovery",
                    serde_json::json!({
                        "fleet": fleet_name,
                        "action": action,
                        "members": members,
                    }),
                );
            }
        }
        Ok(())
    }

    pub(super) fn init_recovery_runtime(&mut self) {
        self.recovery_knowledge_path = default_knowledge_store_path();
        self.pending_recovery_approvals.clear();
        self.granted_recovery_approvals.clear();
        self.recovery_speed_cap = None;
    }
}
