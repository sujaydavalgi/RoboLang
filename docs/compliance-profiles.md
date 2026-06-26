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
- Tamper response policy (`tamper_policy`) for defense and medical profiles
- Secure-boot contract import (`trust.jetson` / `trust.pi`) for defense and medical profiles

Reports include an explicit **template notice** — profiles are engineering templates, not legal accreditation.

## CLI

```bash
spanda verify examples/showcase/policy/warehouse.sd --profile warehouse
spanda verify rover.sd --profile medical --json
spanda readiness rover.sd --profile medical
spanda compliance report examples/showcase/compliance/defense_rover.sd --profile defense
spanda compliance report examples/showcase/compliance/defense_rover.sd --profile defense --json
```

`spanda compliance report` exports an **accreditation bundle** with evidence checklist, audit export ID, and explicit `template_only` status — suitable for engineering audit trails, not legal certification.

## Integration

Built on readiness, capability verification, and assurance evidence checks in `spanda-compliance`.

**Disclaimer:** Profiles are **templates** for engineering discipline, not regulatory approval.

Showcase: `examples/showcase/policy/warehouse.sd`, `examples/showcase/compliance/defense_rover.sd`, `examples/showcase/compliance/medical_rover.sd` · smoke: `scripts/compliance_smoke.sh`, `scripts/gaps_smoke.sh`

See [policy-engine.md](./policy-engine.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
