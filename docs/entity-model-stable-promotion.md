# Unified Entity Model — Stable Promotion

Operational checklist for promoting **Unified Entity Model** from **Experimental** to **Stable** in `docs/feature-status.md`.

**Implementation status:** Phases 1–5 and stabilization items in [entity-model.md](./entity-model.md) are **complete** on `main`.

---

## Automated gate

```bash
chmod +x scripts/entity_model_stable_promotion_gate.sh
chmod +x scripts/enterprise_ops_field_soak_init.sh
chmod +x scripts/security_audit_prep.sh

# One-time: start shared 30-day field soak clock (same file as enterprise ops)
./scripts/enterprise_ops_field_soak_init.sh

# Audit prep packet for reviewers
./scripts/security_audit_prep.sh

# Publish SDKs with entity helpers (after merge):
#   git tag sdk-python-v0.4.1 && git push origin sdk-python-v0.4.1
#   git tag crates-sdk-v0.4.1 && git push origin crates-sdk-v0.4.1  # if not already published

# After soak elapsed + audit sign-off:
./scripts/entity_model_stable_promotion_gate.sh
```

Then update `docs/feature-status.md` — **Unified Entity Model** row to **Stable**.

### CI (implementation checks only)

Job `entity-model-promotion-gate` runs with soak and audit skipped:

```bash
SPANDA_ENTITY_MODEL_SKIP_SOAK=1 SPANDA_ENTITY_MODEL_SKIP_AUDIT=1 \
  ./scripts/entity_model_stable_promotion_gate.sh
```

---

## Environment variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `SPANDA_FIELD_SOAK_START_FILE` | `.spanda/field-soak-start.txt` | Shared soak clock with enterprise ops |
| `SPANDA_FIELD_SOAK_MIN_DAYS` | `30` | Minimum elapsed days |
| `SPANDA_SECURITY_AUDIT_PREP_FILE` | `.spanda/security-audit-prep.json` | Audit prep artifact |
| `SPANDA_ENTITY_MODEL_SKIP_SOAK` | `0` | Skip soak elapsed check |
| `SPANDA_ENTITY_MODEL_SKIP_AUDIT` | `0` | Skip audit prep file check |

---

## What the gate runs

1. **Field soak** — shared with enterprise ops ([field-soak-gate.md](./field-soak-gate.md))
2. **Security audit prep** — [security-audit-third-party.md](./security-audit-third-party.md)
3. **`entity_model_smoke.sh`** — REST mutations + TypeScript + Python + Rust SDK

The enterprise ops promotion gate (`enterprise_ops_stable_promotion_gate.sh`) also runs `entity_model_smoke.sh` after E1–E4 smokes.

---

## SDK publish (entity helpers)

| Package | Version | Tag |
|---------|---------|-----|
| `spanda-sdk` (PyPI) | `0.4.1` | `sdk-python-v0.4.1` |
| `spanda-sdk` (crates.io) | `0.4.1` | `crates-sdk-v0.4.1` |
| `@davalgi-spanda/sdk` (npm) | `0.4.1` | `npm-sdk-v0.4.1` |

See [sdk-publishing.md](./sdk-publishing.md).

---

## Remaining human gates

| Gate | Action |
|------|--------|
| Third-party audit | Sign-off recorded in change management |
| PyPI publish | Push `sdk-python-v0.4.1` tag when ready |
| Feature status | Set **Unified Entity Model** to **Stable** after gate passes |

---

## Related

- [entity-model.md](./entity-model.md) — architecture and phase checklist
- [enterprise-ops-stable-promotion.md](./enterprise-ops-stable-promotion.md) — Control Center E1–E4 promotion
- [stable-hardening-enterprise-ops.md](./stable-hardening-enterprise-ops.md)
