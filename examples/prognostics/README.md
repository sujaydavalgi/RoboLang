# Prognostics examples

Remaining useful life (RUL) and degradation warnings via `prognostics` declarations.

## Files

| File | Focus |
|------|--------|
| [`battery_degradation.sd`](battery_degradation.sd) | Battery RUL prediction and warn thresholds |

## Commands

```bash
spanda check examples/prognostics/battery_degradation.sd
spanda prognostics examples/prognostics/battery_degradation.sd --json
```

Guide: [docs/prognostics.md](../../docs/prognostics.md) · Package: `spanda-prognostics` (`assurance.prognostics`)
