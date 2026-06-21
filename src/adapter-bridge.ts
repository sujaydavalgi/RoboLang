/**
 * Optional subprocess bridges for production Nav2/SLAM adapter backends.
 * @module
 */

import { execFileSync } from "node:child_process";

function bridgeCommand(envKey: string): string | undefined {
  const value = process.env[envKey]?.trim();
  return value ? value : undefined;
}

export function invokeNav2Bridge(goal: string): string | undefined {
  const template = bridgeCommand("SPANDA_NAV2_CMD");
  if (!template) return undefined;
  const commandLine = template.replace("{goal}", goal);
  const parts = commandLine.split(/\s+/).filter(Boolean);
  const program = parts[0];
  if (!program) return undefined;
  try {
    const output = execFileSync(program, parts.slice(1), { encoding: "utf-8" });
    return output.trim();
  } catch {
    return undefined;
  }
}

export function invokeSlamBridge(operation: string): string | undefined {
  const template = bridgeCommand("SPANDA_SLAM_CMD");
  if (!template) return undefined;
  const commandLine = template.replace("{op}", operation);
  const parts = commandLine.split(/\s+/).filter(Boolean);
  const program = parts[0];
  if (!program) return undefined;
  try {
    const output = execFileSync(program, parts.slice(1), { encoding: "utf-8" });
    return output.trim();
  } catch {
    return undefined;
  }
}
