# Device Tree

The device tree models physical ownership from fleet down to individual sensors and actuators.

## Hierarchy

```
fleet
  robots
    compute / controllers
      devices (sensors, actuators, accessories, connectivity, safety)
```

## TOML structure

```toml
[fleet]
id = "warehouse-fleet-a"

[[fleet.robots]]
id = "rover-001"
model = "RoverV1"
hardware_profile = "RoverV1"

[fleet.robots.compute]
id = "jetson-001"
type = "JetsonOrin"
serial = "JTN-001"

[[fleet.robots.compute.devices]]
id = "gps-001"
type = "GPS"
provider = "spanda-gps"
port = "/dev/ttyUSB0"
capabilities = ["read_location", "read_heading"]

[[fleet.robots.compute.devices]]
id = "drive-controller-001"
type = "DifferentialDrive"
provider = "spanda-canbus"
bus = "can0"
capabilities = ["move", "stop", "emergency_stop"]
```

Place fleet/device definitions in `spanda.devices.toml` or `spanda.fleet.toml` and reference them from `[config]` in `spanda.toml`.

## Device fields

| Field | Description |
|-------|-------------|
| `id` | Unique device identifier |
| `logical_name` | Program-facing name for logical-to-physical mapping |
| `type` | Device class (GPS, Lidar, Camera, DifferentialDrive, …) |
| `provider` | Spanda provider package name |
| `port` / `network_port` | Serial/USB path or TCP/UDP port |
| `bus` | CAN or other bus identifier |
| `can_id` | CAN frame identifier (e.g. `0x12`) |
| `mount` | Physical mount location |
| `capabilities` | Capability tokens this device exposes |
| `ip` / `ip_address` | IPv4 address for networked devices |
| `mac` / `mac_address` | MAC address |
| `hostname`, `dns_name`, `mdns_name` | Network naming |
| `endpoint` / `endpoint_url` | Service URL (`rtsp://…`, `https://…`) |
| `protocol` | Wire protocol (`rtsp`, `can`, `mqtt`, …) |
| `serial` | Manufacturer serial number |
| `firmware` / `firmware_version` | Firmware metadata |
| `hardware_revision` | Hardware revision string |
| `trusted` | Trust flag for actuator control |
| `trust_level` | `unverified`, `verified`, `trusted`, `restricted` |
| `identity` / `security_identity` | Security identity for networked devices |
| `certificate_fingerprint` | TLS cert fingerprint for remote endpoints |
| `redundant_group` / `failover_priority` | Redundant device failover metadata |
| `robot_id` | Owning robot when declared in flat `[[devices]]` |

## Flat device registry (`[[devices]]`)

Network and bus devices can be declared outside the fleet hierarchy and merged by `id`:

```toml
[[devices]]
id = "camera-front-001"
type = "Camera"
logical_name = "front_camera"
ip = "192.168.1.42"
mac = "AA:BB:CC:DD:EE:FF"
serial = "CAM-12345"
provider = "spanda-vision"
protocol = "rtsp"
endpoint = "rtsp://192.168.1.42/stream"
capabilities = ["capture_image", "stream_video"]
trust_level = "verified"
security_identity = "camera-front-001"
robot_id = "rover-001"
```

Reference via `[config] network_devices = "spanda.network-devices.toml"` in `spanda.toml`. Records merge with fleet nested devices on matching `id`.

## CLI inspection

```bash
spanda device discover [--subnet 192.168.1.0/24]
spanda device inspect camera-front-001
spanda network scan --subnet 192.168.1.0/24
spanda config report --network
spanda device-tree graph
spanda device-tree inspect rover-001
spanda map verify patrol.sd --config spanda.toml
```

## Logical-to-physical mapping

`LogicalPhysicalMap` connects program-level robot/sensor/actuator names to configured physical devices. Sensors are classified by type (GPS, Lidar, Camera, IMU). Actuators include drive units, arms, and motors.

Safety rules require actuators to declare `emergency_stop` in `capabilities` and reject `trusted = false` on actuator devices.

## Hardware profiles

Each robot may set `hardware_profile` (e.g. `RoverV1`, `JetsonOrin`). Validation checks that configured devices match the profile's expected sensors and actuators.
