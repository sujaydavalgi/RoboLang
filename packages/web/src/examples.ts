export const DEFAULT_SOURCE = `robot Rover {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  ai_model planner: LLM {
    provider: "mock";
    model: "safe-planner";
    temperature: 0.1;
  }

  safety {
    max_speed = 1.0 m/s;
    stop_if lidar.nearest_distance < 0.5 m;
  }

  agent Navigator {
    uses planner;
    tools [lidar, wheels];
    memory short_term;
    goal "Reach destination while avoiding obstacles";

    plan {
      let scan = lidar.read();
      let proposal = planner.reason(prompt: "Create a safe navigation action", input: scan);
      let action = safety.validate(proposal);
      wheels.execute(action);
    }
  }

  behavior run() {
    loop every 100ms {
      Navigator.plan();
    }
  }
}`;

export const EXAMPLES = [
  { name: "AI navigation", source: DEFAULT_SOURCE },
  {
    name: "Lidar avoidance",
    source: `robot Avoider {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  safety {
    max_speed = 0.8 m/s;
    stop_if lidar.nearest_distance < 0.4 m;
  }

  behavior avoid() {
    loop every 100ms {
      let d = lidar.read();
      if d.nearest_distance < 1.0 m {
        wheels.drive(linear: 0.0 m/s, angular: 0.5 rad/s);
      } else {
        wheels.drive(linear: 0.4 m/s, angular: 0.0 rad/s);
      }
    }
  }
}`,
  },
];
