/**
 * index module (security/index.ts).
 * @module
 */

import { sha256 } from "@noble/hashes/sha256";
import { sha512 } from "@noble/hashes/sha512";
import * as ed from "@noble/ed25519";
import { bytesToHex, hexToBytes } from "@noble/hashes/utils";
import { TrustBoundaryRegistry } from "./trust-boundary.js";
import type { AuthenticationMode, EncryptionMode, IntegrityMode } from "../transport/transport-security.js";

export { TrustBoundaryRegistry, parseTrustBoundary, boundaryForTransportName } from "./trust-boundary.js";
export {
  securityCheck,
  securityAudit,
  analyzeProgram,
  reportHasErrors,
  type SecurityReport,
  type SecurityFinding,
} from "./validate.js";

ed.etc.sha512Sync = (...m: Uint8Array[]) => sha512(ed.etc.concatBytes(...m));

export type TrustLevel = "untrusted" | "restricted" | "trusted" | "certified";

const TRUST_RANK: Record<TrustLevel, number> = {
  untrusted: 0,
  restricted: 1,
  trusted: 2,
  certified: 3,
};

export function trustSatisfies(actual: TrustLevel, required: TrustLevel): boolean {
  // TrustSatisfies.
  //
  // Parameters:
  // - `actual` — input value
  // - `required` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = trustSatisfies(actual, required);
  return TRUST_RANK[actual] >= TRUST_RANK[required];
}

export function parseTrustLevel(level: string): TrustLevel | null {
  // ParseTrustLevel.
  //
  // Parameters:
  // - `level` — input value
  //
  // Returns:
  // `Some` / non-null value on success, otherwise `None` / null.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = parseTrustLevel(level);
  if (level in TRUST_RANK) return level as TrustLevel;
  return null;
}

function seedBytes(material: string): Uint8Array {
  // SeedBytes.
  //
  // Parameters:
  // - `material` — input value
  //
  // Returns:
  // `Uint8Array`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = seedBytes(material);
  return sha256(new TextEncoder().encode(material));
}

export function isHexPublicKey(key: string): boolean {
  // IsHexPublicKey.
  //
  // Parameters:
  // - `key` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = isHexPublicKey(key);
  return key.length === 64 && /^[0-9a-fA-F]+$/.test(key);
}

export function publicKeyFromMaterial(material: string): string {
  // PublicKeyFromMaterial.
  //
  // Parameters:
  // - `material` — input value
  //
  // Returns:
  // Text result.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = publicKeyFromMaterial(material);
  const priv = seedBytes(material);
  return bytesToHex(ed.getPublicKey(priv));
}

export function sha256Hex(data: string): string {
  // Sha256Hex.
  //
  // Parameters:
  // - `data` — input value
  //
  // Returns:
  // Text result.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = sha256Hex(data);
  return bytesToHex(sha256(new TextEncoder().encode(data)));
}

export async function signAsync(data: string, keyMaterial: string): Promise<string> {
  // SignAsync.
  //
  // Parameters:
  // - `data` — input value
  // - `keyMaterial` — input value
  //
  // Returns:
  // Success value on completion, or an error.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = signAsync(data, keyMaterial);
  const priv = keyMaterial.length === 64 && isHexPublicKey(keyMaterial)
    ? hexToBytes(keyMaterial)
    : seedBytes(keyMaterial);
  const sig = await ed.signAsync(new TextEncoder().encode(data), priv);
  return bytesToHex(sig);
}

export function sign(data: string, keyMaterial: string): string {
  // Sign.
  //
  // Parameters:
  // - `data` — input value
  // - `keyMaterial` — input value
  //
  // Returns:
  // Text result.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = sign(data, keyMaterial);
  const priv = keyMaterial.length === 64 && isHexPublicKey(keyMaterial)
    ? hexToBytes(keyMaterial)
    : seedBytes(keyMaterial);
  return bytesToHex(ed.sign(new TextEncoder().encode(data), priv));
}

export function verifySignature(data: string, signature: string, key: string): boolean {
  // VerifySignature.
  //
  // Parameters:
  // - `data` — input value
  // - `signature` — input value
  // - `key` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = verifySignature(data, signature, key);
  try {
    const sig = hexToBytes(signature);

    // continue when length differs from 64.
    if (sig.length !== 64) return false;
    const msg = new TextEncoder().encode(data);

    // continue when isHexPublicKey(key).
    if (isHexPublicKey(key)) {
      return ed.verify(sig, msg, hexToBytes(key));
    }
    const priv = seedBytes(key);
    const pub = ed.getPublicKey(priv);
    return ed.verify(sig, msg, pub);
  } catch {
    return false;
  }
}

export type RobotIdentity = {
  id: string;
  publicKey: string;
  trust: TrustLevel;
  signingMaterial(): string;
  verifyingKeyHex(): string;
};

export function createRobotIdentity(
  id: string,
  publicKey: string,
  trust: TrustLevel = "trusted",
): RobotIdentity {
  // CreateRobotIdentity.
  //
  // Parameters:
  // - `id` — input value
  // - `publicKey` — input value
  // - `trust` — optional input
  //
  // Returns:
  // `RobotIdentity`.
  //
  // Options:
  // - `trust` — optional parameter
  //
  // Example:

  // const result = createRobotIdentity(id, publicKey, trust);
  return {
    id,
    publicKey,
    trust,
    signingMaterial() {
      //
      // Parameters:
      // None.
      //
      // Returns:
      //
      // Options:
      // None.
      //
      // Example:

      // continue when publicKey || isHexPublicKey is falsy.
      if (!publicKey || isHexPublicKey(publicKey)) return `spanda-device-${id}`;
      return publicKey;
    },
    verifyingKeyHex() {
      //
      // Parameters:
      // None.
      //
      // Returns:
      //
      // Options:
      // None.
      //
      // Example:

      // continue when isHexPublicKey(publicKey).
      if (isHexPublicKey(publicKey)) return publicKey;
      return publicKeyFromMaterial(this.signingMaterial());
    },
  };
}

export type SecurePolicy = {
  signed: boolean;
  minTrust: TrustLevel | null;
  requires: string[];
  encryption: "none" | "optional" | "required";
  authentication: "none" | "signed" | "mutual";
  integrity: "none" | "required";
  trustedSources: string[];
  rejectUntrusted: boolean;
};

export class CapabilitySet {
  private granted = new Set<string>();
  private permissive = false;

  grant(cap: string): void {
    // Grant.
    //
    // Parameters:
    // - `cap` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = grant(cap);

    this.granted.add(cap);
  }

  grantAll(caps: string[]): void {
    // GrantAll.
    //
    // Parameters:
    // - `caps` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = grantAll(caps);

    for (const c of caps) this.grant(c);
  }

  has(cap: string): boolean {
    // Has.
    //
    // Parameters:
    // - `cap` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = has(cap);

    return this.permissive || this.granted.has(cap);
  }

  require(cap: string): void {
    // Require.
    //
    // Parameters:
    // - `cap` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = require(cap);

    if (!this.has(cap)) throw new Error(`capability denied: ${cap}`);
  }
}

export type SecretSource =
  | { source: "env"; var: string }
  | { source: "literal"; value: string }
  | { source: "file"; path: string };

export class SecretStore {
  private secrets = new Map<string, SecretSource>();

  register(name: string, source: SecretSource): void {
    // Register the value.
    //
    // Parameters:
    // - `name` — input value
    // - `source` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = register(name, source);

    this.secrets.set(name, source);
  }

  resolve(name: string): string {
    // Resolve.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = resolve(name);

    const src = this.secrets.get(name);
    if (!src) throw new Error(`secret not found: ${name}`);
    if (src.source === "literal") return src.value;
    if (src.source === "file") return src.path;
    const val = process.env[src.var];
    if (val === undefined) throw new Error(`environment variable '${src.var}' not set`);
    return val;
  }
}

export class SecureEndpointRegistry {
  private policies = new Map<string, SecurePolicy>();

  register(path: string, policy: SecurePolicy): void {
    // Register the value.
    //
    // Parameters:
    // - `path` — input value
    // - `policy` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = register(path, policy);

    this.policies.set(path, policy);
  }

  policyOrOpen(path: string): SecurePolicy {
    // PolicyOrOpen.
    //
    // Parameters:
    // - `path` — input value
    //
    // Returns:
    // SecurePolicy.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = policyOrOpen(path);

    return this.policies.get(path) ?? {
      signed: false,
      minTrust: null,
      requires: [],
      encryption: "none",
      authentication: "none",
      integrity: "none",
      trustedSources: [],
      rejectUntrusted: false,
    };
  }
}

export class SecurityContext {
  identity: RobotIdentity | null = null;
  trust: TrustLevel = "trusted";
  secrets = new SecretStore();
  capabilities = new CapabilitySet();
  secureEndpoints = new SecureEndpointRegistry();
  trustBoundaries = new TrustBoundaryRegistry();
  transportBoundary: import("./trust-boundary.js").TrustBoundaryKind | null = null;
  busEncryption: EncryptionMode = "none";
  busAuthentication: AuthenticationMode = "none";
  busIntegrity: IntegrityMode = "none";
  strictPermissions = false;
  wireCertPath: string | null = null;
  wireKeySecret: string | null = null;
  securityFaultsActive = new Set<string>();

  enableStrictPermissions(): void {
    // EnableStrictPermissions.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = enableStrictPermissions();

    this.strictPermissions = true;
  }

  grantIfNotStrict(cap: string): void {
    // GrantIfNotStrict.
    //
    // Parameters:
    // - `cap` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = grantIfNotStrict(cap);

    if (!this.strictPermissions) this.capabilities.grant(cap);
  }

  requireOperation(operation: string): void {
    // RequireOperation.
    //
    // Parameters:
    // - `operation` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = requireOperation(operation);

    const map: Record<string, string> = {
      "audit.record": "audit.write",
      "audit.read": "audit.read",
      sign: "identity.sign",
      "identity.sign": "identity.sign",
      "identity.verify": "identity.verify",
      "ledger.anchor": "ledger.anchor",
      "cellular.sim_identity": "cellular.connect",
    };
    const cap = map[operation];
    if (cap) this.capabilities.require(cap);
  }

  authorizePublish(path: string, sourceId: string): void {
    const policy = this.secureEndpoints.policyOrOpen(path);
    if (policy.trustedSources.length > 0) {
      if (!policy.trustedSources.includes(sourceId)) {
        if (policy.rejectUntrusted) {
          throw new Error(`untrusted source rejected: ${sourceId}`);
        }
        throw new Error(`untrusted source '${sourceId}' on ${path}`);
      }
      this.capabilities.require("secure_topic.publish");
    } else if (
      policy.encryption !== "none" ||
      policy.signed ||
      policy.authentication !== "none"
    ) {
      this.capabilities.require("secure_topic.publish");
    }
  }

  authorizeSubscribe(path: string): void {
    const policy = this.secureEndpoints.policyOrOpen(path);
    if (
      policy.encryption !== "none" ||
      policy.signed ||
      policy.authentication !== "none" ||
      policy.trustedSources.length > 0
    ) {
      this.capabilities.require("secure_topic.subscribe");
    }
  }

  verifyInbound(path: string, sourceId?: string | null): void {
    const policy = this.secureEndpoints.policyOrOpen(path);
    if (policy.trustedSources.length > 0) {
      const sid = sourceId ?? "unknown";
      if (!policy.trustedSources.includes(sid)) {
        if (policy.rejectUntrusted) {
          throw new Error(`untrusted source rejected: ${sid}`);
        }
        throw new Error(`untrusted source '${sid}' on ${path}`);
      }
    }
    const secured =
      policy.signed ||
      policy.minTrust ||
      policy.requires.length > 0 ||
      policy.encryption !== "none" ||
      policy.authentication !== "none" ||
      policy.integrity !== "none";
    if (!secured) return;
    for (const cap of policy.requires) this.capabilities.require(cap);
    if (!this.identity) throw new Error(`identity required for ${path}`);
    if (policy.minTrust && !trustSatisfies(this.trust, policy.minTrust)) {
      throw new Error(`trust level insufficient: required ${policy.minTrust}, have ${this.trust}`);
    }
    if (policy.encryption === "required") this.capabilities.require("crypto.decrypt");
    if (policy.signed || policy.integrity === "required") {
      this.capabilities.require("identity.verify");
    }
  }

  setTransportContext(
    boundary: import("./trust-boundary.js").TrustBoundaryKind | null,
    encryption: EncryptionMode,
    authentication: AuthenticationMode,
    integrity: IntegrityMode,
  ): void {
    this.transportBoundary = boundary;
    this.busEncryption = encryption;
    this.busAuthentication = authentication;
    this.busIntegrity = integrity;
  }

  enforceTrustBoundary(messageType: string, endpoint: SecurePolicy): void {
    if (!this.transportBoundary) return;
    if (!this.trustBoundaries.contains(this.transportBoundary)) return;
    const encryption =
      endpoint.encryption !== "none" ? endpoint.encryption : this.busEncryption;
    const authentication =
      endpoint.authentication !== "none" ? endpoint.authentication : this.busAuthentication;
    const integrity = endpoint.integrity !== "none" ? endpoint.integrity : this.busIntegrity;
    this.trustBoundaries.validateChannel(
      this.transportBoundary,
      encryption,
      authentication,
      integrity,
      messageType,
    );
  }

  verifyInboundMessage(path: string, _payload: string, sourceId?: string | null, messageType = "Unknown"): void {
    const policy = this.secureEndpoints.policyOrOpen(path);
    this.enforceTrustBoundary(messageType, policy);
    this.authorizeSubscribe(path);
    this.verifyInbound(path, sourceId);
  }

  signOutbound(path: string, payload: string, sourceId?: string): void {
    if (sourceId) this.authorizePublish(path, sourceId);
    //
    // Parameters:
    // - `path` — input value
    // - `payload` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = signOutbound(path, payload);

    const policy = this.secureEndpoints.policyOrOpen(path);
    for (const cap of policy.requires) this.capabilities.require(cap);
    if (policy.minTrust && !trustSatisfies(this.trust, policy.minTrust)) {
      throw new Error(`trust level insufficient: required ${policy.minTrust}, have ${this.trust}`);
    }
    if (policy.signed || policy.minTrust || policy.requires.length > 0) {
      if (!this.identity) throw new Error(`identity required for ${path}`);
      if (policy.signed) this.capabilities.require("identity.sign");
    }
    if (policy.encryption !== "none") {
      this.capabilities.require("crypto.encrypt");
    }
  }

  wireSessionMaterial(): string {
    // Build session key material from configured cert path and key secret name.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Material string for wire encryption session derivation.
    //
    // Options:
    // None.
    //
    // Example:
    // const material = ctx.wireSessionMaterial();

    return `${this.wireCertPath ?? "spanda-local"}:${this.wireKeySecret ?? "spanda-local-key"}`;
  }

  preparePublish(path: string, payload: string, sourceId: string, messageType = "Unknown"): void {
    // Authorize and validate an outbound publish against secure endpoint policy.
    const policy = this.secureEndpoints.policyOrOpen(path);
    this.enforceTrustBoundary(messageType, policy);
    this.authorizePublish(path, sourceId);
    this.checkSecurityFaults(path, payload);
    if (policy.encryption !== "none") {
      this.capabilities.require("crypto.encrypt");
    }
    this.signOutbound(path, payload);
  }

  checkSecurityFaults(path: string, payload: string): void {
    // Apply injected security fault simulation when enabled.
    if (this.securityFaultsActive.has("InvalidSignature")) {
      throw new Error("signature invalid");
    }
    if (this.securityFaultsActive.has("SecureHandshakeDropped")) {
      throw new Error(`secure handshake dropped on ${path}`);
    }
    if (this.securityFaultsActive.has("ManInTheMiddle")) {
      throw new Error("man-in-the-middle detected");
    }
    if (this.securityFaultsActive.has("ExpiredCertificate")) {
      throw new Error(`certificate expired for ${this.identity?.id ?? "unknown"}`);
    }
    if (this.securityFaultsActive.has("ReplayAttack")) {
      const hash = sha256Hex(`${path}:${payload}`);
      const key = `replay:${path}`;
      if (this.replayHashes?.has(key) && this.replayHashes.get(key)!.has(hash)) {
        throw new Error(`replay detected on ${path}`);
      }
      if (!this.replayHashes) this.replayHashes = new Map();
      const seen = this.replayHashes.get(key) ?? new Set<string>();
      seen.add(hash);
      this.replayHashes.set(key, seen);
    }
  }

  private replayHashes?: Map<string, Set<string>>;
}

export const KNOWN_CAPABILITIES = [
  "audit.write",
  "audit.read",
  "identity.sign",
  "identity.verify",
  "identity.read",
  "ledger.anchor",
  "network.outbound",
  "actuator.execute",
  "crypto.encrypt",
  "crypto.decrypt",
  "crypto.sign",
  "crypto.verify",
  "secret.read",
  "secure_topic.publish",
  "secure_topic.subscribe",
] as const;

export function isKnownCapability(cap: string): boolean {
  // IsKnownCapability.
  //
  // Parameters:
  // - `cap` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = isKnownCapability(cap);
  return (KNOWN_CAPABILITIES as readonly string[]).includes(cap);
}
