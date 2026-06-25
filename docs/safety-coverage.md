# Safety Coverage

**Status:** Stable · **Horizon:** NOW · **Priority:** P0

Coverage reporting for safety scenarios — identify what is protected, partially protected, or uncovered before deploy.

## CLI

```bash
spanda safety-coverage rover.sd
spanda safety-coverage rover.sd --json
spanda safety-coverage rover.sd --format markdown
```

## Scenarios evaluated

| Scenario | Detection source |
|----------|------------------|
| Obstacle avoidance | `safety { stop_if }`, lidar rules, `simulate_compatibility` |
| GPS failure | `recovery_policy`, connectivity policies, hardware assumptions |
| Battery failure | Battery estimation, `stop_if` thresholds, recovery actions |
| Connectivity failure | `requires_network`, failover policies, recovery modes |
| Provider failure | Provider dispatch, fallback handlers, package trust |
| Takeover failure | `continuity_policy`, succession ranking, checkpoint coverage |
| Recovery failure | `recovery_policy` completeness, assurance recovery gates |

## Core types

| Type | Purpose |
|------|---------|
| `SafetyCoverageReport` | Full report with per-scenario status |
| `SafetyScenario` | Named scenario with evaluation criteria |
| `CoverageGap` | Uncovered or partial scenario with remediation hints |

## Output

Per scenario: **Covered** · **Partially Covered** · **Uncovered**

```json
{
  "overall_coverage_pct": 71,
  "scenarios": [
    { "name": "obstacle_avoidance", "status": "covered", "evidence": ["safety.stop_if lidar"] },
    { "name": "gps_failure", "status": "partially_covered", "gaps": ["no_imu_fallback"] },
    { "name": "takeover_failure", "status": "uncovered", "gaps": ["no_continuity_policy"] }
  ],
  "recommendations": [
    "Add continuity_policy with auto_takeover for takeover_failure",
    "Add recovery_policy on gps_loss with imu_dead_reckoning fallback"
  ]
}
```

## Integration

| Engine | Role |
|--------|------|
| Readiness | Safety coverage factor in go/no-go score |
| Safety auditor | Rule inventory for obstacle/collision scenarios |
| Recovery | Recovery policy coverage for failure scenarios |
| Continuity | Takeover/succession coverage |
| Simulation | `simulate_compatibility` fault catalog |
| Mission Contracts | `safety:` clause alignment (NOW) |

## Implementation

Extends `spanda-readiness` safety analysis — no duplicate safety logic. Scenario catalog ships in core; vendor-specific scenarios as packages.

See [differentiation-roadmap.md](./differentiation-roadmap.md) · [safety-auditor.md](./safety-auditor.md) · [minimum-hardware-safety.md](./minimum-hardware-safety.md).
