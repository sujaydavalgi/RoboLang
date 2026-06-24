# Anomaly detection examples

`anomaly_detector` declarations, `on anomaly` handlers, and optional **learned backends** (`learned backend assurance.anomaly`).

## Files

| File | Focus |
|------|--------|
| [`navigation_anomaly.sd`](navigation_anomaly.sd) | Threshold-based navigation anomaly |
| [`learned_navigation.sd`](learned_navigation.sd) | Minimal learned-detector program |

## Commands

```bash
spanda check examples/anomaly/learned_navigation.sd
spanda anomaly scan examples/anomaly/navigation_anomaly.sd --json
```

Optional ONNX model for runtime `scan_learned`:

```bash
export SPANDA_ANOMALY_ONNX_MODEL_PATH=/path/to/model.onnx
spanda run examples/anomaly/learned_navigation.sd --inject-health-faults
```

## Related

- Full showcase: [`../showcase/assurance/rover.sd`](../showcase/assurance/rover.sd)
- Package: `spanda-anomaly` (`assurance.anomaly`) — [docs/anomaly-detection.md](../../docs/anomaly-detection.md)
