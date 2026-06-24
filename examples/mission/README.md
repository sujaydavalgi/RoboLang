# Mission assurance examples

`mission_plan` declarations and mission achievability verification.

## Files

| File | Focus |
|------|--------|
| [`mission_assurance.sd`](mission_assurance.sd) | Mission plan steps, constraints, and assurance hooks |

## Commands

```bash
spanda check examples/mission/mission_assurance.sd
spanda mission verify examples/mission/mission_assurance.sd --json
```

Guide: [docs/mission-verification.md](../../docs/mission-verification.md) · Package: `spanda-mission-planning` (`assurance.mission`)
