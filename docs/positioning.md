# Positioning (GPS / GNSS)

Spanda provides first-class positioning types and sensor support for autonomous navigation.

## Native types

| Type | Purpose |
|------|---------|
| `GpsFix` | GPS fix with lat/lon and motion fields |
| `GnssFix` | Multi-constellation GNSS fix with quality metrics |
| `GeoPoint` | WGS84 coordinate pair |
| `GeoFence` | Circular geofence region |
| `Altitude`, `Heading`, `SpeedOverGround` | Navigation scalars |
| `SatelliteInfo`, `PositionAccuracy`, `NavigationStatus` | Fix metadata |

Create coordinates with the `geo()` builtin:

```spanda
let home: GeoPoint = geo(30.2672, -97.7431);
```

## Sensors

```spanda
sensor gps: GPS on "/gps";
sensor gnss: GNSS on "/gnss";

on gps.fix { update_route(gps); }
on gps.lost { enter degraded_mode; }
on gps.acquired { resume_navigation; }
```

## Standard library

Import types from `std.positioning`:

- `GpsFix`, `GnssFix`, `GeoPoint`, `GeoFence`, `NavigationStatus`, …

## Simulation

Inject positioning faults during compatibility simulation:

```spanda
simulate_compatibility {
  fault GPSLost;
  fault GpsDrift;     // accumulates ~5 cm/s coordinate drift; fires `on gps.drift`
  fault GpsSpoofing;  // offsets lat/lon and lowers fix quality; fires `on gps.spoofed`
}
```

## Security

Grant `gps.read` in robot permissions for signed GPS telemetry access.

See also: [Geofencing](geofencing.md), [Connectivity](connectivity.md).
