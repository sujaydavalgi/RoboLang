export type Diagnostic = {
  message: string;
  line: number;
  column: number;
};

export type CheckResult = {
  ok: boolean;
  diagnostics: Diagnostic[];
};

export type PoseState = {
  x: number;
  y: number;
  theta: number;
  z?: number;
};

export type VelocityState = {
  linear: number;
  angular: number;
};

export type RobotState = {
  pose: PoseState;
  velocity: VelocityState;
  emergency_stop: boolean;
};

export type RunResult = {
  state: RobotState;
  events: string[];
  logs: string[];
};

export type RunOptions = {
  entryBehavior?: string;
  maxLoopIterations?: number;
};

export interface SynapseNative {
  checkSource(source: string): CheckResult;
  runSource(source: string, options?: RunOptions): RunResult;
  coreVersion(): string;
}

let native: SynapseNative | null = null;
let loadAttempted = false;

export function isNativeAvailable(): boolean {
  loadNative();
  return native !== null;
}

function loadNative(): void {
  if (loadAttempted) return;
  loadAttempted = true;
  try {
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const mod = require("./native.js") as SynapseNative;
    native = mod;
  } catch {
    native = null;
  }
}

export function checkSource(source: string): CheckResult | null {
  loadNative();
  if (!native) return null;
  return native.checkSource(source);
}

export function runSource(source: string, options?: RunOptions): RunResult | null {
  loadNative();
  if (!native) return null;
  return native.runSource(source, options);
}

export function coreVersion(): string | null {
  loadNative();
  return native?.coreVersion() ?? null;
}
