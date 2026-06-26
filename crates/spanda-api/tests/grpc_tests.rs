//! gRPC server smoke tests for Control Center.
use spanda_api::grpc::spanda_v1::control_center_client::ControlCenterClient;
use spanda_api::grpc::spanda_v1::{Empty, QueryRequest, ReadinessRequest, TrustPackageRequest};
use spanda_api::{run_control_center_server, ControlCenterOptions};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use tonic::transport::Channel;

fn pick_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral")
        .local_addr()
        .expect("local addr")
        .port()
}

#[tokio::test]
async fn grpc_health_and_dashboard() {
    let http_port = pick_port();
    let grpc_port = pick_port();
    let http_bind = format!("127.0.0.1:{http_port}");
    let grpc_bind = format!("127.0.0.1:{grpc_port}");
    let options = ControlCenterOptions {
        bind: http_bind,
        grpc_bind: Some(grpc_bind.clone()),
        once: true,
        timeout_ms: 500,
        ..Default::default()
    };
    thread::spawn(move || {
        let _ = run_control_center_server(&options);
    });
    thread::sleep(Duration::from_millis(400));
    let mut client = connect(&grpc_bind).await;
    let health = client
        .health(Empty {})
        .await
        .expect("health rpc")
        .into_inner();
    assert_eq!(health.status, "ok");
    let dashboard = client
        .get_dashboard(Empty {})
        .await
        .expect("dashboard rpc")
        .into_inner();
    assert!(dashboard.json.contains("device_pool"));
}

#[tokio::test]
async fn grpc_expanded_endpoints_return_json() {
    let http_port = pick_port();
    let grpc_port = pick_port();
    let http_bind = format!("127.0.0.1:{http_port}");
    let grpc_bind = format!("127.0.0.1:{grpc_port}");
    let options = ControlCenterOptions {
        bind: http_bind,
        grpc_bind: Some(grpc_bind.clone()),
        once: true,
        timeout_ms: 500,
        ..Default::default()
    };
    thread::spawn(move || {
        let _ = run_control_center_server(&options);
    });
    thread::sleep(Duration::from_millis(400));
    let mut client = connect(&grpc_bind).await;

    let devices = client
        .list_devices(Empty {})
        .await
        .expect("devices")
        .into_inner();
    assert!(devices.json.contains("\"devices\""));

    let agents = client
        .list_fleet_agents(Empty {})
        .await
        .expect("agents")
        .into_inner();
    assert!(agents.json.contains("\"agents\""));

    let readiness = client
        .evaluate_readiness(ReadinessRequest {
            body_json: String::new(),
        })
        .await
        .expect("readiness")
        .into_inner();
    assert!(readiness.json.contains("mission_ready"));

    let sre = client
        .get_sre_summary(Empty {})
        .await
        .expect("sre")
        .into_inner();
    assert!(sre.json.contains("availability_percent"));

    let trust = client
        .get_trust_package(TrustPackageRequest {
            package_name: "spanda-mqtt".into(),
        })
        .await
        .expect("trust")
        .into_inner();
    assert!(trust.json.contains("trust"));

    let openapi = client
        .get_open_api(Empty {})
        .await
        .expect("openapi")
        .into_inner();
    assert!(openapi.json.contains("openapi"));

    let health_summary = client
        .get_health_summary(Empty {})
        .await
        .expect("health summary")
        .into_inner();
    assert!(health_summary.json.contains("overall_status"));

    let assurance = client
        .get_assurance_summary(Empty {})
        .await
        .expect("assurance")
        .into_inner();
    assert!(assurance.json.contains("loaded"));

    let diagnosis = client
        .get_diagnosis_summary(Empty {})
        .await
        .expect("diagnosis")
        .into_inner();
    assert!(diagnosis.json.contains("loaded"));

    let ota = client
        .get_ota_status(Empty {})
        .await
        .expect("ota")
        .into_inner();
    assert!(ota.json.contains("version"));

    let metrics = client
        .get_otlp_metrics(Empty {})
        .await
        .expect("otlp metrics")
        .into_inner();
    assert!(metrics.json.contains("resourceMetrics"));

    let discovery = client
        .discover_devices(QueryRequest {
            query: "transport=mdns".into(),
        })
        .await
        .expect("discover devices")
        .into_inner();
    assert!(discovery.json.contains("discovery"));
}

async fn connect(bind: &str) -> ControlCenterClient<Channel> {
    let channel = Channel::from_shared(format!("http://{bind}"))
        .unwrap()
        .connect()
        .await
        .expect("grpc connect");
    ControlCenterClient::new(channel)
}
