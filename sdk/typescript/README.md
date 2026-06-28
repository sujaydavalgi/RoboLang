# @davalgi-spanda/sdk

Official TypeScript SDK for Spanda Control Center API v1.

## Install

```bash
npm install @davalgi-spanda/sdk
```

From this monorepo:

```bash
npm ci --prefix sdk/typescript && npm run build --prefix sdk/typescript
```

## Quick start

```typescript
import { SpandaClient } from "@davalgi-spanda/sdk";

const client = SpandaClient.local();
const report = await client.readiness("rover.sd");
console.log(report.score);
```

## Documentation

- [docs/sdk-typescript.md](../../docs/sdk-typescript.md)
- [docs/sdk-publishing.md](../../docs/sdk-publishing.md)
- [docs/control-center-api.md](../../docs/control-center-api.md)

## Publish

See [docs/sdk-publishing.md](../../docs/sdk-publishing.md) for `NPM_TOKEN`, release tags (`npm-sdk-v*`), and token rotation.
