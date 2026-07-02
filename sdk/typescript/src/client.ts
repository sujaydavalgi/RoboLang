/** Official Spanda TypeScript SDK — thin REST client over Control Center API v1. */

import { ConnectionError, PermissionError, SpandaError } from "./errors.js";
import { Entity, JsonValue, ReadinessReport } from "./types.js";

export interface SpandaClientOptions {
  baseUrl?: string;
  apiKey?: string;
  timeoutMs?: number;
}

export class SpandaClient {
  readonly baseUrl: string;
  readonly apiKey?: string;
  readonly timeoutMs: number;

  constructor(options: SpandaClientOptions = {}) {
    this.baseUrl = (
      options.baseUrl ??
      process.env.SPANDA_CONTROL_CENTER_URL ??
      "http://127.0.0.1:8080"
    ).replace(/\/$/, "");
    this.apiKey = options.apiKey ?? process.env.SPANDA_API_KEY;
    this.timeoutMs = options.timeoutMs ?? 30_000;
  }

  static local(): SpandaClient {
    return new SpandaClient();
  }

  private correlationId(): string {
    return `ts-sdk-${Math.random().toString(16).slice(2, 14)}`;
  }

  private async request(
    method: string,
    path: string,
    body?: JsonValue,
    auth = false,
  ): Promise<JsonValue> {
    const headers: Record<string, string> = {
      Accept: "application/json",
      "X-Correlation-ID": this.correlationId(),
    };
    if (body) {
      headers["Content-Type"] = "application/json";
    }
    if (auth && this.apiKey) {
      headers.Authorization = `Bearer ${this.apiKey}`;
    }
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.timeoutMs);
    try {
      const resp = await fetch(`${this.baseUrl}${path}`, {
        method,
        headers,
        body: body ? JSON.stringify(body) : undefined,
        signal: controller.signal,
      });
      const text = await resp.text();
      const payload = text ? (JSON.parse(text) as JsonValue) : {};
      if (!resp.ok) {
        const message =
          typeof payload.error === "string" ? payload.error : `HTTP ${resp.status}`;
        throw SpandaError.fromStatus(resp.status, message);
      }
      return payload;
    } catch (err) {
      if (err instanceof SpandaError || err instanceof PermissionError) {
        throw err;
      }
      throw new ConnectionError(String(err));
    } finally {
      clearTimeout(timer);
    }
  }

  private programBody(file: string): JsonValue {
    return { file };
  }

  async readiness(fileOrProject: string): Promise<ReadinessReport> {
    const raw = await this.request(
      "POST",
      "/v1/programs/readiness",
      this.programBody(fileOrProject),
    );
    return ReadinessReport.fromApi(raw);
  }

  async assure(fileOrProject: string): Promise<JsonValue> {
    return this.request("POST", "/v1/programs/assure", this.programBody(fileOrProject));
  }

  async diagnose(traceOrFile: string): Promise<JsonValue> {
    return this.request("POST", "/v1/programs/diagnose", this.programBody(traceOrFile));
  }

  async heal(target: string): Promise<JsonValue> {
    return this.request("POST", "/v1/programs/recovery/heal", this.programBody(target));
  }

  async planRecovery(body: Record<string, unknown> = {}): Promise<JsonValue> {
    return this.request("POST", "/v1/recovery/plan", body);
  }

  async simulateRecovery(body: Record<string, unknown> = {}): Promise<JsonValue> {
    return this.request("POST", "/v1/recovery/simulate", body);
  }

  async executeRecovery(body: Record<string, unknown> = {}): Promise<JsonValue> {
    return this.request("POST", "/v1/recovery/execute", body);
  }

  async validateRecovery(body: Record<string, unknown> = {}): Promise<JsonValue> {
    return this.request("POST", "/v1/recovery/validate", body);
  }

  async listRecoveryPolicies(): Promise<JsonValue> {
    return this.request("GET", "/v1/recovery/policies");
  }

  async listRecoveryPlaybooks(): Promise<JsonValue> {
    return this.request("GET", "/v1/recovery/playbooks");
  }

  async getRecoveryHistory(): Promise<JsonValue> {
    return this.request("GET", "/v1/recovery/history");
  }

  async getRecoveryMetrics(): Promise<JsonValue> {
    return this.request("GET", "/v1/recovery/metrics");
  }

  async verifyHardware(project: string): Promise<JsonValue> {
    return this.request(
      "POST",
      "/v1/programs/verify/hardware",
      this.programBody(project),
    );
  }

  async verifyCapabilities(project: string): Promise<JsonValue> {
    return this.request("POST", "/v1/programs/verify/capabilities", {
      file: project,
      traceability: true,
    });
  }

  async listEntities(): Promise<Entity[]> {
    const raw = await this.request("GET", "/v1/entities");
    const list = Array.isArray(raw.entities) ? raw.entities : [];
    return list.map((item) => {
      const row = item as Record<string, unknown>;
      return {
        id: String(row.id ?? ""),
        kind: typeof row.kind === "string" ? row.kind : undefined,
        displayName:
          typeof row.display_name === "string" ? row.display_name : undefined,
        raw: row,
      };
    });
  }

  async getEntity(id: string): Promise<Entity> {
    const raw = await this.request("GET", `/v1/entities/${id}`);
    const row = (raw.entity as Record<string, unknown> | undefined) ?? raw;
    return {
      id: String(row.id ?? id),
      kind: typeof row.kind === "string" ? row.kind : undefined,
      displayName:
        typeof row.display_name === "string" ? row.display_name : undefined,
      raw: row,
    };
  }

  async listDevices(): Promise<JsonValue> {
    return this.request("GET", "/v1/devices", undefined, true);
  }

  async provisionDevice(deviceId: string, body: JsonValue = {}): Promise<JsonValue> {
    return this.request("POST", `/v1/devices/${deviceId}/provision`, body, true);
  }

  async runSimulation(project: string, execute = false): Promise<JsonValue> {
    return this.request("POST", "/v1/programs/simulation", {
      file: project,
      execute,
    });
  }

  async replay(
    trace: string,
    options: { deterministic?: boolean; playback?: boolean } = {},
  ): Promise<JsonValue> {
    return this.request("POST", "/v1/programs/replay", {
      file: trace,
      deterministic: options.deterministic ?? false,
      playback: options.playback ?? false,
    });
  }

  async getHealth(entityId: string): Promise<JsonValue> {
    return this.request("GET", `/v1/entities/${entityId}/health`);
  }

  async entityReadiness(entityId: string): Promise<JsonValue> {
    return this.request("GET", `/v1/entities/${entityId}/readiness`);
  }

  async entityRelationships(entityId: string): Promise<JsonValue> {
    return this.request("GET", `/v1/entities/${entityId}/relationships`);
  }

  async getTrust(entityId: string): Promise<JsonValue> {
    return this.request("GET", `/v1/entities/${entityId}/trust`);
  }

  async verifyEntity(
    entityId: string,
    body: { includeDependencies?: boolean; file?: string } = {},
  ): Promise<JsonValue> {
    return this.request("POST", `/v1/entities/${entityId}/verify`, {
      include_dependencies: body.includeDependencies ?? false,
      file: body.file,
    });
  }

  async entityGraph(): Promise<JsonValue> {
    return this.request("GET", "/v1/entities/graph");
  }

  async entityTraceability(
    query: { entityId?: string; capability?: string; deviceId?: string } = {},
  ): Promise<JsonValue> {
    const params = new URLSearchParams();
    if (query.entityId) params.set("entity_id", query.entityId);
    if (query.capability) params.set("capability", query.capability);
    if (query.deviceId) params.set("device_id", query.deviceId);
    const qs = params.toString();
    return this.request("GET", `/v1/entities/traceability${qs ? `?${qs}` : ""}`);
  }

  async queryEntities(body: JsonValue): Promise<JsonValue> {
    return this.request("POST", "/v1/entities/query", body);
  }

  async registerEntity(body: JsonValue): Promise<JsonValue> {
    return this.request("POST", "/v1/entities/register", body, true);
  }

  async tagEntity(
    entityId: string,
    body: { add?: string[]; remove?: string[] },
  ): Promise<JsonValue> {
    return this.request("POST", `/v1/entities/${entityId}/tags`, body, true);
  }

  async relateEntities(body: {
    from_id: string;
    to_id: string;
    kind: string;
    label?: string;
  }): Promise<JsonValue> {
    return this.request("POST", "/v1/entities/relationships", body, true);
  }

  async syncEntities(): Promise<JsonValue> {
    return this.request("POST", "/v1/entities/sync", {}, true);
  }

  async listDecisions(): Promise<JsonValue> {
    return this.request("GET", "/v1/decisions");
  }

  async getEntityDecisions(entityId: string): Promise<JsonValue> {
    return this.request("GET", `/v1/entities/${entityId}/decisions`);
  }

  async simulateDecision(body: JsonValue): Promise<JsonValue> {
    return this.request("POST", "/v1/decisions/simulate", body);
  }

  async approveEscalation(body: JsonValue): Promise<JsonValue> {
    return this.request("POST", "/v1/decisions/escalate", body, true);
  }

  async listDecisionPolicies(): Promise<JsonValue> {
    return this.request("GET", "/v1/decision-policies");
  }

  async listDecisionTraces(query: {
    file?: string;
    trace?: string;
  } = {}): Promise<JsonValue> {
    const params = new URLSearchParams();
    if (query.file) params.set("file", query.file);
    if (query.trace) params.set("trace", query.trace);
    const qs = params.toString();
    return this.request("GET", `/v1/decisions/traces${qs ? `?${qs}` : ""}`);
  }

  async listDecisionPolicyCache(query: { cache?: string } = {}): Promise<JsonValue> {
    const params = new URLSearchParams();
    if (query.cache) params.set("cache", query.cache);
    const qs = params.toString();
    return this.request("GET", `/v1/decision-policy-cache${qs ? `?${qs}` : ""}`);
  }

  async getPackageTrust(packageName: string, version?: string): Promise<JsonValue> {
    let path = `/v1/trust/package?name=${encodeURIComponent(packageName)}`;
    if (version) {
      path += `&version=${encodeURIComponent(version)}`;
    }
    return this.request("GET", path);
  }

  async healthCheck(): Promise<JsonValue> {
    return this.request("GET", "/v1/health");
  }

  async smartSpacesSummary(): Promise<JsonValue> {
    return this.request("GET", "/v1/smart-spaces/summary");
  }

  async listFacilities(): Promise<JsonValue> {
    return this.request("GET", "/v1/facilities");
  }

  async facilityReadiness(facilityId: string): Promise<JsonValue> {
    return this.request("GET", `/v1/facilities/${facilityId}/readiness`);
  }

  async zoneOccupancy(zoneId: string): Promise<JsonValue> {
    return this.request("GET", `/v1/zones/${zoneId}/occupancy`);
  }

  async listEnergySystems(): Promise<JsonValue> {
    return this.request("GET", "/v1/energy/systems");
  }

  async emergencyStatus(): Promise<JsonValue> {
    return this.request("GET", "/v1/emergency/status");
  }

  async smartSpacesDevices(facilityId?: string): Promise<JsonValue> {
    const qs = facilityId ? `?facility_id=${encodeURIComponent(facilityId)}` : "";
    return this.request("GET", `/v1/smart-spaces/devices${qs}`);
  }

  async facilityHealth(facilityId: string): Promise<JsonValue> {
    return this.request("GET", `/v1/facilities/${facilityId}/health`);
  }

  async facilitySecurity(facilityId: string): Promise<JsonValue> {
    return this.request("GET", `/v1/facilities/${facilityId}/security`);
  }

  async zoneEnvironment(zoneId: string): Promise<JsonValue> {
    return this.request("GET", `/v1/zones/${zoneId}/environment`);
  }

  async energySystem(systemId: string): Promise<JsonValue> {
    return this.request("GET", `/v1/energy/systems/${systemId}`);
  }

  async facilityFloorMap(facilityId: string): Promise<JsonValue> {
    return this.request("GET", `/v1/facilities/${facilityId}/floor-map`);
  }

  private analyticsPath(
    base: string,
    named?: { key: string; value: string },
    all = false,
  ): string {
    const params = new URLSearchParams();
    if (all) params.set("all", "1");
    if (named?.value) params.set(named.key, named.value);
    const qs = params.toString();
    return qs ? `${base}?${qs}` : base;
  }

  async analyticsReadiness(query?: string): Promise<JsonValue> {
    const path = query
      ? `/v1/analytics/readiness?${query}`
      : "/v1/analytics/readiness";
    return this.request("GET", path);
  }

  async analyticsWhatIf(options: {
    scenario?: string;
    all?: boolean;
  } = {}): Promise<JsonValue> {
    return this.request(
      "GET",
      this.analyticsPath(
        "/v1/analytics/what-if",
        options.scenario
          ? { key: "scenario", value: options.scenario }
          : undefined,
        options.all ?? false,
      ),
    );
  }

  async analyticsMissionRisk(): Promise<JsonValue> {
    return this.request("GET", "/v1/analytics/mission-risk");
  }

  async analyticsReadinessForecast(options: {
    horizon?: string;
    all?: boolean;
  } = {}): Promise<JsonValue> {
    return this.request(
      "GET",
      this.analyticsPath(
        "/v1/analytics/readiness-forecast",
        options.horizon
          ? { key: "horizon", value: options.horizon }
          : undefined,
        options.all ?? false,
      ),
    );
  }

  async analyticsTrustGraph(format?: string): Promise<JsonValue> {
    const qs = format ? `?format=${encodeURIComponent(format)}` : "";
    return this.request("GET", `/v1/analytics/trust-graph${qs}`);
  }

  async analyticsMissionTwin(): Promise<JsonValue> {
    return this.request("GET", "/v1/analytics/mission-twin");
  }

  async analyticsCertificationPack(options: { strict?: boolean } = {}): Promise<JsonValue> {
    const qs = options.strict ? "?strict=1" : "";
    return this.request("GET", `/v1/analytics/certification-pack${qs}`);
  }

  async analyticsTimeTravel(options: {
    at: string;
    inspect?: string;
    trace?: string;
  }): Promise<JsonValue> {
    const params = new URLSearchParams();
    params.set("at", options.at);
    if (options.inspect) params.set("inspect", options.inspect);
    if (options.trace) params.set("trace", options.trace);
    return this.request("GET", `/v1/analytics/time-travel?${params.toString()}`);
  }

  async analyticsHumanTeaming(): Promise<JsonValue> {
    return this.request("GET", "/v1/analytics/human-teaming");
  }

  async analyticsGovernance(policy?: string): Promise<JsonValue> {
    const qs = policy ? `?policy=${encodeURIComponent(policy)}` : "";
    return this.request("GET", `/v1/analytics/governance${qs}`);
  }

  async listTwins(): Promise<JsonValue> {
    return this.request("GET", "/v1/twins");
  }

  async getTwin(twinId: string): Promise<JsonValue> {
    return this.request("GET", `/v1/twins/${encodeURIComponent(twinId)}`);
  }

  async syncTwin(twinId?: string): Promise<JsonValue> {
    const qs = twinId ? `?twin_id=${encodeURIComponent(twinId)}` : "";
    return this.request("POST", `/v1/twins/sync${qs}`, {});
  }

  async pushTwinSnapshot(twinId: string, snapshot: JsonValue): Promise<JsonValue> {
    return this.request("POST", `/v1/twins/${encodeURIComponent(twinId)}/snapshots`, snapshot);
  }

  async rpc(method: string, params: JsonValue = {}): Promise<JsonValue> {
    const payload = await this.request("POST", "/v1/rpc", {
      method,
      params,
    });
    return (payload.result as JsonValue | undefined) ?? payload;
  }
}

/** WebSocket event stream URL for Control Center telemetry. */
export class EventStream {
  readonly wsUrl: string;

  constructor(baseUrl?: string) {
    const http =
      baseUrl ??
      process.env.SPANDA_CONTROL_CENTER_URL ??
      "http://127.0.0.1:8080";
    const ws = http.replace(/^https:/, "wss:").replace(/^http:/, "ws:");
    this.wsUrl = `${ws.replace(/\/$/, "")}/v1/stream/telemetry`;
  }

  static local(): EventStream {
    return new EventStream();
  }
}
