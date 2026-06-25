# Dependency Graphs

**Status:** Experimental · **Phase:** Build, Operate · **Priority:** P0.1

Visualize how a Spanda program composes missions, capabilities, hardware, providers, packages, and safety rules.

## CLI

```bash
spanda graph rover.sd
spanda graph rover.sd --format mermaid
spanda graph rover.sd --format dot
spanda graph rover.sd --json
spanda graph rover.sd --config spanda.toml
```

## Graph hierarchy

```
Mission
  ↓ Capabilities
  ↓ Hardware
  ↓ Providers
  ↓ Packages
  ↓ Safety Rules
```

## Data sources

| Node type | Source |
|-----------|--------|
| Mission | `mission { }` declarations |
| Capabilities | `spanda-capability` traceability |
| Hardware | `hardware`, `deploy`, `requires_hardware` |
| Providers | Import paths + provider registry |
| Packages | `spanda.toml` dependencies |
| Safety | kill switches, robot safety blocks |

## Output formats

- **JSON** — machine-readable graph (`nodes`, `edges`, `metadata`)
- **Mermaid** — embed in docs and PRs
- **Graphviz DOT** — `dot -Tpng` for presentations
- **text** — default human-readable listing

## Crate

`spanda-graph` — static analysis only; no runtime changes.

## Integration

Composes with `spanda-capability`, `spanda-config`, and readiness traceability rows.

See [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
