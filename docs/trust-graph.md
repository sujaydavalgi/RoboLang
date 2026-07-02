# Trust Graph

**Status:** Stable · **Horizon:** NEXT (promoted v0.5.0) · **Priority:** P1

Visualize trust-weighted dependencies across the deployment stack.

## CLI

```bash
spanda trust-graph rover.sd
spanda trust-graph rover.sd --format mermaid
spanda trust-graph rover.sd --json
```

## Chain

```
Mission → Capability → Hardware → Package → Provider → Trust Score
```

Each node carries a trust score derived from composite trust categories; edges annotate the weakest link on each path.

## Output

- Per-node trust scores (0–100)
- Composite trust tier from `spanda trust` evaluation
- Mission-to-package paths with minimum trust along the path
- Formats: text (default), JSON, Mermaid, DOT

## Integration

Composes [dependency-graphs.md](./dependency-graphs.md) (`spanda graph`) with [trust-framework.md](./trust-framework.md) (`spanda trust` composite evaluation). Implemented in `spanda-graph` (`trust_graph` module).

See [differentiation-roadmap.md](./differentiation-roadmap.md) · [scorecards.md](./scorecards.md).
