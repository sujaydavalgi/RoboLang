# Rust SDK (`spanda-sdk`)

Official Rust client for Spanda Control Center API v1.

## Install

Add to your `Cargo.toml`:

```toml
[dependencies]
spanda-sdk = { path = "../crates/spanda-sdk" }
# or from crates.io when published
```

## Usage

```rust
use spanda_sdk::SpandaClient;

fn main() -> Result<(), spanda_sdk::SpandaError> {
    let client = SpandaClient::local();
    let report = client.readiness("rover.sd")?;
    println!("score = {:?}", report.score);
    Ok(())
}
```

## `SpandaClient` methods

| Method | API endpoint |
|--------|----------------|
| `readiness(file)` | `POST /v1/programs/readiness` |
| `assure(file)` | `POST /v1/programs/assure` |
| `diagnose(trace_or_file)` | `POST /v1/programs/diagnose` |
| `heal(target)` | `POST /v1/programs/recovery/heal` |
| `verify_hardware(project)` | `POST /v1/programs/verify/hardware` |
| `verify_capabilities(project)` | `POST /v1/programs/verify/capabilities` |
| `list_entities()` | `GET /v1/entities` |
| `get_entity(id)` | `GET /v1/entities/{id}` |
| `list_devices()` | `GET /v1/devices` |
| `provision_device(id, body)` | `POST /v1/devices/{id}/provision` |
| `run_simulation(project)` | `POST /v1/programs/simulation` |
| `replay(trace)` | `POST /v1/programs/replay` |
| `get_health(entity_id)` | `GET /v1/entities/{id}/health` |
| `get_trust(entity_id)` | `GET /v1/entities/{id}/trust` |
| `get_package_trust(name, version)` | `GET /v1/trust/package` |

## Authentication

```rust
let client = SpandaClient::builder()
    .base_url("https://control-center.example.com")
    .api_key(std::env::var("SPANDA_API_KEY").ok())
    .build();
```

## Event stream

```rust
use spanda_sdk::EventStream;

let stream = EventStream::local();
println!("Connect to {}", stream.url());
```

## Error handling

```rust
use spanda_sdk::SpandaError;

match client.readiness("rover.sd") {
    Err(SpandaError::Connection(msg)) => eprintln!("server down: {msg}"),
    Err(SpandaError::Permission(msg)) => eprintln!("auth: {msg}"),
    Err(e) => eprintln!("{e}"),
    Ok(report) => println!("{:?}", report.score),
}
```

## Examples

```bash
cargo run --example readiness -p spanda-sdk
cargo run --example device_inventory -p spanda-sdk
```

## Tests

```bash
cargo test -p spanda-sdk
```
