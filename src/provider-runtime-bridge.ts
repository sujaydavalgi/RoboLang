/**
 * Provider-backed implementation of the runtime provider dispatch boundary.
 * @module
 */

import {
  bootstrapProvidersForPackages,
  dispatchOfficialPackageCall,
  syncCommBusForOfficialPackages,
} from "./providers/index.js";
import type { ProviderRegistry } from "./providers/registry.js";
import type { RoutingCommBus } from "./transport/index.js";
import type { ProviderRuntime } from "./runtime/provider-runtime.js";
import type { RuntimeValue } from "./runtime/interpreter.js";

/** Full provider runtime delegating to providers bootstrap and dispatch. */
export const providerBackedRuntime: ProviderRuntime = {
  bootstrapProvidersForPackages(packageNames: readonly string[]): ProviderRegistry {
    return bootstrapProvidersForPackages([...packageNames]);
  },
  syncCommBus(commBus: RoutingCommBus, registry: ProviderRegistry): void {
    syncCommBusForOfficialPackages(commBus, registry);
  },
  dispatchOfficialPackageCall(
    registry: ProviderRegistry,
    modulePath: string,
    functionName: string,
    args: RuntimeValue[],
  ): RuntimeValue | null {
    return dispatchOfficialPackageCall(registry, modulePath, functionName, args);
  },
};

/** Create a provider runtime backed by the official providers module. */
export function createProviderBackedRuntime(): ProviderRuntime {
  return providerBackedRuntime;
}
