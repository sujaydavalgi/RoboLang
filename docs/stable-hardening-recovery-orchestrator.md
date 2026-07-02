# Stable hardening — Recovery Orchestrator

Checklist for **Stable** tier promotion of the Recovery Orchestrator (`spanda-recovery`, REST `/v1/recovery/*`, gRPC proto **1.0.8**).

## Gate script

```bash
./scripts/recovery_orchestrator_stable_promotion_gate.sh
```

Runs:

1. `scripts/recovery_orchestrator_smoke.sh` — crate, REST API, gRPC, and CLI
2. Control Center probe — `GET /v1/recovery/playbooks`, `GET /v1/recovery/history`, `POST /v1/recovery/plan`

Skip smoke only: `SPANDA_RECOVERY_SKIP_SMOKE=1 ./scripts/recovery_orchestrator_stable_promotion_gate.sh`

## Smoke (CI)

```bash
./scripts/recovery_orchestrator_smoke.sh
```

## Test locations

| Layer | Location |
|-------|----------|
| Orchestrator unit tests | `crates/spanda-recovery/tests/orchestrator_tests.rs` |
| REST contract tests | `crates/spanda-api/tests/recovery_api_tests.rs` |
| gRPC parity | `crates/spanda-api/tests/grpc_tests.rs` (`grpc_recovery_endpoints_with_self_healing_program`) |
| Legacy self-healing | `scripts/self_healing_smoke.sh` |

## Backward compatibility

- `spanda heal`, `spanda recover`, `POST /v1/programs/recovery/heal` unchanged
- Orchestrator wraps assurance — no API breaks

## See also

- [recovery-orchestrator.md](./recovery-orchestrator.md)
- [recovery-api.md](./recovery-api.md)
- [test-plan.md](./test-plan.md)
