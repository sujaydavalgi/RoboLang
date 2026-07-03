import { describe, expect, it } from "vitest";
import { SpandaClient } from "../src/client.js";
import { SpandaError } from "../src/errors.js";
import { ReadinessReport } from "../src/types.js";

describe("SpandaClient", () => {
  it("constructs local client", () => {
    const client = SpandaClient.local();
    expect(client.baseUrl).toContain("127.0.0.1");
  });

  it("maps permission errors", () => {
    const err = SpandaError.fromStatus(403, "forbidden");
    expect(err.name).toBe("PermissionError");
  });

  it("entityReadiness uses readiness path", async () => {
    const client = SpandaClient.local();
    let captured = "";
    (client as unknown as { request: typeof client["request"] }).request = async (
      method,
      path,
    ) => {
      captured = `${method} ${path}`;
      return {};
    };
    await client.entityReadiness("rover-001");
    expect(captured).toBe("GET /v1/entities/rover-001/readiness");
  });

  it("entityRelationships uses relationships path", async () => {
    const client = SpandaClient.local();
    let captured = "";
    (client as unknown as { request: typeof client["request"] }).request = async (
      method,
      path,
    ) => {
      captured = `${method} ${path}`;
      return {};
    };
    await client.entityRelationships("rover-001");
    expect(captured).toBe("GET /v1/entities/rover-001/relationships");
  });

  it("listDecisionTraces uses traces path", async () => {
    const client = SpandaClient.local();
    let captured = "";
    (client as unknown as { request: typeof client["request"] }).request = async (
      method,
      path,
    ) => {
      captured = `${method} ${path}`;
      return {};
    };
    await client.listDecisionTraces({ file: "mission.sd" });
    expect(captured).toBe("GET /v1/decisions/traces?file=mission.sd");
  });

  it("listDecisionPolicyCache uses policy-cache path", async () => {
    const client = SpandaClient.local();
    let captured = "";
    (client as unknown as { request: typeof client["request"] }).request = async (
      method,
      path,
    ) => {
      captured = `${method} ${path}`;
      return {};
    };
    await client.listDecisionPolicyCache({ cache: "/tmp/cache.json" });
    expect(captured).toBe(
      "GET /v1/decision-policy-cache?cache=%2Ftmp%2Fcache.json",
    );
  });

  it("analyticsWhatIf builds query path", async () => {
    const client = SpandaClient.local();
    let captured = "";
    (client as unknown as { request: typeof client["request"] }).request = async (
      _method,
      path,
    ) => {
      captured = path;
      return {};
    };
    await client.analyticsWhatIf({ scenario: "gps_failure", all: true });
    expect(captured).toBe("/v1/analytics/what-if?all=1&scenario=gps_failure");
  });

  it("analyticsTimeTravel encodes timestamp query", async () => {
    const client = SpandaClient.local();
    let captured = "";
    (client as unknown as { request: typeof client["request"] }).request = async (
      _method,
      path,
    ) => {
      captured = path;
      return {};
    };
    await client.analyticsTimeTravel({ at: "T+00:01", inspect: "decisions" });
    expect(captured).toBe(
      "/v1/analytics/time-travel?at=T%2B00%3A01&inspect=decisions",
    );
  });

  it("syncTwin sends auth", async () => {
    const client = SpandaClient.local();
    let auth = false;
    (client as unknown as { request: typeof client["request"] }).request = async (
      _method,
      path,
      _body,
      useAuth,
    ) => {
      auth = useAuth ?? false;
      expect(path).toBe("/v1/twins/sync?twin_id=patrol");
      return {};
    };
    await client.syncTwin("patrol");
    expect(auth).toBe(true);
  });

  it("getTwinHistory uses history path", async () => {
    const client = SpandaClient.local();
    let captured = "";
    (client as unknown as { request: typeof client["request"] }).request = async (
      _method,
      path,
    ) => {
      captured = path;
      return {};
    };
    await client.getTwinHistory("patrol");
    expect(captured).toBe("/v1/twins/patrol/history");
  });
});

describe("ReadinessReport", () => {
  it("extracts score from API envelope", () => {
    const report = ReadinessReport.fromApi({
      report: { score: { total: 88 }, status: "Ready" },
    });
    expect(report.score).toBe(88);
  });
});
