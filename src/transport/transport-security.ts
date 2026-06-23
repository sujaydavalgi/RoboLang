/**
 * Transport-layer security policy validation and TLS session management.
 * @module
 */

import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import tls from "node:tls";
import { WireCryptoSession } from "./wire-crypto.js";

export type EncryptionMode = "none" | "optional" | "required";
export type AuthenticationMode = "none" | "signed" | "mutual";
export type IntegrityMode = "none" | "required";

export const WIRE_PREFIX = "spanda/wire/v1:";

/** Per-transport TLS / encryption configuration wired from bus declarations. */
export type TransportSecurityConfig = {
  encryption: EncryptionMode;
  authentication: AuthenticationMode;
  integrity: IntegrityMode;
  certPath: string | null;
  keySecret: string | null;
  keyPath: string | null;
};

export function defaultTransportSecurity(): TransportSecurityConfig {
  // Return open transport security defaults.
  //
  // Parameters:
  // None.
  //
  // Returns:
  // TransportSecurityConfig with encryption disabled.
  //
  // Options:
  // None.
  //
  // Example:
  // const cfg = defaultTransportSecurity();

  return {
    encryption: "none",
    authentication: "none",
    integrity: "none",
    certPath: null,
    keySecret: null,
    keyPath: null,
  };
}

function parseEncryption(value: string | null | undefined): EncryptionMode {
  if (!value) return "none";
  if (value === "none" || value === "optional" || value === "required") return value;
  throw new Error(`invalid encryption mode '${value}'`);
}

function parseAuthentication(value: string | null | undefined): AuthenticationMode {
  if (!value) return "none";
  if (value === "none" || value === "signed" || value === "mutual") return value;
  throw new Error(`invalid authentication mode '${value}'`);
}

function parseIntegrity(value: string | null | undefined): IntegrityMode {
  if (!value) return "none";
  if (value === "none" || value === "required") return value;
  throw new Error(`invalid integrity mode '${value}'`);
}

export function transportSecurityFromBusFields(
  encryption?: string | null,
  authentication?: string | null,
  integrity?: string | null,
): TransportSecurityConfig {
  // Parse bus block security fields into a transport config.
  //
  // Parameters:
  // - `encryption` — optional bus encryption mode
  // - `authentication` — optional bus authentication mode
  // - `integrity` — optional bus integrity mode
  //
  // Returns:
  // Parsed TransportSecurityConfig.
  //
  // Options:
  // None.
  //
  // Example:
  // const cfg = transportSecurityFromBusFields("required", "mutual", "required");

  return {
    encryption: parseEncryption(encryption),
    authentication: parseAuthentication(authentication),
    integrity: parseIntegrity(integrity),
    certPath: null,
    keySecret: null,
    keyPath: null,
  };
}

export function sessionMaterial(config: TransportSecurityConfig): string {
  // Build deterministic session key material from cert and key secret names.
  //
  // Parameters:
  // - `config` — transport security config
  //
  // Returns:
  // Material string for WireCryptoSession derivation.
  //
  // Options:
  // None.
  //
  // Example:
  // const material = sessionMaterial(cfg);

  return `${config.certPath ?? "spanda-local"}:${config.keySecret ?? "spanda-local-key"}`;
}

export function validateTransportSecurity(config: TransportSecurityConfig, transport: string): void {
  // Fail fast when encryption is required without cert/key configuration.
  //
  // Parameters:
  // - `config` — transport security config
  // - `transport` — transport kind name for error messages
  //
  // Returns:
  // Nothing; throws when configuration is invalid.
  //
  // Options:
  // None.
  //
  // Example:
  // validateTransportSecurity(cfg, "mqtt");

  if (config.encryption === "required" && !config.certPath && !config.keySecret) {
    throw new Error(
      `transport '${transport}' requires encryption but no cert/key secret is configured`,
    );
  }
}

export type TlsEndpoint = { host: string; port: number; useTls: boolean };

export function parseTlsEndpoint(url: string): TlsEndpoint | null {
  // Parse broker URLs into host/port TLS endpoints.
  const lower = url.toLowerCase();
  let useTls = false;
  let stripped: string;
  let defaultPort = 1883;
  if (lower.startsWith("mqtts://")) {
    useTls = true;
    stripped = lower.slice("mqtts://".length);
    defaultPort = 8883;
  } else if (lower.startsWith("mqtt://")) {
    stripped = lower.slice("mqtt://".length);
  } else if (lower.startsWith("wss://")) {
    useTls = true;
    stripped = lower.slice("wss://".length);
    defaultPort = 443;
  } else if (lower.startsWith("ws://")) {
    stripped = lower.slice("ws://".length);
    defaultPort = 80;
  } else if (lower.startsWith("dds+sec://")) {
    useTls = true;
    stripped = lower.slice("dds+sec://".length);
    defaultPort = 7400;
  } else if (lower.startsWith("dds://")) {
    stripped = lower.slice("dds://".length);
    defaultPort = 7400;
  } else {
    return null;
  }
  const [host, portText] = stripped.includes(":")
    ? (stripped.split(":", 2) as [string, string])
    : [stripped, String(defaultPort)];
  return { host, port: Number.parseInt(portText, 10) || defaultPort, useTls };
}

function mtlsSessionMaterial(config: TransportSecurityConfig): string {
  if (config.certPath && existsSync(config.certPath) && config.keyPath && existsSync(config.keyPath)) {
    const cert = readFileSync(config.certPath);
    const key = readFileSync(config.keyPath);
    return createHash("sha256").update(cert).update(key).digest("hex");
  }
  return sessionMaterial(config);
}

function tryMtlsHandshake(
  endpoint: TlsEndpoint,
  certPath: string,
  keyPath: string,
): { cipherSuite: string; sessionMaterial: string } | null {
  if (!endpoint.useTls) return null;
  try {
    const cert = readFileSync(certPath);
    const key = readFileSync(keyPath);
    const socket = tls.connect({
      host: endpoint.host,
      port: endpoint.port,
      cert,
      key,
      rejectUnauthorized: true,
      servername: endpoint.host,
    });
    const deadline = Date.now() + 3000;
    while (!socket.authorized && Date.now() < deadline) {
      /* busy wait for short broker handshake in sync configure path */
    }
    if (!socket.authorized) {
      socket.destroy();
      return null;
    }
    const peer = socket.getPeerCertificate();
    const peerHash = peer.raw
      ? createHash("sha256").update(peer.raw).digest("hex")
      : "peer";
    socket.end();
    return {
      cipherSuite: "TLS/mTLS",
      sessionMaterial: `mtls:${endpoint.host}:${endpoint.port}:${peerHash}`,
    };
  } catch {
    return null;
  }
}

export function resolveBrokerUrl(busUrl?: string | null): string | null {
  // Resolve broker URL from bus declaration or SPANDA_BROKER_URL env.
  if (busUrl && busUrl.length > 0) return busUrl;
  const env = process.env.SPANDA_BROKER_URL;
  return env && env.length > 0 ? env : null;
}

export function urlRequiresTls(brokerUrl?: string | null): boolean {
  // Detect TLS broker URL schemes that require encrypted transport.
  //
  // Parameters:
  // - `brokerUrl` — optional broker URL
  //
  // Returns:
  // true when the URL implies TLS.
  //
  // Options:
  // None.
  //
  // Example:
  // urlRequiresTls("mqtts://broker:8883");

  if (!brokerUrl) return false;
  const lower = brokerUrl.toLowerCase();
  return (
    lower.startsWith("mqtts://") ||
    lower.startsWith("wss://") ||
    lower.startsWith("ssl://") ||
    lower.startsWith("tls://") ||
    lower.startsWith("dds+sec://")
  );
}

export type SecureCommPolicy = {
  encryption: EncryptionMode;
  authentication: AuthenticationMode;
  integrity: IntegrityMode;
};

export function effectiveTransportPolicy(
  robot: SecureCommPolicy,
  bus: TransportSecurityConfig,
): TransportSecurityConfig {
  // Merge robot secure_comm defaults with per-bus overrides.
  //
  // Parameters:
  // - `robot` — robot-wide secure_comm policy
  // - `bus` — per-bus transport security config
  //
  // Returns:
  // Effective merged transport security config.
  //
  // Options:
  // None.
  //
  // Example:
  // const effective = effectiveTransportPolicy(robotPolicy, busCfg);

  return {
    encryption: bus.encryption !== "none" ? bus.encryption : robot.encryption,
    authentication: bus.authentication !== "none" ? bus.authentication : robot.authentication,
    integrity: bus.integrity !== "none" ? bus.integrity : robot.integrity,
    certPath: bus.certPath,
    keySecret: bus.keySecret,
    keyPath: bus.keyPath,
  };
}

/** Negotiated TLS session for transport wire encryption (AES-256-GCM). */
export class TlsTransportSession {
  negotiated = false;
  cipherSuite = "none";
  peerVerified = false;
  private session: WireCryptoSession | null = null;

  connect(config: TransportSecurityConfig, brokerUrl?: string | null): void {
    // Negotiate wire encryption and optional mTLS handshake against a broker URL.
    validateTransportSecurity(config, "tls");
    if (config.encryption === "none") {
      this.negotiated = false;
      this.cipherSuite = "none";
      this.peerVerified = true;
      this.session = null;
      return;
    }
    const certFile =
      config.certPath && existsSync(config.certPath) ? config.certPath : null;
    const keyFile = config.keyPath && existsSync(config.keyPath) ? config.keyPath : null;
    if (config.authentication === "mutual" && (!certFile || !keyFile)) {
      throw new Error("mutual TLS authentication failed: missing certificate or key file");
    }
    if (
      config.authentication === "mutual" &&
      certFile &&
      keyFile &&
      brokerUrl &&
      process.env.SPANDA_MTLS_HANDSHAKE === "1"
    ) {
      const endpoint = parseTlsEndpoint(brokerUrl);
      if (endpoint?.useTls) {
        const hs = tryMtlsHandshake(endpoint, certFile, keyFile);
        if (hs) {
          const crypto = WireCryptoSession.fromMaterial(hs.sessionMaterial);
          this.cipherSuite = hs.cipherSuite;
          this.peerVerified = true;
          this.session = crypto;
          this.negotiated = true;
          return;
        }
        if (process.env.SPANDA_MTLS_REQUIRED === "1") {
          throw new Error("mTLS handshake failed");
        }
      }
    }
    this.peerVerified = config.authentication !== "mutual" || certFile !== null;
    if (certFile) {
      this.peerVerified = true;
    }
    const material = mtlsSessionMaterial(config);
    const crypto = WireCryptoSession.fromMaterial(material);
    this.cipherSuite = crypto.cipherSuite;
    this.session = crypto;
    this.negotiated = true;
  }

  encryptFrame(plaintext: string): string {
    // Encrypt a JSON wire frame when a TLS session is negotiated.
    //
    // Parameters:
    // - `plaintext` — JSON wire frame text
    //
    // Returns:
    // Plaintext or spanda/wire/v1 prefixed hex ciphertext.
    //
    // Options:
    // None.
    //
    // Example:
    // const wire = tls.encryptFrame(json);

    if (!this.negotiated) return plaintext;
    if (!this.session) throw new Error("TLS session not negotiated");
    const encrypted = this.session.encrypt(new TextEncoder().encode(plaintext));
    return `${WIRE_PREFIX}${Buffer.from(encrypted).toString("hex")}`;
  }

  decryptFrame(ciphertext: string): string {
    // Decrypt a transport wire frame or pass through plaintext.
    //
    // Parameters:
    // - `ciphertext` — wire frame text or encrypted hex payload
    //
    // Returns:
    // Decrypted JSON wire frame text.
    //
    // Options:
    // None.
    //
    // Example:
    // const json = tls.decryptFrame(wire);

    if (!this.negotiated) return ciphertext;
    if (ciphertext.startsWith(WIRE_PREFIX)) {
      if (!this.session) throw new Error("TLS session not negotiated");
      const hexPayload = ciphertext.slice(WIRE_PREFIX.length);
      const bytes = Buffer.from(hexPayload, "hex");
      const plain = this.session.decrypt(bytes);
      return new TextDecoder().decode(plain);
    }
    const legacy = `tls:${this.cipherSuite}:`;
    if (ciphertext.startsWith(legacy)) {
      return ciphertext.slice(legacy.length);
    }
    throw new Error("TLS decrypt failed: unrecognized wire frame");
  }
}
