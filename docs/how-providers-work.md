# How Providers Work

Providers are optional domain backends registered in `ProviderRegistry`. The lean core defines trait contracts; official packages supply implementations.

## Provider traits

| Trait | Package examples | Purpose |
|-------|------------------|---------|
| `SensorProvider` | — | Sensor reads (sim HAL by default) |
| `ActuatorProvider` | — | Motion / e-stop |
| `ConnectivityProvider` | spanda-wifi, spanda-ble, spanda-cellular | Wi-Fi, BLE, LTE connect |
| `PositioningProvider` | spanda-gps | GPS/GNSS fixes |
| `TransportProvider` | spanda-mqtt, spanda-ros2, spanda-dds | Pub/sub, services, actions |
| `NavigationProvider` | spanda-nav | Path planning |
| `SlamProvider` | spanda-slam | Localization / mapping |
| `VisionProvider` | spanda-opencv, spanda-yolo | Detection / classification |
| `FleetProvider` | spanda-fleet | Multi-robot orchestration |
| `SimulationProvider` | spanda-gazebo, spanda-webots | External sim stepping |
| `CryptoProvider` | — | Hash / sign / verify |
| `MaintenanceProvider` | spanda-maintenance | Health metrics |
| `LedgerProvider` | spanda-ledger | Audit / provenance |
| `CloudProvider` | spanda-cloud | Remote invoke |
| `RosProvider` | spanda-ros2 | ROS node lifecycle |

## Bootstrap

When you run a program inside a package project, the CLI loads **provenanced** official package names from `spanda.lock` / `spanda.toml` and calls:

```rust
bootstrap_providers_for_packages(&["spanda-gps", "spanda-mqtt", ...])
```

Only registry-resolved dependencies (or a path to the canonical `packages/registry/<name>` tree) count as official for provider wiring. Reusing an official name via an arbitrary path or git URL does **not** register built-in providers — calls fall back to package `.sd` stubs.

This registers transport adapters, positioning stubs, connectivity stubs, and capability grants scoped to provenanced packages.

Transport providers are also attached to `RoutingCommBus` via `sync_comm_bus_for_official_packages`.

## Package → provider dispatch

Imported official-package functions dispatch through the registry when the backing package is installed:

| Import | Function | Provider |
|--------|----------|----------|
| `positioning.gps` | `read()` | `PositioningProvider::read_fix` |
| `connectivity.wifi` | `connect()` | `ConnectivityProvider::connect` |
| `communication.mqtt` | `publish_topic()` | `TransportProvider::publish` |
| `navigation.path_planning` | `navigate()` | `NavigationProvider::navigate_to` |
| `navigation.slam` | `localize()` | `SlamProvider::localize` |
| `vision.opencv` / `vision.yolo` | `detect()` | `VisionProvider::detect` |
| `sim.gazebo` / `sim.webots` | `step()` | `SimulationProvider::step` |
| `robotics.fleet` | `dispatch()` | `FleetProvider::dispatch_task` |

If the package is not **provenanced** in `spanda.lock` (registry source or canonical `packages/registry/` path), calls fall back to the `.sd` stub body (returns placeholder values).

## Capabilities and security

Each provider registration grants capabilities (`mqtt.publish`, `positioning.read`, etc.). Dispatch checks `registry.has_capability()` before invoking a provider. Robot `permissions` and `secure` blocks further restrict communication at runtime.

## Observability

```bash
spanda run program.sd --trace-providers
spanda run program.sd --trace-realtime   # includes providers
```

Metrics include per-provider call counts, failures, and latency (`ProviderMetrics` in runtime telemetry).

## Extending providers

1. Implement the trait in a workspace crate or package adapter.
2. Register in `crates/spanda-providers/src/bootstrap.rs`.
3. Add dispatch mapping in `crates/spanda-providers/src/package_dispatch.rs` if the package exports `.sd` functions.
4. Declare capabilities in `spanda.toml`.

See [provider-interfaces.md](./provider-interfaces.md) for trait signatures.

## See also

- [How Packages Work](./how-packages-work.md)
- [How Runtime Resolution Works](./how-runtime-resolution-works.md)
