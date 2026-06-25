# Policy Engine

**Status:** Experimental (verify-time) · **Phase:** Verify, Operate · **Priority:** P1.5 (verify-time), P3.4 (runtime)

Declarative operational rules enforced at verification and during run/sim when `--enforce-policy` is set.

## Syntax

```spanda
policy WarehousePolicy {
    max_speed = 1.0 m/s;
    requires_kill_switch;
    requires_capability [ gps_navigation, obstacle_avoidance ];
    min_readiness_score = 80;
    operation_hours = "06:00-22:00";
}
```

### Rules (verify-time)

| Rule | Checks |
|------|--------|
| `max_speed = N m/s` | Every robot `safety { max_speed }` is at or below the policy limit |
| `requires_kill_switch` | Program declares at least one `kill_switch` |
| `requires_capability [ … ]` | Every robot `exposes capabilities` includes each listed capability |
| `min_readiness_score = N` | Readiness rollup score meets the threshold |
| `operation_hours = "HH:MM-HH:MM"` | Range format is valid (clock enforcement is runtime, P3.4) |

`policy Name { }` is distinct from swarm `policy round_robin;` — the parser disambiguates by block vs statement form.

## CLI

```bash
spanda verify examples/showcase/policy/warehouse.sd --policy WarehousePolicy
spanda verify examples/showcase/policy/warehouse.sd --policy WarehousePolicy --json
spanda sim examples/showcase/policy/warehouse.sd --enforce-policy WarehousePolicy
spanda run examples/showcase/policy/warehouse.sd --enforce-policy WarehousePolicy
```

Policy evaluation runs after hardware compatibility when `--policy` is set. Failures surface as policy violations in the verify report.

## Core types

- `OperationalPolicyDecl` — named rule set (AST)
- `OperationalPolicyRule` — individual constraint variant
- `PolicyViolation` — failed rule with severity and message
- `PolicyEvaluationReport` — pass/fail rollup for one policy

## Enforcement phases

| Phase | When | Command | Status |
|-------|------|---------|--------|
| 1 | Verify-time | `spanda verify --policy WarehousePolicy` | **Experimental** |
| 2 | Readiness | Policy factor in readiness score | Planned |
| 3 | Runtime | Policy monitor in interpreter (`--enforce-policy`) | **Experimental** |

## Integration

Composes with `spanda-readiness`, `spanda-security`, safety rules, and deployment gates.

## Crate

`spanda-policy` — AST types in `spanda-ast::policy_decl`, parser in `spanda-parser`, evaluator in `evaluate_policy`.

Showcase: `examples/showcase/policy/warehouse.sd` · smoke: `scripts/policy_smoke.sh`

See [deployment-gates.md](./deployment-gates.md) · [compliance-profiles.md](./compliance-profiles.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
