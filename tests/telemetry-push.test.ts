import { afterEach, beforeEach, describe, expect, it } from "vitest";
import http from "node:http";
import {
  envAutoPushEnabled,
  envPushIntervalMs,
  pushOtlpJson,
} from "../src/telemetry-push.js";
import { renderOtlpJson } from "../src/telemetry-otlp.js";

const ENV_KEYS = [
  "SPANDA_OTLP_AUTO_PUSH",
  "SPANDA_OTLP_PUSH",
  "SPANDA_OTLP_PUSH_INTERVAL_MS",
] as const;

describe("telemetry push env", () => {
  let savedEnv: Partial<Record<(typeof ENV_KEYS)[number], string | undefined>>;

  beforeEach(() => {
    savedEnv = {};
    for (const key of ENV_KEYS) {
      savedEnv[key] = process.env[key];
      delete process.env[key];
    }
  });

  afterEach(() => {
    for (const key of ENV_KEYS) {
      if (savedEnv[key] === undefined) {
        delete process.env[key];
      } else {
        process.env[key] = savedEnv[key];
      }
    }
  });

  it("reads auto-push flags", () => {
    expect(envAutoPushEnabled()).toBe(false);
    process.env.SPANDA_OTLP_AUTO_PUSH = "1";
    expect(envAutoPushEnabled()).toBe(true);
    delete process.env.SPANDA_OTLP_AUTO_PUSH;
    process.env.SPANDA_OTLP_PUSH = "true";
    expect(envAutoPushEnabled()).toBe(true);
  });

  it("defaults push interval to 30s", () => {
    expect(envPushIntervalMs()).toBe(30_000);
    process.env.SPANDA_OTLP_PUSH_INTERVAL_MS = "5000";
    expect(envPushIntervalMs()).toBe(5_000);
  });

  it("posts OTLP JSON to a mock collector", async () => {
    let received = "";
    const server = http.createServer((req, res) => {
      const chunks: Buffer[] = [];
      req.on("data", (chunk) => chunks.push(chunk));
      req.on("end", () => {
        received = Buffer.concat(chunks).toString("utf8");
        res.writeHead(204);
        res.end();
      });
    });
    await new Promise<void>((resolve) => server.listen(0, resolve));
    const address = server.address();
    if (!address || typeof address === "string") {
      throw new Error("expected bound TCP port");
    }
    try {
      const body = renderOtlpJson();
      await pushOtlpJson(`http://127.0.0.1:${address.port}/v1/metrics`, body);
      expect(received).toContain("resourceMetrics");
    } finally {
      await new Promise<void>((resolve, reject) => {
        server.close((error) => (error ? reject(error) : resolve()));
      });
    }
  });
});
