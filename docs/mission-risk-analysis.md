# Mission Risk Analysis

**Status:** Stable · **Horizon:** NEXT (promoted v0.5.0) · **Priority:** P1

Assess deployment risk before field operation.

## CLI

```bash
spanda risk mission.sd
spanda risk mission.sd --json
```

## Core types

| Type | Purpose |
|------|---------|
| `MissionRiskAssessment` | Full risk report |
| `MissionRiskScore` | Composite 0–100 score (higher = more risk) |
| `MissionRiskFactor` | Individual contributor with weight |

## Risk contributors (v1)

- Safety coverage gaps
- Recovery coverage gaps
- Readiness score below threshold
- Trust score below threshold
- Unverified assumptions in mission contract
- Hardware margin (memory, battery, timing)
- Fleet/swarm dependency risk
- Provider/package trust

## Output

Risk Score, Risk Contributors, Recommended Mitigations — linked to `spanda explain` and coverage reports.

## Crate

`spanda-risk` — composes readiness, assurance, contract verify, safety/recovery coverage.

See [differentiation-roadmap.md](./differentiation-roadmap.md) · [scorecards.md](./scorecards.md).
