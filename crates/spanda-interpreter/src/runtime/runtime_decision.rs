//! Live distributed decision tree evaluation and fleet consensus recording.

use super::super::super::fleet_http::{
    fetch_fleet_decision_conflict, ingest_fleet_decision_vote, FleetDecisionVoteIngestRequest,
};
use super::{Interpreter, MotionCommand, RobotBackend};
use spanda_decision::{
    escalation_is_approved, layer_str_precedence_key, register_pending_escalation,
    resolve_conflict, CompetingDecision, ConflictResolution,
};
use spanda_error::SpandaError;
use spanda_runtime::decision_runtime::{DecisionActionVerdict, DecisionTreeEvalResult};

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
            || escalation_is_approved(escalation_id)
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
        let all_results = runtime.evaluate_trees(&program, &self.decision_signals);

        let entity = self
            .active_robot_name
            .clone()
            .unwrap_or_else(|| "robot".into());

        let mut candidates: Vec<DecisionTreeEvalResult> = all_results
            .into_iter()
            .filter(|result| {
                let fingerprint = format!("{}:{}", result.tree_name, result.condition_matched);
                if self.decision_tree_emitted.contains(&fingerprint) {
                    return false;
                }
                if let Some(changed_key) = changed {
                    return result.condition_matched.contains(changed_key)
                        || self.signal_matches_tree(changed_key, &result.condition_matched);
                }
                true
            })
            .collect();

        if self.decision_signals.get("obstacle.detected").copied() == Some(true) {
            candidates.push(DecisionTreeEvalResult {
                tree_name: "SafetyReflex".into(),
                layer: "reflex".into(),
                condition_matched: "obstacle.detected".into(),
                actions: vec!["emergency_stop".into()],
                tree_hash: "safety-reflex".into(),
            });
        }

        if candidates.is_empty() {
            return;
        }

        let (results_to_dispatch, rejected_alternatives): (Vec<_>, Vec<_>) =
            if candidates.len() > 1 {
                let competing: Vec<CompetingDecision> = candidates
                    .iter()
                    .map(|r| CompetingDecision {
                        layer_precedence: layer_str_precedence_key(&r.layer).into(),
                        entity_id: entity.clone(),
                        action: r.actions.first().cloned().unwrap_or_default(),
                        reason: format!(
                            "tree '{}' matched '{}'",
                            r.tree_name, r.condition_matched
                        ),
                    })
                    .collect();
                let mesh_winner = self.resolve_decision_via_fleet_mesh(&entity, &competing);
                if let Some(resolution) = mesh_winner.or_else(|| resolve_conflict(&competing)) {
                    let winner = resolution.winner.clone();
                    let rejected: Vec<serde_json::Value> = resolution
                        .rejected
                        .iter()
                        .map(|d| {
                            serde_json::json!({
                                "entity_id": d.entity_id,
                                "action": d.action,
                                "reason": d.reason,
                                "precedence_applied": resolution.precedence_applied,
                            })
                        })
                        .collect();
                    let winners: Vec<_> = candidates
                        .into_iter()
                        .filter(|r| {
                            r.actions.first().map(String::as_str) == Some(winner.action.as_str())
                                || r.tree_name == "SafetyReflex"
                                    && winner.action.contains("emergency")
                        })
                        .collect();
                    (winners, rejected)
                } else {
                    (candidates, vec![])
                }
            } else {
                (candidates, vec![])
            };

        for result in results_to_dispatch {
            self.emit_decision_tree_result(&program, &entity, &result, &rejected_alternatives);
        }
    }

    fn emit_decision_tree_result(
        &mut self,
        program: &spanda_ast::nodes::Program,
        entity: &str,
        result: &DecisionTreeEvalResult,
        rejected_alternatives: &[serde_json::Value],
    ) {
        let fingerprint = format!("{}:{}", result.tree_name, result.condition_matched);
        if self.decision_tree_emitted.contains(&fingerprint) {
            return;
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
            entity,
            serde_json::json!({
                "tree": result.tree_name,
                "condition": result.condition_matched,
                "actions": result.actions,
                "tree_hash": result.tree_hash,
                "policy_version": "1.0.0",
                "signals": self.decision_signals,
                "central_connected": self.central_connected(),
                "offline_minutes": self.offline_minutes(),
                "rejected_alternatives": rejected_alternatives,
            }),
        );
        if let Err(err) = self.dispatch_decision_tree_actions(program, entity, &result.actions) {
            self.log(format!("decision_tree: action dispatch error: {err}"));
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
            let _ = register_pending_escalation(id, entity_id, action, &verdict.reason);
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

    fn resolve_decision_via_fleet_mesh(
        &self,
        entity: &str,
        competing: &[CompetingDecision],
    ) -> Option<ConflictResolution> {
        let url = std::env::var("SPANDA_FLEET_MESH_URL").ok()?;
        let token = std::env::var("SPANDA_FLEET_MESH_TOKEN").ok();
        let round_id = format!("local-{entity}-{}", self.sim_time_ms as u64);
        for vote in competing {
            let _ = ingest_fleet_decision_vote(
                &url,
                &FleetDecisionVoteIngestRequest {
                    round_id: round_id.clone(),
                    entity_id: vote.entity_id.clone(),
                    action: vote.action.clone(),
                    layer_precedence: vote.layer_precedence.clone(),
                    reason: vote.reason.clone(),
                    fleet_name: None,
                },
                token.as_deref(),
            );
        }
        let conflict = fetch_fleet_decision_conflict(&url, &round_id, token.as_deref()).ok()?;
        Some(ConflictResolution {
            winner: CompetingDecision {
                layer_precedence: conflict.resolution.winner.layer_precedence,
                entity_id: conflict.resolution.winner.entity_id,
                action: conflict.resolution.winner.action,
                reason: conflict.resolution.winner.reason,
            },
            rejected: conflict
                .resolution
                .rejected
                .into_iter()
                .map(|d| CompetingDecision {
                    layer_precedence: d.layer_precedence,
                    entity_id: d.entity_id,
                    action: d.action,
                    reason: d.reason,
                })
                .collect(),
            precedence_applied: conflict.resolution.precedence_applied,
        })
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
        let round_id = format!("{event}-{}", self.sim_time_ms as u64);
        let mesh_url = std::env::var("SPANDA_FLEET_MESH_URL").ok();
        let mesh_token = std::env::var("SPANDA_FLEET_MESH_TOKEN").ok();
        let mut resolved_action = selected_action.to_string();
        let mut mesh_resolution: Option<serde_json::Value> = None;

        if let Some(url) = mesh_url.as_deref() {
            for member in members {
                let member_action = member_fleet_vote_action(member, selected_action);
                let _ = ingest_fleet_decision_vote(
                    url,
                    &FleetDecisionVoteIngestRequest {
                        round_id: round_id.clone(),
                        entity_id: member.clone(),
                        action: member_action,
                        layer_precedence: "fleet_coordination".into(),
                        reason: format!("{event} fleet vote"),
                        fleet_name: None,
                    },
                    mesh_token.as_deref(),
                );
            }
            if let Ok(conflict) =
                fetch_fleet_decision_conflict(url, &round_id, mesh_token.as_deref())
            {
                resolved_action = conflict.resolution.winner.action.clone();
                mesh_resolution = Some(serde_json::json!({
                    "round_id": conflict.round_id,
                    "winner": conflict.resolution.winner,
                    "rejected": conflict.resolution.rejected,
                    "precedence_applied": conflict.resolution.precedence_applied,
                }));
            }
        }

        let votes: Vec<(String, String, f64)> = members
            .iter()
            .enumerate()
            .map(|(i, m)| {
                (
                    m.clone(),
                    resolved_action.clone(),
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
                "mesh_resolution": mesh_resolution,
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

fn member_fleet_vote_action(member: &str, default_action: &str) -> String {
    let key = format!(
        "SPANDA_FLEET_MEMBER_VOTE_{}",
        member.to_ascii_uppercase().replace('-', "_")
    );
    std::env::var(&key).unwrap_or_else(|_| default_action.to_string())
}
