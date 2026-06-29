/**
 * Adapter-backed implementation of the runtime adapter bridge boundary.
 * @module
 */

import { invokeNav2Bridge, invokeSlamBridge } from "./adapter-bridge.js";
import type { AdapterRuntime } from "./runtime/adapter-runtime.js";

/** Full adapter runtime delegating to optional subprocess Nav2/SLAM bridges. */
export const adapterBackedRuntime: AdapterRuntime = {
  invokeNav2Bridge(goal: string): string | undefined {
    return invokeNav2Bridge(goal);
  },
  invokeSlamBridge(operation: string): string | undefined {
    return invokeSlamBridge(operation);
  },
};

/** Create an adapter runtime backed by adapter-bridge subprocess hooks. */
export function createAdapterBackedRuntime(): AdapterRuntime {
  return adapterBackedRuntime;
}
