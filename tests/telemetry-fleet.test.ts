import { describe, expect, it } from "vitest";
import { mergeFleetOtlpJson } from "../src/telemetry-fleet.js";

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
});
