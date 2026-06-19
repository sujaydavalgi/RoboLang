# RoboLang

A safe, readable, strongly typed programming language for robot control, sensors, actuators, motion planning, automation, and simulation.

RoboLang is designed for robotics engineers and students who want **deterministic**, **safety-first** robot programs that run in simulation first and can connect to hardware later via a ROS2 adapter.

## Features

- **Strong typing with physical units** — `m`, `s`, `ms`, `rad`, `m/s`, `rad/s`
- **Robot-centric syntax** — sensors, actuators, safety blocks, and behaviors
- **Deterministic loop scheduling** — `loop every 50ms { ... }`
- **Safety rules** — always evaluated before motion commands
- **Simulation backend** — test without hardware
- **ROS2 adapter interface** — stub ready for future integration

## Quick Start

```bash
npm install
npm test
npm run robolang -- run examples/lidar_avoidance.rl
npm run robolang -- sim examples/differential_drive.rl
```

## Language Overview

```robolang
robot Rover {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  safety {
    max_speed = 1.5 m/s;
    stop_if lidar.read().nearest_distance < 0.5 m;
  }

  behavior avoid_obstacles() {
    loop every 50ms {
      let scan = lidar.read();

      if scan.nearest_distance < 0.5 m {
        wheels.stop();
      } else {
        wheels.drive(linear: 0.8 m/s, angular: 0.2 rad/s);
      }
    }
  }
}
```

## CLI

| Command | Description |
|---------|-------------|
| `robolang run <file.rl>` | Run with simulated backend |
| `robolang sim <file.rl>` | Run simulation with detailed output |
| `robolang check <file.rl>` | Type-check only |

## Project Structure

```
src/
  lexer/       Tokenizer
  parser/      Recursive descent parser
  ast/         AST node definitions
  types/       Type checker with unit validation
  runtime/     Tree-walking interpreter
  simulator/   Physics-lite simulation backend
  safety/      Safety rule evaluation
  ros2/        ROS2 adapter stub (future hardware)
  cli/         Command-line interface
examples/      Sample RoboLang programs
tests/         Lexer, parser, type, safety, interpreter, simulator tests
```

## Core Concepts

| Concept | Description |
|---------|-------------|
| `robot` | Top-level container for a robot definition |
| `sensor` | Input device (Lidar, IMU, GPS, AltitudeSensor, …) |
| `actuator` | Output device (DifferentialDrive, RoboticArm, DroneRotors, …) |
| `safety` | Rules enforced before every motion command |
| `behavior` | Named control loop or task |
| `loop every Nms` | Deterministic periodic execution |

## Examples

- `examples/hello_robot.rl` — minimal robot
- `examples/differential_drive.rl` — wheeled robot motion
- `examples/lidar_avoidance.rl` — obstacle avoidance with safety
- `examples/robotic_arm_pick_place.rl` — arm pick-and-place sequence
- `examples/drone_altitude_hold.rl` — altitude control loop

## Safety Model

Safety rules in the `safety { }` block are evaluated **before every motion command**:

1. **`max_speed = X m/s`** — clamps drive velocity
2. **`stop_if <condition>`** — triggers emergency stop when true

When a safety rule blocks motion, the actuator receives a `stop()` command and the simulation enters emergency-stop state.

## ROS2 Integration (Future)

The `src/ros2/` module defines a `Ros2Adapter` interface mapping RoboLang concepts to ROS2 nodes, topics, services, and actions. The current implementation is a stub for development.

## License

Apache-2.0 — see [LICENSE](LICENSE).
