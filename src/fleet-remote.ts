/**
 * Remote fleet peer relay client and agent registry.
 * @module
 */

import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import type { PeerDelivery } from "./fleet-orchestrator.js";
import { remoteFetch } from "./http-fetch.js";

export type FleetAgentEntry = {
  robotName: string;
  url: string;
  token?: string;
};

export type FleetAgentRegistry = {
  agents: FleetAgentEntry[];
};

export type PeerRelayResponse = {
  ok: boolean;
  toRobot: string;
  topic: string;
  step: string;
  error?: string;
};

export function defaultFleetAgentsPath(): string {
  return process.env.SPANDA_FLEET_AGENTS ?? ".spanda/fleet-agents.json";
}

export function emptyFleetAgentRegistry(): FleetAgentRegistry {
  return { agents: [] };
}

export function loadFleetAgentRegistry(text: string | null): FleetAgentRegistry {
  if (!text) return emptyFleetAgentRegistry();
  try {
    const parsed = JSON.parse(text) as {
      agents?: Array<{ robot_name?: string; robotName?: string; url: string; token?: string }>;
    };
    return {
      agents: (parsed.agents ?? []).map((entry) => ({
        robotName: entry.robotName ?? entry.robot_name ?? "",
        url: entry.url,
        token: entry.token,
      })),
    };
  } catch {
    return emptyFleetAgentRegistry();
  }
}

export function serializeFleetAgentRegistry(registry: FleetAgentRegistry): string {
  return JSON.stringify(
    {
      agents: registry.agents.map((entry) => ({
        robot_name: entry.robotName,
        url: entry.url,
        token: entry.token,
      })),
    },
    null,
    2,
  );
}

export function readFleetAgentRegistryFromDisk(path = defaultFleetAgentsPath()): FleetAgentRegistry {
  if (!existsSync(path)) return emptyFleetAgentRegistry();
  return loadFleetAgentRegistry(readFileSync(path, "utf-8"));
}

export function writeFleetAgentRegistryToDisk(
  registry: FleetAgentRegistry,
  path = defaultFleetAgentsPath(),
): void {
  const abs = resolve(path);
  mkdirSync(dirname(abs), { recursive: true });
  writeFileSync(abs, serializeFleetAgentRegistry(registry));
}

export function registerFleetAgent(
  registry: FleetAgentRegistry,
  robotName: string,
  url: string,
  token?: string,
): FleetAgentRegistry {
  if (!url.startsWith("http://") && !url.startsWith("https://")) {
    throw new Error(`fleet agent URL must start with http:// or https:// (got ${url})`);
  }
  const agents = registry.agents.filter((entry) => entry.robotName !== robotName);
  agents.push({ robotName, url, token });
  agents.sort((a, b) => a.robotName.localeCompare(b.robotName));
  return { agents };
}

export function lookupFleetAgent(
  registry: FleetAgentRegistry,
  robotName: string,
): FleetAgentEntry | undefined {
  return registry.agents.find((entry) => entry.robotName === robotName);
}

async function agentFetch(
  entry: FleetAgentEntry,
  method: string,
  path: string,
  body?: string,
): Promise<Response> {
  const base = entry.url.replace(/\/$/, "");
  const headers: Record<string, string> = { Accept: "application/json" };
  if (body) headers["Content-Type"] = "application/json";
  if (entry.token) headers.Authorization = `Bearer ${entry.token}`;
  return remoteFetch(`${base}${path}`, { method, headers, body });
}

export async function fleetAgentHealth(entry: FleetAgentEntry): Promise<boolean> {
  const response = await agentFetch(entry, "GET", "/v1/health");
  if (!response.ok) return false;
  const body = (await response.json()) as { ok?: boolean };
  return body.ok === true;
}

export async function relayPeerDelivery(
  entry: FleetAgentEntry,
  delivery: PeerDelivery,
): Promise<PeerRelayResponse> {
  const response = await agentFetch(
    entry,
    "POST",
    "/v1/peer",
    JSON.stringify({
      from_robot: delivery.fromRobot,
      to_robot: delivery.toRobot,
      topic: delivery.topic,
      step: delivery.step,
    }),
  );
  if (!response.ok) {
    throw new Error(`fleet agent HTTP ${response.status}: ${await response.text()}`);
  }
  const body = (await response.json()) as {
    ok?: boolean;
    to_robot?: string;
    toRobot?: string;
    topic?: string;
    step?: string;
    error?: string;
  };
  return {
    ok: body.ok === true,
    toRobot: body.toRobot ?? body.to_robot ?? delivery.toRobot,
    topic: body.topic ?? delivery.topic,
    step: body.step ?? delivery.step,
    error: body.error,
  };
}

export async function relayPeerDeliveries(
  deliveries: PeerDelivery[],
  registry: FleetAgentRegistry,
): Promise<{ relayed: number; failed: number }> {
  let relayed = 0;
  let failed = 0;
  for (const delivery of deliveries) {
    const agent = lookupFleetAgent(registry, delivery.toRobot);
    if (!agent) {
      failed += 1;
      continue;
    }
    try {
      const resp = await relayPeerDelivery(agent, delivery);
      if (resp.ok) relayed += 1;
      else failed += 1;
    } catch {
      failed += 1;
    }
  }
  return { relayed, failed };
}
