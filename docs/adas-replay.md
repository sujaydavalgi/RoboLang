# ADAS Replay

Mission trace record, deterministic replay, and explainability for intelligent vehicles.

**Fixture:** `examples/solutions/adas/src/highway_drive.trace`

---

## Workflow

```bash
# Record a drive
spanda sim examples/solutions/adas/src/highway_drive.sd --record

# Deterministic replay
spanda replay examples/solutions/adas/src/highway_drive.trace --deterministic

# Diagnose events in trace
spanda diagnose examples/solutions/adas/src/highway_drive.sd \
  examples/solutions/adas/src/highway_drive.trace

# Explain decisions
spanda explain examples/solutions/adas/src/highway_drive.trace
```

---

## Replay event types

| Event | Description | Example frame |
|-------|-------------|---------------|
| `readiness_check` | Pre-drive go/no-go | Score 94, all factors pass |
| `safety_event` | Lane departure, AEB activation | Steering correction applied |
| `sensor_degradation` | Camera obstruction, radar failure | Device + severity |
| `recovery_action` | Switch sensor, restart provider | Passed safety + capability verification |
| `continuity_decision` | Degraded mode, speed reduction | Driver takeover required? |
| `driver_takeover` | Handoff to driver | Audit trail entry |

---

## Scenario replay library

Record traces for each simulation scenario:

```bash
# Collision / near miss
spanda sim automatic_emergency_braking/aeb.sd --record

# Driver takeover
spanda sim driver_takeover/driver_takeover.sd --record

# Sensor failure + recovery
spanda sim sensor_failure_recovery/camera_failure.sd --record

# Weather degradation
spanda sim src/highway_drive.sd --record --fault weather_degraded
```

---

## Golden fixtures

Committed traces for CI regression:

| Trace | Scenario |
|-------|----------|
| `src/highway_drive.trace` | Camera obstruction → radar failover → degraded mode |
| Generated AEB trace | Emergency braking activation |
| Generated takeover trace | Driver distraction → takeover request |

Smoke script validates deterministic replay: `./scripts/adas_smoke.sh`

---

## Control Center replay viewer

Launch Control Center with the ADAS blueprint:

```bash
spanda control-center serve \
  --config examples/solutions/adas/spanda.toml \
  --program examples/solutions/adas/src/highway_drive.sd
```

ADAS dashboard tab includes replay viewer linking to trace files and diagnosis summaries.

API: `GET /v1/diagnosis/summary`, `GET /v1/observability/traces`

---

## Assurance integration

Replay references are included in assurance evidence bundles:

```bash
spanda compliance report src/highway_drive.sd --profile iso26262
```

Assurance case evidence: `simulation_replay`

---

## Related

- [replay.md](./replay.md) — General replay reference
- [diagnostics.md](./diagnostics.md) — Diagnosis and explainability
- [adas-assurance.md](./adas-assurance.md) — Assurance evidence
- [solutions/adas.md](./solutions/adas.md) — Blueprint architecture
