# Digital Mission Twin

**Status:** Planned · **Horizon:** LATER (6–12 months) · **Priority:** P2

Maintain a digital representation of mission state — progress, health, readiness, risks, and recovery status.

## Core types

| Type | Purpose |
|------|---------|
| `MissionTwin` | Live mission state mirror |
| `MissionStateModel` | Progress, checkpoints, objectives |
| `MissionRiskModel` | Active risks and forecasts |
| `MissionForecast` | Projected completion and degradation |

## Integration

Extends existing `twin` blocks and `spanda-readiness` twin module. Local twin in core; cloud sync via `spanda-twin-cloud` package (future).

Feeds What-If (NEXT) and Mission Time Travel (LATER).

See [differentiation-roadmap.md](./differentiation-roadmap.md) · [replay.md](./replay.md).
