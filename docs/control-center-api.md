# Control Center API Reference (v1)

Stable Control Center API served by `spanda control-center serve` and implemented in `crates/spanda-api`.

For the Rust/TypeScript **compiler crate index**, see [api-reference.md](./api-reference.md).

## Transports

| Transport | Entry |
|-----------|--------|
| REST | `http://host:8080/v1/*` |
| OpenAPI | `GET /v1/openapi.json` |
| gRPC | `ControlCenter` service (see `proto/spanda/v1/control_center.proto`) — program ops: `EvaluateProgramReadiness`, `ListEntities`, … |
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
| `POST /v1/programs/simulation` | `spanda sim` (metadata) |
| `POST /v1/programs/replay` | `spanda replay` (trace load) |
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

| Endpoint | Description |
|----------|-------------|
| `GET /v1/entities` | Humans + devices unified list |
| `GET /v1/entities/{id}` | Entity by id |
| `GET /v1/entities/{id}/health` | Device health |
| `GET /v1/entities/{id}/trust` | Entity trust metadata |

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
- Policy: `GET /v1/version`
- OpenAPI parity enforced by `crates/spanda-api/tests/openapi_parity_tests.rs`

## Authentication

- Bearer token: `Authorization: Bearer $SPANDA_API_KEY`
- RBAC enforced on mutations (provision, OTA, config approvals)
- Correlation: `X-Correlation-ID` header (optional, echoed in responses)

## JSON-RPC gateway (`POST /v1/rpc`)

gRPC-compatible JSON gateway for clients without tonic. Example:

```json
{
  "method": "spanda.v1.ControlCenter/EvaluateProgramReadiness",
  "params": { "body_json": "{\"file\":\"rover.sd\"}" }
}
```

Supported SDK methods include `ListEntities`, `GetEntity`, `GetEntityHealth`, `GetEntityTrust`, `EvaluateProgramReadiness`, `EvaluateProgramAssure`, `EvaluateProgramDiagnose`, `EvaluateProgramHeal`, `VerifyProgramHardware`, `VerifyProgramCapabilities`, `VerifyProgramMission`, `RunProgramSimulation`, `ReplayProgram`, and `GetTrustProgram`.

## Event types (WebSocket)

- `health_changed`
- `readiness_changed`
- `mission_started` / `mission_paused`
- `recovery_triggered`
- `device_offline`
- `tamper_detected`
- `kill_switch_triggered`

See [SDK overview](sdk.md) for client usage.
