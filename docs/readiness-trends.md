# Readiness Trend Analysis

**Status:** Planned · **Phase:** Operate · **Priority:** P2.2

Predict readiness degradation from historical evaluations.

## Types

- `ReadinessHistory` — time-series of `ReadinessReport` snapshots
- `ReadinessTrend` — slope and volatility per factor
- `ReadinessForecast` — predicted score and risk window

## Storage

Local history file: `.spanda/readiness-history.json` (append on each `spanda readiness --record`).

## CLI

```bash
spanda readiness rover.sd --record
spanda readiness trends rover.sd
spanda readiness trends rover.sd --forecast 7d --json
```

## Output

- Readiness over time (per factor)
- Predicted readiness at horizon
- Risk warnings when forecast crosses policy threshold

## Integration

Extends `spanda-readiness` engine; feeds scorecard and deployment gate trends.

See [readiness.md](./readiness.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
