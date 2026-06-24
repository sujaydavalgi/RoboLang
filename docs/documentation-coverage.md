# Documentation Coverage Report

Generated: 2026-06-24

This report is produced by `scripts/validate_documentation.py`. See [coding-standards.md](./coding-standards.md) for the required docstring format.

## Summary

| Metric | Count |
|--------|------:|
| Total methods / functions audited | 4223 |
| Fully documented (structured standard) | 4223 |
| Undocumented or incomplete | 0 |
| Coverage | 100.0% |

## Coverage by module

| Module | Total | Documented | Coverage |
|--------|------:|-----------:|---------:|
| `python/scripts` | 54 | 54 | 100.0% |
| `rust/spanda-ai` | 74 | 74 | 100.0% |
| `rust/spanda-assurance` | 107 | 107 | 100.0% |
| `rust/spanda-ast` | 23 | 23 | 100.0% |
| `rust/spanda-audit` | 54 | 54 | 100.0% |
| `rust/spanda-bridge` | 31 | 31 | 100.0% |
| `rust/spanda-capability` | 63 | 63 | 100.0% |
| `rust/spanda-certify` | 19 | 19 | 100.0% |
| `rust/spanda-cli` | 170 | 170 | 100.0% |
| `rust/spanda-codegen` | 9 | 9 | 100.0% |
| `rust/spanda-comm` | 30 | 30 | 100.0% |
| `rust/spanda-concurrency` | 17 | 17 | 100.0% |
| `rust/spanda-connectivity` | 22 | 22 | 100.0% |
| `rust/spanda-connectivity-runtime` | 17 | 17 | 100.0% |
| `rust/spanda-core` | 314 | 314 | 100.0% |
| `rust/spanda-dap` | 8 | 8 | 100.0% |
| `rust/spanda-debug` | 7 | 7 | 100.0% |
| `rust/spanda-deploy-http` | 20 | 20 | 100.0% |
| `rust/spanda-docs` | 54 | 54 | 100.0% |
| `rust/spanda-driver` | 58 | 58 | 100.0% |
| `rust/spanda-error` | 4 | 4 | 100.0% |
| `rust/spanda-ffi` | 14 | 14 | 100.0% |
| `rust/spanda-fleet` | 95 | 95 | 100.0% |
| `rust/spanda-format` | 59 | 59 | 100.0% |
| `rust/spanda-hal` | 40 | 40 | 100.0% |
| `rust/spanda-hardware` | 41 | 41 | 100.0% |
| `rust/spanda-interpreter` | 233 | 233 | 100.0% |
| `rust/spanda-lexer` | 22 | 22 | 100.0% |
| `rust/spanda-lib-registry` | 44 | 44 | 100.0% |
| `rust/spanda-lint` | 13 | 13 | 100.0% |
| `rust/spanda-llvm` | 42 | 42 | 100.0% |
| `rust/spanda-modules` | 6 | 6 | 100.0% |
| `rust/spanda-node` | 8 | 8 | 100.0% |
| `rust/spanda-ota` | 61 | 61 | 100.0% |
| `rust/spanda-package` | 207 | 207 | 100.0% |
| `rust/spanda-parser` | 188 | 188 | 100.0% |
| `rust/spanda-providers` | 157 | 157 | 100.0% |
| `rust/spanda-readiness` | 84 | 84 | 100.0% |
| `rust/spanda-regex-lang` | 6 | 6 | 100.0% |
| `rust/spanda-ros2-rclrs-native` | 9 | 9 | 100.0% |
| `rust/spanda-rt` | 23 | 23 | 100.0% |
| `rust/spanda-runtime` | 218 | 218 | 100.0% |
| `rust/spanda-runtime-host` | 39 | 39 | 100.0% |
| `rust/spanda-safety` | 22 | 22 | 100.0% |
| `rust/spanda-security` | 133 | 133 | 100.0% |
| `rust/spanda-sir` | 42 | 42 | 100.0% |
| `rust/spanda-transport` | 47 | 47 | 100.0% |
| `rust/spanda-transport-dds` | 18 | 18 | 100.0% |
| `rust/spanda-transport-mqtt` | 24 | 24 | 100.0% |
| `rust/spanda-transport-ros2` | 68 | 68 | 100.0% |
| `rust/spanda-transport-routing` | 61 | 61 | 100.0% |
| `rust/spanda-transport-websocket` | 19 | 19 | 100.0% |
| `rust/spanda-typecheck` | 139 | 139 | 100.0% |
| `rust/spanda-wasm` | 7 | 7 | 100.0% |
| `spanda/examples` | 2 | 2 | 100.0% |
| `ts/ai` | 23 | 23 | 100.0% |
| `ts/cli` | 41 | 41 | 100.0% |
| `ts/comm` | 7 | 7 | 100.0% |
| `ts/ffi` | 11 | 11 | 100.0% |
| `ts/hal` | 2 | 2 | 100.0% |
| `ts/lexer` | 8 | 8 | 100.0% |
| `ts/lib` | 9 | 9 | 100.0% |
| `ts/lsp` | 28 | 28 | 100.0% |
| `ts/modules` | 4 | 4 | 100.0% |
| `ts/native` | 9 | 9 | 100.0% |
| `ts/navigation` | 1 | 1 | 100.0% |
| `ts/network` | 2 | 2 | 100.0% |
| `ts/parser` | 163 | 163 | 100.0% |
| `ts/providers` | 11 | 11 | 100.0% |
| `ts/root` | 341 | 341 | 100.0% |
| `ts/ros2` | 3 | 3 | 100.0% |
| `ts/runtime` | 77 | 77 | 100.0% |
| `ts/safety` | 4 | 4 | 100.0% |
| `ts/security` | 30 | 30 | 100.0% |
| `ts/simulator` | 2 | 2 | 100.0% |
| `ts/soc` | 3 | 3 | 100.0% |
| `ts/transport` | 32 | 32 | 100.0% |
| `ts/types` | 50 | 50 | 100.0% |
| `ts/units` | 8 | 8 | 100.0% |
| `ts/web` | 8 | 8 | 100.0% |

## Coverage by language

| Language | Total | Documented | Coverage |
|----------|------:|-----------:|---------:|
| python | 54 | 54 | 100.0% |
| rust | 3290 | 3290 | 100.0% |
| spanda | 2 | 2 | 100.0% |
| typescript | 877 | 877 | 100.0% |

## Remaining gaps (public APIs, sample)

Public APIs missing one or more required sections. Run `python3 scripts/validate_documentation.py --warn` for the full list.


## CI enforcement

CI runs `python3 scripts/validate_documentation.py --warn --report` on every pull request. Warnings are emitted for public APIs that lack structured documentation; builds do not fail yet.

## Regenerating

```bash
python3 scripts/validate_documentation.py --report
python3 scripts/migrate_legacy_inline_docs.py
python3 scripts/add_structured_api_docs.py
python3 scripts/normalize_inline_docs.py
```
