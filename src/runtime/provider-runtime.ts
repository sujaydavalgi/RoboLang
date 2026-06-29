/**
 * Injectable provider dispatch boundary for official package runtime wiring.
 * @module
 */

import { ProviderRegistry } from "../providers/registry.js";
import type { RoutingCommBus } from "../transport/index.js";
import type { RuntimeValue } from "./interpreter.js";

/** Extension points for official package provider bootstrap and dispatch. */
export interface ProviderRuntime {
  bootstrapProvidersForPackages(packageNames: readonly string[]): ProviderRegistry;
  syncCommBus(commBus: RoutingCommBus, registry: ProviderRegistry): void;
  dispatchOfficialPackageCall(
    registry: ProviderRegistry,
    modulePath: string,
    functionName: string,
    args: RuntimeValue[],
  ): RuntimeValue | null;
}

/** Built-in provider runtime with empty bootstrap and no official package dispatch. */
export class BuiltinProviderRuntime implements ProviderRuntime {
  bootstrapProvidersForPackages(packageNames: readonly string[]): ProviderRegistry {
    const registry = new ProviderRegistry();
    registry.setOfficialPackages([...packageNames]);
    return registry;
  }

  syncCommBus(_commBus: RoutingCommBus, _registry: ProviderRegistry): void {}

  dispatchOfficialPackageCall(
    _registry: ProviderRegistry,
    _modulePath: string,
    _functionName: string,
    _args: RuntimeValue[],
  ): RuntimeValue | null {
    return null;
  }
}

/** Default built-in provider runtime for direct interpreter use without providers crate. */
export function defaultProviderRuntime(): ProviderRuntime {
  return new BuiltinProviderRuntime();
}
