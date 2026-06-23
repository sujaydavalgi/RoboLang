# Verification diagnostics

Structured verification output for capability traceability, minimum-hardware safety, health checks, and kill switch policies — surfaced in the CLI, LSP, and CI.

**Example:** [`examples/hardware/capability_verification.sd`](../examples/hardware/capability_verification.sd)

---

## CLI: `spanda check --verification-json`

Emits span-aware diagnostics alongside type-check results:

```bash
spanda check examples/hardware/capability_verification.sd --verification-json
spanda check rover.sd --verification-json > verification.json
```

Categories include:

| Category | Checks |
|----------|--------|
| `capability` | `requires_capability`, robot `exposes capabilities` |
| `traceability` | Hardware ↔ robot capability matrix |
| `minimum-hardware` | Minimum sensors/actuators for safety |
| `health` | `health_check` / `health_policy` analysis |
| `kill-switch` | Kill switch wiring, `remote_signed` policy |

### Severity and suggested fixes

Since Phase 30, diagnostics may include `suggested_fix` hints consumed by the LSP as quick-fix code actions. Phase 35 upgraded `remote_signed` without signed communication to **error** severity at verify time.

```bash
spanda verify examples/security/remote_signed_kill_switch.sd --health
```

---

## Verify flags

Combine with hardware verification:

```bash
spanda verify rover.sd --health --capabilities --traceability --minimum-capabilities
spanda trace health rover.sd
spanda trace capabilities rover.sd
spanda safety check rover.sd --capabilities
```

See [capability-traceability.md](./capability-traceability.md), [health-checks.md](./health-checks.md), [kill-switch.md](./kill-switch.md).

---

## LSP integration

The VS Code extension and `@spanda/lsp` cache verification diagnostics from the native CLI. Quick-fix code actions map `suggested_fix` strings to editor actions when available.

Requires a built native CLI (`target/release/spanda`). See [debugging.md](./debugging.md) for the extension workflow.

---

## CI wiring

Use JSON output in pipelines:

```bash
spanda check src/main.sd --verification-json | jq '.diagnostics[] | select(.severity == "error")'
```

Hardware verify in CI: [ci-verify.md](./ci-verify.md).

Golden path: `./scripts/ci_verify_golden_path.sh`

---

## Related

- [testing.md](./testing.md) — `expect_compile_error` and `spanda test`
- [typed-handler-io.md](./typed-handler-io.md) — handler return type validation
- [feature-status.md](./feature-status.md) — stable vs experimental matrix
