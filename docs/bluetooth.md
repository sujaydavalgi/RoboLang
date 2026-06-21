# Bluetooth and BLE

Spanda models Bluetooth classic and BLE at the language level for sensor pairing and trusted-device policies.

## Robot configuration

```spanda
robot Rover {
  bluetooth {
    scan for devices where name matches /^sensor-/;
    pair trusted_only;
  }
}
```

- **scan** — regex filter on discovered device names
- **pair** — `trusted_only` rejects untrusted BLE devices (security integration)

## BLE services

Declare GATT services for typed BLE interaction:

```spanda
ble_service HeartRateSensor {
  uuid: "180D";
}
```

## Triggers

```spanda
on bluetooth.device_connected { authenticate_device(); }
```

## Types

- `BluetoothConnection` — classic Bluetooth link
- `BleConnection` — BLE link
- Namespace: `std.bluetooth`

## Security

| Capability | Purpose |
|------------|---------|
| `bluetooth.scan` | Device discovery |
| `bluetooth.pair` | Pairing and trust establishment |

Untrusted devices are rejected when `pair trusted_only` is configured.

## Simulation

```spanda
simulate_compatibility {
  fault BluetoothDisconnect;
}
```

See also: [Connectivity](connectivity.md).
