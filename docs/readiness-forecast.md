# Readiness Forecasting

**Status:** Experimental · **Horizon:** NEXT (3–6 months) · **Priority:** P1

Predict future readiness scores from current evaluation, local history, and time-based degradation heuristics.

## CLI

```bash
spanda readiness forecast rover.sd
spanda readiness forecast rover.sd --target RoverV1
spanda readiness forecast rover.sd --horizon 14d --json
spanda readiness forecast rover.sd --all
spanda readiness forecast rover.sd --history .spanda/readiness-history.json
```

## Types

- `ReadinessPrediction` — projected score at a horizon with risks
- `ReadinessForecastReport` — current score, degradation rate, and predictions
- `ReadinessHistory` / `ReadinessForecast` — from [readiness-trends.md](./readiness-trends.md)

## Evaluates

- Current readiness (`evaluate_readiness` with simulation)
- Historical slope when `.spanda/readiness-history.json` has samples
- Heuristic degradation from readiness issues, safety coverage gaps, and factor weakness

## Output

Per horizon: current → predicted score, degradation rate, projected risks, policy warnings (exit 1 when below minimum).

## Integration

Extends `spanda-readiness` trends module; feeds scorecard and deployment gate signals. ML backends ship as `spanda-readiness-ml` package.

See [differentiation-roadmap.md](./differentiation-roadmap.md) · [readiness-trends.md](./readiness-trends.md).
