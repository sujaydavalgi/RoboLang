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

Authenticate mutations with `Authorization: Bearer <SPANDA_API_KEY>`.

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

**Experimental** (Phase E1). Provisioning workflow, gRPC, and full CLI parity APIs ship in Phase E2–E3.
