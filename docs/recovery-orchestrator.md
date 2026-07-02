# Recovery Orchestrator

The **Recovery Orchestrator** (`spanda-recovery`) is the platform-wide recovery intelligence for Spanda. It coordinates planning, simulation, validation, execution, evidence, and learning across every entity type — without replacing existing recovery APIs.

## Architecture

```
Detect → Diagnose → Plan → Validate → Execute → Verify → Audit
         ↑ existing assurance/fleet layers
         ↓ Recovery Orchestrator coordinates all stages
```

### Layer placement

| Component | Crate | Role |
|-----------|-------|------|
| **Recovery Orchestrator** | `spanda-recovery` | Planning, graph, policies, playbooks, decision engine |
| Recovery assurance | `spanda-assurance` | Legacy planner, validation gates, execution |
| Recovery types | `spanda-runtime` | Shared DTOs (`RecoveryPlan`, `RecoveryEvidence`, …) |
| Entity model | `spanda-config` | Universal recoverable entities |
| Fleet relay | `spanda-fleet` | Mesh recovery execution |
| Control Center | `spanda-api` | REST `/v1/recovery/*` |

### Core service

```rust
use spanda_recovery::RecoveryOrchestrator;

let orchestrator = RecoveryOrchestrator::new();
let report = orchestrator.plan_recovery(&program, &registry, resolved, &request);
```

### Responsibilities

- Recovery planning and strategy selection
- Dependency and impact analysis (recovery graph)
- Policy evaluation (TOML + program declarations)
- Playbook execution
- Predictive recovery from telemetry
- Validation through health, readiness, trust, security gates
- Immutable evidence generation
- Rule-based learning (historical statistics, no ML initially)

### Backward compatibility

- `spanda heal`, `spanda recover`, `POST /v1/programs/recovery/heal` unchanged
- Orchestrator wraps `spanda-assurance` — does not replace it
- Existing `RecoveryLevel` (autonomy) coexists with `RecoveryEscalationLevel` (0–8)

## Escalation levels

| Level | Name | Example strategies |
|-------|------|-------------------|
| 0 | Retry | `retry` |
| 1 | Restart component | `restart_component`, `graceful_degradation` |
| 2 | Restart package | `restart_package`, `switch_provider` |
| 3 | Recover device | `reinitialize`, `switch_sensor` |
| 4 | Recover robot | `restart_robot` |
| 5 | Mission reassignment | `transfer_mission`, `delegate_mission`, `takeover_mission` |
| 6 | Fleet redistribution | `restart_fleet`, `switch_fleet` |
| 7 | Human intervention | `human_escalation` |
| 8 | Emergency shutdown | `emergency_shutdown`, `safe_shutdown` |

## CLI

```bash
spanda recovery plan rover.sd --entity robot-1 --failure gps_loss
spanda recovery simulate rover.sd --failure sensor_failure
spanda recovery dry-run rover.sd --entity robot-1
spanda recovery execute rover.sd --force
spanda recovery validate rover.sd
spanda recovery history
spanda recovery metrics rover.sd
spanda recovery graph rover.sd --entity robot-1
spanda recovery playbooks
spanda recovery explain rover.sd --entity robot-1 --failure gps_loss
```

## Integration points

- **Entity Model** — all entities are recoverable via generic APIs
- **Health / Readiness / Trust** — validation gates
- **Diagnosis** — failure classification feeds decision engine
- **Mission Continuity** — delegation, takeover, succession strategies
- **Fleet** — fleet redistribution playbooks
- **Plugins** — `[recovery.extensions]` in `spanda.plugin.toml`; `on_recovery_completed` hook after execute
- **gRPC** — proto **1.0.8** mirrors REST (`ListRecoveryPlans`, `PlanRecovery`, …)
- **Control Center** — Recovery dashboard and graph visualization

## CI & promotion

```bash
./scripts/recovery_orchestrator_smoke.sh
./scripts/recovery_orchestrator_stable_promotion_gate.sh
```

See [stable-hardening-recovery-orchestrator.md](./stable-hardening-recovery-orchestrator.md).

## See also

- [recovery-policies.md](./recovery-policies.md)
- [recovery-playbooks.md](./recovery-playbooks.md)
- [recovery-graph.md](./recovery-graph.md)
- [recovery-simulation.md](./recovery-simulation.md)
- [predictive-recovery.md](./predictive-recovery.md)
- [recovery-api.md](./recovery-api.md)
- [recovery-sdk.md](./recovery-sdk.md)
- [self-healing.md](./self-healing.md)
