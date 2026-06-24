# Policy Engine

**Status:** Planned · **Phase:** Verify, Operate · **Priority:** P1.5 (verify-time), P3.4 (runtime)

Declarative operational rules enforced at verification and, later, at runtime.

## Syntax (planned)

```spanda
policy WarehousePolicy {
    max_speed = 2 m/s;
    operation_hours = 06:00-22:00;
}
```

## Core types

- `Policy` — named rule set
- `PolicyRule` — individual constraint
- `PolicyViolation` — failed rule with context

## Enforcement phases

| Phase | When | Command |
|-------|------|---------|
| 1 | Verify-time | `spanda verify --policy WarehousePolicy` |
| 2 | Readiness | Policy factor in readiness score |
| 3 | Runtime | Policy monitor in interpreter (feature-gated) |

## Integration

Composes with `spanda-readiness`, `spanda-security`, safety rules, and deployment gates.

## Crate

`spanda-policy` — parser extension + rule evaluator.

See [deployment-gates.md](./deployment-gates.md) · [compliance-profiles.md](./compliance-profiles.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
