# Connectivity

Spanda extends the existing `network` and communication systems with first-class wireless connectivity modeling, verification, and failover.

## Connection types

| Type | Description |
|------|-------------|
| `WifiConnection` | Wi-Fi link state |
| `BluetoothConnection` / `BleConnection` | Bluetooth classic and BLE |
| `CellularConnection`, `LTEConnection`, `FourGConnection`, `FiveGConnection` | Cellular links |
| `EthernetConnection`, `MeshConnection` | Wired and mesh backhaul |
| `NetworkStatus`, `SignalStrength`, `Bandwidth`, `Latency`, `PacketLoss`, `RoamingStatus` | Link metrics |

Namespaces: `std.connectivity`, `std.wifi`, `std.cellular`, `std.network`.

## Hardware profiles

Declare available radios on a hardware target:

```spanda
hardware RoverV2 {
  connectivity [
    WiFi6,
    Bluetooth5,
    LTE,
    GPS
  ];
  network { bandwidth: 100 Mbps; latency: 15 ms; }
}
```

`spanda verify` checks that deployment targets expose required connectivity.

## Requirements

Use `requires_connectivity` alongside `requires_network`:

```spanda
requires_connectivity {
  gps: required;
  wifi: optional;
  cellular: required;
  latency <= 100ms;
  bandwidth >= 5 Mbps;
  packet_loss <= 1%;
}
```

## Triggers

```spanda
on network.disconnected { buffer_telemetry(); }
on cellular.roaming { reduce_bandwidth_usage(); }
```

## Failover policies

```spanda
connectivity_policy RoverNetwork {
  preferred: wifi;
  fallback: cellular;
  emergency: bluetooth;
  switch_if latency > 200ms;
  switch_if packet_loss > 5%;
}
```

## Offline and degraded modes

```spanda
mode offline { disable_cloud_ai(); buffer_telemetry(); }
mode weak_signal { use_compressed_telemetry(); }

on network.disconnected { enter offline_mode; }
```

## Simulation faults

| Fault | Effect |
|-------|--------|
| `NetworkOutage`, `LteOutage` | Zero bandwidth, high latency |
| `WeakWifi` | Reduced bandwidth |
| `NetworkLatencySpike`, `LatencySpike` | 2 s latency |
| `FiveGHandoff` | Brief latency increase |
| `BluetoothDisconnect` | Removes BT from profile |
| `PacketLoss` | 10% packet loss |

Timed faults: `fault GPSLost at T+30s;`, `fault NetworkLatencySpike duration 10s;`

## Security capabilities

| Capability | Purpose |
|------------|---------|
| `network.status` | Read link state |
| `wifi.connect` | Join Wi-Fi networks |
| `cellular.connect` | Cellular data (placeholder identity) |
| `network.failover` | Switch preferred link |
| `bluetooth.scan`, `bluetooth.pair` | Bluetooth discovery and pairing |

See also: [Bluetooth](bluetooth.md), [Cellular](cellular.md), [Positioning](positioning.md).
