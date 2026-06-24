use spanda_telemetry_store::{
    configure_session_persist, env_persist_enabled, persist_enabled, record_health_event,
    resolve_store_path, PersistentTelemetryStore, TelemetryEvent, TelemetryQuery,
};
use tempfile::tempdir;

#[test]
fn append_and_query_device_and_sensor_events() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("telemetry.jsonl");
    let heartbeat_path = dir.path().join("heartbeats.json");
    let mut store = PersistentTelemetryStore::open(store_path.clone(), heartbeat_path);

    store
        .append(TelemetryEvent::Device {
            device_id: "robot-001".into(),
            metric: "battery".into(),
            value: serde_json::json!({"kind":"number","value":82.0}),
            timestamp_ms: 1000.0,
            robot_id: Some("Rover".into()),
        })
        .unwrap();
    store
        .append(TelemetryEvent::Sensor {
            sensor_id: "lidar".into(),
            sensor_type: "Lidar".into(),
            value: serde_json::json!({"kind":"scan","nearest_distance":1.2}),
            timestamp_ms: 1100.0,
            robot_id: Some("Rover".into()),
        })
        .unwrap();

    let events = store
        .query(&TelemetryQuery {
            device_id: Some("robot-001".into()),
            ..TelemetryQuery::default()
        })
        .unwrap();
    assert_eq!(events.len(), 1);

    let latest = store.latest_sensor("lidar").unwrap().unwrap();
    assert!(matches!(latest, TelemetryEvent::Sensor { .. }));
    assert!(store_path.exists());
}

#[test]
fn heartbeat_index_updates_and_history_is_throttled() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("telemetry.jsonl");
    let heartbeat_path = dir.path().join("heartbeats.json");
    let mut store = PersistentTelemetryStore::open(store_path, heartbeat_path.clone());

    store
        .touch_heartbeat("control", 1000.0, 5000.0, Some("Rover"))
        .unwrap();
    store
        .touch_heartbeat("control", 2000.0, 5000.0, Some("Rover"))
        .unwrap();
    store
        .touch_heartbeat("control", 7000.0, 5000.0, Some("Rover"))
        .unwrap();

    let heartbeats: Vec<_> = store
        .read_all()
        .unwrap()
        .into_iter()
        .filter(|event| matches!(event, TelemetryEvent::Heartbeat { .. }))
        .collect();
    assert_eq!(heartbeats.len(), 2);
    assert_eq!(store.heartbeat_index().tasks.get("control"), Some(&7000.0));
    assert!(heartbeat_path.exists());
}

#[test]
fn persist_enabled_respects_session_and_env() {
    configure_session_persist(false);
    std::env::remove_var("SPANDA_TELEMETRY_STORE");
    assert!(!persist_enabled());

    configure_session_persist(true);
    assert!(persist_enabled());

    configure_session_persist(false);
    std::env::set_var("SPANDA_TELEMETRY_STORE", "1");
    assert!(persist_enabled());
    std::env::remove_var("SPANDA_TELEMETRY_STORE");
    assert!(!persist_enabled());
}

#[test]
fn record_health_event_appends_to_store() {
    let dir = tempdir().unwrap();
    std::env::set_var(
        "SPANDA_TELEMETRY_STORE_PATH",
        dir.path().join("telemetry.jsonl").to_string_lossy().to_string(),
    );
    configure_session_persist(true);
    record_health_event("overall", "Degraded", 1500.0).unwrap();
    let store = PersistentTelemetryStore::open(
        resolve_store_path(),
        dir.path().join("heartbeats.json"),
    );
    let events = store.read_all().unwrap();
    assert_eq!(events.len(), 1);
    assert!(matches!(
        &events[0],
        TelemetryEvent::Health {
            target,
            status,
            timestamp_ms,
        } if target == "overall" && status == "Degraded" && *timestamp_ms == 1500.0
    ));
    configure_session_persist(false);
    std::env::remove_var("SPANDA_TELEMETRY_STORE_PATH");
}

#[test]
fn device_heartbeat_updates_index_and_history() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("telemetry.jsonl");
    let heartbeat_path = dir.path().join("heartbeats.json");
    let mut store = PersistentTelemetryStore::open(store_path, heartbeat_path.clone());

    store
        .touch_device_heartbeat("temp-1", 1000.0, 5000.0, None, Some("mqtt"))
        .unwrap();
    store
        .touch_device_heartbeat("temp-1", 2000.0, 5000.0, None, Some("mqtt"))
        .unwrap();
    store
        .touch_device_heartbeat("temp-1", 7000.0, 5000.0, None, Some("mqtt"))
        .unwrap();

    let device_heartbeats: Vec<_> = store
        .read_all()
        .unwrap()
        .into_iter()
        .filter(|event| matches!(event, TelemetryEvent::DeviceHeartbeat { .. }))
        .collect();
    assert_eq!(device_heartbeats.len(), 2);
    assert_eq!(
        store.heartbeat_index().devices.get("temp-1"),
        Some(&7000.0)
    );
}

#[test]
fn is_heartbeat_metric_detects_liveness_names() {
    assert!(spanda_telemetry_store::is_heartbeat_metric("heartbeat"));
    assert!(spanda_telemetry_store::is_heartbeat_metric("Liveness"));
    assert!(!spanda_telemetry_store::is_heartbeat_metric("battery"));
}

#[test]
fn record_topic_publish_stores_device_event() {
    use spanda_runtime::value::RuntimeValue;
    let dir = tempdir().unwrap();
    std::env::set_var(
        "SPANDA_TELEMETRY_STORE_PATH",
        dir.path().join("telemetry.jsonl").to_string_lossy().to_string(),
    );
    configure_session_persist(true);
    spanda_telemetry_store::record_topic_publish(
        Some("Rover"),
        "/telemetry",
        &RuntimeValue::String {
            value: "ok".into(),
        },
        1200.0,
    )
    .unwrap();
    let store = PersistentTelemetryStore::open(
        resolve_store_path(),
        dir.path().join("heartbeats.json"),
    );
    let latest = store.latest_device("Rover", "/telemetry").unwrap().unwrap();
    assert!(matches!(
        latest,
        TelemetryEvent::Device {
            device_id,
            metric,
            robot_id: Some(rid),
            ..
        } if device_id == "Rover" && metric == "/telemetry" && rid == "Rover"
    ));
    configure_session_persist(false);
    std::env::remove_var("SPANDA_TELEMETRY_STORE_PATH");
}

#[test]
fn env_persist_enabled_accepts_true_literal() {
    std::env::set_var("SPANDA_TELEMETRY_STORE", "true");
    assert!(env_persist_enabled());
    std::env::remove_var("SPANDA_TELEMETRY_STORE");
}
