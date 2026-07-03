# Test Coverage Plan

## Rust (`cargo test --workspace`)

| Area | Tests | Location |
|------|-------|----------|
| Lexer | unit suffixes, keywords, `%` | `lexer.rs` |
| Parser | robot, HAL, AI, foundations, hardware | `parser.rs`, `tests/foundations.rs` |
| Type checker | units, safety, capabilities | `types.rs`, `tests/type_system.rs` |
| Runtime | match, tasks, interpreter, contracts | `runtime.rs`, `tests/runtime_hardening.rs` |
| Hardware verify | sensors, timing, power, faults, matrix | `hardware.rs`, `tests/hardware_compat.rs` |
| Scheduler | multi-task multiplex | `tests/scheduler.rs` |
| Fusion | observe + fusion | `tests/fusion.rs` |
| Twin replay | mirror, replay frames | `tests/twin_replay.rs` |
| Integration | all `examples/*.sd` compile + run | `tests/integration.rs` |
| Continuity | runtime takeover, checkpoints, CLI JSON, auto-trigger | `crates/spanda-interpreter/tests/continuity_runtime.rs`, `crates/spanda-cli/tests/continuity_cli.rs`, `crates/spanda-assurance/src/continuity_checkpoint.rs` |
| Distributed decisions | decision trees, offline policy, authority, simulation | `crates/spanda-decision/tests/distributed.rs`, `spanda decision list|inspect|simulate` |
| Decision runtime gates | offline policy block, central approval escalation | `crates/spanda-interpreter/tests/decision_runtime.rs` (5 tests) |
| Decision trace emission | v3 payloads, live trees, smoke on showcase | `scripts/distributed_decisions_smoke.sh`, `crates/spanda-api/tests/decision_traces_api_tests.rs` |
| Signed offline policy | signature, trust key, persisted cache, `sign-policy` CLI | `crates/spanda-decision/tests/distributed.rs`, `spanda decision sign-policy` |
| Policy cache CLI/API | `cache show|sync|clear`, REST + gRPC cache listing, SDK wrappers | `spanda decision cache`, `GET /v1/decision-policy-cache`, `ListDecisionPolicyCache`, smoke script |
| Program sim traces | API sim emits v3 frames + mission trace | `program_simulation` with `decision_trace`/`record_trace`, Control Center button |
| Distributed decisions demo | One-command evaluator path | `spanda demo distributed-decisions`, smoke script |
| Differentiation decision trail | audit + explain decision on v3 trace | `examples/showcase/differentiation/decision_trail/`, `scripts/differentiation_smoke.sh` |
| What-if analysis | scenario impact, risk, recovery plan, probability | `crates/spanda-whatif/tests/gps_failure.rs`, `scripts/what_if_smoke.sh` |
| Mission risk scoring | deployment risk score and mitigations | `crates/spanda-risk/tests/deployment_risk.rs`, `scripts/risk_smoke.sh` |
| Readiness forecast | horizon predictions and degradation | `crates/spanda-readiness/tests/forecast_tests.rs`, `scripts/readiness_forecast_smoke.sh` |
| Trust graph | trust-weighted mission → provider paths | `crates/spanda-graph/tests/graph_tests.rs`, `scripts/trust_graph_smoke.sh` |
| Scorecard rollup | executive multi-pillar score | `scripts/scorecard_smoke.sh`, `examples/showcase/scorecard/executive.sd` |
| Differentiation analytics API | what-if, risk, forecast, trust graph REST | `crates/spanda-api/tests/differentiation_analytics_api_tests.rs` |
| Differentiation analytics gRPC | GetAnalytics* RPC parity | `crates/spanda-api/tests/grpc_tests.rs` (`grpc_analytics_endpoints_with_forecast_program`) |
| Differentiation analytics SDK | REST + gRPC analytics wrappers (NEXT + LATER) | `crates/spanda-sdk`, `sdk/python`, `sdk/typescript` client tests |
| LATER differentiation | twin mission, certify pack, team verify, governance, replay time travel | `scripts/later_differentiation_smoke.sh`, `crates/spanda-runtime/tests/time_travel_tests.rs` |
| LATER analytics API | mission twin, certification pack, time travel, human teaming, governance REST + gRPC | `crates/spanda-api/tests/later_analytics_api_tests.rs`, `scripts/later_analytics_smoke.sh` |
| Twin Cloud SaaS | push/pull/list/sync/import-replay + history + gRPC + RBAC | `crates/spanda-twin-cloud`, `scripts/twin_cloud_unified_path.sh`, `scripts/hosted_twin_cloud_smoke.sh`, `deploy/twin-cloud-hosted/`, `cargo test -p spanda-api twin_cloud` |
| Differentiation promotion gate | 15 pillars smoke + showcase check + topic guides | `scripts/differentiation_promotion_gate.sh` |
| What-If Stable promotion | soak + smoke + analytics API + Control Center probe | `scripts/what_if_stable_promotion_gate.sh`, [stable-hardening-what-if.md](./stable-hardening-what-if.md) |
| NEXT Stable promotion (risk, forecast, trust graph, scorecard) | per-pillar gates + Control Center probes | `scripts/{risk,forecast,trust_graph,scorecard}_stable_promotion_gate.sh`, CI `next-differentiation-stable-gates` |
| LATER Stable promotion | soak + later smoke + topic guides Stable | `scripts/later_differentiation_stable_promotion_gate.sh`, CI `later-differentiation-stable-gates` |
| Trust Framework Stable promotion | soak + audit + trust smokes + Control Center `/v1/trust/program` | `scripts/trust_framework_stable_promotion_gate.sh`, CI `trust-framework-stable-gate` |
| Bundled examples sync | NEXT/LATER showcases + decision-trail trace | `scripts/sync_bundled_examples.sh` |
| Decision diagnostics | `decision_tree` / `offline_policy` / authority parity Rust ↔ TS | `crates/spanda-decision/src/diagnostics.rs`, `src/decision-diagnostics.ts` |
| Swarm continuity | member-lost handoff + mesh relay | `crates/spanda-fleet/src/swarm_continuity.rs`, `crates/spanda-fleet/tests/mesh_integration.rs` |
| Self-healing runtime | auto-trigger, approval retry, mesh relay | `crates/spanda-interpreter/tests/recovery_runtime.rs`, `scripts/self_healing_smoke.sh` |
| **Recovery Orchestrator** | plan/simulate/execute, graph, playbooks, gRPC, plugins | `crates/spanda-recovery/tests/orchestrator_tests.rs`, `crates/spanda-api/tests/recovery_api_tests.rs`, `grpc_tests` (`grpc_recovery_*`), `scripts/recovery_orchestrator_smoke.sh` |
| Fleet field validation | multi-process agents + mesh orchestrate | `scripts/fleet_field_validation.sh` |
| gRPC Control Center | tonic (see `GET /v1/version` → `grpc.rpc_count`; full REST parity except `/v1/rpc`) | `crates/spanda-api/tests/grpc_tests.rs`, `grpc_live_probe.rs` |
| API rate limit + versioning | `SPANDA_API_RATE_LIMIT_PER_MINUTE`, `GET /v1/version`, `X-Spanda-Api-Version` | `crates/spanda-api/tests/api_policy_tests.rs` |
| OpenAPI REST parity | `GET /v1/openapi.json` documents all `/v1/*` routes | `crates/spanda-api/tests/openapi_parity_tests.rs` |
| Live OTA execute | `POST /v1/ota/execute` against deploy agent | `crates/spanda-api/tests/ota_execute_live.rs`, `scripts/ota_fleet_execute_smoke.sh` |
| OTA fleet soak | Multi-agent version bumps + canary progression | `crates/spanda-ota/tests/fleet_soak.rs`, `scripts/ota_fleet_soak.sh` |
| Failover drill | Redundant chain selection + recovery actions | `crates/spanda-config/tests/failover_drill.rs`, `scripts/failover_drill_smoke.sh` |
| Remote CLI parity | `spanda control-center` routes vs OpenAPI registry | `crates/spanda-cli/tests/control_center_openapi_parity.rs` |
| Control Center instance status | `GET /v1/instance`, `spanda control-center status` | `crates/spanda-api/src/handlers.rs` (`instance_endpoint_reports_runtime_paths`), `spanda control-center status --discover` |
| Control Center stop | `spanda control-center stop` kills verified local listeners | `crates/spanda-cli/src/control_center_cli.rs`, `scripts/lib/control_center_smoke_lib.sh` |
| Control Center UI token persistence | Embedded HTML `localStorage` restore, verify-on-save, Forget token | Manual browser check: `control-center serve` → paste token with **Remember** → refresh → **Forget** clears `spanda.control_center.bearer_token.v1:<host>` (`crates/spanda-api/src/static/control-center.html`) |
| Discovery registry runtime | `spanda-discovery-mdns` package wrap | `crates/spanda-config/src/discovery_registry.rs` |
| OTLP metrics (Control Center) | `GET /v1/observability/otlp/metrics`, `POST /v1/observability/otlp/export-metrics` | `crates/spanda-ops/src/otlp_metrics.rs`, `scripts/enterprise_ops_smoke.sh` |
| Fleet agent interpreter recovery | HTTP deploy + `/v1/recovery/execute` | `scripts/fleet_agent_recovery_smoke.sh`, `crates/spanda-fleet/tests/mesh_integration.rs` |
| Operational drift (full) | program + agent dimensions | `crates/spanda-config/src/operational_drift.rs` |
| Platform architecture validation | layer classification, zero-waiver baseline, SCC, TypeScript layers, manifest sync | `scripts/validate_architecture.py --check-manifest-sync` (CI `rust` job) |
| Platform event transitions | health/readiness/degraded emit-on-change | `crates/spanda-readiness/tests/platform_events.rs` |
| Plugin system | manifest, security, lifecycle, hooks, registry fetch | `crates/spanda-plugin/tests/integration.rs` |
| Solution blueprint governance | no workspace crates or Rust in blueprint trees | `scripts/validate_blueprints.py` (CI `rust` job) |
| Enterprise ops API | Control Center handlers, device pool | `crates/spanda-api/tests/` |
| Smart Spaces live-building | BACnet registry script dispatch (`live-building` feature) | `crates/spanda-providers/src/iot_live.rs` (`live_bacnet_registry_script_reads_mock_stdout`) |
| Negative | `ai_safety_violation.sd` fails | `tests/integration.rs` |

**Current count:** ~115+ Rust unit/integration tests (workspace total grows with platform crates).

## TypeScript (`npm test`)

| Area | Status |
|------|--------|
| Lexer, parser, typechecker | Passing |
| Foundations + phases 4–7 | enum, struct literal, trait impl, twin replay |
| Runtime hardening | contracts, capabilities, verify |
| Golden (Rust CLI) | `tests/golden/rust.test.ts` |
| LSP diagnostics | `tests/lsp.test.ts` via `spanda check` + `spanda verify` |
| Mission continuity mirror | `tests/mission-continuity.test.ts`, `tests/continuity-diagnostics.test.ts` |

**Current count:** 121+ vitest tests.

## CLI smoke scripts

| Script | Coverage |
|--------|----------|
| `scripts/showcase_smoke.sh` | Bundled demos (continuity, maturity, enterprise ops, policy, trust, …) |
| `scripts/continuity_smoke.sh` | Continuity CLI + demo |
| `scripts/policy_smoke.sh` | Verify-time policy |
| `scripts/policy_runtime_smoke.sh` | Runtime `--enforce-policy` |
| `scripts/maturity_smoke.sh` | Graph, explain, trust, deploy gate |
| `scripts/enterprise_ops_smoke.sh` | Control Center E1–E4 API surface (compliance catalog, report schedules, discovery TLS, audit prep) |
| `scripts/field_soak_gate.sh` | 30-day field pilot gate before Stable promotion |
| `scripts/spatial_computing_smoke.sh` | Spatial Computing blueprint (human registry, readiness, examples) |
| `scripts/hri_stable_promotion_gate.sh` | HRI Stable promotion (soak + audit prep + spatial smoke + Control Center HRI API probe) |
| `scripts/adas_smoke.sh` | ADAS Solution Blueprint (verify, readiness, replay, compliance, examples) |
| `scripts/smart_spaces_smoke.sh` | Smart Spaces blueprint (six apps, verify + check) |
| `scripts/smart_spaces_promotion_gate.sh` | Smart Spaces scaffold promotion (smoke, live IoT, BMS sidecar, `live-building` test, API tests, OpenAPI parity, Control Center probe) |
| `scripts/smart_spaces_stable_init.sh` | Start field soak clock + security audit artifacts for Smart Spaces |
| `scripts/smart_spaces_live_iot_smoke.sh` | BACnet/KNX/Thread/Z-Wave/HA bridges (mock CI; `SPANDA_LIVE_IOT_HARDWARE=1` for bacpypes3/xknx) |
| `scripts/smart_spaces_bms_sidecar_smoke.sh` | Home Assistant REST sidecar (mock CI; `SPANDA_BMS_SIDECAR_LIVE=1` for live HA) |
| `scripts/adas_stable_promotion_gate.sh` | ADAS Stable promotion (soak + audit prep + smoke + Control Center ADAS API probe) |
| `scripts/adas_automotive_sensors_smoke.sh` | Automotive sensor hub + live `SPANDA_*_CMD` bridge tests |
| `scripts/hri_field_soak_init.sh` | Start 30-day HRI field soak clock |
| `scripts/hri_security_audit_prep.sh` | HRI security audit intake artifact |
| `scripts/security_audit_prep.sh` | Third-party audit intake artifact |
| `scripts/verify_sdk_publish_ready.sh` | PyPI + npm pack readiness (no publish) |
| `scripts/verify_desktop_release_ready.sh` | Desktop version sync + Tauri compile smoke |

## CLI verification

```bash
cargo test -p spanda-core --test hardware_compat
spanda verify examples/hardware/rover_deploy.sd
spanda verify examples/hardware/full_compat.sd   # expect incompatible (ESP32 in matrix)
spanda readiness examples/showcase/policy/warehouse.sd --policy WarehousePolicy
```

## CI

`.github/workflows/ci.yml`: TypeScript tests, Rust tests, architecture validation (`validate_architecture.py`), WASM + web build, enterprise ops smoke (when enabled), `smart-spaces-promotion-gate` (API + live probe after smoke).

## Acceptance criteria per feature

Each feature merges when:

- Rust unit + integration tests pass
- New examples in `examples/` compile; hardware examples verify as expected
- Relevant `docs/` updated
- Golden manifest updated for stable fixtures (when applicable)
- Smoke script added or extended for user-visible CLI paths

## Future tests

1. Verify JSON output schema conformance (`api-contract.json`)
2. LSP verify diagnostic golden files
3. Per-fault simulation coverage matrix
4. Cross-profile deploy matrix CI job (`--all-targets` on main examples)
5. Multi-process fleet agent field validation — **shipped** (`scripts/fleet_field_validation.sh`)
