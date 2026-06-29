/**
 * Durable mission checkpoint persistence (TypeScript mirror).
 * @module
 */

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import type { MissionStateSnapshot } from "./runtime/continuity-types.js";

export type ContinuityCheckpointStore = {
  entries: Record<string, MissionStateSnapshot>;
};

function storeKey(mission: string, robot: string): string {
  return `${mission}::${robot}`;
}

export function defaultCheckpointStorePath(): string {
  return process.env.SPANDA_CONTINUITY_CHECKPOINTS ?? ".spanda/mission-checkpoints.json";
}

export function loadCheckpointStore(path = defaultCheckpointStorePath()): ContinuityCheckpointStore {
  if (!existsSync(path)) {
    return { entries: {} };
  }
  try {
    return JSON.parse(readFileSync(path, "utf-8")) as ContinuityCheckpointStore;
  } catch {
    return { entries: {} };
  }
}

export function saveCheckpointStore(
  store: ContinuityCheckpointStore,
  path = defaultCheckpointStorePath(),
): void {
  const dir = dirname(path);
  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }
  writeFileSync(path, JSON.stringify(store, null, 2));
}

export function recordCheckpoint(
  store: ContinuityCheckpointStore,
  mission: string,
  robot: string,
  snapshot: MissionStateSnapshot,
): ContinuityCheckpointStore {
  return {
    entries: {
      ...store.entries,
      [storeKey(mission, robot)]: snapshot,
    },
  };
}

export function loadCheckpoint(
  store: ContinuityCheckpointStore,
  mission: string,
  robot: string,
): MissionStateSnapshot | undefined {
  return store.entries[storeKey(mission, robot)];
}

export function persistCheckpoint(
  mission: string,
  robot: string,
  snapshot: MissionStateSnapshot,
  path = defaultCheckpointStorePath(),
): void {
  const store = loadCheckpointStore(path);
  saveCheckpointStore(recordCheckpoint(store, mission, robot, snapshot), path);
}
