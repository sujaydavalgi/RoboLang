# Compliance Profiles

**Status:** Experimental · **Phase:** Verify, Deploy · **Priority:** P2.4

Industry-specific verification templates — not accredited certifications.

## Profiles

| Profile | Typical use |
|---------|-------------|
| `industrial` | Factory AMRs, fixed safety zones |
| `warehouse` | Speed caps, shift hours, pedestrian zones |
| `medical` | Stricter health evidence, audit trails |
| `agriculture` | Outdoor connectivity, GPS reliance |
| `defense` | Signed comm, capability minimization |
| `research` | Relaxed gates with explicit warnings |

## Each profile defines

- Required safety rules (kill switch, max speed)
- Required health checks
- Required evidence (assurance cases)
- Required capabilities
- Required readiness thresholds
- Secure communication (defense)

## CLI

```bash
spanda verify examples/showcase/policy/warehouse.sd --profile warehouse
spanda verify rover.sd --profile medical --json
spanda readiness rover.sd --profile medical
```

## Integration

Built on readiness, capability verification, and assurance evidence checks in `spanda-compliance`.

**Disclaimer:** Profiles are **templates** for engineering discipline, not regulatory approval.

Showcase: `examples/showcase/policy/warehouse.sd` · smoke: `scripts/compliance_smoke.sh`

See [policy-engine.md](./policy-engine.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
