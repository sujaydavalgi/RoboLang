# Deployment Gates

**Status:** Experimental · **Phase:** Deploy · **Priority:** P0.2

Prevent unsafe deployment when operational gates fail.

## Types

- `DeploymentGatePolicy` — named thresholds (`default`, `production`)
- `DeploymentGate` — single pass/fail check with message
- `DeploymentGateReport` — composite gate result

## Example gates

| Gate | Condition |
|------|-----------|
| Readiness | Score ≥ threshold and `mission_ready` |
| Safety | Safety audit has no critical/high findings |
| Capability | Capability traceability matrix PASS |
| Package trust | Configured packages meet trust threshold (with `--config`) |
| Official provenance | **Production:** no official package name path/git overrides (`official_provenance`) |
| Registry signatures | **Production:** `SPANDA_REGISTRY_REQUIRE_SIGNATURE=1` and valid lockfile registry signatures |
| Composite trust | Program composite trust score ≥ 60 (`spanda-trust`) |
| Secure boot | Secure-boot contract trust passes when `trust.jetson` / `trust.pi` imported |
| Operational policy | Named `policy { }` passes verify-time rules (`--operational-policy <name>`) |
| Health | No high-severity health readiness issues |

## CLI

```bash
spanda deploy gate rover.sd
spanda deploy gate rover.sd --policy production
spanda deploy gate rover.sd --operational-policy WarehousePolicy
spanda deploy gate rover.sd --json --config spanda.toml
```

**Production policy** (`--policy production`) additionally requires:

- No official package name overrides without registry provenance (no `spanda-mqtt = { path = "../evil" }`)
- `SPANDA_REGISTRY_REQUIRE_SIGNATURE=1` in the environment
- Every registry dependency in `spanda.lock` verifies against signed checksums in `registry/index.json`

Example CI:

```bash
export SPANDA_REGISTRY_REQUIRE_SIGNATURE=1
spanda deploy gate src/main.sd --policy production --config spanda.toml
```

Deployment is **blocked** (exit code 1) when any gate fails.

## Foundation

Extends `spanda-readiness` (`evaluate_deployment_gates`), safety auditor, and capability traceability. Complements `deploy rollout --require-certify`.

## Integration

Composes `spanda-readiness`, capability verification, health framework, and assurance evidence.

See [readiness.md](./readiness.md) · [ci-verify.md](./ci-verify.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
