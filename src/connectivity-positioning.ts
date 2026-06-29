/**
 * Positioning and wireless connectivity helpers for parse, verify, and runtime.
 * @module
 */

import type {
  BleServiceDecl,
  ConnectivityPolicyDecl,
  GeofenceDecl,
  RequiresConnectivityDecl,
  RequiresHardwareDecl,
  RequiresNetworkDecl,
  ResourceBudgetDecl,
} from "./foundations.js";
import type { TransportKind } from "./comm/decls.js";
import type { CompatItem } from "./rust-bridge-types.js";
import type { HardwareProfile } from "./hardware-profile-types.js";

export type ConnectivityRequirement = "required" | "optional";

export type GeofenceRuntime = {
  name: string;
  centerLat: number;
  centerLon: number;
  radiusM: number;
};

export type ConnectivityPolicyRuntime = {
  name: string;
  preferred: string;
  fallback: string;
  emergency: string | null;
  switchIfLatencyMs: number | null;
  switchIfPacketLossPct: number | null;
};

export function haversineM(lat1: number, lon1: number, lat2: number, lon2: number): number {
  // Description:
  //     HaversineM.
  //
  // Inputs:
  //     lat1: number
  //         Caller-supplied lat1.
  //     lon1: number
  //         Caller-supplied lon1.
  //     lat2: number
  //         Caller-supplied lat2.
  //     lon2: number
  //         Caller-supplied lon2.
  //
  // Outputs:
  //     result: number
  //         Return value from `haversineM`.
  //
  // Example:
  //     const result = haversineM(lat1, lon1, lat2, lon2);
  // Description:
  //     HaversineM.
  //
  // Inputs:
  //     lat1: number
  //         Caller-supplied lat1.
  //     lon1: number
  //         Caller-supplied lon1.
  //     lat2: number
  //         Caller-supplied lat2.
  //     lon2: number
  //         Caller-supplied lon2.
  //
  // Outputs:
  //     result: number
  //         Return value from `haversineM`.
  //
  // Example:

  //     const result = haversineM(lat1, lon1, lat2, lon2);

  const r = 6_371_000;
  const dLat = ((lat2 - lat1) * Math.PI) / 180;
  const dLon = ((lon2 - lon1) * Math.PI) / 180;
  const a =
    Math.sin(dLat / 2) ** 2 +
    Math.cos((lat1 * Math.PI) / 180) *
      Math.cos((lat2 * Math.PI) / 180) *
      Math.sin(dLon / 2) ** 2;

  return 2 * r * Math.asin(Math.sqrt(a));
}

export function geofenceContains(fence: GeofenceRuntime, lat: number, lon: number): boolean {
  // Description:
  //     GeofenceContains.
  //
  // Inputs:
  //     fence: GeofenceRuntime
  //         Caller-supplied fence.
  //     lat: number
  //         Caller-supplied lat.
  //     lon: number
  //         Caller-supplied lon.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `geofenceContains`.
  //
  // Example:
  //     const result = geofenceContains(fence, lat, lon);
  // Description:
  //     GeofenceContains.
  //
  // Inputs:
  //     fence: GeofenceRuntime
  //         Caller-supplied fence.
  //     lat: number
  //         Caller-supplied lat.
  //     lon: number
  //         Caller-supplied lon.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `geofenceContains`.
  //
  // Example:

  //     const result = geofenceContains(fence, lat, lon);

  return haversineM(lat, lon, fence.centerLat, fence.centerLon) <= fence.radiusM;
}

export function geofenceFromDecl(decl: GeofenceDecl): GeofenceRuntime {
  // Description:
  //     GeofenceFromDecl.
  //
  // Inputs:
  //     decl: GeofenceDecl
  //         Caller-supplied decl.
  //
  // Outputs:
  //     result: GeofenceRuntime
  //         Return value from `geofenceFromDecl`.
  //
  // Example:
  //     const result = geofenceFromDecl(decl);
  // Description:
  //     GeofenceFromDecl.
  //
  // Inputs:
  //     decl: GeofenceDecl
  //         Caller-supplied decl.
  //
  // Outputs:
  //     result: GeofenceRuntime
  //         Return value from `geofenceFromDecl`.
  //
  // Example:

  //     const result = geofenceFromDecl(decl);

  return {
    name: decl.name,
    centerLat: decl.centerLat,
    centerLon: decl.centerLon,
    radiusM: decl.radiusM,
  };
}

export function connectivityPolicyFromDecl(
  decl: ConnectivityPolicyDecl,
): ConnectivityPolicyRuntime {
  // Description:
  //     ConnectivityPolicyFromDecl.
  //
  // Inputs:
  //     decl: ConnectivityPolicyDecl
  //         Caller-supplied decl.
  //
  // Outputs:
  //     result: ConnectivityPolicyRuntime
  //         Return value from `connectivityPolicyFromDecl`.
  //
  // Example:
  //     const result = connectivityPolicyFromDecl(decl);
  // Description:
  //     ConnectivityPolicyFromDecl.
  //
  // Inputs:
  //     decl: ConnectivityPolicyDecl
  //         Caller-supplied decl.
  //
  // Outputs:
  //     result: ConnectivityPolicyRuntime
  //         Return value from `connectivityPolicyFromDecl`.
  //
  // Example:

  //     const result = connectivityPolicyFromDecl(decl);

  return {
    name: decl.name,
    preferred: decl.preferred,
    fallback: decl.fallback,
    emergency: decl.emergency,
    switchIfLatencyMs: decl.switchIfLatencyMs,
    switchIfPacketLossPct: decl.switchIfPacketLossPct,
  };
}

export function connectivityCapabilities(): string[] {
  // Description:
  //     ConnectivityCapabilities.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string[]
  //         Return value from `connectivityCapabilities`.
  //
  // Example:
  //     const result = connectivityCapabilities();
  // Description:
  //     ConnectivityCapabilities.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string[]
  //         Return value from `connectivityCapabilities`.
  //
  // Example:

  //     const result = connectivityCapabilities();

  return [
    "gps.read",
    "network.status",
    "wifi.connect",
    "bluetooth.scan",
    "bluetooth.pair",
    "cellular.connect",
    "network.failover",
  ];
}

export function positioningSensorTypes(): string[] {
  // Description:
  //     PositioningSensorTypes.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string[]
  //         Return value from `positioningSensorTypes`.
  //
  // Example:
  //     const result = positioningSensorTypes();
  // Description:
  //     PositioningSensorTypes.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string[]
  //         Return value from `positioningSensorTypes`.
  //
  // Example:

  //     const result = positioningSensorTypes();

  return ["GPS", "GNSS"];
}

export function connectivityLinkTypes(): string[] {
  // Description:
  //     ConnectivityLinkTypes.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string[]
  //         Return value from `connectivityLinkTypes`.
  //
  // Example:
  //     const result = connectivityLinkTypes();
  // Description:
  //     ConnectivityLinkTypes.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string[]
  //         Return value from `connectivityLinkTypes`.
  //
  // Example:

  //     const result = connectivityLinkTypes();

  return ["WiFi", "WiFi6", "Bluetooth", "Bluetooth5", "LTE", "5G", "GPS", "Satellite"];
}

export function connectivityFaultNames(): string[] {
  // Description:
  //     ConnectivityFaultNames.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string[]
  //         Return value from `connectivityFaultNames`.
  //
  // Example:
  //     const result = connectivityFaultNames();
  // Description:
  //     ConnectivityFaultNames.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string[]
  //         Return value from `connectivityFaultNames`.
  //
  // Example:

  //     const result = connectivityFaultNames();

  return [
    "GPSLost",
    "GpsFailure",
    "GpsDrift",
    "GpsSpoofing",
    "NetworkOutage",
    "NetworkLatencySpike",
    "WeakWifi",
    "LteOutage",
    "SatelliteOutage",
    "FiveGHandoff",
    "BluetoothDisconnect",
    "PacketLoss",
    "LatencySpike",
  ];
}

export function connectivityKeyToProfileTokens(key: string): string[] {
  // Description:
  //     ConnectivityKeyToProfileTokens.
  //
  // Inputs:
  //     key: string
  //         Caller-supplied key.
  //
  // Outputs:
  //     result: string[]
  //         Return value from `connectivityKeyToProfileTokens`.
  //
  // Example:
  //     const result = connectivityKeyToProfileTokens(key);
  // Description:
  //     ConnectivityKeyToProfileTokens.
  //
  // Inputs:
  //     key: string
  //         Caller-supplied key.
  //
  // Outputs:
  //     result: string[]
  //         Return value from `connectivityKeyToProfileTokens`.
  //
  // Example:

  //     const result = connectivityKeyToProfileTokens(key);

  switch (key) {
    case "gps":
      return ["GPS"];
    case "gnss":
      return ["GNSS", "GPS"];
    case "wifi":
      return ["WiFi", "WiFi6"];
    case "bluetooth":
      return ["Bluetooth", "Bluetooth5", "BLE"];
    case "cellular":
      return ["LTE", "FourG", "4G", "FiveG", "5G"];
    case "ethernet":
      return ["Ethernet"];
    case "mesh":
      return ["Mesh"];
    case "satellite":
      return ["Satellite"];
    default:
      return [];
  }
}

export function faultToConnectivity(
  fault: string,
): {
  // Description:
  //     FaultToConnectivity.
  //
  // Inputs:
  //     fault: string
  //         Caller-supplied fault.
  //
  // Outputs:
  //     None.
  //
  // Example:

 // const result = faultToConnectivity(fault);
 domain: string; event: string } | null {
  // Map a simulation fault to a connectivity trigger domain and event.
  //
  // Parameters:
  // - `fault` — fault name from simulate_compatibility or comm bus
  //
  // Returns:
  // Trigger pair, or null when the fault is unrelated.
  //
  // Options:
  // None.
  //
  // Example:

  // const evt = faultToConnectivity("NetworkOutage");

  switch (fault) {
    case "NetworkOutage":
    case "LteOutage":
    case "SatelliteOutage":
    case "WeakWifi":
      return { domain: "network", event: "disconnected" };
    case "BluetoothDisconnect":
      return { domain: "bluetooth", event: "device_disconnected" };
    case "FiveGHandoff":
      return { domain: "cellular", event: "roaming" };
    case "GpsSpoofing":
      return { domain: "gps", event: "spoofed" };
    case "GpsDrift":
      return { domain: "gps", event: "drift" };
    default:
      return null;
  }
}

export function connectivityLinkToTransport(link: string): TransportKind {
  // Description:
  //     ConnectivityLinkToTransport.
  //
  // Inputs:
  //     link: string
  //         Caller-supplied link.
  //
  // Outputs:
  //     result: TransportKind
  //         Return value from `connectivityLinkToTransport`.
  //
  // Example:
  //     const result = connectivityLinkToTransport(link);
  // Description:
  //     ConnectivityLinkToTransport.
  //
  // Inputs:
  //     link: string
  //         Caller-supplied link.
  //
  // Outputs:
  //     result: TransportKind
  //         Return value from `connectivityLinkToTransport`.
  //
  // Example:

  //     const result = connectivityLinkToTransport(link);

  switch (link.toLowerCase()) {
    case "wifi":
      return "mqtt";
    case "cellular":
    case "lte":
    case "4g":
    case "fiveg":
    case "5g":
      return "dds";
    case "bluetooth":
    case "ble":
      return "websocket";
    case "ethernet":
      return "ros2";
    case "satellite":
      return "websocket";
    case "network":
      return "sim";
    default:
      return "sim";
  }
}

export function applyGpsPositionFaults(
  faults: Set<string>,
  trueLat: number,
  trueLon: number,
  simTimeMs: number,
): {
  // Description:
  //     ApplyGpsPositionFaults.
  //
  // Inputs:
  //     faults: Set<string>
  //         Caller-supplied faults.
  //     trueLat: number
  //         Caller-supplied trueLat.
  //     trueLon: number
  //         Caller-supplied trueLon.
  //     simTimeMs: number
  //         Caller-supplied simTimeMs.
  //
  // Outputs:
  //     None.
  //
  // Example:

 // const result = applyGpsPositionFaults(faults, trueLat, trueLon, simTimeMs);
 lat: number; lon: number; fixQuality: number } {
  // Apply GPS drift or spoofing simulation to WGS84 coordinates.
  //
  // Parameters:
  // - `faults` — active injected fault names
  // - `trueLat`, `trueLon` — ground-truth degrees
  // - `simTimeMs` — simulation clock for drift accumulation
  //
  // Returns:
  // Adjusted latitude, longitude, and fix quality.
  //
  // Options:
  // None.
  //
  // Example:

  // const pos = applyGpsPositionFaults(faults, 30, -97, 1000);

  if (faults.has("GpsSpoofing")) {
    return { lat: trueLat + 0.009, lon: trueLon + 0.012, fixQuality: 0.3 };
  }
  if (faults.has("GpsDrift")) {
    const driftM = (simTimeMs / 1000) * 0.05;
    const dDeg = driftM / 111_000;
    return { lat: trueLat + dDeg, lon: trueLon + dDeg * 0.5, fixQuality: 0.8 };
  }
  return { lat: trueLat, lon: trueLon, fixQuality: 1.0 };
}

export function isCellularLink(link: string): boolean {
  // Description:
  //     IsCellularLink.
  //
  // Inputs:
  //     link: string
  //         Caller-supplied link.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isCellularLink`.
  //
  // Example:
  //     const result = isCellularLink(link);
  // Description:
  //     IsCellularLink.
  //
  // Inputs:
  //     link: string
  //         Caller-supplied link.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isCellularLink`.
  //
  // Example:

  //     const result = isCellularLink(link);

  const lower = link.toLowerCase();
  return ["cellular", "lte", "4g", "fourg", "fiveg", "5g"].includes(lower);
}

export function isSatelliteLink(link: string): boolean {
  // Description:
  //     IsSatelliteLink.
  //
  // Inputs:
  //     link: string
  //         Caller-supplied link.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isSatelliteLink`.
  //
  // Example:
  //     const result = isSatelliteLink(link);
  // Description:
  //     IsSatelliteLink.
  //
  // Inputs:
  //     link: string
  //         Caller-supplied link.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isSatelliteLink`.
  //
  // Example:

  //     const result = isSatelliteLink(link);

  return link.toLowerCase() === "satellite";
}

export function isModemBearer(link: string): boolean {
  // Description:
  //     IsModemBearer.
  //
  // Inputs:
  //     link: string
  //         Caller-supplied link.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isModemBearer`.
  //
  // Example:
  //     const result = isModemBearer(link);
  // Description:
  //     IsModemBearer.
  //
  // Inputs:
  //     link: string
  //         Caller-supplied link.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isModemBearer`.
  //
  // Example:

  //     const result = isModemBearer(link);

  return isCellularLink(link) || isSatelliteLink(link);
}

export function isWifiLink(link: string): boolean {
  // Description:
  //     IsWifiLink.
  //
  // Inputs:
  //     link: string
  //         Caller-supplied link.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isWifiLink`.
  //
  // Example:
  //     const result = isWifiLink(link);
  // Description:
  //     IsWifiLink.
  //
  // Inputs:
  //     link: string
  //         Caller-supplied link.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isWifiLink`.
  //
  // Example:

  //     const result = isWifiLink(link);

  const lower = link.toLowerCase();
  return lower === "wifi" || lower === "wi-fi" || lower === "wifi6";
}

export function isLinkImpaired(link: string, faults: Set<string> | Iterable<string>): boolean {
  // Description:
  //     IsLinkImpaired.
  //
  // Inputs:
  //     link: string
  //         Caller-supplied link.
  //     faults: Set<string> | Iterable<string>
  //         Caller-supplied faults.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isLinkImpaired`.
  //
  // Example:
  //     const result = isLinkImpaired(link, faults);
  // Description:
  //     IsLinkImpaired.
  //
  // Inputs:
  //     link: string
  //         Caller-supplied link.
  //     faults: Set<string> | Iterable<string>
  //         Caller-supplied faults.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isLinkImpaired`.
  //
  // Example:

  //     const result = isLinkImpaired(link, faults);

  const lower = link.toLowerCase();
  for (const fault of faults) {
    switch (fault) {
      case "NetworkOutage":
        if (isSatelliteLink(link) || lower === "bluetooth" || lower === "ble") continue;
        if (isWifiLink(link) || isCellularLink(link) || lower === "network" || lower === "ethernet") {
          return true;
        }
        break;
      case "WeakWifi":
        if (isWifiLink(link) || lower === "network") return true;
        break;
      case "LteOutage":
        if (isCellularLink(link)) return true;
        break;
      case "SatelliteOutage":
        if (isSatelliteLink(link)) return true;
        break;
    }
  }
  return false;
}

export function runtimeSimIdentity(link: string, attested: boolean) {
  // Description:
  //     RuntimeSimIdentity.
  //
  // Inputs:
  //     link: string
  //         Caller-supplied link.
  //     attested: boolean
  //         Caller-supplied attested.
  //
  // Outputs:
  //     None.
  //
  // Example:
  //     const result = runtimeSimIdentity(link, attested);
  // Description:
  //     RuntimeSimIdentity.
  //
  // Inputs:
  //     link: string
  //         Caller-supplied link.
  //     attested: boolean
  //         Caller-supplied attested.
  //
  // Outputs:
  //     None.
  //
  // Example:

  //     const result = runtimeSimIdentity(link, attested);

  const lower = link.toLowerCase();
  let hash = 0;
  for (let i = 0; i < lower.length; i++) {
    hash = (hash * 31 + lower.charCodeAt(i)) >>> 0;
  }
  const iccid = `89${String(hash % 10_000_000_000).padStart(10, "0")}00000000000`;
  const carrier = isSatelliteLink(link)
    ? "sim-satellite"
    : lower.includes("5g") || lower.includes("fiveg")
      ? "sim-5g"
      : isCellularLink(link)
        ? "sim-lte"
        : "sim-unknown";
  const esim = lower.includes("5g") || lower.includes("fiveg");
  return {
    kind: "object" as const,
    typeName: "SimIdentity",
    fields: {
      iccid: { kind: "string" as const, value: iccid },
      carrier: { kind: "string" as const, value: carrier },
      esim: { kind: "bool" as const, value: esim },
      attested: { kind: "bool" as const, value: attested },
    },
  };
}

function compatItem(
  category: string,
  message: string,
  severity: CompatItem["severity"],
  line: number,
  column: number,
): CompatItem {
  // Description:
  //     CompatItem.
  //
  // Inputs:
  //     category: string
  //         Caller-supplied category.
  //     message: string
  //         Caller-supplied message.
  //     severity: CompatItem["severity"]
  //         Caller-supplied severity.
  //     line: number
  //         Caller-supplied line.
  //     column: number
  //         Caller-supplied column.
  //
  // Outputs:
  //     result: CompatItem
  //         Return value from `compatItem`.
  //
  // Example:

  //     const result = compatItem(category, message, severity, line, column);

  return { category, message, severity, line, column };
}

export function verifyRequiresConnectivity(
  req: RequiresConnectivityDecl,
  profile: HardwareProfile,
): CompatItem[] {
  // Description:
  //     VerifyRequiresConnectivity.
  //
  // Inputs:
  //     req: RequiresConnectivityDecl
  //         Caller-supplied req.
  //     profile: HardwareProfile
  //         Caller-supplied profile.
  //
  // Outputs:
  //     result: CompatItem[]
  //         Return value from `verifyRequiresConnectivity`.
  //
  // Example:
  //     const result = verifyRequiresConnectivity(req, profile);
  // Description:
  //     VerifyRequiresConnectivity.
  //
  // Inputs:
  //     req: RequiresConnectivityDecl
  //         Caller-supplied req.
  //     profile: HardwareProfile
  //         Caller-supplied profile.
  //
  // Outputs:
  //     result: CompatItem[]
  //         Return value from `verifyRequiresConnectivity`.
  //
  // Example:

  //     const result = verifyRequiresConnectivity(req, profile);

  const items: CompatItem[] = [];
  const line = req.span.start.line;
  const column = req.span.start.column;
  const profileSet = new Set(profile.connectivity ?? []);

  for (const [key, level] of req.channels) {
    if (level !== "required") continue;
    const tokens = connectivityKeyToProfileTokens(key);
    if (tokens.length === 0) {
      items.push(
        compatItem(
          "connectivity",
          `Unknown connectivity key '${key}' in requires_connectivity`,
          "warning",
          line,
          column,
        ),
      );
      continue;
    }
    const satisfied = tokens.some((t) => profileSet.has(t));
    if (satisfied) {
      items.push(
        compatItem(
          "connectivity",
          `Required connectivity '${key}' present on '${profile.name}'`,
          "pass",
          line,
          column,
        ),
      );
    } else {
      items.push(
        compatItem(
          "connectivity",
          `Required connectivity '${key}' not on '${profile.name}' [${[...profileSet].join(", ")}]`,
          "error",
          line,
          column,
        ),
      );
    }
  }

  if (req.bandwidthMbpsMin != null) {
    const minBw = req.bandwidthMbpsMin;
    const bw = profile.networkBandwidthMbps;
    if (bw == null) {
      items.push(
        compatItem(
          "connectivity",
          "Target bandwidth unknown — cannot verify connectivity bandwidth requirement",
          "warning",
          line,
          column,
        ),
      );
    } else if (bw >= minBw) {
      items.push(
        compatItem(
          "connectivity",
          `Bandwidth ${bw} Mbps meets connectivity requirement >= ${minBw} Mbps`,
          "pass",
          line,
          column,
        ),
      );
    } else {
      items.push(
        compatItem(
          "connectivity",
          `Connectivity bandwidth requirement ${minBw} Mbps exceeds target ${bw} Mbps`,
          "error",
          line,
          column,
        ),
      );
    }
  }

  if (req.latencyMsMax != null) {
    const maxLat = req.latencyMsMax;
    const lat = profile.networkLatencyMs;
    if (lat == null) {
      items.push(
        compatItem(
          "connectivity",
          "Target latency unknown — cannot verify connectivity latency requirement",
          "warning",
          line,
          column,
        ),
      );
    } else if (lat <= maxLat) {
      items.push(
        compatItem(
          "connectivity",
          `Latency ${lat} ms meets connectivity requirement <= ${maxLat} ms`,
          "pass",
          line,
          column,
        ),
      );
    } else {
      items.push(
        compatItem(
          "connectivity",
          `Connectivity latency requirement ${maxLat} ms exceeded by target ${lat} ms`,
          "error",
          line,
          column,
        ),
      );
    }
  }

  if (req.packetLossPctMax != null) {
    const maxLoss = req.packetLossPctMax;
    const loss = profile.packetLossPct;
    if (loss == null) {
      items.push(
        compatItem(
          "connectivity",
          "Target packet loss unknown — cannot verify packet_loss requirement",
          "warning",
          line,
          column,
        ),
      );
    } else if (loss <= maxLoss) {
      items.push(
        compatItem(
          "connectivity",
          `Packet loss ${loss}% meets requirement <= ${maxLoss}%`,
          "pass",
          line,
          column,
        ),
      );
    } else {
      items.push(
        compatItem(
          "connectivity",
          `Packet loss ${loss}% exceeds requirement <= ${maxLoss}%`,
          "error",
          line,
          column,
        ),
      );
    }
  }

  return items;
}

export function validateGeofence(geofence: GeofenceDecl): CompatItem[] {
  // Description:
  //     ValidateGeofence.
  //
  // Inputs:
  //     geofence: GeofenceDecl
  //         Caller-supplied geofence.
  //
  // Outputs:
  //     result: CompatItem[]
  //         Return value from `validateGeofence`.
  //
  // Example:
  //     const result = validateGeofence(geofence);
  // Description:
  //     ValidateGeofence.
  //
  // Inputs:
  //     geofence: GeofenceDecl
  //         Caller-supplied geofence.
  //
  // Outputs:
  //     result: CompatItem[]
  //         Return value from `validateGeofence`.
  //
  // Example:

  //     const result = validateGeofence(geofence);

  const line = geofence.span.start.line;
  const column = geofence.span.start.column;
  if (geofence.centerLat < -90 || geofence.centerLat > 90) {
    return [
      compatItem(
        "geofence",
        `Geofence '${geofence.name}' center latitude ${geofence.centerLat} out of range [-90, 90]`,
        "error",
        line,
        column,
      ),
    ];
  }
  if (geofence.centerLon < -180 || geofence.centerLon > 180) {
    return [
      compatItem(
        "geofence",
        `Geofence '${geofence.name}' center longitude ${geofence.centerLon} out of range [-180, 180]`,
        "error",
        line,
        column,
      ),
    ];
  }
  if (geofence.radiusM <= 0) {
    return [
      compatItem(
        "geofence",
        `Geofence '${geofence.name}' radius must be positive`,
        "error",
        line,
        column,
      ),
    ];
  }
  return [
    compatItem(
      "geofence",
      `Geofence '${geofence.name}' geometry valid`,
      "pass",
      line,
      column,
    ),
  ];
}

export function validateConnectivityPolicy(policy: ConnectivityPolicyDecl): CompatItem[] {
  // Description:
  //     ValidateConnectivityPolicy.
  //
  // Inputs:
  //     policy: ConnectivityPolicyDecl
  //         Caller-supplied policy.
  //
  // Outputs:
  //     result: CompatItem[]
  //         Return value from `validateConnectivityPolicy`.
  //
  // Example:
  //     const result = validateConnectivityPolicy(policy);
  // Description:
  //     ValidateConnectivityPolicy.
  //
  // Inputs:
  //     policy: ConnectivityPolicyDecl
  //         Caller-supplied policy.
  //
  // Outputs:
  //     result: CompatItem[]
  //         Return value from `validateConnectivityPolicy`.
  //
  // Example:

  //     const result = validateConnectivityPolicy(policy);

  const line = policy.span.start.line;
  const column = policy.span.start.column;
  const items: CompatItem[] = [
    compatItem(
      "connectivity_policy",
      `Connectivity policy '${policy.name}' parsed: preferred=${policy.preferred}, fallback=${policy.fallback}`,
      "pass",
      line,
      column,
    ),
  ];
  if (policy.preferred === policy.fallback) {
    items.push(
      compatItem(
        "connectivity_policy",
        `Policy '${policy.name}' preferred and fallback are the same link`,
        "warning",
        line,
        column,
      ),
    );
  }
  if (policy.emergency && (policy.emergency === policy.preferred || policy.emergency === policy.fallback)) {
    items.push(
      compatItem(
        "connectivity_policy",
        `Policy '${policy.name}' emergency link duplicates preferred or fallback`,
        "warning",
        line,
        column,
      ),
    );
  }
  return items;
}

export type { BleServiceDecl };
