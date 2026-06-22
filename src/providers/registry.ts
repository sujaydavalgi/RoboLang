/**
 * In-memory provider registry mirrored from Rust `providers/registry.rs`.
 * @module
 */

import type { RuntimeValue } from "../runtime/interpreter.js";
import type { TransportConfig } from "../transport/index.js";

/** Stable identifier for a registered provider implementation. */
export type ProviderId = {
  package: string;
  name: string;
};

/** Transport provider surface used by registry-backed comm-bus routing. */
export type TransportProvider = {
  connect(config: TransportConfig): void;
  disconnect(): void;
  isConnected(): boolean;
  publish(topic: string, messageType: string, value: RuntimeValue): void;
  subscribe(topic: string): void;
  receive(topic: string): RuntimeValue | null;
};

/** Registry key for a package-scoped transport provider. */
export function transportRegistryKey(packageName: string): string {
  return `${packageName}::project`;
}

/** In-memory registry of installed provider implementations (TS fallback). */
export class ProviderRegistry {
  private readonly transports = new Map<string, TransportProvider>();
  private readonly capabilities = new Set<string>();
  private officialPackageList: string[] = [];

  setOfficialPackages(names: string[]): void {
    this.officialPackageList = names;
  }

  officialPackages(): readonly string[] {
    return this.officialPackageList;
  }

  isOfficialPackage(name: string): boolean {
    return this.officialPackageList.includes(name);
  }

  grantCapability(cap: string): void {
    this.capabilities.add(cap);
  }

  hasCapability(cap: string): boolean {
    return this.capabilities.has(cap);
  }

  registerTransport(key: string, provider: TransportProvider): void {
    this.transports.set(key, provider);
  }

  withTransport<R>(key: string, fn: (provider: TransportProvider) => R): R | undefined {
    const provider = this.transports.get(key);
    if (!provider) return undefined;
    return fn(provider);
  }

  listTransports(): ProviderId[] {
    return [...this.transports.keys()].map((key) => {
      const [pkg, name] = key.split("::");
      return { package: pkg, name: name ?? "project" };
    });
  }
}
