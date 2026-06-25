# Memory Leak Detection

Spanda tracks memory growth over time and raises warnings when configured thresholds are exceeded.

## Configuration

```spanda
memory_watch RoverRuntime {
    threshold growth > 100 MB over 10 min;
    action {
        mark Warning;
        create MemoryLeakEvent;
    }
}

on memory_leak detected {
    mark Degraded;
    notify_operator;
}
```

## Detection logic

1. Record baseline memory at window start
2. Sample memory at regular intervals
3. Compare growth against threshold over the configured window
4. Emit `MemoryLeakEvent` and record in mission trace

## Readiness impact

A memory leak warning reduces the health readiness factor score by 15 points.

## CLI

```bash
spanda fault scan rover.sd --inject-memory-leak
spanda fault report rover.sd --json
```

## See also

- [Runtime Fault Detection](runtime-fault-detection.md)
- [Runtime Health](runtime-health.md)
