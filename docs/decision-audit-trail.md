# Decision Audit Trail

**Status:** Stable (trace parse v1) · **Horizon:** NOW · **Priority:** P0

Trace every important autonomous decision through the operational stack.

## Trace chain

```
Mission → Decision → Evidence → Safety Check → Action
```

## Core types

| Type | Purpose |
|------|---------|
| `DecisionRecord` | Single decision event with timestamp, actor, and outcome |
| `DecisionEvidence` | Sensor readings, planner output, context snapshots |
| `DecisionTimeline` | Ordered sequence of decisions for a mission session |
| `DecisionChain` | Linked chain from mission intent to executed action |

## Decision record schema (v3 trace)

```json
{
  "version": 3,
  "decision_id": "d-20260624-143200-001",
  "mission": "WarehousePatrol",
  "timestamp": "2026-06-24T14:32:00.123Z",
  "decision": "execute_motion",
  "reason": "planner_proposal_within_safety_bounds",
  "evidence": {
    "lidar_nearest_m": 2.4,
    "proposal_speed_mps": 0.8,
    "battery_pct": 72
  },
  "alternatives_considered": [
    { "action": "stop", "rejected_reason": "path_clear" },
    { "action": "reduce_speed", "rejected_reason": "already_below_cap" }
  ],
  "safety_checks": [
    { "rule": "max_speed", "passed": true },
    { "rule": "stop_if_obstacle", "passed": true }
  ],
  "action": { "type": "SafeAction", "actuator": "wheels", "command": "forward" }
}
```

## Emission

- Enabled during `run` / `sim` when `SPANDA_DECISION_TRACE=1` (default on with `--record`)
- Distributed decision layers emit v3 payloads at continuity, recovery, safety reflex (kill switch, emergency stop, `safety.validate` rejection), live `decision_tree` evaluation during sim, and fleet mesh consensus when `SPANDA_DECISION_TRACE=1` or `--record` (see [distributed-decisions.md](./distributed-decisions.md))
- Stored in mission trace files and telemetry store
- Audit records cross-reference via `decision_id`

## CLI

```bash
spanda audit decisions mission.trace
spanda audit decisions mission.trace --json
spanda decision trace mission.trace              # distributed decision timeline
spanda decision explain mission.trace            # plain-language explanations
spanda explain mission.trace                     # human-readable decision explanations
```

## Integration

| Engine | Role |
|--------|------|
| Explainability | `spanda explain` renders decision chains |
| Replay | Time-travel inspects decisions at timestamp (LATER) |
| Audit | `spanda-audit` persists decision records |
| Assurance | Diagnosis links failures to decision points |
| Safety | Safety check results embedded in records |
| Recovery | Recovery triggers reference triggering decision |

## Crate

`spanda-decision` — trace schema, emission hooks, audit integration.

See [differentiation-roadmap.md](./differentiation-roadmap.md) · [explainability.md](./explainability.md) · [replay.md](./replay.md) · [distributed-decisions.md](./distributed-decisions.md).
