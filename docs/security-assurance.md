# Security Assurance

**Status:** Planned · **Phase:** Verify, Operate · **Priority:** P1.2, P3.1

Rollup security posture combining threat modeling, integrity verification, trust scoring, and tamper detection.

## CLI

```bash
spanda security assurance rover.sd
spanda security assurance rover.sd --json --format markdown
```

## Report contents

| Section | Source |
|---------|--------|
| Attack surface | `spanda threat-model` |
| Integrity status | `spanda integrity` |
| Trust score | `spanda trust` |
| Tamper status | `spanda tamper-check` |
| Secure comm audit | `spanda security audit` |
| Recommendations | Cross-engine synthesis |

## Output formats

JSON · Markdown · HTML

## Integration

Composes `spanda-threat`, `spanda-tamper`, `spanda-trust`, `spanda-security`, and `spanda-audit` without duplicating analysis.

## Questions answered

- Can this system be trusted?
- Has anything been modified or spoofed?
- Has anything been compromised?
- What should happen next?

See [threat-modeling.md](./threat-modeling.md) · [tamper-detection.md](./tamper-detection.md) · [security-architecture.md](./security-architecture.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
