# Human / Robot Teaming

**Status:** Planned · **Horizon:** LATER (6–12 months) · **Priority:** P2

Support collaborative autonomy with verified approval, escalation, and fallback paths.

## Core types

| Type | Purpose |
|------|---------|
| `HumanApproval` | Required human sign-off before action |
| `HumanOverride` | Operator override of autonomous decision |
| `HumanEscalation` | Escalation chain on critical events |
| `HumanReview` | Post-hoc review request |

## Example

```spanda
mission Patrol {
    requires approval Operator;

    escalation on critical_fault -> Supervisor;
    fallback on approval_timeout -> enter_safe_mode;
}
```

## Verification

| Path | Validated by |
|------|--------------|
| Approval path | `requires approval` + recovery approval hooks |
| Escalation path | Escalation chain completeness |
| Fallback path | Recovery policy on timeout |

Builds on existing `requires approval`, `SPANDA_OPERATOR_APPROVAL`, and Recovery approval integration. Console UI ships as `spanda-approval-console` package.

See [differentiation-roadmap.md](./differentiation-roadmap.md) · [self-healing.md](./self-healing.md) · [mission-contracts.md](./mission-contracts.md).
