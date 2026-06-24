# Takeover

Hot takeover when a patrol robot's battery becomes critical.

```bash
spanda takeover examples/showcase/takeover/patrol.sd \
  --failed RoverAlpha --successor RoverBeta --trigger battery_critical --progress 45
```

Expected: **hot_takeover** mode with immediate successor replacement.
