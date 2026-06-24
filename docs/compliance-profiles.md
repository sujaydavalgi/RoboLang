# Compliance Profiles

**Status:** Planned (Future) · **Phase:** Verify, Deploy · **Priority:** P2.4

Industry-specific verification templates — not accredited certifications.

## Profiles

| Profile | Typical use |
|---------|-------------|
| Industrial | Factory AMRs, fixed safety zones |
| Warehouse | Speed caps, shift hours, pedestrian zones |
| Medical | Stricter health evidence, audit trails |
| Agriculture | Outdoor connectivity, GPS reliance |
| Defense | Signed comm, capability minimization |
| Research | Relaxed gates with explicit warnings |

## Each profile defines

- Required safety rules
- Required health checks
- Required evidence (assurance cases)
- Required capabilities
- Required readiness thresholds

## CLI

```bash
spanda verify rover.sd --profile warehouse
spanda readiness rover.sd --profile medical --json
```

## Integration

Built on policy engine + readiness + capability verification + assurance evidence.

**Disclaimer:** Profiles are **templates** for engineering discipline, not regulatory approval.

See [policy-engine.md](./policy-engine.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
