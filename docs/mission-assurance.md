# Mission Assurance

Spanda provides NASA-style mission assurance as a **lean-core platform layer** integrated with existing readiness, health, traceability, and safety systems.

## Architecture

```
.sd declarations  →  spanda-assurance (core interfaces + static analysis)
                   →  spanda-readiness (operational readiness hub)
                   →  spanda-capability (health, traceability)
                   →  optional packages (spanda-anomaly, spanda-diagnosis, …)
```

Core defines **interfaces and data models**. Heavy algorithms (ML anomaly detection, advanced prognostics) live in optional packages under `packages/registry/`.

## Language constructs

| Construct | Purpose |
|-----------|---------|
| `knowledge_model` | System model, components, dependencies |
| `state_estimator` | Sensor fusion inputs and estimate type |
| `anomaly_detector` | Expected behavior bounds |
| `on anomaly …` | Automated reactions |
| `prognostics` | RUL prediction and degradation warnings |
| `mitigation` | Conditional recovery actions |
| `operating_mode` | Normal / degraded / safe / emergency |
| `mission_plan` | Steps and constraints |
| `resilience_policy` | Fault tolerance strategies |
| `assurance_case` | Links evidence sources |

At runtime, `state_estimator` wires `SensorFusion` bindings after robot sensors are registered. One estimator also aliases `fusion` for parity with `observe { }` programs.

## CLI

```bash
spanda assure rover.sd
spanda anomaly scan rover.sd
spanda state estimate rover.sd
spanda diagnose mission.trace    # or program .sd
spanda prognostics rover.sd
spanda mission verify mission.sd
spanda resilience check rover.sd
spanda mitigation plan rover.sd
```

All commands support `--json`, `--markdown`, and `--html`.

### One-command demo

```bash
spanda demo assurance
```

Runs `check`, `assure`, `anomaly scan`, `state estimate`, `prognostics`, `mission verify`, `resilience check`, `mitigation plan`, and `readiness` on [`examples/showcase/assurance/rover.sd`](../examples/showcase/assurance/rover.sd).

## Official packages (mission assurance)

| Package | Import path | Role |
|---------|-------------|------|
| `spanda-assurance` | `assurance.evidence` | Assurance case and evidence scaffolds |
| `spanda-knowledge-model` | `assurance.knowledge` | Knowledge models |
| `spanda-anomaly` | `assurance.anomaly` | Learned anomaly backends (`scan_learned`, optional ONNX) |
| `spanda-fusion` | `assurance.fusion` | Weighted fusion weights (`weight_for_sensor`, `confidence_for_types`) |
| `spanda-diagnosis` | `assurance.diagnosis` | Fault diagnosis |
| `spanda-prognostics` | `assurance.prognostics` | RUL and degradation |
| `spanda-mission-planning` | `assurance.mission` | Mission planning assurance |
| `spanda-resilience` | `assurance.resilience` | Resilience and recovery |

Lean core provides static analysis (`spanda-assurance` crate) and runtime wiring (fusion, learned anomaly polling). Packages extend algorithms via provider dispatch.

### Learned anomaly + ONNX

```spanda
anomaly_detector NavigationML {
    learned backend assurance.anomaly;
    expected localization.confidence >= 0.80;
}
```

Runtime invokes `scan_learned(detector, observed, volatility)`. Set `SPANDA_ANOMALY_ONNX_MODEL_PATH` (or `SPANDA_ONNX_MODEL_PATH`) for ONNX inference (2-feature input: observed, volatility). See [anomaly-detection.md](anomaly-detection.md).

### Weighted sensor fusion

`state_estimator` and `fusion.read()` use per-sensor-type weights in lean core. Optional package:

```spanda
// In a project with spanda-fusion installed:
// import assurance.fusion;
// assurance.fusion.confidence_for_types("GPS,Lidar,IMU");
```

See [state-estimation.md](state-estimation.md).

## Reports

- **Assurance report** — evidence, verification, traceability
- **Anomaly report** — detectors, violations, handler coverage
- **State estimation report** — estimators, belief state, fusion inputs
- **Diagnosis report** — root cause, causal graph, trace timeline
- **Prognostics report** — RUL, degradation warnings
- **Mitigation plan** — recovery actions and mode transitions
- **Mission assurance report** — plan verification + readiness mission checks
- **Resilience report** — policies, recovery, readiness score

## Integration

Mission assurance **composes** with:

- `spanda verify` / hardware verification
- `spanda trace` / traceability matrices
- `spanda health` / health checks (not duplicated)
- `spanda readiness` / fleet readiness
- `spanda audit` / provenance
- Digital twin, replay, kill switch

## Examples

### Showcase (start here)

| Example | Command |
|---------|---------|
| [`examples/showcase/assurance/rover.sd`](../examples/showcase/assurance/rover.sd) | `spanda demo assurance` |

### By domain

| Directory | Example | CLI |
|-----------|---------|-----|
| [`examples/assurance/`](../examples/assurance/README.md) | `rover_assurance.sd` | `spanda assure` |
| [`examples/anomaly/`](../examples/anomaly/README.md) | `learned_navigation.sd`, `navigation_anomaly.sd` | `spanda anomaly scan` |
| [`examples/diagnostics/`](../examples/diagnostics/README.md) | `gps_failure.sd` | `spanda diagnose` |
| [`examples/prognostics/`](../examples/prognostics/README.md) | `battery_degradation.sd` | `spanda prognostics` |
| [`examples/resilience/`](../examples/resilience/README.md) | `degraded_mode_recovery.sd` | `spanda resilience check`, `mitigation plan` |
| [`examples/mission/`](../examples/mission/README.md) | `mission_assurance.sd` | `spanda mission verify` |

### Sensor fusion (related)

| Example | Topic |
|---------|--------|
| [`examples/basics/11_observe_and_fusion.sd`](../examples/basics/11_observe_and_fusion.sd) | `observe { }` + `fusion.read()` |
| [`examples/robotics/sensor_fusion.sd`](../examples/robotics/sensor_fusion.sd) | Multi-sensor fusion patrol |
| [`examples/showcase/world_model_patrol.sd`](../examples/showcase/world_model_patrol.sd) | Fusion → belief hook |

## Related docs

- [Knowledge models](knowledge-models.md)
- [State estimation](state-estimation.md)
- [Anomaly detection](anomaly-detection.md)
- [Diagnostics](diagnostics.md)
- [Prognostics](prognostics.md)
- [Resilience](resilience.md)
- [Assurance cases](assurance-cases.md)
- [Readiness](readiness.md)
