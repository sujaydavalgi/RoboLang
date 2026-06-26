//! Live gRPC probe against a running Control Center (`SPANDA_GRPC_BIND`).
use spanda_api::grpc::spanda_v1::control_center_client::ControlCenterClient;
use spanda_api::grpc::spanda_v1::{
    DriftRequest, Empty, ReadinessRequest, TrustPackageRequest,
};
use tonic::transport::Channel;

async fn connect(bind: &str) -> ControlCenterClient<Channel> {
    let channel = Channel::from_shared(format!("http://{bind}"))
        .expect("grpc url")
        .connect()
        .await
        .expect("grpc connect");
    ControlCenterClient::new(channel)
}

#[tokio::test]
async fn grpc_live_control_center_endpoints() {
    let Some(bind) = std::env::var("SPANDA_GRPC_BIND").ok() else {
        return;
    };
    let mut client = connect(&bind).await;
    let health = client
        .health(Empty {})
        .await
        .expect("health")
        .into_inner();
    assert_eq!(health.status, "ok");

    let devices = client
        .list_devices(Empty {})
        .await
        .expect("list devices")
        .into_inner();
    assert!(devices.json.contains("\"devices\""));

    let agents = client
        .list_fleet_agents(Empty {})
        .await
        .expect("list fleet agents")
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
    assert!(openapi.json.contains("Spanda"));

    if let Ok(baseline_id) = std::env::var("SPANDA_GRPC_BASELINE_ID") {
        let drift = client
            .detect_drift(DriftRequest { baseline_id })
            .await
            .expect("drift")
            .into_inner();
        assert!(drift.json.contains("dimensions_checked"));
    }
}
