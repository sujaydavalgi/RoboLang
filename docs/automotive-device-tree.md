# Automotive Device Tree

Device hierarchy and capability mapping for the ADAS Solution Blueprint.

**Fixture:** `examples/solutions/adas/spanda.devices.toml` · **CLI:** `spanda device-tree inspect vehicle-001 --config spanda.toml`

---

## Hierarchy

```
Vehicle (vehicle-001)
├── Compute Platform (compute-001, JetsonOrin)
│   ├── Front Camera          → lane_detection, traffic_sign_recognition, pedestrian_detection
│   ├── Rear Camera           → obstacle_detection, parking_assist
│   ├── Stereo Camera         → obstacle_detection, parking_assist
│   ├── Front Radar           → obstacle_detection, adaptive_speed_control
│   ├── Front LiDAR           → obstacle_detection, localization
│   ├── Ultrasonic Array      → parking_assist
│   ├── GPS Receiver          → localization, route_following
│   ├── IMU                   → localization
│   └── Driver Monitor Camera → driver_monitoring
├── Steering ECU              → steering_control
├── Brake ECU                 → emergency_braking
├── Powertrain ECU            → adaptive_speed_control
└── Communication Gateway     → secure_communication, v2x_relay
```

---

## Capability mapping

| Logical capability | Primary device | Redundant device |
|--------------------|----------------|------------------|
| `lane_detection` | Front camera | Stereo camera |
| `obstacle_detection` | Front radar | Front LiDAR, front camera |
| `emergency_braking` | Brake ECU | — |
| `adaptive_speed_control` | Front radar + powertrain ECU | Front LiDAR |
| `steering_control` | Steering ECU | Differential drive |
| `localization` | GPS + IMU | Front LiDAR |
| `route_following` | GPS | IMU + visual odometry |
| `driver_monitoring` | Driver monitor camera | — |
| `parking_assist` | Ultrasonic array | Stereo camera, rear camera |

---

## Device types

| Type | Spanda sensor/actuator | Provider |
|------|------------------------|----------|
| Camera | `Camera` | `spanda-opencv` |
| DepthCamera | `DepthCamera` | `spanda-opencv` |
| Radar | `Radar` | `spanda-radar` |
| Lidar | `Lidar` | `spanda-lidar` |
| Ultrasonic | `Ultrasonic` | `spanda-ultrasonic` |
| GPS | `GPS` | `spanda-gps` |
| IMU | `IMU` | `spanda-imu` |
| SteeringController | `SteeringController` | `spanda-canbus` |
| BrakeController | `BrakeController` | `spanda-canbus` |
| PowertrainController | `PowertrainController` | `spanda-canbus` |
| CommunicationGateway | — | `spanda-automotive-ethernet` |

Wheel speed, steering angle, brake, and tire pressure sensors attach to ECUs via CAN and surface as health-check inputs.

---

## CLI

```bash
spanda device-tree inspect vehicle-001 --config examples/solutions/adas/spanda.toml
spanda device-tree graph --config examples/solutions/adas/spanda.toml --json
```

Control Center: `GET /v1/device-tree` with `--config spanda.toml`.

---

## Application variants

Adjust the device tree per application without changing core types:

| Application | Add / remove devices |
|-------------|---------------------|
| Passenger vehicle | Full suite (default fixture) |
| Commercial truck | Long-range radar, additional rear radar |
| Mining vehicle | Ruggedized LiDAR, redundant GPS |
| Agricultural | RTK GPS, remove highway radar |
| Delivery | Reduce LiDAR, add rear ultrasonic |
| Airport ground | Remove driver monitor, add geofence provider |
| Campus shuttle | Full pedestrian detection suite |
| Construction | Remove ACC, add 360° obstacle detection |

Copy `spanda.devices.toml` and edit `[[fleet.robots.compute.devices]]` entries for each deployment.

---

## Related

- [device-tree.md](./device-tree.md) — General device tree reference
- [solutions/adas.md](./solutions/adas.md) — ADAS blueprint architecture
- [configuration.md](./configuration.md) — Cascading config layers
