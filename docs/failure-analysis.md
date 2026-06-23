# Failure Impact Analysis

Failure impact analysis answers: **what happens if a component breaks?**

It evaluates common failure scenarios against the robot's declared hardware, connectivity, and capabilities.

## CLI

```bash
spanda analyze-failure examples/showcase/failure_analysis/rover.sd
spanda analyze-failure examples/showcase/failure_analysis/rover.sd --json
```

## Scenarios analyzed

| Failure | Typical mitigation |
|---------|-------------------|
| GPS | Switch to visual odometry |
| Camera | Obstacle avoidance degraded; reduce speed |
| Lidar | Halt autonomous motion |
| LTE / WiFi | Offline mode; queue telemetry |
| Battery | Return to base; reduce mission scope |
| Provider / Package | Fallback provider or safe stop |

## Output

```
If GPS fails:
  Switch to visual odometry

If Camera fails:
  Reduce speed and rely on Lidar

If LTE fails:
  Offline mode activated; queue telemetry locally
```

## API

```rust
use spanda_readiness::{analyze_failure, FailureAnalysisReport};
```

Combine with [simulation fault injection](../examples/showcase/health_monitoring/) and [root cause analysis](root-cause-analysis.md) for full failure lifecycle coverage.
