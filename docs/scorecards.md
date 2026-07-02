# Autonomous Systems Scorecard

**Status:** Experimental · **Phase:** Operate · **Priority:** P1.4

Executive-level visibility into system health across platform pillars.

## CLI

```bash
spanda score rover.sd
spanda score examples/showcase/scorecard/executive.sd --json
spanda score rover.sd --format markdown
```

## Categories

| Category | Source engine |
|----------|---------------|
| Safety | Safety auditor, minimum-hardware analysis |
| Health | Health checks, runtime health |
| Readiness | `evaluate_readiness` |
| Security | Threat model, composite program trust, secure-boot coverage (when `trust.jetson` / `trust.pi` imported) |
| Resilience | Assurance recovery + chaos results |
| Verification | `spanda verify` |
| Assurance | `spanda assure` evidence |

## Output

- Per-category scores (0–100)
- Overall weighted score
- Top recommendations (actionable, linked to `spanda explain`)

## Integration

Pure composition layer — calls existing engines; no duplicate scoring logic.

## Crate

`spanda-score` — rollup over readiness, assurance, threat, trust.

See [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
