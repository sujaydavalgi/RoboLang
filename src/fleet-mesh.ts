/**
 * Multi-host fleet mesh coordinator client.
 * @module
 */

import type { PeerDelivery } from "./fleet-orchestrator.js";

export type MeshRelayResponse = {
  ok: boolean;
  relayed: number;
  failed: number;
  error?: string;
};

export function defaultFleetMeshUrl(): string | undefined {
  return process.env.SPANDA_FLEET_MESH_URL;
}

async function meshFetch(
  meshUrl: string,
  path: string,
  body: string,
  token?: string,
): Promise<Response> {
  const base = meshUrl.replace(/\/$/, "");
  const headers: Record<string, string> = {
    Accept: "application/json",
    "Content-Type": "application/json",
  };
  if (token) headers.Authorization = `Bearer ${token}`;
  return fetch(`${base}${path}`, { method: "POST", headers, body });
}

export async function relayDeliveriesViaMesh(
  meshUrl: string,
  deliveries: PeerDelivery[],
  token?: string,
): Promise<MeshRelayResponse> {
  const response = await meshFetch(
    meshUrl,
    "/v1/mesh/relay",
    JSON.stringify({ deliveries: deliveries.map((d) => ({
      from_robot: d.fromRobot,
      to_robot: d.toRobot,
      topic: d.topic,
      step: d.step,
      delivered: d.delivered,
    })) }),
    token,
  );
  if (!response.ok) {
    throw new Error(`fleet mesh HTTP ${response.status}: ${await response.text()}`);
  }
  const body = (await response.json()) as MeshRelayResponse;
  return body;
}
