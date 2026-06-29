/**
 * Hardware profile type definitions shared across verify and runtime layers.
 * @module
 */

export type HardwareProfile = {
  name: string;
  cpu: string | null;
  memoryMb: number | null;
  storageMb: number | null;
  gpuTops: number | null;
  gpuRequired: boolean;
  sensors: string[];
  actuators: string[];
  connectivity: string[];
  batteryWh: number | null;
  networkBandwidthMbps: number | null;
  networkLatencyMs: number | null;
  packetLossPct: number | null;
  minControlPeriodMs: number;
  powerDrawW: number;
};
