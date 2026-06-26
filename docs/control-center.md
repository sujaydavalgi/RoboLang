# Control Center

Web-based operational visibility for fleets, devices, readiness, and alerts. Phase E1 ships a REST API v1 and embedded UI served by the native CLI.

**Related:** [enterprise-operations-roadmap.md](./enterprise-operations-roadmap.md) · [telemetry-store.md](./telemetry-store.md) · [configuration.md](./configuration.md)

---

## Quick start

```bash
# Start API + UI (default http://127.0.0.1:8080)
export SPANDA_API_KEY="your-operator-key"
spanda control-center serve

# With project configuration (device pool from spanda.toml)
spanda control-center serve --config spanda.toml --bind 0.0.0.0:8080
```

Open `http://127.0.0.1:8080/` for the Control Center UI, or use the **Control Center** view in `@spanda/web` (set API URL to the serve address).

---

## REST API v1

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/v1/health` | GET | — | Liveness |
| `/v1/dashboard` | GET | — | Device pool summary, fleet agent count, alerts |
| `/v1/devices` | GET | — | Device pool entries |
| `/v1/devices/{id}` | PATCH | Bearer | Update `lifecycle_state` |
| `/v1/fleet/agents` | GET | — | Registered fleet agents (`.spanda/fleet-agents.json`) |
| `/v1/alerts` | GET | — | Alert history |
| `/v1/alerts/test` | POST | Bearer | Dispatch test alert |
| `/v1/secrets` | GET | Bearer | Secret metadata (no values) |
| `/v1/rbac/matrix` | GET | — | Role permission matrix |
| `/v1/provision` | POST | Bearer | Run discover → ready workflow |
| `/v1/discovery` | GET | — | Package-backed discovery (`?transport=mdns` or `subnet`) |
| `/v1/config/snapshots` | GET/POST | POST: Bearer | List or save configuration snapshots |
| `/v1/health/summary` | GET | — | Device pool health rollup |
| `/v1/assurance/summary` | GET | — | Assurance policy from resolved config |
| `/v1/diagnosis/summary` | GET | — | Diagnosis policy from resolved config |
| `/v1/openapi.json` | GET | — | OpenAPI 3.1 specification |
| `/v1/drift` | GET | — | Operational drift vs baseline snapshot (`?baseline_id=`) |
| `/v1/ota/plan` | POST | Bearer | Plan canary / staged / blue_green rollout |
| `/v1/ota/status` | GET | — | OTA deploy state (`.spanda/deploy-state.json`) |
| `/v1/trust/package` | GET | — | Package trust evaluation (`?name=&version=`) |
| `/v1/sre/summary` | GET | — | Availability and alert rollup |
| `/v1/observability/traces` | GET | — | Recent API trace records |
| `/v1/operator/quarantine` | POST | Bearer | Quarantine a device |
| `/v1/operator/mission/approve` | POST | Bearer | Approve or reject a mission |
| `/v1/rpc` | POST | — | gRPC-compatible JSON gateway |

Authenticate mutations with `Authorization: Bearer <SPANDA_API_KEY>`.

Pass optional `X-Correlation-ID` on any request; the server echoes it on the response and records traces for `/v1/observability/traces`.

---

## Python SDK

```bash
pip install -e packages/sdk-python
export SPANDA_API_KEY=your-key
python -c "from spanda_sdk import ControlCenterClient; print(ControlCenterClient().health())"
```

Integration tests: `SPANDA_SDK_INTEGRATION=1 SPANDA_CONTROL_CENTER_URL=http://127.0.0.1:8080 pytest packages/sdk-python/tests`

---

## Device Pool lifecycle

Devices in `[[devices]]` or the device tree carry optional lifecycle fields:

| State | Meaning |
|-------|---------|
| `discovered` | Seen but not verified |
| `quarantined` | Blocked pending review |
| `verified` | Identity and trust checks passed |
| `assigned` | Bound to a robot |
| `healthy` / `degraded` / `offline` / `failed` | Runtime posture |
| `retired` | Removed from active pool |

Set in TOML:

```toml
[[devices]]
id = "lidar-front"
type = "lidar"
lifecycle_state = "healthy"
assigned_robot = "rover-1"
```

---

## Alerting

Configure delivery channels via environment variables:

| Variable | Effect |
|----------|--------|
| `SPANDA_ALERT_WEBHOOK_URL` | POST JSON alert payload |
| `SPANDA_ALERT_EMAIL_TO` | Email recipient (logs if `SPANDA_SMTP_HOST` unset) |
| `SPANDA_ALERT_EMAIL_DRY_RUN=1` | Log email without sending |

Default: log to stderr.

---

## Smoke test

```bash
./scripts/enterprise_ops_smoke.sh
```

---

## Status

**Experimental** (Phase E1–E3). Phase E3 adds operational drift, OTA rollout planning, package trust API, SRE/observability traces, operator workflows, JSON-RPC gateway, OpenAPI spec, and the Python SDK. Native gRPC (tonic) and full CLI parity remain Phase E4 scope.
