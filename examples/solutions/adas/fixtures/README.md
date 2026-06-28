# ADAS scenario trace fixtures

Narrative mission traces for **playback**, **diagnose**, and **explain** demos. Not used for `--deterministic` replay (use `src/highway_drive.trace` from `spanda sim --record`).

| Trace | Scenario |
|-------|----------|
| `aeb_activation.trace` | Emergency braking activation |
| `camera_failure_recovery.trace` | Camera obstruction → radar failover → degraded mode |
| `driver_takeover.trace` | Driver distraction → takeover request |

```bash
spanda replay fixtures/aeb_activation.trace --playback
spanda diagnose src/highway_drive.sd fixtures/camera_failure_recovery.trace
spanda explain driver_takeover/driver_takeover.sd fixtures/driver_takeover.trace
```
