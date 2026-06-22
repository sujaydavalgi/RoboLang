# spanda-runtime

Runtime kernel pieces extracted from `spanda-core` for the Phase 4 lean-core split:

- **scheduler** — sim vs wall-clock tick helpers
- **provider_types** — `ProviderId`, `ProviderRegistry` metadata types
- **classification** — module ownership audit table
- **robotics** — `MissionRuntime`, `FleetRegistry`, zone registries

The interpreter (`Interpreter`), `RuntimeValue`, and provider trait registry remain in `spanda-core` for now.
