/**
 * Spanda run pipeline — interpreter execution wired from the interfaces layer.
 *
 * Mirrors Rust `spanda-driver` run path with platform service bridges injected
 * at the CLI boundary (telemetry, security, providers, adapters).
 *
 * @module
 */

import { readFileSync } from "node:fs";
import type { Program } from "../ast/nodes.js";
import { compile, compileFile, compileWithRegistry } from "../compile.js";
import type { ModuleRegistry } from "../modules/index.js";
import {
  Interpreter,
  type RobotBackend,
  type RobotState,
} from "../runtime/index.js";
import { createDefaultSimulator } from "../simulator/index.js";
import {
  certificationRuntimeEnabledFromEnv,
  enforceCertificationRuntime,
} from "../certify-runtime.js";
import { resolveTraceOutputPath, saveMissionTrace } from "../replay.js";
import { defaultTelemetrySink, type TelemetrySink } from "../runtime/telemetry-sink.js";
import { type SecurityRuntime } from "../runtime/security-runtime.js";
import { defaultProviderRuntime, type ProviderRuntime } from "../runtime/provider-runtime.js";
import { defaultAdapterRuntime, type AdapterRuntime } from "../runtime/adapter-runtime.js";
import { createTelemetryStoreSink } from "../telemetry-store-bridge.js";
import { createSecurityBackedRuntime } from "../security-runtime-bridge.js";
import { createProviderBackedRuntime } from "../provider-runtime-bridge.js";
import { createAdapterBackedRuntime } from "../adapter-runtime-bridge.js";

export type RunOptions = {
  backend: RobotBackend;
  entryBehavior?: string;
  maxLoopIterations?: number;
  onMotionBlocked?: (reason: string) => void;
  onLog?: (message: string) => void;
  /** When set, attempt Rust CLI run before TS interpreter */
  rustCli?: boolean;
  moduleRegistry?: ModuleRegistry;
  recordTrace?: boolean;
  traceSource?: string;
  schedulerClock?: "sim" | "wall";
  secure?: boolean;
  injectSecurityFaults?: boolean;
  enforceCertify?: boolean;
  persistTelemetry?: boolean;
  telemetrySink?: TelemetrySink;
  securityRuntime?: SecurityRuntime;
  providerRuntime?: ProviderRuntime;
  adapterRuntime?: AdapterRuntime;
};

export type TestRunResult = {
  passed: number;
  failed: number;
  logs: string[];
};

function resolveRunInjections(options: RunOptions): {
  telemetrySink: TelemetrySink;
  securityRuntime: SecurityRuntime;
  providerRuntime: ProviderRuntime;
  adapterRuntime: AdapterRuntime;
} {
  const useStore = options.persistTelemetry ?? false;
  return {
    telemetrySink:
      options.telemetrySink ??
      (useStore ? createTelemetryStoreSink() : defaultTelemetrySink()),
    securityRuntime:
      options.securityRuntime ?? createSecurityBackedRuntime(),
    providerRuntime: options.providerRuntime ?? createProviderBackedRuntime(),
    adapterRuntime: options.adapterRuntime ?? createAdapterBackedRuntime(),
  };
}

function buildInterpreterOptions(
  options: RunOptions,
  injections: ReturnType<typeof resolveRunInjections>,
  moduleRegistry?: ModuleRegistry,
): ConstructorParameters<typeof Interpreter>[0] {
  return {
    backend: options.backend,
    maxLoopIterations: options.maxLoopIterations,
    onMotionBlocked: options.onMotionBlocked,
    onLog: options.onLog,
    moduleRegistry: moduleRegistry ?? options.moduleRegistry,
    recordTrace: options.recordTrace,
    traceSource: options.traceSource,
    schedulerClock: options.schedulerClock,
    secure: options.secure,
    injectSecurityFaults: options.injectSecurityFaults,
    telemetrySink: injections.telemetrySink,
    securityRuntime: injections.securityRuntime,
    providerRuntime: injections.providerRuntime,
    adapterRuntime: injections.adapterRuntime,
  };
}

export function run(program: Program, options: RunOptions): RobotState {
  const injections = resolveRunInjections(options);
  const { telemetrySink } = injections;

  telemetrySink.configureSessionPersist(options.persistTelemetry ?? false);
  if (options.persistTelemetry) {
    telemetrySink.beginRunSession(options.traceSource);
  }
  if (options.enforceCertify || certificationRuntimeEnabledFromEnv()) {
    enforceCertificationRuntime(program, true);
  }

  const interpreter = new Interpreter(buildInterpreterOptions(options, injections));
  const state = interpreter.run(program, options.entryBehavior);

  if (options.persistTelemetry) {
    const trace = interpreter.takeMissionTrace();
    let missionTracePath: string | undefined;
    let traceFrameCount = 0;
    if (options.recordTrace && trace) {
      missionTracePath = resolveTraceOutputPath(options.traceSource);
      saveMissionTrace(trace, missionTracePath);
      traceFrameCount = trace.frames.length;
    }
    telemetrySink.endRunSession(
      missionTracePath,
      interpreter.collectRuntimeMetrics(traceFrameCount),
      interpreter.getSimTimeMs(),
    );
  }
  return state;
}

export async function runSource(source: string, options: RunOptions): Promise<RobotState> {
  if (options.rustCli) {
    try {
      const { isCliAvailable, runViaCli } = await import("../rust-bridge.js");
      if (isCliAvailable()) {
        const result = runViaCli(source);
        return {
          pose: {
            x: result.state.pose.x,
            y: result.state.pose.y,
            theta: result.state.pose.theta,
            z: result.state.pose.z,
          },
          velocity: {
            linear: result.state.velocity.linear,
            angular: result.state.velocity.angular,
          },
          emergencyStop: result.state.emergency_stop,
        };
      }
    } catch {
      /* fall through to TS */
    }
  }
  const { program } = compile(source);
  return run(program, options);
}

export function runFile(path: string, options: RunOptions): RobotState {
  const { program } = compileFile(path);
  return run(program, options);
}

export function runTestsWithRegistry(
  source: string,
  registry?: ModuleRegistry,
): TestRunResult {
  const { program } = compileWithRegistry(source, registry);
  const logs: string[] = [];
  const backend = createDefaultSimulator();
  const injections = resolveRunInjections({
    backend,
    providerRuntime: createProviderBackedRuntime(),
    adapterRuntime: createAdapterBackedRuntime(),
  });

  try {
    const interpreter = new Interpreter({
      backend,
      maxLoopIterations: 10,
      moduleRegistry: registry,
      onLog: (msg) => logs.push(msg),
      ...injections,
    });
    interpreter.runTests(program);
    return { passed: program.tests.length, failed: 0, logs };
  } catch (e) {
    logs.push(e instanceof Error ? e.message : String(e));
    return { passed: 0, failed: Math.max(program.tests.length, 1), logs };
  }
}

export function runTests(source: string): TestRunResult {
  return runTestsWithRegistry(source, undefined);
}
