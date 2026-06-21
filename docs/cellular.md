# Cellular (LTE / 4G / 5G)

Spanda models cellular connectivity as part of hardware profiles, requirements, and failover policies.

## Hardware connectivity

```spanda
hardware RoverV2 {
  connectivity [ WiFi6, LTE, FiveG, GPS ];
}
```

Supported profile tokens: `LTE`, `FourG`, `4G`, `FiveG`, `5G`, `Cellular`, `Satellite` (placeholder).

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
| `FiveGHandoff` | Temporary latency spike |
| `NetworkOutage` | Full network loss |

## Security

- `cellular.connect` — cellular data capability (identity placeholder for SIM/eSIM attestation)
- Audit connectivity changes via `audit.write`

See also: [Connectivity](connectivity.md).
