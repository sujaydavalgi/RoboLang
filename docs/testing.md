# Testing in Spanda

In-language tests, compile-fail assertions, and CLI test runners (Phases 27–28).

**Examples:**

- [`examples/basics/07_in_language_tests.sd`](../examples/basics/07_in_language_tests.sd) — basic `test` blocks
- [`examples/basics/12_compile_fail_tests.sd`](../examples/basics/12_compile_fail_tests.sd) — `expect_compile_error`

---

## `test` blocks

```spanda
test "clamp accepts in-range speed" {
    assert(clamp_speed(0.5) <= 1.0);
}
```

Run from a project or single file:

```bash
spanda test
spanda test examples/basics/07_in_language_tests.sd
```

---

## `expect_compile_error`

Validate that inner statements fail type-checking:

```spanda
test "rejects bad assignment" {
    expect_compile_error {
        let x: Int = "not an int";
    }
    assert(true);
}
```

The test passes when the nested block produces a compile error. If the block type-checks, the test fails.

**Example:** [`examples/basics/12_compile_fail_tests.sd`](../examples/basics/12_compile_fail_tests.sd)

---

## CLI flags

| Flag | Purpose |
|------|---------|
| `--json` | Machine-readable test report |
| `--filter <pattern>` | Run tests matching name substring |
| `--compile-fail` | Expect compile failure for negative tests |

```bash
spanda test tests/ --json
spanda test rover.sd --filter "health"
```

---

## Project layout

```
my_robot/
├── spanda.toml
├── src/main.sd
└── tests/
    └── smoke.sd
```

See [packages.md](./packages.md) and [Spanda 101 Lesson 9](./spanda-101/09-packages-and-tests.md).

---

## CI

Combine with verification and verify in pipelines:

```bash
spanda test
spanda check src/main.sd --verification-json
spanda verify src/main.sd --json --target RoverV1
```

See [ci-verify.md](./ci-verify.md) and [verification-diagnostics.md](./verification-diagnostics.md).

---

## Related

- [test-plan.md](./test-plan.md) — coverage plan
- [feature-status.md](./feature-status.md) — test block stability tier
