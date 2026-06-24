import { describe, expect, it } from "vitest";
import http from "node:http";
import {
  envFleetAutoIngestEnabled,
  envRobotId,
  ingestFleetTelemetry,
  mergeFleetOtlpJson,
} from "../src/telemetry-fleet.js";

describe("fleet telemetry aggregation", () => {
  it("merges robot OTLP shards with robot id attributes", () => {
    const shard = JSON.stringify({
      resourceMetrics: [{
        resource: { attributes: [] },
        scopeMetrics: [],
      }],
    });
    const merged = mergeFleetOtlpJson([
      { robotId: "rover-a", otlpJson: shard },
      { robotId: "rover-b", otlpJson: shard },
    ]);
    expect(merged).toContain("rover-a");
    expect(merged).toContain("rover-b");
    expect(merged).toContain("spanda.robot.id");
  });

  it("reads fleet auto-ingest env flags", () => {
    const saved = process.env.SPANDA_FLEET_TELEMETRY_AUTO_INGEST;
    delete process.env.SPANDA_FLEET_TELEMETRY_AUTO_INGEST;
    expect(envFleetAutoIngestEnabled()).toBe(false);
    process.env.SPANDA_FLEET_TELEMETRY_AUTO_INGEST = "1";
    expect(envFleetAutoIngestEnabled()).toBe(true);
    if (saved === undefined) {
      delete process.env.SPANDA_FLEET_TELEMETRY_AUTO_INGEST;
    } else {
      process.env.SPANDA_FLEET_TELEMETRY_AUTO_INGEST = saved;
    }
  });

  it("prefers SPANDA_ROBOT_ID for fleet ingest identity", () => {
    const saved = process.env.SPANDA_ROBOT_ID;
    process.env.SPANDA_ROBOT_ID = "rover-alpha";
    expect(envRobotId()).toBe("rover-alpha");
    if (saved === undefined) {
      delete process.env.SPANDA_ROBOT_ID;
    } else {
      process.env.SPANDA_ROBOT_ID = saved;
    }
  });

  it("posts OTLP shards to a mock fleet mesh ingest endpoint", async () => {
    let received = "";
    const server = http.createServer((req, res) => {
      const chunks: Buffer[] = [];
      req.on("data", (chunk) => chunks.push(chunk));
      req.on("end", () => {
        received = Buffer.concat(chunks).toString("utf8");
        res.writeHead(200, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ ok: true, robots: 1 }));
      });
    });
    await new Promise<void>((resolve) => server.listen(0, resolve));
    const address = server.address();
    if (!address || typeof address === "string") {
      throw new Error("expected bound TCP port");
    }
    try {
      const shard = JSON.stringify({
        resourceMetrics: [{ resource: { attributes: [] }, scopeMetrics: [] }],
      });
      await ingestFleetTelemetry(
        `http://127.0.0.1:${address.port}`,
        { robotId: "rover-a", otlpJson: shard },
      );
      expect(received).toContain("rover-a");
      expect(received).toContain("resourceMetrics");
    } finally {
      await new Promise<void>((resolve, reject) => {
        server.close((error) => (error ? reject(error) : resolve()));
      });
    }
  });
});
