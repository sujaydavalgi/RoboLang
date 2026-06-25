# Mission Contracts

**Status:** Stable (static analysis v1) · **Horizon:** NOW

Make missions **verifiable first-class entities** with explicit guarantees, constraints, assumptions, invariants, and objectives.

## Language syntax

```spanda
mission WarehousePatrol {
    guarantees:
        all_checkpoints_visited;
        mission_completed_within_duration;

    safety:
        no_collision;
        within_geofence;

    continuity:
        auto_takeover;

    constraint max_duration = 2 h;
    constraint max_speed = 1.5 m/s;

    assumption gps_available;
    assumption connectivity_min = 50%;

    invariant battery_level > 20%;
    invariant lidar_operational;

    objective visit_all_zones;
    objective return_to_base;
}
```

## Core types

| Type | Purpose |
|------|---------|
| `MissionContract` | Root contract binding mission name to all clauses |
| `MissionGuarantee` | Post-conditions the mission must satisfy on completion |
| `MissionConstraint` | Hard limits (duration, speed, resource bounds) |
| `MissionAssumption` | Preconditions assumed true at mission start |
| `MissionInvariant` | Conditions that must hold throughout execution |
| `MissionObjective` | Goals to achieve (may be partial on abort) |

## CLI

```bash
spanda contract verify warehouse.sd
spanda contract verify warehouse.sd --json
spanda mission verify warehouse.sd    # extends existing mission verify
```

## Verification scope

| Check | Source |
|-------|--------|
| Guarantee achievability | Mission planner + capability traceability |
| Assumption coverage | Hardware verify + provider availability |
| Constraint feasibility | Timing budgets, battery estimation |
| Invariant enforceability | Safety rules, health policies |
| Continuity alignment | `continuity_policy` declarations |
| Objective traceability | Capability matrix |

## Integration

| Engine | Role |
|--------|------|
| Readiness | Contract completeness factor in go/no-go |
| Assurance | Links to `assurance_case` evidence |
| Continuity | `continuity:` clause maps to `continuity_policy` |
| Capability verify | Objectives → required capabilities |
| Hardware verify | Assumptions → hardware profile fit |
| Recovery | Constraint violations → recovery triggers |

## Crate

`spanda-contract` — parser extensions, static analysis, readiness/assurance hooks.

## Package backends

| Package | Role |
|---------|------|
| `spanda-mission-planning` | Planner feasibility for objectives |
| `spanda-mission-continuity` | Continuity clause validation |

See [differentiation-roadmap.md](./differentiation-roadmap.md) · [mission-verification.md](./mission-verification.md) · [mission-continuity.md](./mission-continuity.md).
