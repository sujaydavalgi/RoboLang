# What-If Analysis

**Status:** Experimental · **Horizon:** NEXT (3–6 months) · **Priority:** P1

Predict mission outcomes under failure scenarios without executing on hardware.

## CLI

```bash
spanda what-if mission.sd
spanda what-if mission.sd --scenario gps_failure
spanda what-if mission.sd --all --json
```

## Scenarios

| Scenario | Simulation method |
|----------|-------------------|
| GPS failure | Fault injection + recovery planner |
| Battery failure | Battery model + mission duration |
| Connectivity loss | Network policy + offline mode |
| Robot failure | Health fault + continuity handoff |
| Fleet failure | Fleet mesh peer loss |
| Swarm failure | Swarm quorum breach |
| Provider failure | Provider dispatch timeout |
| Package failure | Package trust + fallback |

## Output

Per scenario: **Impact**, **Risk**, **Recovery Plan**, **Probability** (heuristic v1).

```json
{
  "scenario": "gps_failure",
  "impact": "navigation_degraded",
  "risk": "medium",
  "recovery_plan": "GpsRecovery → imu_dead_reckoning",
  "probability": 0.15,
  "mission_completion_likely": true
}
```

## Integration

Composes Recovery planner, Assurance resilience, Sim fault injection, Readiness degradation model. ML probability backends ship as `spanda-whatif-ml` package.

See [differentiation-roadmap.md](./differentiation-roadmap.md) · [chaos-engineering.md](./chaos-engineering.md).
