# Diagnostics examples

Fault diagnosis and root-cause analysis with `spanda diagnose`.

## Files

| File | Focus |
|------|--------|
| [`gps_failure.sd`](gps_failure.sd) | GPS loss scenario for diagnosis reports |

## Commands

```bash
spanda check examples/diagnostics/gps_failure.sd
spanda diagnose examples/diagnostics/gps_failure.sd --json
```

Guide: [docs/diagnostics.md](../../docs/diagnostics.md) · Package: `spanda-diagnosis` (`assurance.diagnosis`)
