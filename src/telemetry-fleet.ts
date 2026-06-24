/**
 * Fleet OTLP aggregation helpers for TypeScript telemetry tooling.
 * @module
 */

export type FleetTelemetryShard = {
  robotId: string;
  otlpJson: string;
};

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
