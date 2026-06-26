//! Native gRPC server (tonic) for Control Center CLI parity.
//!
use crate::state::SharedState;
use tonic::{transport::Server, Request, Response, Status};

pub mod spanda_v1 {
    tonic::include_proto!("spanda.v1");
}

use spanda_v1::control_center_server::{ControlCenter, ControlCenterServer};
use spanda_v1::{
    DriftRequest, Empty, HealthResponse, JsonResponse, ReadinessRequest, TrustPackageRequest,
};

struct GrpcControlCenter {
    state: SharedState,
}

impl GrpcControlCenter {
    fn with_state<F>(&self, f: F) -> Result<JsonResponse, Status>
    where
        F: FnOnce(&crate::state::ControlCenterState) -> String,
    {
        let guard = self.state.lock().map_err(|e| Status::internal(e.to_string()))?;
        let json = f(&guard);
        Ok(JsonResponse { json })
    }
}

#[tonic::async_trait]
impl ControlCenter for GrpcControlCenter {
    async fn health(&self, _request: Request<Empty>) -> Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse {
            status: "ok".into(),
        }))
    }

    async fn get_dashboard(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.with_state(|state| {
            let registry = state.device_registry();
            let fleet =
                spanda_fleet::load_fleet_agent_registry(&spanda_fleet::default_fleet_agents_path());
            let json = serde_json::json!({
                "version": "v1",
                "device_pool": registry.pool_summary(),
                "fleet_agent_count": fleet.agents.len(),
                "alert_count": state.alert_store.list().len(),
            });
            serde_json::to_string(&json).unwrap_or_else(|_| "{}".into())
        })
        .map(Response::new)
    }

    async fn list_devices(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.with_state(|state| crate::handlers::devices_list_json(state))
            .map(Response::new)
    }

    async fn list_fleet_agents(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        Ok(Response::new(JsonResponse {
            json: crate::handlers::fleet_agents_json(),
        }))
    }

    async fn evaluate_readiness(
        &self,
        request: Request<ReadinessRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let body = request.into_inner().body_json;
        self.with_state(|state| crate::handlers::readiness_run_json(state, &body))
            .map(Response::new)
    }

    async fn get_sre_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.with_state(|state| crate::handlers::sre_summary_json(state))
            .map(Response::new)
    }

    async fn get_trust_package(
        &self,
        request: Request<TrustPackageRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let package_name = request.into_inner().package_name;
        let query = format!("name={package_name}");
        Ok(Response::new(JsonResponse {
            json: crate::handlers::trust_package_json(&query),
        }))
    }

    async fn get_open_api(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        Ok(Response::new(JsonResponse {
            json: crate::handlers::openapi_json(),
        }))
    }

    async fn detect_drift(
        &self,
        request: Request<DriftRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let baseline_id = request.into_inner().baseline_id;
        self.with_state(|state| {
            let query = format!("baseline_id={baseline_id}");
            crate::e3::drift_report(state, &query).body
        })
        .map(Response::new)
    }
}

/// Start tonic gRPC server on `bind` (blocks the current thread's tokio runtime).
pub async fn serve_grpc(bind: String, state: SharedState) -> Result<(), String> {
    // Serve ControlCenter gRPC on a dedicated listener.
    //
    // Parameters:
    // - `bind` — socket address (for example `127.0.0.1:50051`)
    // - `state` — shared Control Center state
    //
    // Returns:
    // Ok when the server stops, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // serve_grpc("127.0.0.1:50051".into(), state).await?;

    let service = GrpcControlCenter { state };
    Server::builder()
        .add_service(ControlCenterServer::new(service))
        .serve(bind.parse().map_err(|e: std::net::AddrParseError| e.to_string())?)
        .await
        .map_err(|e| e.to_string())
}

/// Spawn gRPC server on a background thread with its own tokio runtime.
pub fn spawn_grpc_server(bind: String, state: SharedState) {
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("grpc tokio runtime");
        if let Err(error) = runtime.block_on(serve_grpc(bind.clone(), state)) {
            eprintln!("gRPC server on {bind} stopped: {error}");
        }
    });
}
