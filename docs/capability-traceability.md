# Capability Traceability

Traceability matrices link capabilities to hardware, packages, providers, safety rules, and verification status.

**Example:** [`examples/hardware/capability_verification.sd`](../examples/hardware/capability_verification.sd)

---

## Hardware traceability

```bash
spanda trace hardware rover.sd
spanda verify rover.sd --traceability
spanda verify rover.sd --traceability-json
```

Columns: Hardware Component | Used By | Source Location | Capability | Provider | Verified | Safety Rule

## Capability traceability

```bash
spanda trace capabilities rover.sd
spanda trace capabilities rover.sd --json
spanda verify rover.sd --traceability --capabilities
```

Columns: Capability | Required By | Provided By | Hardware | Package | Provider | Safety Rule | Status

## Package contributions

Packages register capabilities in the capability registry (e.g. `spanda-nav` → `obstacle_avoidance`, `spanda-gps` → `gps_navigation`).

See [Hardware Traceability](./hardware-traceability.md) for hardware-to-code mapping.

---

## Verification JSON

Combine traceability with structured diagnostics:

```bash
spanda check examples/hardware/capability_verification.sd --verification-json
spanda verify rover.sd --traceability --capabilities --health
```

See [verification-diagnostics.md](./verification-diagnostics.md).
