# Reboot Detection

Spanda detects unexpected robot/device reboots and OS reboots, tracking boot identity, uptime, and shutdown reason.

## Tracked signals

- Boot ID changes
- Uptime resets
- Last shutdown reason
- Reboot reason
- Unexpected reboot count

## Configuration

```spanda
heartbeat RoverRuntime every 1s timeout 5s {
    on_missed {
        mark Degraded;
        audit.record("runtime_heartbeat_missed");
    }
}

on reboot unexpected {
    run post_reboot_diagnostics;
}
```

## Readiness impact

An unexpected reboot:

- Lowers readiness score
- Requires operator review before deployment
- Triggers post-reboot diagnostics

## CLI

```bash
spanda fault scan rover.sd --inject-reboot
spanda runtime health rover.sd
```

## See also

- [Runtime Fault Detection](runtime-fault-detection.md)
- [Runtime Health](runtime-health.md)
