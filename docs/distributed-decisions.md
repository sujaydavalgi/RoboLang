# Distributed Decision Architecture

Spanda implements a **brain / spinal cord / reflex** model for hierarchical distributed autonomy. Decisions are made at the appropriate layer — closest to the device for safety and latency, centrally for strategy and governance.

## Decision layers

| Layer | Name | Latency | Purpose |
|-------|------|---------|---------|
| 0 | **Reflex** | milliseconds | Immediate safety (stop motor, e-stop, reject untrusted signal) |
| 1 | **Local Entity** | ms–seconds | Bounded local autonomy (degraded mode, sensor failover, offline continue) |
| 2 | **Group / Fleet** | seconds | Multi-entity coordination (reassign, delegate, swarm consensus) |
| 3 | **Control Center** | seconds–minutes | Strategy, policy, assurance, human approval |

## Quality rules

Local decisions **never bypass**:

- Safety validation
- Kill switch
- Trust policy
- Capability verification
- Hardware limits

Central orchestration **must not block** immediate safety reflexes.

Offline operation is **bounded by signed policy** with expiration.

Every distributed decision is **auditable and replayable**.

## Language syntax

Declare per-entity authority inside `robot` blocks:

```sd
robot Rover001 {
    local_decision_authority [emergency_stop, degraded_mode, return_home];
    requires_central_approval [override_safety_policy, update_firmware];
}
```

Define local decision trees:

```sd
decision_tree GPSLossRecovery local {
    when gps.status == Failed {
        if visual_odometry.available { enter degraded_mode; }
        else { pause_mission; }
    }
}
```

Define offline bounds:

```sd
offline_policy RoverOffline {
    max_duration = 30 min;
    allowed_actions [continue_current_safe_mission, return_home];
    forbidden_actions [disable_safety, accept_unknown_device];
}
```

## Runtime emission

During `run` / `sim` with `SPANDA_DECISION_TRACE=1` or `--record`:

| Event | Layer | When |
|-------|-------|------|
| `kill_switch_activated` | reflex | Kill switch activation |
| `emergency_stop` | reflex | `emergency_stop` statement or scheduler halt |
| `safety_validate_rejected` | reflex | `safety.validate()` rejects a proposal |
| `decision_tree_eval` | local / fleet | Live tree match on health-fault injection or scheduler poll |
| `decision_action_blocked` / `decision_escalation_pending` | local / control_center | Offline policy or `requires_central_approval` gate at action dispatch |
| `continuity_takeover` | local / fleet | Continuity handoff |
| `fleet_mesh_continuity` / `fleet_mesh_recovery` | group_fleet | Fleet mesh relay with consensus trace |

`DecisionRuntime` is injected from CLI (`DecisionBackedRuntime`); TypeScript `collectDecisionDiagnostics` mirrors Rust `collect_decision_diagnostics` for `spanda check --readiness-json`.

## CLI

```bash
spanda decision list mission.sd
spanda decision inspect mission.sd --entity Rover001
spanda decision simulate mission.sd --offline
spanda decision trace mission.trace
spanda decision explain mission.trace
spanda decision policy mission.sd
spanda decision sign-policy mission.sd --policy RoverOffline --write-cache
spanda decision cache show|sync|clear ...
spanda decision security-audit
```

## API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v1/decisions` | List authorities, trees, offline policies |
| GET | `/v1/entities/{id}/decisions` | Entity-scoped decision evaluation |
| POST | `/v1/decisions/simulate` | Simulate under failure scenarios |
| POST | `/v1/decisions/escalate` | Approve pending escalation |
| GET | `/v1/decision-policies` | Offline policy specs from loaded program |
| GET | `/v1/decisions/traces` | List v3 decision frames from mission trace (`?file=` or `?trace=`) |
| GET | `/v1/decision-policy-cache` | Persisted signed offline policy cache on disk |
| POST | `/v1/programs/simulation` | Run sim; set `decision_trace` and `record_trace` for v3 frames |

Env knobs for runtime gates in sim: `SPANDA_CENTRAL_CONNECTED`, `SPANDA_OFFLINE_MINUTES`, `SPANDA_DECISION_ESCALATION_APPROVED`.

## SDK

```rust
client.list_decisions()?;
client.list_decision_traces(None, None)?;
client.list_decision_policy_cache(None)?;
client.get_entity_decisions("Rover001")?;
client.simulate_decision(&body)?;
client.approve_escalation(&body)?;
```

## Related docs

- [Local decision trees](./local-decision-trees.md)
- [Decision authority](./decision-authority.md)
- [Offline decisions](./offline-decisions.md)
- [Decision traceability](./decision-traceability.md)
- [Conflict resolution](./decision-conflict-resolution.md)
- [Decision audit trail](./decision-audit-trail.md)

## Examples

`examples/showcase/distributed_decisions/` — GPS loss recovery, obstacle reflex, offline continue, fleet takeover, swarm consensus, control center escalation.
