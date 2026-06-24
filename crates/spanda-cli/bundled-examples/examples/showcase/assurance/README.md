# Mission assurance showcase

NASA-style autonomous operations in one program: knowledge model, state estimation, anomaly detection, prognostics, mitigation, resilience, and assurance evidence.

## One command

```bash
spanda demo assurance
```

## Manual path

```bash
spanda check examples/showcase/assurance/rover.sd
spanda assure examples/showcase/assurance/rover.sd --json
spanda anomaly scan examples/showcase/assurance/rover.sd
spanda state estimate examples/showcase/assurance/rover.sd
spanda prognostics examples/showcase/assurance/rover.sd
spanda mission verify examples/showcase/assurance/rover.sd
spanda resilience check examples/showcase/assurance/rover.sd
spanda mitigation plan examples/showcase/assurance/rover.sd
spanda readiness examples/showcase/assurance/rover.sd --target RoverV1 --json
```

See [docs/mission-assurance.md](../../../docs/mission-assurance.md).
