/**
 * Injectable security runtime boundary for interpreter identity and comm enforcement.
 * @module
 */

import {
  boundaryForTransportName,
  parseTrustBoundary,
  parseTrustLevel,
  type AuthenticationMode,
  type EncryptionMode,
  type IntegrityMode,
  type SecretSource,
  type SecurePolicyConfig,
  type TrustBoundaryKind,
  type TrustLevel,
} from "./security-types.js";

export type {
  AuthenticationMode,
  EncryptionMode,
  IntegrityMode,
  SecretSource,
  SecurePolicyConfig,
  TrustBoundaryKind,
  TrustLevel,
} from "./security-types.js";
export {
  boundaryForTransportName,
  parseTrustBoundary,
  parseTrustLevel,
} from "./security-types.js";

/** Extension points for security enforcement at runtime. */
export interface SecurityRuntime {
  reset(): void;
  enableStrictPermissions(): void;
  injectSecurityFault(fault: string): void;
  grantCapabilities(caps: readonly string[]): void;
  grantIfNotStrict(cap: string): void;
  setTrust(level: TrustLevel): void;
  registerSecret(name: string, source: SecretSource): void;
  setIdentity(id: string, publicKey: string): void;
  declareTrustBoundary(boundary: TrustBoundaryKind): void;
  setWireCert(path: string): void;
  setWireKeySecret(name: string): void;
  setTransportContext(
    boundary: TrustBoundaryKind | null,
    encryption: EncryptionMode,
    authentication: AuthenticationMode,
    integrity: IntegrityMode,
  ): void;
  registerSecureEndpoint(path: string, policy: SecurePolicyConfig): void;
  preparePublish(path: string, payload: string, sourceId: string, messageType?: string): void;
  authorizeSubscribe(path: string): void;
  verifyInboundMessage(
    path: string,
    payload: string,
    sourceId?: string | null,
    messageType?: string,
  ): void;
  requireOperation(operation: string): void;
  identityId(): string | undefined;
}

/** Permissive built-in security runtime for simulation without the security crate. */
export class BuiltinSecurityRuntime implements SecurityRuntime {
  private trust: TrustLevel = "trusted";
  private strictPermissions = false;
  private capabilities = new Set<string>();
  private identity: { id: string; publicKey: string } | null = null;
  private secureEndpoints = new Map<string, SecurePolicyConfig>();
  private trustBoundaries = new Set<TrustBoundaryKind>();
  private transportBoundary: TrustBoundaryKind | null = null;
  private busEncryption: EncryptionMode = "none";
  private busAuthentication: AuthenticationMode = "none";
  private busIntegrity: IntegrityMode = "none";
  private securityFaultsActive = new Set<string>();
  private secrets = new Map<string, SecretSource>();
  private wireCertPath: string | null = null;
  private wireKeySecret: string | null = null;

  reset(): void {
    this.trust = "trusted";
    this.strictPermissions = false;
    this.capabilities.clear();
    this.identity = null;
    this.secureEndpoints.clear();
    this.trustBoundaries.clear();
    this.transportBoundary = null;
    this.busEncryption = "none";
    this.busAuthentication = "none";
    this.busIntegrity = "none";
    this.securityFaultsActive.clear();
    this.secrets.clear();
    this.wireCertPath = null;
    this.wireKeySecret = null;
  }

  enableStrictPermissions(): void {
    this.strictPermissions = true;
  }

  injectSecurityFault(fault: string): void {
    this.securityFaultsActive.add(fault);
  }

  grantCapabilities(caps: readonly string[]): void {
    for (const cap of caps) {
      this.capabilities.add(cap);
    }
  }

  grantIfNotStrict(cap: string): void {
    if (!this.strictPermissions) {
      this.capabilities.add(cap);
    }
  }

  setTrust(level: TrustLevel): void {
    this.trust = level;
  }

  registerSecret(name: string, source: SecretSource): void {
    this.secrets.set(name, source);
  }

  setIdentity(id: string, publicKey: string): void {
    this.identity = { id, publicKey };
  }

  declareTrustBoundary(boundary: TrustBoundaryKind): void {
    this.trustBoundaries.add(boundary);
  }

  setWireCert(path: string): void {
    this.wireCertPath = path;
  }

  setWireKeySecret(name: string): void {
    this.wireKeySecret = name;
  }

  setTransportContext(
    boundary: TrustBoundaryKind | null,
    encryption: EncryptionMode,
    authentication: AuthenticationMode,
    integrity: IntegrityMode,
  ): void {
    this.transportBoundary = boundary;
    this.busEncryption = encryption;
    this.busAuthentication = authentication;
    this.busIntegrity = integrity;
  }

  registerSecureEndpoint(path: string, policy: SecurePolicyConfig): void {
    this.secureEndpoints.set(path, policy);
  }

  preparePublish(path: string, _payload: string, _sourceId: string, _messageType = "Unknown"): void {
    if (this.securityFaultsActive.has("InvalidSignature")) {
      throw new Error(`security fault: invalid signature on ${path}`);
    }
  }

  authorizeSubscribe(_path: string): void {}

  verifyInboundMessage(_path: string, _payload: string, _sourceId?: string | null, _messageType = "Unknown"): void {}

  requireOperation(_operation: string): void {}

  identityId(): string | undefined {
    return this.identity?.id;
  }
}

/** Default permissive security runtime for direct interpreter use. */
export function defaultSecurityRuntime(): SecurityRuntime {
  return new BuiltinSecurityRuntime();
}
