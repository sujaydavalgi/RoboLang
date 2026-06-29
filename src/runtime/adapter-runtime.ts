/**
 * Injectable adapter bridge boundary for Nav2/SLAM subprocess backends.
 * @module
 */

/** Extension points for optional production navigation/SLAM adapter bridges. */
export interface AdapterRuntime {
  invokeNav2Bridge(goal: string): string | undefined;
  invokeSlamBridge(operation: string): string | undefined;
}

/** Built-in no-op adapter runtime for simulation without external bridges. */
export const builtinAdapterRuntime: AdapterRuntime = {
  invokeNav2Bridge() {
    return undefined;
  },
  invokeSlamBridge() {
    return undefined;
  },
};

/** Default built-in adapter runtime for direct interpreter use. */
export function defaultAdapterRuntime(): AdapterRuntime {
  return builtinAdapterRuntime;
}
