# Compliance profile showcases

Template compliance profiles with secure boot, tamper policies, and assurance cases — not legal accreditation.

## Profiles

| Program | Profile | Standard |
|---------|---------|----------|
| `defense_rover.sd` | `defense` | Defense / secure comm |
| `medical_rover.sd` | `medical` | Medical devices |
| `automotive_rover.sd` | `iso26262` | ISO 26262 automotive |
| `machinery_rover.sd` | `iso13849` | ISO 13849 machinery (PL-oriented) |
| `iec61508_rover.sd` | `iec61508` | IEC 61508 functional safety (SIL-oriented) |

## Commands

```bash
spanda verify examples/showcase/compliance/defense_rover.sd --profile defense
spanda verify examples/showcase/compliance/medical_rover.sd --profile medical
spanda verify examples/showcase/compliance/automotive_rover.sd --profile iso26262
spanda verify examples/showcase/compliance/machinery_rover.sd --profile iso13849
spanda verify examples/showcase/compliance/iec61508_rover.sd --profile iec61508
spanda compliance report examples/showcase/compliance/defense_rover.sd --profile defense
spanda deploy gate examples/showcase/compliance/defense_rover.sd
```

Smoke: `scripts/compliance_smoke.sh`, `scripts/gaps_smoke.sh`

See [docs/compliance-profiles.md](../../../docs/compliance-profiles.md).
