# Troubleshooting

Symptom-first fixes for CLI install, language errors, verification, runtime, fleet, integrations, and enterprise ops. For capability tiers and honest scope limits, see [known-limitations.md](./known-limitations.md). For language-specific mistakes, see [spanda-for-dummies/06-common-mistakes.md](./spanda-for-dummies/06-common-mistakes.md).

**Related:** [installation.md](./installation.md) · [getting-started.md](./getting-started.md) · [control-center.md](./control-center.md) · [ci-verify.md](./ci-verify.md)

---

## Quick diagnosis

| Symptom | Start here |
|---------|------------|
| `spanda: command not found` | [CLI not on PATH](#cli-not-on-path) |
| `Unknown command: control-center` or `Unknown argument: generate` | [Stale CLI binary](#stale-cli-binary) |
| `check` / compile errors | [Compile and type errors](#compile-and-type-errors) |
| `verify` fails in CI or locally | [Hardware verify failures](#hardware-verify-failures) |
| `run` / `sim` hangs or shows nothing | [Run and simulation](#run-and-simulation) |
| Package / import / provider issues | [Packages and registry](#packages-and-registry) |
| `deploy gate` / rollout blocked | [Deploy and OTA gates](#deploy-and-ota-gates) |
| Control Center 401, blank UI, port errors | [Control Center](#control-center) |
| Fleet / mesh / remote orchestrate fails | [Fleet and mesh](#fleet-and-mesh) |
| ROS2 topics silent | [ROS2 integration](#ros2-integration) |
| AI always returns mock output | [Live AI and extern bridges](#live-ai-and-extern-bridges) |
| IoT / MQTT / Modbus not live | [Live IoT and transports](#live-iot-and-transports) |
| Plugin install / CLI namespace errors | [Plugins](#plugins) |
| VS Code diagnostics or debug broken | [Editor, LSP, and debugging](#editor-lsp-and-debugging) |
| CI passes locally but fails in pipeline | [CI and release builds](#ci-and-release-builds) |

**First commands on any issue:**

```bash
which spanda && spanda --version
spanda check <file.sd> --json    # structured diagnostics
spanda verify <file.sd> --json --target <HardwareProfile>
```

---

## CLI installation

### CLI not on PATH

```text
spanda: command not found
```

**Fix:**

```bash
# From a clone
npm run build:rust
export PATH="$PWD/target/release:$PATH"

# Or install to ~/.cargo/bin
./scripts/install.sh
```

Confirm: `spanda check examples/hello_world.sd`

### Stale CLI binary

Several installs can coexist (prebuilt release, `cargo install`, repo `target/debug` or `target/release`). The shell uses the first match on `PATH`.

```bash
which spanda
spanda --version
ls -la "$(which spanda)"
```

| Symptom | Likely cause |
|---------|----------------|
| `spanda --version` is old; `control-center` missing | Stale `~/.cargo/bin/spanda` from an earlier install |
| Works in repo, fails in a new terminal | Different `PATH` between shells |
| `which spanda` → `target/debug/spanda` | Repo dev build (expected); reinstall if outdated |

**Fix:**

```bash
./scripts/install.sh
# or:
cargo install --path crates/spanda-cli --locked --force
```

Verify enterprise-ops commands:

```bash
spanda control-center --help
spanda control-center api-key generate --export
```

### `Unknown command: control-center`

The `spanda` on your `PATH` predates Control Center CLI support.

```bash
spanda control-center --help
```

If that fails, reinstall (above) or use a current [GitHub Release](https://github.com/Davalgi/Spanda/releases). Inside the repo without installing:

```bash
cargo run -p spanda -- control-center serve --bind 127.0.0.1:8080
```

### `Unknown argument: generate` (misleading)

```bash
spanda control-center api-key generate --export
# Unknown argument: generate
# (usage has no control-center section)
```

Same stale-binary issue: the CLI treats `api-key` as a filename, then fails on `generate`. Reinstall and retry.

### Typo: `api-keygenerate`

```bash
# Wrong
spanda control-center api-keygenerate --export

# Correct — space between subcommands
spanda control-center api-key generate --export
```

### Wrong file or project root

```text
Error reading foo.sd: No such file or directory
Error loading manifest: ...
```

| Cause | Fix |
|-------|-----|
| Path typo | Run from repo root or pass absolute path |
| `spanda run` without `spanda.toml` parent | `cd` to project root or use `--project <dir>` |
| Missing `spanda install` after `spanda add` | Run `spanda install` or `spanda build` |

---

## Compile and type errors

Run `spanda check <file.sd> --json` and fix the **first** diagnostic. Common cases:

| Error / message | Cause | Fix |
|-----------------|-------|-----|
| `ActionProposal` cannot be used where `SafeAction` expected | Executing raw AI output | `let action = safety.validate(proposal);` then `execute(action)` |
| Unit mismatch / expected unit | Missing `m/s`, `rad`, `ms` | Attach units: `0.5 m/s`, `0.1 rad/s` |
| Module / import not found | Wrong path or missing package | Match `module` to file layout; run `spanda install` |
| Unknown identifier / type error | Typo or missing `hardware` / `sensor` decl | Compare with working example in `examples/basics/` |
| `remote_signed` kill switch errors | No signature material configured | Add trust key / signature env — see [kill-switch.md](./kill-switch.md) |

**Examples:** broken vs fixed AI safety — `examples/showcase/ai_safety_violation.sd` vs `rover_navigation.sd`.

Full beginner list: [06-common-mistakes.md](./spanda-for-dummies/06-common-mistakes.md).

---

## Hardware verify failures

`spanda verify` answers: *can this program run on the declared deploy target?*

```bash
spanda verify src/main.sd --json --target RoverV1
```

| Symptom | Cause | Fix |
|---------|-------|-----|
| `compatible: false`, sensor mismatch | Sensor declared but not on hardware profile | Add sensor to `hardware { sensors [...] }` or remove from robot |
| Memory / timing / battery errors | Program exceeds profile limits | Read JSON `items[]`; reduce tasks, models, or change target |
| Missing deploy mapping | No `deploy Robot to Hardware;` | Add deploy line matching your `--target` |
| CI fails only on one matrix cell | `--all-targets` found a weak profile | Filter nightly matrix or fix per-target issues |
| `verify` unavailable | Native CLI not built | `npm run build:rust` — TS fallback covers most checks, not all |

**CI rule:** fail on `ok: false` or `compatible: false` in JSON — do not parse human stdout. Templates: [ci-verify.md](./ci-verify.md).

---

## Run and simulation

| Symptom | Cause | Fix |
|---------|-------|-----|
| `run` appears to do nothing | Simulation stops quickly or drives slowly | `spanda sim file.sd`; add `--trace-scheduler` / `--trace-tasks` |
| Hangs / runs forever | Control loop with `loop every` | Normal — sim uses `maxLoopIterations`; see [06-common-mistakes.md](./spanda-for-dummies/06-common-mistakes.md) |
| Mock sensor values only | Live providers not enabled | Set env flags (`SPANDA_ROS2_LIVE=1`, `SPANDA_LIVE_AI=1`, etc.) |
| `--enforce-certify` fails | Program lacks certify metadata | `spanda certify prove file.sd` or remove flag for dev |
| Trace empty on replay | Forgot `--record` | `spanda run file.sd --record` then `spanda replay mission.trace` |
| Deterministic replay differs | Wall-clock or non-deterministic provider | `spanda replay mission.trace --deterministic` |
| Scheduler trace noisy | Many periodic tasks | Use `--trace-scheduler` only when debugging timing |

Physics is **2D lite** — logic and safety testing, not Gazebo-class fidelity. See [known-limitations.md](./known-limitations.md).

---

## Packages and registry

| Symptom | Cause | Fix |
|---------|-------|-----|
| Provider stub only (no live I/O) | Official name overridden with `path` / `git` | Use registry version in `spanda.toml`; see [how-packages-work.md](./how-packages-work.md) |
| `official_provenance` warning | Name squatting on official package | Point to `packages/registry/<name>` or use version dep |
| `spanda add` then import fails | Lockfile not refreshed | `spanda install` or `spanda update` |
| Registry fetch fails offline | Remote index unreachable | Set `SPANDA_REGISTRY_URL` or use bundled slice (auto when unset) |
| Signature / checksum errors | Production policy enabled | `SPANDA_REGISTRY_REQUIRE_SIGNATURE=1` needs signed `registry/index.json` entries |
| `spanda publish` no remote upload | No registry URL | `export SPANDA_REGISTRY_URL=...` — local mirror still updates `registry/packages/` |

**Production gate:**

```bash
export SPANDA_REGISTRY_REQUIRE_SIGNATURE=1
spanda deploy gate src/main.sd --policy production --config spanda.toml
```

See [deployment-gates.md](./deployment-gates.md) · [registry.md](./registry.md).

---

## Deploy and OTA gates

| Symptom | Cause | Fix |
|---------|-------|-----|
| `deploy gate` exit 1 | Readiness, safety, trust, or provenance failed | `spanda deploy gate file.sd --json` — read failing gate name |
| `--policy production` fails | Path override, missing signatures, low trust | Fix lockfile provenance; enable signature env |
| `deploy rollout --remote` connection refused | Agent not running or wrong URL | `spanda deploy agent start`; `spanda deploy agent register` |
| `--require-certify` blocked | No certify proof | `spanda certify prove file.sd` |
| OTA execute fails live test | Deploy agent down or token missing | Start agent; set mesh/API tokens per [ota docs](./enterprise-operations-roadmap.md) |
| Rollback not triggered | Readiness gate env unset | `SPANDA_OTA_ROLLBACK_ON_READINESS_FAIL=1` for auto-rollback |

Agent readiness probe:

```bash
spanda deploy agent readiness Rover@JetsonOrin --runtime --json
```

---

## Control Center

### API key workflow

```bash
eval "$(spanda control-center api-key generate --export)"
spanda control-center serve --bind 127.0.0.1:8080
```

Manual dev key: `export SPANDA_API_KEY="my-local-dev-key"`. Same value as `Authorization: Bearer …` for mutations. Full guide: [control-center.md — Authentication](./control-center.md#authentication--api-keys).

### Serve and UI

| Issue | What to try |
|-------|-------------|
| Port already in use | `spanda control-center status --discover`; stop orphans with `spanda control-center stop`, or `--bind 127.0.0.1:9090` for a new server |
| Mutations 401/403 | Set `SPANDA_API_KEY` or `SPANDA_API_KEYS_FILE` on server; paste token in embedded UI banner |
| Token not remembered after refresh | Ensure **Remember on this browser** is checked before saving; token is scoped per `host:port` (`spanda.control_center.bearer_token.v1:<host>`) — a new bind port needs a fresh paste; private/incognito mode may block `localStorage` |
| Clear stored browser token | Click **Forget token** in the UI banner, or DevTools → Application → Local Storage → remove `spanda.control_center.bearer_token.v1:<host>` |
| Token save fails in UI | Server must accept the token (`POST /v1/alerts/test`); restart `serve` after changing `SPANDA_API_KEYS_FILE`; reinstall CLI if embedded HTML is stale (`cargo install --path crates/spanda-cli --force`) |
| Remote CLI cannot reach API | `export SPANDA_CONTROL_CENTER_URL=http://host:port` |
| Desktop app blank | Start API first; `VITE_CONTROL_CENTER_URL` — [control-center.md](./control-center.md#access-the-ui) |
| gRPC client fails | Start with `--grpc-bind 127.0.0.1:50051`; check firewall |
| Rate limited | `SPANDA_API_RATE_LIMIT_PER_MINUTE` — [control-center-rate-limits.md](./control-center-rate-limits.md) |

### Integration tests

```bash
export SPANDA_SDK_INTEGRATION=1
./scripts/enterprise_ops_smoke.sh
cargo test -p spanda-api
```

---

## Fleet and mesh

Distributed fleet ops need **agents running before** remote orchestration.

| Symptom | Cause | Fix |
|---------|-------|-----|
| `fleet orchestrate --remote` no effect | Agents not registered | `spanda fleet agent start`; `spanda fleet agent register` |
| Mesh relay silent | `SPANDA_FLEET_MESH_URL` unset | Export mesh URL + token; start `spanda fleet mesh start` |
| Recovery / continuity not on edge | Program not deployed to agent | `POST /v1/program` or deploy workflow — [readiness.md](./readiness.md) |
| `connection refused` on agent URL | Wrong bind, firewall, HTTP vs HTTPS | Match registered URL; check `--token` |
| In-process sim only | Default for multi-robot examples | Expected for local demos — wire agents for field validation |
| Tamper / fleet correlation empty | No mesh ingest | `spanda tamper-check --mesh-url <url>` after shards ingested |

**Smoke scripts:** `scripts/fleet_field_validation.sh`, `scripts/fleet_agent_recovery_smoke.sh`, `scripts/failover_drill_smoke.sh`.

Field validation checklist: [test-plan.md](./test-plan.md).

---

## ROS2 integration

Golden path: **rclpy bridge** with `SPANDA_ROS2_LIVE=1`. See [ros2-golden-path.md](./ros2-golden-path.md).

| Symptom | Cause | Fix |
|---------|-------|-----|
| `/cmd_vel` silent | Live flag off or ROS not sourced | `export SPANDA_ROS2_LIVE=1`; `source /opt/ros/humble/setup.bash` |
| `spanda ros2 check` fails | Missing `ROS_DISTRO` or rclpy | `python3 -c "import rclpy"` after sourcing distro |
| Mock topics in `sim` | Expected without live env | Enable live only for hardware-in-loop runs |
| Mixed rclrs + rclpy confusion | Multiple transport modes | Pick one path per deployment — do not mix blindly |
| Bridge script not found | Run outside repo | `export SPANDA_PYTHON_BRIDGE=/path/to/scripts/spanda_python_bridge.py` |

**Validate before live run:**

```bash
source /opt/ros/humble/setup.bash
spanda ros2 check --json
spanda check examples/ros2_bridge.sd
```

ROS2 adapter targets **Humble** on Linux; Windows golden paths are narrower in CI.

---

## Live AI and extern bridges

| Symptom | Cause | Fix |
|---------|-------|-----|
| Planner always returns mock text | No API key / live flag | `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, or `SPANDA_ONNX_MODEL_PATH` |
| Forced mock despite key | `SPANDA_LIVE_AI=0` | Unset or set `SPANDA_LIVE_AI=1` |
| `extern python` no-op | Handler not registered | Add function to `scripts/spanda_python_bridge.py` |
| ONNX load error | Bad model path or runtime | Verify path; see [live-ai-provider.md](./live-ai-provider.md) |
| Rate limit / timeout | Provider API | Retry; use mock for CI |

```bash
export ANTHROPIC_API_KEY=sk-ant-...
spanda run examples/features/live_anthropic.sd
```

---

## Live IoT and transports

Live paths require **feature flags at build time** and **runtime env** — defaults are in-memory mock.

| Transport | Typical env | Build note |
|-----------|-------------|------------|
| MQTT | `SPANDA_LIVE_MQTT=1` | `--features live-iot` where documented |
| Modbus | `SPANDA_LIVE_MODBUS=1` | `live-iot` feature |
| OPC-UA | `SPANDA_LIVE_OPCUA=1` | Python asyncua bridge |
| BACnet / KNX (Smart Spaces) | `SPANDA_LIVE_IOT_*`, package scripts | `live-building` feature on CLI |
| WebSocket / DDS | respective `SPANDA_LIVE_*` | DDS is UDP JSON shim, not full middleware |

| Symptom | Fix |
|---------|-----|
| Reads return fixture data | Enable live env; confirm Python bridge reachable |
| Smart Spaces panels empty | `spanda control-center serve --config spanda.toml`; check device pool in config |
| Telemetry not persisted | `SPANDA_TELEMETRY_STORE=1` or `--persist-telemetry` |

See [iot.md](./iot.md) · [smart-space-bms-bridge.md](./smart-space-bms-bridge.md).

---

## Plugins

| Symptom | Cause | Fix |
|---------|-------|-----|
| `Unknown plugin subcommand` | Typo or plugin disabled | `spanda plugin list`; `spanda plugin enable <name>` |
| Install rejected (security) | Trust tier / signature | `spanda plugin trust <name> <tier>`; `--approve-dangerous` for dev only |
| Namespaced command no output | No plugin declares namespace | `spanda plugin inspect <name>` — check `[cli.commands]` |
| WASM load failure | Missing artifact or wrong arch | Rebuild plugin; use manifest-only for dev |
| Control Center panel missing | Plugin type not `control-center-ui` | Enable plugin; `GET /v1/plugins/control-center` |

```bash
spanda plugin search <query>
spanda plugin install <name> --approve-dangerous   # dev only
spanda plugin inspect <name> --json
```

See [plugins.md](./plugins.md).

---

## Configuration and drift

| Symptom | Cause | Fix |
|---------|-------|-----|
| Config merge surprises | Layer order | base → environment → deployment → robot — [configuration.md](./configuration.md) |
| `device.ip_unreachable` warning | Network probe on | `SPANDA_CONFIG_PROBE_NETWORK=1` pings declared IPs |
| Drift scan empty baseline | No snapshot saved | `spanda control-center snapshots save --label baseline` |
| Agent drift always differs | Program not uploaded to agent | Deploy or `POST /v1/program` before compare |

```bash
spanda config validate --config spanda.toml
spanda drift scan file.sd --json
```

---

## Security, trust, and tamper

| Symptom | Cause | Fix |
|---------|-------|-----|
| `tamper-check` fails TPM path | Backend not configured | Set `SPANDA_TPM_BACKEND` (file/script/tpm2/vendor) — [hardware-attestation.md](./hardware-attestation.md) |
| Trust score low | Unsigned packages or policy gaps | `spanda trust <package>`; review `spanda security audit` |
| Capability denial in trace | Runtime policy block | `spanda diagnose tamper mission.trace`; check capability matrix |
| Signed offline policy blocked | `SPANDA_DECISION_REQUIRE_SIGNED_OFFLINE_POLICY=1` | `spanda decision sign-policy`; set trust key |

Smoke: `scripts/tamper_smoke.sh`, `scripts/security_assurance_smoke.sh`.

---

## Editor, LSP, and debugging

| Issue | Fix |
|-------|-----|
| No diagnostics in VS Code | Build CLI; set `spanda.cliPath` to `target/release/spanda` |
| Verify warnings missing | Run **Spanda: Verify Deploy Target** or enable verify in LSP settings |
| `spanda-dap not found` | `cargo build -p spanda-dap -p spanda-cli --release` |
| Breakpoints never hit | Set breakpoint **inside** task/`every` body; use Step Over |
| LSP works, debug fails | Debug uses `spanda-dap`, separate from LSP |

See [editor/vscode/README.md](../editor/vscode/README.md) · [debugging.md](./debugging.md).

---

## CI and release builds

| Symptom | Cause | Fix |
|---------|-------|-----|
| CI uses old Spanda | Cached installer or wrong PATH | `cargo build -p spanda-cli --release` + `GITHUB_PATH` |
| `verify --json` parse errors | Script expects wrong schema | Fail on `compatible: false` — [ci-verify.md](./ci-verify.md) |
| Smoke script fails mid-run | Port conflict or missing `SPANDA_BIN` | `SPANDA_BIN=target/release/spanda ./scripts/<smoke>.sh` |
| Cross-compile package fails | Missing zig/xwin | `./scripts/package-release.sh --install-cross` |
| Feature-gated test skipped | Default build without features | `cargo test -p spanda-providers --features live-iot` |

**Contributor pre-flight:**

```bash
npm install && npm run build:rust
cargo test --workspace
npm test
```

Gate index: [scripts/gates/README.md](../scripts/gates/README.md).

---

## Decision runtime and offline policy

| Symptom | Cause | Fix |
|---------|-------|-----|
| Tree action blocked offline | Policy signature required | Sign policy; cache at `.spanda/decision-policy-cache.json` |
| Central approval pending forever | `requires_central_approval` | Approve via Control Center or set operator approval env |
| No decision traces | Trace env off | `SPANDA_DECISION_TRACE=1` or `--record` on run/sim |
| `GET /v1/decisions/traces` empty | Server not recording | Run program with trace emission; check API URL |

See [offline-decisions.md](./offline-decisions.md) · [distributed-decisions.md](./distributed-decisions.md).

---

## Environment variable quick reference

High-impact variables that commonly cause "it works on my machine" failures:

| Variable | When unset (typical) | When set |
|----------|----------------------|----------|
| `SPANDA_API_KEY` | Control Center mutations rejected | Bearer auth for ops API |
| `SPANDA_CONTROL_CENTER_URL` | Remote CLI defaults to `http://127.0.0.1:8080` | Points CLI at remote server |
| `SPANDA_ROS2_LIVE` | Mock ROS2 transport | Live rclpy bridge |
| `SPANDA_LIVE_AI` | Mock AI (unless keys alone enable) | Force live provider path |
| `SPANDA_FLEET_MESH_URL` | No mesh relay | Fleet recovery / continuity relay |
| `SPANDA_REGISTRY_REQUIRE_SIGNATURE` | Lenient registry install | Production checksum enforcement |
| `SPANDA_REGISTRY_URL` | Bundled registry slice | Remote package index |
| `SPANDA_TELEMETRY_STORE` | Ephemeral telemetry | Persist to `.spanda/` store |
| `SPANDA_DECISION_TRACE` | No decision trace emission | v3 traces in run/sim |

Full config cascade: [configuration.md](./configuration.md) · [spanda-toml.md](./spanda-toml.md).

---

## Smoke scripts (by area)

Use these to reproduce CI failures locally:

| Area | Script |
|------|--------|
| Enterprise ops / Control Center | `scripts/enterprise_ops_smoke.sh` |
| Fleet field validation | `scripts/fleet_field_validation.sh` |
| Self-healing / recovery | `scripts/self_healing_smoke.sh` |
| Distributed decisions | `scripts/distributed_decisions_smoke.sh` |
| ROS2 / adoption | `scripts/showcase_smoke.sh` |
| Smart Spaces | `scripts/smart_spaces_smoke.sh` |
| OTA fleet execute | `scripts/ota_fleet_execute_smoke.sh` |
| Maturity gate | `scripts/maturity_smoke.sh` |

Pass custom binary: `SPANDA_BIN=target/release/spanda ./scripts/<script>.sh`

---

## Still stuck?

1. **Reproduce minimally** — smallest `.sd` file and one command.
2. **Capture context** — `which spanda`, `spanda --version`, OS, env vars (redact secrets).
3. **Structured output** — prefer `--json` flags for bug reports.
4. **Compare with examples** — `examples/basics/`, `examples/showcase/`.
5. **Run targeted tests** — `cargo test -p <crate> <test_name>`.
6. **File an issue** — [GitHub Issues](https://github.com/Davalgi/Spanda/issues) with reproducer and logs.

If behavior contradicts [known-limitations.md](./known-limitations.md), that's a bug — please report it.
