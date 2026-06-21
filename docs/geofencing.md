# Geofencing

Spanda supports WGS84 geofences for safety-critical boundary enforcement, extending the existing local `safety { zone ... }` model with GPS-native regions.

## Declaring a geofence

```spanda
geofence SafeZone {
  center: geo(30.2672, -97.7431);
  radius: 100.0;
}
```

The compiler validates latitude/longitude ranges and positive radius during `spanda verify`.

## Safety triggers

```spanda
on geofence SafeZone exited {
  stop_all_actuators();
  notify_operator();
}
```

Supported phases: `entered`, `exited` (also available as keywords).

## Verification

Combine geofences with runtime `verify` rules and safety stops:

```spanda
verify {
  not robot.exits_geofence("SafeZone");
}
```

> Note: `robot.in_zone()` remains for local metric safety zones inside `safety { }`. Geofences operate in WGS84 coordinates via GPS/GNSS sensors.

## Standard library

Types live in `std.geofence`: `GeoFence`, `GeoPoint`.

See also: [Positioning](positioning.md), [Connectivity](connectivity.md).
