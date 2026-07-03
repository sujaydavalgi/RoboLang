# Distributed Decision Security

Security validation, attack simulations, and enforcement status for Spanda's hierarchical distributed decision architecture.

**Status: Stable** — live attack simulations exercise enforcement code paths with constructed adversarial inputs.

## What is production-ready

| Capability | Status | Evidence |
|------------|--------|----------|
| Reflex safety without central dependency | **Tested** | `rule_enforcement::reflex_safety_actions_run_without_central_approval` |
| Signed offline policy verification | **Tested** | Ed25519 via `spanda-audit`; `sign-policy`, `cache sync` |
| Offline duration and forbidden action gates | **Tested** | `validate_offline_action`, runtime bridge |
| Central approval for high-risk actions | **Tested** | `requires_central_approval` + escalation trace |
| Nonce replay rejection | **Tested** | `NonceRegistry`, `register_decision_nonce` |
| Policy tamper detection | **Tested** | Signature mismatch on tampered payload |
| Split-brain precedence resolution | **Tested** | `resolve_conflict` + `CONFLICT_PRECEDENCE` |
| v3 decision trace proof fields | **Tested** | `validate_decision_trace_payload` |
| Live attack simulations | **Tested** | `run_attack_simulation`, CI smoke |

## What is simulated (not live adversarial)

Attack simulations exercise **enforcement code paths** with constructed adversarial inputs. They do not spin up fleet mesh, inject network packets, or compromise real devices.

| Scenario | CLI | What runs |
|----------|-----|-----------|
| Policy tampering | `spanda decision simulate-attack policy-tamper` | Sign policy, tamper allowed_actions, verify signature fails |
| Replayed decision | `spanda decision simulate-attack replayed-decision` | Register nonce twice, second rejected |
| Fake coordinator | `spanda decision simulate-attack fake-coordinator` | Untrusted entity blocked from takeover |
| Offline abuse | `spanda decision simulate-attack offline-abuse` | Duration exceeded + forbidden action blocked |
| Compromised robot | `spanda decision simulate-attack compromised-robot` | `disable_safety` forbidden while offline |
| Poisoned telemetry | `spanda decision simulate-attack poisoned-telemetry` | Stale timestamp rejected |
| Split-brain coordinator | `spanda decision simulate-attack split-brain-coordinator` | Safety precedence wins |

All simulations exit non-zero when the unsafe decision is **not** blocked.

## Signed policy validation

Offline policies use canonical JSON signing via `spanda-audit`:

```bash
export SPANDA_DECISION_POLICY_SIGNING_KEY=<material>
spanda decision sign-policy mission.sd --policy RoverOffline --write-cache
export SPANDA_DECISION_REQUIRE_SIGNED_OFFLINE_POLICY=1
export SPANDA_DECISION_POLICY_TRUST_KEY=<material>
```

Enforcement checks:

- **Policy version** — pinned in envelope and offline policy spec
- **Policy hash** — fingerprint of signing payload (`policy_hash`)
- **Policy expiration** — `expires_at_ms` on offline policy
- **Decision tree hash** — fingerprint match via `verify_decision_tree_hash` (not cryptographic)
- **Decision nonce** — replay protection via global registry
- **Decision timestamp** — staleness bounds via `validate_decision_timestamp`
- **Authority scope** — layer alignment via `validate_authority_scope`

**Note:** Decision tree hash is a non-crypto fingerprint for tamper detection in tests and traces. Offline policy signatures use real Ed25519 — do not mock in production paths.

## Rule enforcement tests

Run the full enforcement suite:

```bash
cargo test -p spanda-decision --test rule_enforcement
cargo test -p spanda-decision --test attack_simulations
./scripts/distributed_decisions_smoke.sh
```

Tests prove:

1. Reflex safety actions run without central approval
2. Local decisions cannot bypass safety validation
3. Local decisions cannot bypass kill switch
4. Local decisions cannot bypass trust policy
5. High-risk actions require approval/quorum/central authorization
6. Offline decisions expire
7. Cached policies must be signed when required
8. Decision tree hash must match
9. Replayed decisions are rejected
10. Untrusted entities cannot issue takeover decisions
11. Split-brain conflicts resolve using documented precedence

## Threat model

```bash
spanda decision threat-model
spanda decision security-audit [--json]
```

Static catalog documents mitigations. Live simulations (`simulate-attack`) produce JSON evidence.

## Production-ready controls

| Control | Status |
|---------|--------|
| Runtime conflict resolution | **Stable** — wired in interpreter |
| Persistent escalation store | **Stable** — disk + API + Control Center |
| Decision tree Ed25519 signing | **Stable** — `sign-tree` + cache |
| Persisted nonce registry | **Stable** — disk-backed replay protection |
| v3 envelope signatures | **Stable** — Ed25519 when signing key configured |
| Fleet mesh conflict aggregation | **Stable** — `POST /v1/fleet/decisions/vote/ingest`, `GET /v1/fleet/decisions/conflicts` |
| Shared fleet nonce registry | **Stable** — `POST /v1/fleet/decisions/nonce/register` (+ local mirror) |
| Pluggable signing backend | **Stable** — `SPANDA_CRYPTO_BACKEND=software|mock_hsm`, `SPANDA_DECISION_SIGNING_KEY_ID` |

## Former enhancements (resolved)

| Enhancement | Resolution |
|-------------|------------|
| Fleet mesh coordinator aggregation | Mesh ingest + conflict resolution; runtime uses coordinator winner when `SPANDA_FLEET_MESH_URL` set |
| Distributed nonce store | Shared mesh registry with local file fallback |
| Hardware-backed signing keys | `sign_with_backend` abstraction in `spanda-audit` (`mock_hsm` for CI; production HSM via key id) |

## Former gaps (resolved)

| Gap | Resolution |
|-----|------------|
| `resolve_conflict` not wired at runtime | Wired in `evaluate_live_decision_trees` |
| Escalation approval in-memory only | `PersistedEscalationStore` + API + Control Center |
| Decision tree signing | Ed25519 `sign-tree` + cache merge |
| Global nonce registry | `.spanda/decision-nonce-registry.json` |
| Crypto signature on v3 envelope | Ed25519 envelope signing + verification |

## CI

The `distributed-decisions` CI job runs:

- `cargo test -p spanda-decision`
- Rule enforcement and attack simulation tests
- Attack simulation CLI commands
- GPS loss recovery flagship demo
- Decision trace validation and audit

See `.github/workflows/ci.yml` job `distributed-decisions`.

## Related

- [distributed-decisions.md](./distributed-decisions.md)
- [distributed-decision-demo.md](./distributed-decision-demo.md)
- [decision-traceability.md](./decision-traceability.md)
- [offline-decisions.md](./offline-decisions.md)
