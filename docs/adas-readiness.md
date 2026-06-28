# ADAS Readiness

Operational go/no-go gates before enabling ADAS functions.

**Config:** `examples/solutions/adas/spanda.readiness.toml` · **Profile:** ISO 26262 (min score 90)

---

## Pre-drive checklist

Before highway pilot, lane keeping, or any ADAS function activates, the Readiness Engine evaluates:

| Factor | Weight | Checks |
|--------|--------|--------|
| Sensor availability | 25% | Required devices online and healthy |
| Calibration valid | 20% | Camera, radar, IMU within max age |
| Firmware approved | 15% | ECU and compute versions in allowlist |
| Vehicle health | 15% | Battery, tire pressure, brake system |
| Safety systems | 10% | Kill switch, emergency stop, secure comm |
| Connectivity | 5% | LTE/WiFi when OTA or V2X required |
| Trust score | 10% | Package trust, tamper status, attestation |

---

## Required sensors

From `spanda.readiness.toml`:

**Required:** front camera, front radar, GPS, IMU, brake ECU

**Optional (degraded mode without):** front LiDAR, stereo camera, driver monitor

If a required sensor is offline, readiness score drops below threshold and ADAS functions are blocked.

---

## CLI

```bash
# Full readiness report
spanda readiness examples/solutions/adas/src/highway_drive.sd \
  --profile iso26262 \
  --config examples/solutions/adas/spanda.toml \
  --json

# With runtime health (--runtime flag)
spanda readiness examples/solutions/adas/src/highway_drive.sd \
  --profile iso26262 \
  --config examples/solutions/adas/spanda.toml \
  --runtime

# Per-function readiness (check before enabling specific ADAS)
spanda readiness examples/solutions/adas/lane_keeping/lane_keeping.sd \
  --profile iso26262
```

---

## Function-specific gates

| ADAS function | Minimum capabilities | Minimum readiness |
|---------------|---------------------|-------------------|
| Lane Keeping Assist | `lane_detection`, `steering_control` | 85 |
| Adaptive Cruise Control | `adaptive_speed_control`, `obstacle_detection` | 90 |
| Automatic Emergency Braking | `emergency_braking`, `obstacle_detection` | 95 |
| Highway Pilot | Full capability set + `driver_monitoring` | 90 |
| Parking Assist | `parking_assist` | 80 |
| Low-Speed Autonomy | `steering_control`, `obstacle_detection` | 85 |

Function-specific thresholds are enforced via mission `requires capabilities` blocks; readiness score gates the overall deploy decision.

---

## Control Center

```bash
spanda control-center serve \
  --config examples/solutions/adas/spanda.toml \
  --program examples/solutions/adas/src/highway_drive.sd
```

- `POST /v1/readiness/run` — evaluate readiness with device pool
- `GET /v1/analytics/readiness` — readiness trends
- ADAS dashboard tab — readiness score, factor breakdown, blocked functions

---

## Integration with compliance

ISO 26262 profile requires min readiness score 90, kill switch, secure comm, tamper policy, assurance case, and ≥2 health checks. ADAS blueprint adds ≥4 health checks and sensor/calibration gates.

```bash
spanda verify src/highway_drive.sd --profile iso26262 --json
```

---

## Related

- [readiness.md](./readiness.md) — Readiness engine reference
- [solutions/adas.md](./solutions/adas.md) — Blueprint architecture
- [compliance-profiles.md](./compliance-profiles.md) — ISO 26262 profile
