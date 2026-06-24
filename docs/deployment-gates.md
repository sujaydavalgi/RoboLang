# Deployment Gates

**Status:** Planned · **Phase:** Deploy · **Priority:** P0.2

Prevent unsafe deployment when operational gates fail.

## Types

- `DeploymentPolicy` — named set of gate rules
- `DeploymentGate` — single pass/fail check with threshold

## Example gates

| Gate | Condition |
|------|-----------|
| Readiness | Score > 90 |
| Safety | Safety audit PASS |
| Health | All `health_check` healthy |
| Capability verification | Traceability matrix PASS |
| Certify | `spanda certify prove` valid (extends `--require-certify`) |

## CLI

```bash
spanda deploy gate rover.sd
spanda deploy gate rover.sd --policy production
spanda deploy rollout plan.json --gate
```

Deployment is **blocked** when any required gate fails.

## Foundation

Extends existing `deploy rollout --require-certify`, readiness `mission_ready`, and safety auditor.

## Integration

Composes `spanda-readiness`, capability verification, health framework, and assurance evidence.

See [readiness.md](./readiness.md) · [ci-verify.md](./ci-verify.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
