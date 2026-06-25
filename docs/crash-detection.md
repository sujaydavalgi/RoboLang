# Crash Detection

Spanda detects process crashes, runtime panics, provider crashes, package crashes, sensor/actuator driver crashes, network stack crashes, and abnormal shutdowns.

## Detection

Crash detection triggers on:

- Abnormal process exit codes
- Unhandled runtime panics
- Provider or package termination
- Sensor or actuator driver failure
- Network stack crash

## Configuration

```spanda
restart_policy ProviderRuntime {
    max_restarts: 3 within 5 min;
    on_exceeded {
        enter degraded_mode;
        disable provider;
        request_operator_review;
    }
}

on runtime crash {
    diagnose root_cause;
    recover using RestartRuntime;
}
```

## CLI

```bash
spanda fault scan rover.sd --inject-crash
spanda fault report rover.sd --json
```

## Recovery

On crash, Spanda can:

1. Diagnose root cause from trace and static analysis
2. Attempt recovery via configured `recovery_policy`
3. Record crash event in mission trace for replay
4. Lower readiness score and block deployment if critical

Recovery actions never bypass safety validation.

## See also

- [Runtime Fault Detection](runtime-fault-detection.md)
- [Recovery policies](../examples/showcase/continuity/)
