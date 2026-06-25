# Runtime Fault Detection

Spanda provides runtime fault detection for autonomous systems — detecting memory leaks, crashes, reboots, hangs, restarts, and resource exhaustion. Fault detection integrates with health, readiness, diagnosis, recovery, replay, audit, and assurance systems.

## Overview

Runtime fault detection monitors:

- **Memory leaks** — sustained memory growth over configurable windows
- **Crashes** — process, provider, package, and runtime panics
- **Reboots** — unexpected robot/device/OS reboots
- **Watchdog timeouts** — task and pipeline deadline violations
- **Restart loops** — repeated crash/restart within policy windows
- **Resource pressure** — CPU, memory, disk, GPU, battery, network
- **Deadlocks / starvation** — task not progressing, queue stuck
- **Heartbeat loss** — runtime heartbeat missed beyond timeout

## Declarations

### Heartbeat monitoring

```spanda
heartbeat RoverRuntime every 1s timeout 5s {
    on_missed {
        mark Degraded;
        audit.record("runtime_heartbeat_missed");
    }
}
```

### Memory leak detection

```spanda
memory_watch RoverRuntime {
    threshold growth > 100 MB over 10 min;
    action {
        mark Warning;
        create MemoryLeakEvent;
    }
}
```

### Resource pressure

```spanda
resource_watch {
    memory > 85%;
    cpu > 90% for 30s;
    disk_free < 500 MB;
}
```

### Restart loop policy

```spanda
restart_policy ProviderRuntime {
    max_restarts: 3 within 5 min;
    on_exceeded {
        enter degraded_mode;
        disable provider;
        request_operator_review;
    }
}
```

### Runtime fault triggers

```spanda
on runtime crash {
    diagnose root_cause;
    recover using RestartRuntime;
}

on memory_leak detected {
    mark Degraded;
    notify_operator;
}

on reboot unexpected {
    run post_reboot_diagnostics;
}

on restart_loop detected {
    enter SafeMode;
}
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `spanda fault scan <file.sd>` | Scan program for runtime faults |
| `spanda runtime health <file.sd>` | Show runtime health summary |
| `spanda runtime diagnose <mission.trace>` | Diagnose faults from mission trace |
| `spanda fault report <file.sd>` | Full fault report with readiness impact |
| `spanda fault report <file.sd> --json` | JSON fault report |
| `spanda fault report <file.sd> --html` | HTML fault report |
| `spanda replay <trace> --show-faults` | Show fault events in mission trace |

### Injection flags (testing)

- `--inject-crash` — simulate process crash
- `--inject-memory-leak` — simulate memory leak
- `--inject-reboot` — simulate unexpected reboot
- `--inject-heartbeat-loss` — simulate missed heartbeat
- `--inject-resource-pressure` — simulate resource pressure

## Integration

### Readiness

Runtime faults affect readiness scoring:

- Recent crash lowers readiness to **NotReady**
- Restart loop blocks deployment
- Memory leak warning reduces health factor score
- Unexpected reboot requires operator review

### Assurance

Runtime reliability evidence includes uptime, crash-free duration, reboot history, memory stability, watchdog coverage, and restart policy configuration.

### Diagnosis

The diagnosis engine explains: what crashed, when, likely cause, affected components, and recovery success.

### Recovery

Allowed recovery actions (always safety-validated):

- Restart provider / package
- Switch provider
- Enter degraded mode
- Pause mission
- Trigger kill switch (actuator faults)
- Request operator approval

### Replay

Fault events are recorded in mission traces as `fault_crash`, `fault_reboot`, `fault_watchdog_timeout`, `fault_memory_growth`, `fault_restart_loop`, and `fault_resource_pressure` frames.

## Examples

See `examples/showcase/runtime_faults/`:

- `memory_leak_detection.sd`
- `crash_detection.sd`
- `reboot_detection.sd`
- `restart_loop.sd`
- `watchdog_timeout.sd`

## Related docs

- [Crash Detection](crash-detection.md)
- [Reboot Detection](reboot-detection.md)
- [Memory Leak Detection](memory-leak-detection.md)
- [Runtime Health](runtime-health.md)
- [Health Checks](health-checks.md)
- [Readiness](readiness.md)
- [Replay](replay.md)
- [Watchdogs](watchdogs.md)
