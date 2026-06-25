# Architecture Decision Records

**Status:** Experimental · **Phase:** Build · **Priority:** P2.5

Capture design rationale inferred from Spanda program declarations.

## CLI

```bash
spanda adr examples/showcase/policy/warehouse.sd
spanda adr rover.sd --json
spanda adr rover.sd --out .spanda/adr
```

## Generated records

ADR generation inspects deploy targets, capabilities, safety caps, missions, kill switches, health/recovery/operational policies, and assurance cases.

Each record includes:

- **Context** — why the decision exists
- **Decision** — what the program declares
- **Consequences** — downstream verification and operations impact

## Output

- Markdown to stdout (default)
- JSON with `--json`
- `architecture-decisions.md` in `--out <dir>`

## Crate

`spanda-adr` — AST-driven documentation artifact generation.

Smoke: `scripts/adr_smoke.sh`

See [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
