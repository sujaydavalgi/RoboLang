/**
 * Fleet OTLP aggregation helpers for TypeScript telemetry tooling.
 * @module
 */

import { renderOtlpJson } from "./telemetry-otlp.js";

export type FleetTelemetryShard = {
  robotId: string;
  otlpJson: string;
};

function envTruthy(name: string): boolean {
  const value = process.env[name];
  return value === "1" || value?.toLowerCase() === "true";
}

/** True when `SPANDA_FLEET_TELEMETRY_AUTO_INGEST=1` or `SPANDA_FLEET_TELEMETRY_INGEST=1`. */
export function envFleetAutoIngestEnabled(): boolean {
  return envTruthy("SPANDA_FLEET_TELEMETRY_AUTO_INGEST") || envTruthy("SPANDA_FLEET_TELEMETRY_INGEST");
}

/** Fleet mesh coordinator URL from `SPANDA_FLEET_MESH_URL`. */
export function envFleetMeshUrl(): string | undefined {
  return process.env.SPANDA_FLEET_MESH_URL;
}

/** Robot identity for fleet ingest (`SPANDA_ROBOT_ID`, then hostname env vars). */
export function envRobotId(): string {
  return process.env.SPANDA_ROBOT_ID
    ?? process.env.HOSTNAME
    ?? process.env.COMPUTERNAME
    ?? "robot";
}

function fleetIngestUrl(meshUrl: string): string {
  return meshUrl.endsWith("/")
    ? `${meshUrl}v1/fleet/telemetry/ingest`
    : `${meshUrl}/v1/fleet/telemetry/ingest`;
}

/** Merge per-robot OTLP/JSON bodies into one export payload. */
export function mergeFleetOtlpJson(shards: FleetTelemetryShard[]): string {
  const resourceMetrics: unknown[] = [];
  for (const shard of shards) {
    const parsed = JSON.parse(shard.otlpJson) as {
      resourceMetrics?: Array<Record<string, unknown>>;
    };
    const entries = parsed.resourceMetrics ?? [];
    for (const entry of entries) {
      const resource = (entry.resource ?? {}) as { attributes?: Array<Record<string, unknown>> };
      const attributes = [...(resource.attributes ?? [])];
      attributes.push({
        key: "spanda.robot.id",
        value: { stringValue: shard.robotId },
      });
      resourceMetrics.push({
        ...entry,
        resource: { ...resource, attributes },
      });
    }
  }
  return JSON.stringify({ resourceMetrics }, null, 2);
}

/** Fetch merged fleet OTLP/JSON from a mesh coordinator. */
export async function fetchFleetTelemetry(meshUrl: string, token?: string): Promise<string> {
  const url = meshUrl.endsWith("/")
    ? `${meshUrl}v1/fleet/telemetry`
    : `${meshUrl}/v1/fleet/telemetry`;
  const headers: Record<string, string> = {};
  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }
  const { remoteFetch } = await import("./http-fetch.js");
  const response = await remoteFetch(url, { headers });
  if (!response.ok) {
    const body = await response.text();
    throw new Error(`fleet telemetry HTTP ${response.status}${body ? `: ${body}` : ""}`);
  }
  return response.text();
}

/** POST one robot OTLP snapshot to the fleet mesh ingest endpoint. */
export async function ingestFleetTelemetry(
  meshUrl: string,
  shard: FleetTelemetryShard,
  token?: string,
): Promise<void> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }
  const { remoteFetch } = await import("./http-fetch.js");
  const response = await remoteFetch(fleetIngestUrl(meshUrl), {
    method: "POST",
    headers,
    body: JSON.stringify({
      robot_id: shard.robotId,
      otlp_json: shard.otlpJson,
    }),
  });
  if (!response.ok) {
    const body = await response.text();
    throw new Error(`fleet telemetry ingest HTTP ${response.status}${body ? `: ${body}` : ""}`);
  }
}

/** Ingest the current store OTLP snapshot to the fleet mesh after a session ends. */
export async function maybeAutoIngestFleetAfterSession(): Promise<void> {
  if (!envFleetAutoIngestEnabled()) {
    return;
  }
  const meshUrl = envFleetMeshUrl();
  if (!meshUrl) {
    console.error("SPANDA_FLEET_TELEMETRY_AUTO_INGEST set but SPANDA_FLEET_MESH_URL is missing");
    return;
  }
  const robotId = envRobotId();
  const token = process.env.SPANDA_FLEET_MESH_TOKEN;
  try {
    await ingestFleetTelemetry(meshUrl, {
      robotId,
      otlpJson: renderOtlpJson(),
    }, token);
    console.error(`Auto-ingested OTLP metrics for ${robotId} to ${meshUrl}`);
  } catch (error) {
    console.error(`Fleet telemetry auto-ingest failed: ${error instanceof Error ? error.message : error}`);
  }
}
