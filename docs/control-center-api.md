# Control Center API Reference (v1)

Stable Control Center API served by `spanda control-center serve` and implemented in `crates/spanda-api`.

For the Rust/TypeScript **compiler crate index**, see [api-reference.md](./api-reference.md).

## Transports

| Transport | Entry |
|-----------|--------|
| REST | `http://host:8080/v1/*` |
| OpenAPI | `GET /v1/openapi.json` |
| gRPC | `ControlCenter` service — **82 RPCs**, proto semver **1.0.2** (`proto/spanda/v1/control_center.proto`); pin via `GET /v1/version` → `grpc` |
| WebSocket | `WS /v1/stream/telemetry` |
| JSON-RPC gateway | `POST /v1/rpc` |

## SDK program operations (CLI parity)

These endpoints delegate to the same Rust crates as CLI commands:

| Endpoint | CLI equivalent |
|----------|----------------|
| `POST /v1/programs/readiness` | `spanda readiness <file.sd>` |
| `POST /v1/programs/assure` | `spanda assure <file.sd>` |
| `POST /v1/programs/diagnose` | `spanda diagnose <file.sd\|.trace>` |
| `POST /v1/programs/recovery/heal` | `spanda heal` |
| `POST /v1/programs/verify/hardware` | `spanda verify` |
| `POST /v1/programs/verify/capabilities` | `spanda verify --capabilities` |
| `POST /v1/programs/verify/mission` | `spanda verify mission` |
| `POST /v1/programs/simulation` | `spanda sim` (set `"execute": true` to run) |
| `POST /v1/programs/replay` | `spanda replay` (set `"deterministic"` or `"playback"`) |
| `GET /v1/trust/program` | `spanda trust <file.sd>` |

### Request body (program ops)

```json
{
  "file": "rover.sd",
  "target": "jetson-orin",
  "include_runtime": false,
  "traceability": true
}
```

`file` is optional when Control Center was started with `--program`.

## Entity registry

Read endpoints are unauthenticated by default; mutations require Bearer `SPANDA_API_KEY`.

| Endpoint | Auth | Description |
|----------|------|-------------|
| `GET /v1/entities` | — | Unified entity inventory (optional query filters) |
| `GET /v1/entities/graph` | — | Full entity graph |
| `GET /v1/entities/traceability` | — | Unified traceability (entity + program graph) |
| `POST /v1/entities/query` | — | Structured query body |
| `GET /v1/entities/{id}` | — | Entity by id |
| `GET /v1/entities/{id}/relationships` | — | Relationship edges and impact analysis |
| `GET /v1/entities/{id}/health` | — | Health snapshot |
| `GET /v1/entities/{id}/readiness` | — | Readiness snapshot |
| `GET /v1/entities/{id}/trust` | — | Trust and security metadata |
| `POST /v1/entities/register` | Bearer | Register or update entity overlay |
| `POST /v1/entities/{id}/tags` | Bearer | Add or remove tags |
| `POST /v1/entities/relationships` | Bearer | Relate two entities |
| `POST /v1/entities/sync` | Bearer | Sync overlay to TOML fragments |

### gRPC parity (`--grpc-bind`)

| gRPC RPC | REST equivalent |
|----------|-----------------|
| `ListEntities` | `GET /v1/entities` |
| `GetEntity` | `GET /v1/entities/{id}` |
| `GetEntityHealth` | `GET /v1/entities/{id}/health` |
| `GetEntityTrust` | `GET /v1/entities/{id}/trust` |
| `GetEntityGraph` | `GET /v1/entities/graph` |
| `GetEntityTraceability` | `GET /v1/entities/traceability` |
| `QueryEntities` | `POST /v1/entities/query` |
| `GetEntityRelationships` | `GET /v1/entities/{id}/relationships` |
| `GetEntityReadiness` | `GET /v1/entities/{id}/readiness` |
| `RegisterEntity` | `POST /v1/entities/register` |
| `TagEntity` | `POST /v1/entities/{id}/tags` |
| `RelateEntities` | `POST /v1/entities/relationships` |
| `SyncEntities` | `POST /v1/entities/sync` |

Rust `GrpcClient` (`spanda-sdk` `grpc` feature) mirrors these; mutations send Bearer from `SPANDA_API_KEY`. See [entity-model.md](./entity-model.md) and [sdk-rust.md](./sdk-rust.md).

## Device registry

| Endpoint | Description |
|----------|-------------|
| `GET /v1/devices` | Device pool |
| `POST /v1/devices/discover` | Discovery scan |
| `POST /v1/devices/{id}/provision` | Provision workflow |

## Trust & assurance summaries

| Endpoint | Description |
|----------|-------------|
| `GET /v1/trust/package` | Package trust score |
| `GET /v1/assurance/summary` | Config assurance policy |
| `GET /v1/diagnosis/summary` | Config diagnosis policy |
| `POST /v1/readiness/run` | **Device pool** readiness impact (not program scoring) |

## Shared schemas

Domain types are defined in Rust with `serde` and documented in OpenAPI:

- `ReadinessReport` — `spanda_readiness::types`
- `AssuranceReport` / `MissionAssuranceSummary` — `spanda_assurance`
- `DiagnosisReport` — `spanda_assurance::diagnosis`
- `RecoveryReport` — `spanda_assurance::recovery`
- `HealthReport` — `spanda_capability::health`
- `TrustScoreReport` — `spanda_package::trust`
- `CompatibilityReport` — `spanda_hardware`

SDK wrappers mirror these in each language (`spanda-sdk` types modules).

## Versioning

- API version prefix: `/v1/`
- Policy: `GET /v1/version` (includes `grpc.proto_semver` and `grpc.rpc_count`)
- OpenAPI parity enforced by `crates/spanda-api/tests/openapi_parity_tests.rs`

## Authentication

- Bearer token: `Authorization: Bearer $SPANDA_API_KEY`
- RBAC enforced on mutations (provision, OTA, config approvals, entity overlay writes)
- Correlation: `X-Correlation-ID` header (optional, echoed in responses)

## JSON-RPC gateway (`POST /v1/rpc`)

gRPC-compatible JSON gateway for clients without tonic. Example:

```json
{
  "method": "spanda.v1.ControlCenter/EvaluateProgramReadiness",
  "params": { "body_json": "{\"file\":\"rover.sd\"}" }
}
```

Supported SDK methods include program ops (`EvaluateProgramReadiness`, `EvaluateProgramAssure`, `EvaluateProgramDiagnose`, `EvaluateProgramHeal`, `VerifyProgramHardware`, `VerifyProgramCapabilities`, `VerifyProgramMission`, `RunProgramSimulation`, `ReplayProgram`, `GetTrustProgram`) and entity reads (`ListEntities`, `GetEntity`, `GetEntityHealth`, `GetEntityTrust`, `GetEntityGraph`, `GetEntityTraceability`, `QueryEntities`, `GetEntityRelationships`, `GetEntityReadiness`). Entity mutations are **gRPC-only** (not exposed on the JSON-RPC gateway).

## Event types (WebSocket)

- `health_changed`
- `readiness_changed`
- `mission_started` / `mission_paused`
- `recovery_triggered`
- `device_offline`
- `tamper_detected`
- `kill_switch_triggered`

See [SDK overview](sdk.md) for client usage.
