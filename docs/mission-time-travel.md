# Mission Time Travel

**Status:** Planned · **Horizon:** LATER (6–12 months) · **Priority:** P2

Replay mission state at any point in time for incident investigation.

## CLI

```bash
spanda replay mission.trace --at 2026-06-24T14:32:00Z
spanda replay mission.trace --at 14:32:00 --inspect decisions
spanda replay mission.trace --at 14:32:00 --inspect health|readiness|safety
```

## Core types

`MissionTimeTravel`, `HistoricalMissionState`, `TimelineExplorer`.

## Capabilities

- Inspect robot/mission state at timestamp
- Inspect decisions (requires Decision Audit Trail v3 traces)
- Inspect health, readiness, and safety posture at point in time

Extends [replay.md](./replay.md) with state snapshots and decision records embedded in trace v3.

See [differentiation-roadmap.md](./differentiation-roadmap.md) · [decision-audit-trail.md](./decision-audit-trail.md).
