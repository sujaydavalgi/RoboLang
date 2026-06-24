# Resilience examples

`resilience_policy`, operating modes, and recovery from degraded operation.

## Files

| File | Focus |
|------|--------|
| [`degraded_mode_recovery.sd`](degraded_mode_recovery.sd) | Graceful degradation and recovery strategies |

## Commands

```bash
spanda check examples/resilience/degraded_mode_recovery.sd
spanda resilience check examples/resilience/degraded_mode_recovery.sd --json
spanda mitigation plan examples/resilience/degraded_mode_recovery.sd
```

Guide: [docs/resilience.md](../../docs/resilience.md) · Package: `spanda-resilience` (`assurance.resilience`)
