# Autonomous Rover — Flagship Platform Demo

Primary end-to-end demonstration of Spanda as an autonomous-systems platform:

**Language → Packages → Providers → Runtime → Hardware → Simulation → Verification → Deployment**

## Prerequisites

From the project directory:

```bash
cd examples/showcase/autonomous_rover
spanda install    # resolve spanda-gps, spanda-mqtt, spanda-wifi, spanda-nav
```

## User flow

### 1. Verify hardware fit

```bash
spanda verify src/rover.sd
spanda verify src/rover.sd --json --target RoverV1
```

Checks memory, sensors, connectivity, package requirements, and task budgets before deploy.

### 2. Simulate without hardware

```bash
spanda sim src/rover.sd
spanda sim src/rover.sd --record
```

Runs patrol logic with simulated lidar/camera, safety `stop_if`, GPS/connectivity triggers, and AI planning.

### 3. Run with provider wiring

```bash
spanda run src/rover.sd --trace-providers
spanda run src/rover.sd --trace-realtime --metrics-json
```

Official packages (`spanda-gps`, `spanda-mqtt`, `spanda-wifi`) bootstrap providers at runtime. Imported module calls (`read()`, `connect()`, `publish_topic()`) dispatch through `ProviderRegistry`.

### 4. Replay a mission

```bash
spanda replay src/rover.trace
spanda replay src/rover.trace --deterministic
```

## What this demonstrates

| Capability | How |
|------------|-----|
| GPS | `import positioning.gps` → `GpsPositioningStub` via provider registry |
| MQTT | `import communication.mqtt` → transport provider publish |
| WiFi | `import connectivity.wifi` → connectivity provider |
| AI planning | `planner.reason()` + `safety.validate()` gate |
| Safety | `stop_if`, `safety.validate()`, compile-time ActionProposal gate |
| Hardware verify | `requires_hardware`, `deploy … to RoverV1` |
| Triggers | `on gps.lost`, `on network.disconnected` |
| Simulation | `simulate_compatibility` faults |
| Replay | `--record` → `spanda replay` |
| Observability | `--trace-providers`, `--trace-realtime` |

## Related docs

- [How Packages Work](../../../docs/how-packages-work.md)
- [How Providers Work](../../../docs/how-providers-work.md)
- [How Runtime Resolution Works](../../../docs/how-runtime-resolution-works.md)
- [Killer demo walkthrough](../../../docs/killer-demo.md)
