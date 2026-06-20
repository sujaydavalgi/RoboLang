use spanda_core::{check, lint, run, RunOptions};

#[test]
fn agent_mailbox_send_recv_in_plan() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  agent Vision {
    goal "see";
    plan {
      send_agent("Planner", 1);
    }
  }
  agent Planner {
    goal "plan";
    plan {
      let msg = recv_agent();
      let _ = msg;
      wheels.stop();
    }
  }
  Vision -> Planner;
  behavior run() {
    Vision.plan();
    Planner.plan();
  }
}
"#;
    check(source).expect("agent mailbox should type-check");
    run(source, RunOptions::default()).expect("agent mailbox should run");
}

#[test]
fn peer_send_delivers_to_subscriber() {
    let source = r#"
robot FleetBot {
  bus local;
  robot RoverA;
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  behavior run() {
    subscribe RoverA.pose;
    peer_send("RoverA", "pose", pose(x: 1.0 m, y: 2.0 m, theta: 0.0 rad));
    receive RoverA.pose to p;
    wheels.stop();
  }
}
"#;
    check(source).expect("peer_send should type-check");
    run(source, RunOptions::default()).expect("peer_send should run");
}

#[test]
fn runtime_budget_skips_over_budget_task() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  task heavy low every 10ms {
    budget {
      cpu <= 1%;
    }
    wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
  }
  task heavy2 low every 10ms {
    budget {
      cpu <= 1%;
    }
    wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
  }
}
"#;
    let result = run(
        source,
        RunOptions {
            max_loop_iterations: 4,
            trace_tasks: true,
            ..Default::default()
        },
    )
    .expect("budget enforcement should run");
    assert!(
        result.logs.iter().any(|l| l.contains("budget exceeded")),
        "expected budget skip log, got: {:?}",
        result.logs
    );
    assert!(
        result
            .metrics
            .tasks
            .values()
            .any(|t| t.budget_violations > 0),
        "expected budget violation metrics"
    );
}

#[test]
fn lint_warns_recv_without_send() {
    let source = r#"
module m;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let ch = channel();
    select {
      recv(ch) => {
        wheels.stop();
      }
    };
  }
}
"#;
    let report = lint(source).expect("lint should parse");
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule == "channel-recv-without-send"),
        "expected channel flow warning, got: {:?}",
        report.issues
    );
}
