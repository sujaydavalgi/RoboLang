# Web playground

[← Overview](./README.md)

Browser-based check/run via WASM (optional; native CLI is authoritative for full features).

## Prerequisites

- Node.js 18+
- Rust toolchain (for WASM build)

## Run locally

From the repository root:

```bash
npm install
npm run build:wasm
npm run web:dev       # http://localhost:5173
```

The playground lives under `packages/web/` (`@davalgi-spanda/web`).

## What works in the browser

- Type-check and run subset of programs
- Lighter surface than the native `spanda` CLI (no full fleet, deploy agents, or all provider bridges)

For production workflows use the native CLI: [installation.md](../installation.md) · [getting-started.md](../getting-started.md)
