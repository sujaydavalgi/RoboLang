/**
 * Security-backed implementation of the runtime security boundary.
 * @module
 */

import {
  SecurityContext,
  createRobotIdentity,
  type SecurePolicy,
} from "./security/index.js";
import type { SecurityRuntime } from "./runtime/security-runtime.js";
import type {
  AuthenticationMode,
  EncryptionMode,
  IntegrityMode,
  SecretSource,
  SecurePolicyConfig,
  TrustBoundaryKind,
  TrustLevel,
} from "./runtime/security-types.js";

function mapPolicy(policy: SecurePolicyConfig): SecurePolicy {
  return {
    signed: policy.signed,
    minTrust: policy.minTrust,
    requires: policy.requires,
    encryption: policy.encryption,
    authentication: policy.authentication,
    integrity: policy.integrity,
    trustedSources: policy.trustedSources,
    rejectUntrusted: policy.rejectUntrusted,
  };
}

class SecurityBackedRuntime implements SecurityRuntime {
  private ctx = new SecurityContext();

  reset(): void {
    this.ctx = new SecurityContext();
  }

  enableStrictPermissions(): void {
    this.ctx.enableStrictPermissions();
  }

  injectSecurityFault(fault: string): void {
    this.ctx.securityFaultsActive.add(fault);
  }

  grantCapabilities(caps: readonly string[]): void {
    this.ctx.capabilities.grantAll([...caps]);
  }

  grantIfNotStrict(cap: string): void {
    this.ctx.grantIfNotStrict(cap);
  }

  setTrust(level: TrustLevel): void {
    this.ctx.trust = level;
  }

  registerSecret(name: string, source: SecretSource): void {
    this.ctx.secrets.register(name, source);
  }

  setIdentity(id: string, publicKey: string): void {
    this.ctx.identity = createRobotIdentity(id, publicKey, this.ctx.trust);
  }

  declareTrustBoundary(boundary: TrustBoundaryKind): void {
    this.ctx.trustBoundaries.declare(boundary);
  }

  setWireCert(path: string): void {
    this.ctx.wireCertPath = path;
  }

  setWireKeySecret(name: string): void {
    this.ctx.wireKeySecret = name;
  }

  setTransportContext(
    boundary: TrustBoundaryKind | null,
    encryption: EncryptionMode,
    authentication: AuthenticationMode,
    integrity: IntegrityMode,
  ): void {
    this.ctx.setTransportContext(boundary, encryption, authentication, integrity);
  }

  registerSecureEndpoint(path: string, policy: SecurePolicyConfig): void {
    this.ctx.secureEndpoints.register(path, mapPolicy(policy));
  }

  preparePublish(path: string, payload: string, sourceId: string, messageType = "Unknown"): void {
    this.ctx.preparePublish(path, payload, sourceId, messageType);
  }

  authorizeSubscribe(path: string): void {
    this.ctx.authorizeSubscribe(path);
  }

  verifyInboundMessage(
    path: string,
    payload: string,
    sourceId?: string | null,
    messageType = "Unknown",
  ): void {
    this.ctx.verifyInboundMessage(path, payload, sourceId, messageType);
  }

  requireOperation(operation: string): void {
    this.ctx.requireOperation(operation);
  }

  identityId(): string | undefined {
    return this.ctx.identity?.id;
  }
}

/** Create a security runtime backed by the full security platform module. */
export function createSecurityBackedRuntime(): SecurityRuntime {
  return new SecurityBackedRuntime();
}
