# Recovery Coverage

**Status:** Planned · **Horizon:** NOW (0–3 months) · **Priority:** P0

Measure recovery readiness — which known failures have recovery plans and which paths are missing.

## CLI

```bash
spanda recovery-coverage rover.sd
spanda recovery-coverage rover.sd --json
spanda recovery-coverage rover.sd --format markdown
```

## Core types

| Type | Purpose |
|------|---------|
| `RecoveryCoverageReport` | Full report with coverage percentage |
| `RecoveryScenario` | Named failure mode with expected recovery |
| `RecoveryGap` | Failure without adequate recovery path |

## Known failure catalog (v1)

Derived from hardware profile, capability matrix, connectivity policies, and fleet configuration:

- GPS loss, battery critical, connectivity loss
- Sensor failure (lidar, IMU, camera)
- Actuator failure, provider timeout
- Fleet peer loss, swarm member failure
- Package/provider unavailable
- Human approval timeout

## Output

```json
{
  "coverage_pct": 64,
  "known_failures": 11,
  "covered": 7,
  "partially_covered": 2,
  "uncovered": 2,
  "recovery_plans": [
    { "failure": "gps_loss", "policy": "GpsRecovery", "actions": ["enter_degraded_mode", "imu_navigate"] }
  ],
  "missing_paths": [
    { "failure": "swarm_member_loss", "recommendation": "Add continuity_policy with succession" }
  ]
}
```

## Integration

| Engine | Role |
|--------|------|
| Recovery planner | Maps `recovery_policy` to failure modes |
| Assurance resilience | `resilience_policy` completeness |
| Continuity | Reassign/takeover as recovery paths |
| Readiness | Recovery coverage factor in go/no-go |
| Chaos (platform-maturity) | Validates coverage under injected faults (NEXT) |
| Safety Coverage | Complementary — safety scenarios vs recovery paths |

## Implementation

Extends `spanda-assurance` recovery module — composes `RecoveryPlanner` and failure classification. No duplicate recovery logic.

See [differentiation-roadmap.md](./differentiation-roadmap.md) · [self-healing.md](./self-healing.md) · [recovery-policies.md](./recovery-policies.md).
