# TypeScript SDK (`@davalgi-spanda/sdk`)

Official TypeScript/JavaScript client for Control Center UI, web dashboards, VS Code extensions, and cloud integrations.

## Install

From npm (released package):

```bash
npm install @davalgi-spanda/sdk
```

From this monorepo (development):

```bash
cd sdk/typescript && npm install && npm run build
```

In a monorepo workspace:

```json
{ "dependencies": { "@davalgi-spanda/sdk": "file:../../sdk/typescript" } }
```

## Usage

```typescript
import { SpandaClient } from "@davalgi-spanda/sdk";

const client = SpandaClient.local();
const report = await client.readiness("rover.sd");
console.log(report.score);
```

## Authentication

```typescript
const client = new SpandaClient({
  baseUrl: process.env.SPANDA_CONTROL_CENTER_URL,
  apiKey: process.env.SPANDA_API_KEY,
});
```

## Event stream

```typescript
import { EventStream } from "@davalgi-spanda/sdk";

const stream = EventStream.local();
// Connect with ws package or browser WebSocket to stream.wsUrl
console.log(stream.wsUrl);
```

## Control Center integration pattern

```typescript
const client = SpandaClient.local();
const [health, entities] = await Promise.all([
  client.healthCheck(),
  client.listEntities(),
]);
```

## Examples

```bash
npx tsx examples/sdk/typescript/readiness.ts
npx tsx examples/sdk/typescript/control_center.ts
```

## Tests

```bash
cd sdk/typescript && npm test
```

## Publishing

See [Publishing SDKs](sdk-publishing.md) for npm org `@davalgi-spanda`, `NPM_TOKEN`, release tags (`npm-sdk-v*`), and 90-day token rotation.
