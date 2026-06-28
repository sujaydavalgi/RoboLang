# ADAS Assurance

Assurance evidence for intelligent vehicle deployments.

**Config:** `examples/solutions/adas/spanda.assurance.toml`

---

## Evidence bundle

Each ADAS deployment produces an assurance bundle containing:

| Evidence type | Source | CLI |
|---------------|--------|-----|
| Sensor readiness | Device tree + health checks | `spanda readiness --json` |
| Calibration status | Device metadata | `spanda device-tree inspect` |
| Capability verification | Program + registry | `spanda verify --capabilities --traceability` |
| Hardware verification | Hardware profiles | `spanda verify --target JetsonAutomotive` |
| Safety validation | Safety blocks, kill switch | `spanda verify --profile iso26262` |
| Traceability | Capability → sensor mapping | `spanda trace capabilities` |
| Replay references | Golden trace fixtures | `spanda replay --deterministic` |
| Software versions | Package lock + OTA history | `spanda compliance report` |
| OTA history | Control Center OTA API | `GET /v1/ota/status` |

---

## Assurance case

Declare in program source:

```spanda
assurance_case AdasDeployCase {
  evidence hardware_verification;
  evidence capability_traceability;
  evidence health_checks;
  evidence simulation_replay;
  evidence sensor_readiness;
  evidence calibration_status;
}
```

ISO 26262 profile requires an assurance case (`requires_assurance_case: true`).

---

## CLI workflow

```bash
# Full verification + traceability
spanda verify examples/solutions/adas/src/highway_drive.sd \
  --profile iso26262 \
  --capabilities \
  --traceability \
  --json

# Accreditation export (engineering template, not legal certification)
spanda compliance report examples/solutions/adas/src/highway_drive.sd \
  --profile iso26262 \
  --json

# Capability traceability matrix
spanda trace capabilities examples/solutions/adas/src/highway_drive.sd
```

---

## Control Center export

```bash
curl -H "Authorization: Bearer $SPANDA_API_KEY" \
  "http://127.0.0.1:8080/v1/compliance/export?profile=iso26262"

curl http://127.0.0.1:8080/v1/compliance/evidence
curl http://127.0.0.1:8080/v1/assurance/summary
```

Immutable evidence append log: `.spanda/evidence-append.jsonl`

---

## Golden replay fixtures

Retain committed traces for regression:

- `examples/solutions/adas/src/highway_drive.trace` — sensor degradation + recovery
- Generated via `spanda sim src/highway_drive.sd --record`

Replay verification:

```bash
spanda replay src/highway_drive.trace --deterministic
```

---

## Post-incident assurance

After emergency braking, driver takeover, or sensor failure:

1. `spanda diagnose` — root cause report
2. `spanda explain` — decision narrative
3. `spanda compliance report` — updated evidence bundle with incident timestamp
4. Control Center ADAS tab — assurance report with replay link

---

## Related

- [assurance-cases.md](./assurance-cases.md) — Assurance case language
- [mission-assurance.md](./mission-assurance.md) — Mission assurance CLI
- [compliance-profiles.md](./compliance-profiles.md) — ISO 26262 template
- [solutions/adas.md](./solutions/adas.md) — Blueprint architecture
