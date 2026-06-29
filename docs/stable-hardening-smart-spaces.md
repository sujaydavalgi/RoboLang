# Smart Spaces & Ambient Intelligence — Stable Hardening Checklist

The Smart Spaces Solution Blueprint is shipped at **Experimental** tier with CI smoke (`scripts/smart_spaces_smoke.sh`). This checklist tracks promotion gates before moving the blueprint to **Stable**.

**Related:** [solutions/smart-spaces.md](./solutions/smart-spaces.md) · [feature-status.md](./feature-status.md) · [control-center.md](./control-center.md#smart-spaces-dashboard) · [field-soak-gate.md](./field-soak-gate.md)

---

## Promotion criteria

| Gate | Requirement | Status |
|------|-------------|--------|
| Blueprint smoke | `scripts/smart_spaces_smoke.sh` green on `main` | **Shipped** |
| Scaffold promotion gate | `scripts/smart_spaces_promotion_gate.sh` (smoke + API + Control Center probe) | **Shipped** |
| Readiness profile | `spanda readiness --profile smart_space` on blueprint apps | **Shipped** |
| Control Center REST | `/v1/facilities`, readiness, occupancy, energy, emergency, summary | **Shipped** |
| Control Center UI | Smart Spaces tab (buildings, gateways, zones, energy, continuity) | **Shipped** |
| OpenAPI parity | `openapi_parity_tests` documents all Smart Spaces routes | **Shipped** |
| Registry packages | Nine optional packages + provider dispatch stubs | **Shipped** (experimental) |
| Grafana dashboard | `control-center-smart-spaces.json` template | **Shipped** |
| Golden traces | Emergency / mode-change deterministic replay | **In progress** — `fixtures/fire_panel_activation.trace` |
| Bundled offline registry | Smart Spaces packages in `bundled-registry` | **Shipped** |
| Live building I/O | BACnet/KNX/Thread/Z-Wave/HA env bridges | **Pending** |
| Field soak | 30-day smart-building pilot without regression | **Pending** — `.spanda/smart-spaces-field-soak-start.txt` |
| Security audit | Third-party review of life-safety and access-control paths | **Pending** — `./scripts/smart_spaces_security_audit_prep.sh` |

---

## Running the promotion gate

```bash
# Start 30-day pilot clock (UTC) — one-time
./scripts/smart_spaces_field_soak_init.sh

# Generate Smart Spaces security audit intake artifact
./scripts/smart_spaces_security_audit_prep.sh

# Scaffold gate (soak/audit skipped by default for experimental tier):
chmod +x scripts/smart_spaces_promotion_gate.sh
./scripts/smart_spaces_promotion_gate.sh

# Full gate after soak and audit artifact:
SPANDA_SMART_SPACES_SKIP_SOAK=0 SPANDA_SMART_SPACES_SKIP_AUDIT=0 ./scripts/smart_spaces_promotion_gate.sh

# CI after smart-spaces-smoke (skip duplicate smoke):
SPANDA_SMART_SPACES_SKIP_SOAK=1 SPANDA_SMART_SPACES_SKIP_AUDIT=1 SPANDA_SMART_SPACES_SKIP_SMOKE=1 ./scripts/smart_spaces_promotion_gate.sh
```

The gate runs:

1. Field soak check (unless `SPANDA_SMART_SPACES_SKIP_SOAK=1`, default **skip**)
2. Security audit prep artifact (unless `SPANDA_SMART_SPACES_SKIP_AUDIT=1`, default **skip**)
3. `scripts/smart_spaces_smoke.sh`
4. `cargo test -p spanda-api smart_spaces` and OpenAPI parity
5. Live Control Center probe (`/v1/facilities`, readiness, occupancy, energy, emergency, summary)

---

## Remaining before Stable tier label

1. **30-day field soak** — commercial or residential pilot with gateway failover exercised
2. **Security audit sign-off** — life-safety overrides, access control, health opt-in, tamper policy on edge gateways
3. **Live protocol bridges** — production BACnet/KNX/Matter/Thread adapters beyond stub dispatch
4. **Simulation matrix** — fire, flood, power loss, gateway failure scenarios with recorded traces
5. **gRPC parity** — tonic RPCs for Smart Spaces REST surface (optional; REST is canonical today)
