//! Recovery playbook framework — versioned, reusable recovery workflows.
//!
use crate::types::{OrchestratorStrategy, PlaybookStep, RecoveryEscalationLevel, RecoveryPlaybook};
use spanda_config::entity::EntityKind;
use spanda_config::resolved::ResolvedSystemConfig;

/// Built-in recovery playbooks shipped with the orchestrator.
pub fn default_playbooks() -> Vec<RecoveryPlaybook> {
    vec![
        battery_low_playbook(),
        sensor_failure_playbook(),
        connectivity_loss_playbook(),
        fleet_member_loss_playbook(),
        mission_transfer_playbook(),
    ]
}

/// Load playbooks from config, merging with defaults.
pub fn load_playbooks(cfg: Option<&ResolvedSystemConfig>) -> Vec<RecoveryPlaybook> {
    // Load playbooks from config, merging with defaults.
    //
    // Parameters:
    // - `cfg` — optional resolved system config
    //
    // Returns:
    // Merged playbook list.
    //
    // Options:
    // None.
    //
    // Example:
    // let playbooks = load_playbooks(resolved.as_ref());

    let mut playbooks = default_playbooks();
    if let Some(cfg) = cfg {
        if let Some(section) = cfg.recovery_config() {
            if let Some(arr) = section.get("playbooks").and_then(|v| v.as_array()) {
                for item in arr {
                    if let Some(pb) = parse_playbook_value(item) {
                        if let Some(idx) = playbooks.iter().position(|p| p.name == pb.name) {
                            playbooks[idx] = pb;
                        } else {
                            playbooks.push(pb);
                        }
                    }
                }
            }
        }
    }
    playbooks.sort_by(|a, b| a.name.cmp(&b.name));
    playbooks
}

/// Find playbook by name.
pub fn find_playbook<'a>(
    playbooks: &'a [RecoveryPlaybook],
    name: &str,
) -> Option<&'a RecoveryPlaybook> {
    playbooks.iter().find(|p| p.name == name)
}

/// Match playbooks to a failure trigger string.
pub fn match_playbooks<'a>(
    playbooks: &'a [RecoveryPlaybook],
    failure: &str,
) -> Vec<&'a RecoveryPlaybook> {
    let failure_lower = failure.to_ascii_lowercase();
    playbooks
        .iter()
        .filter(|pb| {
            pb.trigger.to_ascii_lowercase().contains(&failure_lower)
                || failure_lower.contains(&pb.trigger.to_ascii_lowercase())
        })
        .collect()
}

/// Merge plugin-contributed playbook extensions into the playbook list.
pub fn merge_plugin_playbooks(
    playbooks: &mut Vec<RecoveryPlaybook>,
    registry: &crate::plugin::RecoveryPluginRegistry,
) {
    // Register plugin playbooks that are not already present by name.
    for ext in registry.list(crate::plugin::PLUGIN_KIND_PLAYBOOK) {
        if playbooks.iter().any(|pb| pb.name == ext.name) {
            continue;
        }
        let trigger = ext.description.clone();
        playbooks.push(RecoveryPlaybook {
            name: ext.name.clone(),
            version: "plugin".into(),
            description: ext.description.clone(),
            trigger,
            steps: vec![PlaybookStep {
                order: 1,
                description: format!("Plugin playbook: {}", ext.name),
                strategy: OrchestratorStrategy::Custom(ext.name.clone()),
                escalation_level: RecoveryEscalationLevel::Level3RecoverDevice,
                timeout_secs: 120,
                requires_validation: true,
            }],
            entity_kinds: Vec::new(),
        });
    }
    playbooks.sort_by(|a, b| a.name.cmp(&b.name));
}

fn battery_low_playbook() -> RecoveryPlaybook {
    RecoveryPlaybook {
        name: "battery_low".into(),
        version: "1.0.0".into(),
        description:
            "Battery below threshold — navigate to charger, assign backup, transfer mission".into(),
        trigger: "battery".into(),
        entity_kinds: vec![EntityKind::Robot, EntityKind::Drone, EntityKind::Vehicle],
        steps: vec![
            PlaybookStep {
                order: 1,
                description: "Navigate to nearest charger".into(),
                strategy: OrchestratorStrategy::GracefulDegradation,
                escalation_level: RecoveryEscalationLevel::Level0Retry,
                timeout_secs: 120,
                requires_validation: false,
            },
            PlaybookStep {
                order: 2,
                description: "Assign backup robot from fleet".into(),
                strategy: OrchestratorStrategy::DelegateMission,
                escalation_level: RecoveryEscalationLevel::Level5MissionReassign,
                timeout_secs: 60,
                requires_validation: true,
            },
            PlaybookStep {
                order: 3,
                description: "Transfer active mission to backup".into(),
                strategy: OrchestratorStrategy::TransferMission,
                escalation_level: RecoveryEscalationLevel::Level5MissionReassign,
                timeout_secs: 90,
                requires_validation: true,
            },
            PlaybookStep {
                order: 4,
                description: "Resume mission on backup robot".into(),
                strategy: OrchestratorStrategy::RestartRobot,
                escalation_level: RecoveryEscalationLevel::Level4RecoverRobot,
                timeout_secs: 30,
                requires_validation: true,
            },
        ],
    }
}

fn sensor_failure_playbook() -> RecoveryPlaybook {
    RecoveryPlaybook {
        name: "sensor_failure".into(),
        version: "1.0.0".into(),
        description: "Sensor failure — recalibrate, switch redundant sensor, degrade gracefully"
            .into(),
        trigger: "sensor".into(),
        entity_kinds: vec![EntityKind::Sensor, EntityKind::Camera, EntityKind::Gps],
        steps: vec![
            PlaybookStep {
                order: 1,
                description: "Retry sensor read".into(),
                strategy: OrchestratorStrategy::Retry,
                escalation_level: RecoveryEscalationLevel::Level0Retry,
                timeout_secs: 10,
                requires_validation: false,
            },
            PlaybookStep {
                order: 2,
                description: "Reinitialize sensor device".into(),
                strategy: OrchestratorStrategy::Reinitialize,
                escalation_level: RecoveryEscalationLevel::Level3RecoverDevice,
                timeout_secs: 30,
                requires_validation: true,
            },
            PlaybookStep {
                order: 3,
                description: "Switch to redundant sensor".into(),
                strategy: OrchestratorStrategy::SwitchSensor,
                escalation_level: RecoveryEscalationLevel::Level3RecoverDevice,
                timeout_secs: 15,
                requires_validation: true,
            },
            PlaybookStep {
                order: 4,
                description: "Enter degraded mode".into(),
                strategy: OrchestratorStrategy::GracefulDegradation,
                escalation_level: RecoveryEscalationLevel::Level1RestartComponent,
                timeout_secs: 5,
                requires_validation: true,
            },
        ],
    }
}

fn connectivity_loss_playbook() -> RecoveryPlaybook {
    RecoveryPlaybook {
        name: "connectivity_loss".into(),
        version: "1.0.0".into(),
        description: "Connectivity loss — reconnect, switch gateway, switch network".into(),
        trigger: "connectivity".into(),
        entity_kinds: vec![EntityKind::Gateway, EntityKind::Device, EntityKind::Robot],
        steps: vec![
            PlaybookStep {
                order: 1,
                description: "Retry connection".into(),
                strategy: OrchestratorStrategy::Retry,
                escalation_level: RecoveryEscalationLevel::Level0Retry,
                timeout_secs: 15,
                requires_validation: false,
            },
            PlaybookStep {
                order: 2,
                description: "Reconnect transport".into(),
                strategy: OrchestratorStrategy::Reconnect,
                escalation_level: RecoveryEscalationLevel::Level3RecoverDevice,
                timeout_secs: 30,
                requires_validation: true,
            },
            PlaybookStep {
                order: 3,
                description: "Switch to backup gateway".into(),
                strategy: OrchestratorStrategy::SwitchGateway,
                escalation_level: RecoveryEscalationLevel::Level3RecoverDevice,
                timeout_secs: 45,
                requires_validation: true,
            },
        ],
    }
}

fn fleet_member_loss_playbook() -> RecoveryPlaybook {
    RecoveryPlaybook {
        name: "fleet_member_loss".into(),
        version: "1.0.0".into(),
        description: "Fleet member loss — redistribute tasks, reassign missions".into(),
        trigger: "fleet".into(),
        entity_kinds: vec![EntityKind::Fleet, EntityKind::Swarm, EntityKind::Robot],
        steps: vec![
            PlaybookStep {
                order: 1,
                description: "Detect failed fleet member".into(),
                strategy: OrchestratorStrategy::Retry,
                escalation_level: RecoveryEscalationLevel::Level0Retry,
                timeout_secs: 10,
                requires_validation: false,
            },
            PlaybookStep {
                order: 2,
                description: "Redistribute swarm tasks".into(),
                strategy: OrchestratorStrategy::RestartFleet,
                escalation_level: RecoveryEscalationLevel::Level6FleetRedistribute,
                timeout_secs: 60,
                requires_validation: true,
            },
            PlaybookStep {
                order: 3,
                description: "Reassign missions to healthy members".into(),
                strategy: OrchestratorStrategy::TransferMission,
                escalation_level: RecoveryEscalationLevel::Level5MissionReassign,
                timeout_secs: 90,
                requires_validation: true,
            },
        ],
    }
}

fn mission_transfer_playbook() -> RecoveryPlaybook {
    RecoveryPlaybook {
        name: "mission_transfer".into(),
        version: "1.0.0".into(),
        description: "Robot failure during mission — delegate or takeover to backup".into(),
        trigger: "mission".into(),
        entity_kinds: vec![EntityKind::Mission, EntityKind::Robot],
        steps: vec![
            PlaybookStep {
                order: 1,
                description: "Pause mission safely".into(),
                strategy: OrchestratorStrategy::GracefulDegradation,
                escalation_level: RecoveryEscalationLevel::Level1RestartComponent,
                timeout_secs: 15,
                requires_validation: true,
            },
            PlaybookStep {
                order: 2,
                description: "Select backup operator or robot".into(),
                strategy: OrchestratorStrategy::DelegateMission,
                escalation_level: RecoveryEscalationLevel::Level5MissionReassign,
                timeout_secs: 30,
                requires_validation: true,
            },
            PlaybookStep {
                order: 3,
                description: "Execute takeover with state transfer".into(),
                strategy: OrchestratorStrategy::TakeoverMission,
                escalation_level: RecoveryEscalationLevel::Level5MissionReassign,
                timeout_secs: 60,
                requires_validation: true,
            },
        ],
    }
}

fn parse_playbook_value(value: &toml::Value) -> Option<RecoveryPlaybook> {
    let table = value.as_table()?;
    let name = table.get("name")?.as_str()?.to_string();
    let version = table
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("1.0.0")
        .to_string();
    let description = table
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let trigger = table
        .get("trigger")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let entity_kinds = table
        .get("entity_kinds")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(EntityKind::parse))
                .collect()
        })
        .unwrap_or_default();
    let steps = table
        .get("steps")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .enumerate()
                .filter_map(|(i, step)| parse_playbook_step(step, i as u32 + 1))
                .collect()
        })
        .unwrap_or_default();
    Some(RecoveryPlaybook {
        name,
        version,
        description,
        trigger,
        steps,
        entity_kinds,
    })
}

fn parse_playbook_step(value: &toml::Value, default_order: u32) -> Option<PlaybookStep> {
    let table = value.as_table()?;
    let order = table
        .get("order")
        .and_then(|v| v.as_integer())
        .map(|n| n as u32)
        .unwrap_or(default_order);
    let description = table
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("recovery step")
        .to_string();
    let strategy = table
        .get("strategy")
        .and_then(|v| v.as_str())
        .map(|s| crate::policy::parse_strategy_for_playbook(s))
        .unwrap_or(OrchestratorStrategy::Retry);
    let escalation_level = table
        .get("escalation_level")
        .and_then(|v| v.as_integer())
        .and_then(|n| RecoveryEscalationLevel::from_u8(n as u8))
        .unwrap_or_else(|| strategy.default_escalation_level());
    let timeout_secs = table
        .get("timeout_secs")
        .and_then(|v| v.as_integer())
        .unwrap_or(30) as u64;
    let requires_validation = table
        .get("requires_validation")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    Some(PlaybookStep {
        order,
        description,
        strategy,
        escalation_level,
        timeout_secs,
        requires_validation,
    })
}
