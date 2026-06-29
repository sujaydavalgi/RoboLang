# Entity Model Overview

The **Unified Entity Model** is Spanda's foundational abstraction: every managed object — robots, humans, devices, packages, providers, missions, and facilities — is an **Entity** with consistent identity, evaluation, and lifecycle semantics.

## Why it exists

| Before | With Entity Model |
|--------|-------------------|
| Separate verify/readiness paths per kind | `verify_entity`, `evaluate_entity_readiness`, … |
| Fragmented Control Center navigation | **Entities** tab + domain pages |
| Ad-hoc dependency questions | Entity graph + query language |

## Architecture at a glance

```text
Entity
 ├── Health      (evaluate_entity_health)
 ├── Readiness   (evaluate_entity_readiness)
 ├── Trust       (evaluate_entity_trust)
 ├── Verification (verify_entity)
 ├── Relationships & graph traversal
 └── Capabilities, lifecycle, security
```

## Quick start

```bash
CONFIG=crates/spanda-config/tests/fixtures/warehouse/spanda.toml
spanda entity list --config "$CONFIG"
spanda entity verify rover-001 --config "$CONFIG"
spanda entity readiness rover-001 --config "$CONFIG"
```

## Documentation map

| Guide | Topic |
|-------|-------|
| [entity-model.md](./entity-model.md) | Core concepts and migration phases |
| [entity-apis.md](./entity-apis.md) | REST and gRPC endpoint reference |
| [entity-sdk.md](./entity-sdk.md) | Rust, TypeScript, and Python clients |
| [entity-registry.md](./entity-registry.md) | Registry projection and inventory |
| [entity-graph.md](./entity-graph.md) | Graph traversal and impact analysis |
| [entity-relationships.md](./entity-relationships.md) | Relationship taxonomy |
| [entity-query-language.md](./entity-query-language.md) | Structured queries |
| [entity-verification.md](./entity-verification.md) | `verify_entity` |
| [entity-readiness.md](./entity-readiness.md) | `evaluate_entity_readiness` |
| [entity-health.md](./entity-health.md) | `evaluate_entity_health` |
| [entity-trust.md](./entity-trust.md) | `evaluate_entity_trust` |
| [entity-best-practices.md](./entity-best-practices.md) | Adoption patterns |
| [entity-migration-guide.md](./entity-migration-guide.md) | Backward-compatible migration |
| [entity-integration-report.md](./entity-integration-report.md) | Phase status report |

## Examples

Runnable programs: [examples/entity/](../examples/entity/)

## Backward compatibility

Existing `/v1/robots`, `/v1/devices`, `/v1/fleets`, `spanda verify`, and domain TOML schemas are unchanged. Entity APIs are additive.
