//! Tests for readiness forecasting.

use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_readiness::{
    evaluate_readiness_forecast, ReadinessFactorScore, ReadinessForecastOptions, ReadinessHistory,
    ReadinessHistoryEntry,
};
use std::path::PathBuf;

fn parse_program(source: &str) -> spanda_ast::nodes::Program {
    let tokens = tokenize(source).expect("tokenize");
    parse(tokens).expect("parse")
}

#[test]
fn forecasts_heuristic_degradation_without_history() {
    let source = r#"
hardware RoverV1 { sensors [ GPS, Lidar ]; actuators [ DifferentialDrive ]; }
robot Rover {
  uses hardware RoverV1;
  sensor gps: GPS;
  sensor lidar: Lidar;
  actuator wheels: DifferentialDrive;
  exposes capabilities [ gps_navigation ];
  mission Patrol { requires capabilities [ gps_navigation ]; patrol; }
  safety { max_speed = 1 m/s; stop_if lidar.nearest_distance < 0.5 m; }
  behavior patrol() {}
}
deploy Rover to RoverV1;
"#;
    let program = parse_program(source);
    let report = evaluate_readiness_forecast(
        &program,
        "rover.sd",
        &ReadinessForecastOptions {
            horizons_days: vec![7],
            minimum_score: 80,
            history_path: Some(PathBuf::from("/tmp/spanda-forecast-missing-history.json")),
            target: None,
        },
    );
    assert_eq!(report.program, "rover.sd");
    assert_eq!(report.predictions.len(), 1);
    assert!(report.predictions[0].predicted_score <= report.current_score);
}

#[test]
fn forecasts_with_history_slope() {
    let dir = std::env::temp_dir().join(format!("spanda-forecast-{}", std::process::id()));
    let path = dir.join("history.json");
    let _ = std::fs::remove_dir_all(&dir);
    let history = ReadinessHistory {
        version: 1,
        entries: vec![
            ReadinessHistoryEntry {
                recorded_at: "2026-06-01T00:00:00Z".into(),
                program: "rover.sd".into(),
                mission_ready: false,
                total_score: 90,
                maximum_score: 100,
                factors: vec![ReadinessFactorScore {
                    factor: "hardware".into(),
                    score: 90,
                    weight: 20,
                    weighted: 18.0,
                }],
            },
            ReadinessHistoryEntry {
                recorded_at: "2026-06-08T00:00:00Z".into(),
                program: "rover.sd".into(),
                mission_ready: true,
                total_score: 70,
                maximum_score: 100,
                factors: vec![ReadinessFactorScore {
                    factor: "hardware".into(),
                    score: 70,
                    weight: 20,
                    weighted: 14.0,
                }],
            },
        ],
    };
    spanda_readiness::save_readiness_history(&path, &history).unwrap();

    let source = r#"
hardware RoverV1 { sensors [ GPS ]; actuators [ DifferentialDrive ]; }
robot Rover {
  uses hardware RoverV1;
  sensor gps: GPS;
  actuator wheels: DifferentialDrive;
  exposes capabilities [ gps_navigation ];
  mission Patrol { requires capabilities [ gps_navigation ]; patrol; }
  safety { max_speed = 1 m/s; }
  behavior patrol() {}
}
deploy Rover to RoverV1;
"#;
    let program = parse_program(source);
    let report = evaluate_readiness_forecast(
        &program,
        "rover.sd",
        &ReadinessForecastOptions {
            horizons_days: vec![7],
            minimum_score: 80,
            history_path: Some(path.clone()),
            target: None,
        },
    );
    assert_eq!(report.history_samples, 2);
    assert!(report.degradation_per_day < 0.0);
    let _ = std::fs::remove_dir_all(&dir);
}
