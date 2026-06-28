# Trust Framework

**Status:** Experimental · **Phase:** Verify, Operate · **Priority:** P0.4 (package), P3.1 (composite)

Unified trust scoring across packages, devices, firmware, configuration, and runtime behavior.

## Composite TrustScore (0–100)

| Input | Weight (initial) | Source |
|-------|------------------|--------|
| Package trust | 20% | `spanda trust <package>` (registry provenance required for `official_framework` factor when `--project` is set) |
| Device integrity | 20% | Agent attestation (package adapter) |
| Firmware integrity | 15% | Hash vs declared firmware |
| Configuration integrity | 20% | `spanda integrity` |
| Identity validation | 15% | Signed comm, `trust_boundary` |
| Safety integrity | 10% | Safety auditor + certify |

## CLI

```bash
spanda trust rover.sd
spanda trust rover.sd --json
```

## Package integrity status

Per package: **Trusted** · **Modified** · **Unknown**

## Integration

Feeds readiness trust factor, deployment gates (`official_provenance`, `registry_signatures` under `--policy production`), scorecard security category, tamper response policies, and `spanda explain` composite_trust section.

See [package-trust.md](./package-trust.md) · [tamper-detection.md](./tamper-detection.md) · [trust-boundaries.md](./trust-boundaries.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
