//! Live distributed decision tree evaluation and fleet consensus recording.

use super::{Interpreter, MotionCommand, RobotBackend};
use spanda_error::SpandaError;
use spanda_runtime::decision_runtime::DecisionActionVerdict;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn decision_runtime(
        &self,
    ) -> spanda_runtime::decision_runtime::SharedDecisionRuntime {
        self.decision_runtime.clone()
    }

    /// Update a decision signal and re-evaluate trees when the value changes.
    pub(super) fn set_decision_signal(&mut self, key: impl Into<String>, value: bool) {
        let key = key.into();
        let prev = self.decision_signals.get(&key).copied();
        if prev == Some(value) {
            return;
        }
        self.decision_signals.insert(key.clone(), value);
        self.evaluate_live_decision_trees(Some(&key));
    }

    fn central_connected(&self) -> bool {
        std::env::var("SPANDA_CENTRAL_CONNECTED")
            .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or_else(|_| {
                let faults = self.hardware_monitor.injected_faults();
                !faults.iter().any(|f| {
                    let lower = f.to_ascii_lowercase();
                    lower.contains("lte")
                        || lower.contains("wifi")
                        || lower.contains("connectivity")
                        || lower.contains("network")
                })
            })
    }

    fn offline_minutes(&self) -> u32 {
        if let Ok(v) = std::env::var("SPANDA_OFFLINE_MINUTES") {
            return v.parse().unwrap_or(0);
        }
        self.offline_since_ms
            .map(|since| ((self.sim_time_ms - since).max(0.0) / 60_000.0) as u32)
            .unwrap_or(0)
    }

    fn track_offline_state(&mut self) {
        if self.central_connected() {
            self.offline_since_ms = None;
        } else if self.offline_since_ms.is_none() {
            self.offline_since_ms = Some(self.sim_time_ms);
        }
    }

    fn escalation_approved(&self, escalation_id: &str) -> bool {
        self.granted_decision_escalations.contains(escalation_id)
            || std::env::var("SPANDA_DECISION_ESCALATION_APPROVED")
                .map(|v| matches!(v.as_str(), "1" | "true" | "yes") || v == escalation_id)
                .unwrap_or(false)
    }

    /// Sync decision signals from hardware monitor, safety state, and faults.
    pub(super) fn sync_decision_signals_from_runtime(&mut self) {
        self.track_offline_state();
        let gps_failed = self
            .hardware_monitor
            .injected_faults()
            .iter()
            .any(|f| f.contains("GPS") || f.contains("gps"));
        self.decision_signals
            .insert("gps.status == Failed".into(), gps_failed);

        let obstacle = self
            .safety_monitor
            .as_ref()
            .map(|m| m.is_emergency_stop())
            .unwrap_or(false)
            || self.backend.get_state().emergency_stop;
        self.decision_signals
            .insert("obstacle.detected".into(), obstacle);

        let coordinator_failed = std::env::var("SPANDA_FLEET_COORDINATOR_FAILED")
            .map(|v| matches!(v.as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);
        self.decision_signals
            .insert("fleet.coordinator.failed".into(), coordinator_failed);

        let visual_odom = !gps_failed;
        self.decision_signals
            .insert("visual_odometry.available".into(), visual_odom);

        let operator = std::env::var("SPANDA_OPERATOR_AVAILABLE")
            .map(|v| matches!(v.as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);
        self.decision_signals
            .insert("operator.available".into(), operator);
    }

    /// Evaluate decision trees and emit trace records for newly matched conditions.
    pub(super) fn evaluate_live_decision_trees(&mut self, changed: Option<&str>) {
        let Some(program) = self.health_program.clone() else {
            return;
        };
        self.sync_decision_signals_from_runtime();
        let runtime = self.decision_runtime();
        let results = runtime.evaluate_trees(&program, &self.decision_signals);
        for result in results {
            let fingerprint = format!("{}:{}", result.tree_name, result.condition_matched);
            if self.decision_tree_emitted.contains(&fingerprint) {
                continue;
            }
            if let Some(changed_key) = changed {
                if !result.condition_matched.contains(changed_key)
                    && !self.signal_matches_tree(changed_key, &result.condition_matched)
                {
                    continue;
                }
            }
            self.decision_tree_emitted.insert(fingerprint);
            self.log(format!(
                "decision_tree '{}': {} → [{}]",
                result.tree_name,
                result.condition_matched,
                result.actions.join(", ")
            ));
            let layer = if result.layer.contains("reflex") {
                "reflex"
            } else if result.layer.contains("group") || result.layer.contains("fleet") {
                "group_fleet"
            } else {
                "local_entity"
            };
            let entity = self
                .active_robot_name
                .clone()
                .unwrap_or_else(|| "robot".into());
            self.record_decision_trace(
                "decision_tree_eval",
                "local_decision",
                &format!(
                    "tree '{}' matched '{}' → {}",
                    result.tree_name,
                    result.condition_matched,
                    result.actions.join(", ")
                ),
                layer,
                &entity,
                serde_json::json!({
                    "tree": result.tree_name,
                    "condition": result.condition_matched,
                    "actions": result.actions,
                    "tree_hash": result.tree_hash,
                    "policy_version": "1.0.0",
                    "signals": self.decision_signals,
                    "central_connected": self.central_connected(),
                    "offline_minutes": self.offline_minutes(),
                }),
            );
            if let Err(err) =
                self.dispatch_decision_tree_actions(&program, &entity, &result.actions)
            {
                self.log(format!("decision_tree: action dispatch error: {err}"));
            }
        }
    }

    fn authorize_decision_action(
        &self,
        program: &spanda_ast::nodes::Program,
        entity_id: &str,
        action: &str,
    ) -> DecisionActionVerdict {
        self.decision_runtime().authorize_action(
            program,
            entity_id,
            action,
            self.offline_minutes(),
            self.central_connected(),
        )
    }

    fn record_action_verdict(
        &mut self,
        action: &str,
        verdict: &DecisionActionVerdict,
        entity_id: &str,
    ) {
        if verdict.permitted {
            return;
        }
        let event = if verdict.requires_escalation {
            "decision_escalation_pending"
        } else {
            "decision_action_blocked"
        };
        self.record_decision_trace(
            event,
            "policy_gate",
            &verdict.reason,
            if verdict.requires_escalation {
                "control_center"
            } else {
                "local_entity"
            },
            entity_id,
            serde_json::json!({
                "action": action,
                "escalation_id": verdict.escalation_id,
                "policy_version": verdict.policy_version,
                "offline_minutes": self.offline_minutes(),
                "central_connected": self.central_connected(),
                "rejected_alternatives": [{"action": action, "rejected_reason": verdict.reason}],
            }),
        );
        if let Some(id) = &verdict.escalation_id {
            self.pending_decision_escalations.insert(id.clone());
        }
    }

    /// Execute actions selected by a live decision tree evaluation.
    pub(super) fn dispatch_decision_tree_actions(
        &mut self,
        program: &spanda_ast::nodes::Program,
        entity_id: &str,
        actions: &[String],
    ) -> Result<(), SpandaError> {
        for action in actions {
            let verdict = self.authorize_decision_action(program, entity_id, action);
            if !verdict.permitted {
                self.record_action_verdict(action, &verdict, entity_id);
                if verdict.requires_escalation {
                    if let Some(id) = &verdict.escalation_id {
                        if !self.escalation_approved(id) {
                            self.log(format!(
                                "decision: blocked '{action}' pending escalation {id}"
                            ));
                            continue;
                        }
                        self.granted_decision_escalations.insert(id.clone());
                        self.pending_decision_escalations.remove(id);
                    } else {
                        continue;
                    }
                } else {
                    self.log(format!("decision: blocked '{action}' — {}", verdict.reason));
                    continue;
                }
            }
            let lower = action.to_lowercase();
            if lower.contains("trigger") && lower.contains("emergency_stop") {
                if let Some(monitor) = &mut self.safety_monitor {
                    monitor.set_emergency_stop(true);
                }
                self.backend.set_emergency_stop(true);
                self.backend.execute_motion(MotionCommand::Stop {
                    actuator: "all".into(),
                });
                self.log("decision_tree: triggered emergency stop".into());
                continue;
            }
            if lower.contains("cut_actuator_power") {
                self.backend.set_emergency_stop(true);
                self.log("decision_tree: cut actuator power".into());
                continue;
            }
            self.dispatch_recovery_action(action)?;
        }
        Ok(())
    }

    fn signal_matches_tree(&self, changed: &str, condition: &str) -> bool {
        condition.contains(changed) || self.decision_signals.get(changed).copied().unwrap_or(false)
    }

    /// Record fleet mesh consensus decision after coordinator relay.
    pub(super) fn record_fleet_mesh_consensus(
        &mut self,
        event: &str,
        members: &[String],
        selected_action: &str,
        relayed: u32,
        failed: u32,
    ) {
        let votes: Vec<(String, String, f64)> = members
            .iter()
            .enumerate()
            .map(|(i, m)| {
                (
                    m.clone(),
                    selected_action.to_string(),
                    1.0 - (i as f64 * 0.1),
                )
            })
            .collect();
        let quorum = if members.is_empty() {
            0.5
        } else {
            (relayed as f64 / members.len() as f64).clamp(0.0, 1.0)
        };
        let consensus = self
            .decision_runtime()
            .resolve_fleet_consensus(&votes, quorum);
        self.record_decision_trace(
            event,
            "fleet_consensus",
            &format!(
                "{} → {} (quorum={}, votes={})",
                consensus.strategy,
                consensus.selected_action,
                consensus.quorum_met,
                consensus.vote_count
            ),
            "group_fleet",
            "fleet_coordinator",
            serde_json::json!({
                "selected_action": consensus.selected_action,
                "strategy": consensus.strategy,
                "quorum_met": consensus.quorum_met,
                "relayed": relayed,
                "failed": failed,
                "members": members,
                "policy_version": "1.0.0",
            }),
        );
    }

    /// Poll decision trees on scheduler ticks (bounded rate).
    pub(super) fn poll_live_decision_trees(&mut self) -> Result<(), SpandaError> {
        self.sync_decision_signals_from_runtime();
        self.evaluate_live_decision_trees(None);
        Ok(())
    }
}
