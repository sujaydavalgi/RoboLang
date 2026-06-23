# Spanda Documentation

<p align="center">
  <img src="../../assets/image/low_res_logo.png" alt="Spanda logo" width="280">
</p>

**Spanda is an Autonomous Systems Platform with a safety-first programming language at its core.** The `.sd` language, verification engine, simulation, replay, health framework, fleet tooling, and package registry ship as one lean-core, package-first toolchain.

Platform overview: [Platform overview](../../docs/platform-overview.md)

## Quick links

| Topic | Guide |
|-------|-------|
| Platform components | [Platform Overview](../../docs/platform-overview.md) |
| Install & first program | [Getting Started](../../docs/getting-started.md) |
| Language syntax | [Language Guide](../../docs/spanda-language.md) |
| Safety & verification | [Architecture](../../docs/architecture.md) |
| Hardware profiles | [Hardware Compatibility](../../docs/hardware-compatibility.md) |
| Capabilities & traceability | [Capability Traceability](../../docs/capability-traceability.md) |
| Health monitoring | [Health Checks](../../docs/health-checks.md) |
| Tests | [Test Plan](../../docs/test-plan.md) |

Build locally:

```bash
mdbook build docs-site
mdbook serve docs-site
```

Or from npm (when configured):

```bash
npm run docs:build
```
