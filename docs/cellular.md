# Cellular (LTE / 4G / 5G)

Spanda models cellular connectivity as part of hardware profiles, requirements, and failover policies.

## Hardware connectivity

```spanda
hardware RoverV2 {
  connectivity [ WiFi6, LTE, FiveG, GPS ];
}
```

Supported profile tokens: `LTE`, `FourG`, `4G`, `FiveG`, `5G`, `Cellular`, `Satellite` (maps to websocket transport for emergency backhaul).

## Requirements

```spanda
requires_connectivity {
  cellular: required;
  latency <= 100ms;
}
```

## Types

| Type | Description |
|------|-------------|
| `CellularConnection` | Generic cellular link |
| `LTEConnection`, `FourGConnection`, `FiveGConnection` | Generation-specific |
| `RoamingStatus` | Roaming state |
| `SimIdentity` | SIM/eSIM ICCID and attestation (`iccid`, `carrier`, `esim`, `attested`) |

Namespace: `std.cellular`.

## Triggers and failover

```spanda
on cellular.roaming { reduce_bandwidth_usage(); }

connectivity_policy FleetNet {
  preferred: wifi;
  fallback: cellular;
  switch_if packet_loss > 5%;
}
```

## Simulation

| Fault | Effect |
|-------|--------|
| `LteOutage` | Removes cellular connectivity |
| `SatelliteOutage` | Removes satellite backhaul |
| `FiveGHandoff` | Temporary latency spike |
| `NetworkOutage` | Full network loss |

## Security

- `cellular.connect` — required to read `robot.sim_identity()` under strict permissions
- `robot.sim_identity()` returns `SimIdentity` with deterministic ICCID for simulation attestation
- Audit connectivity changes via `audit.write`

See also: [Connectivity](connectivity.md).
