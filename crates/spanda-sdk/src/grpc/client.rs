//! Native tonic gRPC client — optional `grpc` feature on `spanda-sdk`.
//!
use crate::error::{SpandaError, SpandaResult};
use serde_json::Value;
use tonic::transport::Channel;

pub mod spanda_v1 {
    tonic::include_proto!("spanda.v1");
}

use spanda_v1::control_center_client::ControlCenterClient;
use spanda_v1::{EntityBodyRequest, EntityIdRequest, JsonBodyRequest, QueryRequest};

/// Async gRPC client for Control Center (`spanda.v1.ControlCenter`).
pub struct GrpcClient {
    inner: ControlCenterClient<Channel>,
    api_key: Option<String>,
}

impl GrpcClient {
    /// Connect to a gRPC endpoint (for example `http://127.0.0.1:50051`).
    pub async fn connect(endpoint: impl Into<String>) -> SpandaResult<Self> {
        let api_key = std::env::var("SPANDA_API_KEY")
            .ok()
            .filter(|key| !key.is_empty());
        let channel = Channel::from_shared(endpoint.into())
            .map_err(|e| SpandaError::connection(e.to_string()))?
            .connect()
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Ok(Self {
            inner: ControlCenterClient::new(channel),
            api_key,
        })
    }

    fn bearer_metadata(&self) -> Option<tonic::metadata::MetadataValue<tonic::metadata::Ascii>> {
        self.api_key
            .as_ref()
            .and_then(|key| tonic::metadata::MetadataValue::try_from(format!("Bearer {key}")).ok())
    }

    fn with_bearer<T>(&self, mut request: tonic::Request<T>) -> tonic::Request<T> {
        if let Some(value) = self.bearer_metadata() {
            request.metadata_mut().insert("authorization", value);
        }
        request
    }

    /// Blocking connect helper for scripts without an async runtime.
    pub fn connect_blocking(endpoint: impl Into<String>) -> SpandaResult<Self> {
        tokio::runtime::Runtime::new()
            .map_err(|e| SpandaError::connection(e.to_string()))?
            .block_on(Self::connect(endpoint))
    }

    fn parse_json(raw: String) -> SpandaResult<Value> {
        serde_json::from_str(&raw).map_err(|e| SpandaError::validation(e.to_string()))
    }

    fn program_body(file: &str, extra: Value) -> String {
        let mut body = serde_json::json!({ "file": file });
        if let Some(obj) = body.as_object_mut() {
            if let Some(extra_obj) = extra.as_object() {
                for (key, value) in extra_obj {
                    obj.insert(key.clone(), value.clone());
                }
            }
        }
        body.to_string()
    }

    /// Evaluate program readiness via `EvaluateProgramReadiness`.
    pub async fn readiness(&mut self, file: &str) -> SpandaResult<Value> {
        let body = Self::program_body(file, Value::Null);
        let resp = self
            .inner
            .evaluate_program_readiness(JsonBodyRequest { body_json: body })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Evaluate program assurance via `EvaluateProgramAssure`.
    pub async fn assure(&mut self, file: &str) -> SpandaResult<Value> {
        let body = Self::program_body(file, Value::Null);
        let resp = self
            .inner
            .evaluate_program_assure(JsonBodyRequest { body_json: body })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Run program simulation via `RunProgramSimulation`.
    pub async fn run_simulation(&mut self, file: &str, execute: bool) -> SpandaResult<Value> {
        let body = Self::program_body(file, serde_json::json!({ "execute": execute }));
        let resp = self
            .inner
            .run_program_simulation(JsonBodyRequest { body_json: body })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Replay or inspect a mission trace via `ReplayProgram`.
    pub async fn replay(
        &mut self,
        file: &str,
        deterministic: bool,
        playback: bool,
    ) -> SpandaResult<Value> {
        let body = Self::program_body(
            file,
            serde_json::json!({
                "deterministic": deterministic,
                "playback": playback,
            }),
        );
        let resp = self
            .inner
            .replay_program(JsonBodyRequest { body_json: body })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// List unified entities via `ListEntities`.
    pub async fn list_entities(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .list_entities(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Fetch one entity via `GetEntity`.
    pub async fn get_entity(&mut self, entity_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_entity(EntityIdRequest {
                entity_id: entity_id.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Fetch the entity graph via `GetEntityGraph`.
    pub async fn entity_graph(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_entity_graph(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Unified traceability via `GetEntityTraceability`.
    pub async fn entity_traceability(
        &mut self,
        entity_id: Option<&str>,
        capability: Option<&str>,
        device_id: Option<&str>,
    ) -> SpandaResult<Value> {
        let mut parts = Vec::new();
        if let Some(id) = entity_id {
            parts.push(format!("entity_id={id}"));
        }
        if let Some(cap) = capability {
            parts.push(format!("capability={cap}"));
        }
        if let Some(dev) = device_id {
            parts.push(format!("device_id={dev}"));
        }
        let query = parts.join("&");
        let resp = self
            .inner
            .get_entity_traceability(QueryRequest { query })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Query entities via `QueryEntities`.
    pub async fn query_entities(&mut self, body: &Value) -> SpandaResult<Value> {
        let resp = self
            .inner
            .query_entities(JsonBodyRequest {
                body_json: body.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Relationship edges via `GetEntityRelationships`.
    pub async fn entity_relationships(&mut self, entity_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_entity_relationships(EntityIdRequest {
                entity_id: entity_id.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Health snapshot via `GetEntityHealth`.
    pub async fn entity_health(&mut self, entity_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_entity_health(EntityIdRequest {
                entity_id: entity_id.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Readiness snapshot via `GetEntityReadiness`.
    pub async fn entity_readiness(&mut self, entity_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_entity_readiness(EntityIdRequest {
                entity_id: entity_id.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Trust evaluation via `GetEntityTrust`.
    pub async fn entity_trust(&mut self, entity_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_entity_trust(EntityIdRequest {
                entity_id: entity_id.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Unified verification via `VerifyEntity`.
    pub async fn entity_verify(&mut self, entity_id: &str, body: &Value) -> SpandaResult<Value> {
        let resp = self
            .inner
            .verify_entity(EntityBodyRequest {
                entity_id: entity_id.to_string(),
                body_json: body.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Register or update an entity via `RegisterEntity` (Bearer API key required).
    pub async fn register_entity(&mut self, body: &Value) -> SpandaResult<Value> {
        let request = self.with_bearer(tonic::Request::new(JsonBodyRequest {
            body_json: body.to_string(),
        }));
        let resp = self
            .inner
            .register_entity(request)
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Add or remove tags via `TagEntity` (Bearer API key required).
    pub async fn tag_entity(&mut self, entity_id: &str, body: &Value) -> SpandaResult<Value> {
        let request = self.with_bearer(tonic::Request::new(EntityBodyRequest {
            entity_id: entity_id.to_string(),
            body_json: body.to_string(),
        }));
        let resp = self
            .inner
            .tag_entity(request)
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Relate two entities via `RelateEntities` (Bearer API key required).
    pub async fn relate_entities(&mut self, body: &Value) -> SpandaResult<Value> {
        let request = self.with_bearer(tonic::Request::new(JsonBodyRequest {
            body_json: body.to_string(),
        }));
        let resp = self
            .inner
            .relate_entities(request)
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Sync overlay to TOML via `SyncEntities` (Bearer API key required).
    pub async fn sync_entities(&mut self) -> SpandaResult<Value> {
        let request = self.with_bearer(tonic::Request::new(spanda_v1::Empty {}));
        let resp = self
            .inner
            .sync_entities(request)
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Smart Spaces summary via `GetSmartSpacesSummary`.
    pub async fn smart_spaces_summary(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_smart_spaces_summary(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// List facilities via `ListFacilities`.
    pub async fn list_facilities(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .list_facilities(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Facility readiness via `GetFacilityReadiness`.
    pub async fn facility_readiness(&mut self, facility_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_facility_readiness(EntityIdRequest {
                entity_id: facility_id.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Zone occupancy via `GetZoneOccupancy`.
    pub async fn zone_occupancy(&mut self, zone_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_zone_occupancy(EntityIdRequest {
                entity_id: zone_id.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Energy systems via `ListEnergySystems`.
    pub async fn list_energy_systems(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .list_energy_systems(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Emergency status via `GetEmergencyStatus`.
    pub async fn emergency_status(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_emergency_status(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Smart Spaces device inventory via `ListSmartSpacesDevices` (`facility_id=` query).
    pub async fn smart_spaces_devices(&mut self, facility_id: Option<&str>) -> SpandaResult<Value> {
        let query = facility_id
            .map(|id| format!("facility_id={id}"))
            .unwrap_or_default();
        let resp = self
            .inner
            .list_smart_spaces_devices(QueryRequest { query })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Smart Spaces gateway status via `ListSmartSpacesGateways` (`facility_id=` query).
    pub async fn smart_spaces_gateways(
        &mut self,
        facility_id: Option<&str>,
    ) -> SpandaResult<Value> {
        let query = facility_id
            .map(|id| format!("facility_id={id}"))
            .unwrap_or_default();
        let resp = self
            .inner
            .list_smart_spaces_gateways(QueryRequest { query })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Facility health panel via `GetFacilityHealth`.
    pub async fn facility_health(&mut self, facility_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_facility_health(EntityIdRequest {
                entity_id: facility_id.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Facility security panel via `GetFacilitySecurity`.
    pub async fn facility_security(&mut self, facility_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_facility_security(EntityIdRequest {
                entity_id: facility_id.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Facility floor map via `GetFacilityFloorMap`.
    pub async fn facility_floor_map(&mut self, facility_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_facility_floor_map(EntityIdRequest {
                entity_id: facility_id.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Zone environment readings via `GetZoneEnvironment`.
    pub async fn zone_environment(&mut self, zone_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_zone_environment(EntityIdRequest {
                entity_id: zone_id.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Energy system detail via `GetEnergySystem`.
    pub async fn energy_system(&mut self, system_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_energy_system(EntityIdRequest {
                entity_id: system_id.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// List devices via `ListDevices`.
    pub async fn list_devices(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .list_devices(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Readiness trends via `GetAnalyticsReadiness`.
    pub async fn analytics_readiness(&mut self, query: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_analytics_readiness(spanda_v1::QueryRequest {
                query: query.into(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// What-if analysis via `GetAnalyticsWhatIf`.
    pub async fn analytics_what_if(&mut self, query: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_analytics_what_if(spanda_v1::QueryRequest {
                query: query.into(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Mission risk via `GetAnalyticsMissionRisk`.
    pub async fn analytics_mission_risk(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_analytics_mission_risk(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Readiness forecast via `GetAnalyticsReadinessForecast`.
    pub async fn analytics_readiness_forecast(&mut self, query: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_analytics_readiness_forecast(spanda_v1::QueryRequest {
                query: query.into(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Trust graph via `GetAnalyticsTrustGraph`.
    pub async fn analytics_trust_graph(&mut self, query: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_analytics_trust_graph(spanda_v1::QueryRequest {
                query: query.into(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Mission twin via `GetAnalyticsMissionTwin`.
    pub async fn analytics_mission_twin(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_analytics_mission_twin(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Certification pack via `GetAnalyticsCertificationPack`.
    pub async fn analytics_certification_pack(&mut self, query: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_analytics_certification_pack(spanda_v1::QueryRequest {
                query: query.into(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Time travel via `GetAnalyticsTimeTravel`.
    pub async fn analytics_time_travel(&mut self, query: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_analytics_time_travel(spanda_v1::QueryRequest {
                query: query.into(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Human teaming via `GetAnalyticsHumanTeaming`.
    pub async fn analytics_human_teaming(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_analytics_human_teaming(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Governance via `GetAnalyticsGovernance`.
    pub async fn analytics_governance(&mut self, query: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_analytics_governance(spanda_v1::QueryRequest {
                query: query.into(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    fn recovery_body(extra: Value) -> String {
        extra.to_string()
    }

    /// List recovery plans via `ListRecoveryPlans`.
    pub async fn list_recovery_plans(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .list_recovery_plans(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Get recovery history via `GetRecoveryHistory`.
    pub async fn get_recovery_history(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_recovery_history(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Plan recovery via `PlanRecovery`.
    pub async fn plan_recovery(&mut self, body: &Value) -> SpandaResult<Value> {
        let resp = self
            .inner
            .plan_recovery(JsonBodyRequest {
                body_json: Self::recovery_body(body.clone()),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Simulate recovery via `SimulateRecovery`.
    pub async fn simulate_recovery(&mut self, body: &Value) -> SpandaResult<Value> {
        let resp = self
            .inner
            .simulate_recovery(JsonBodyRequest {
                body_json: Self::recovery_body(body.clone()),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Execute recovery via `ExecuteRecovery`.
    pub async fn execute_recovery(&mut self, body: &Value) -> SpandaResult<Value> {
        let resp = self
            .inner
            .execute_recovery(JsonBodyRequest {
                body_json: Self::recovery_body(body.clone()),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Validate recovery via `ValidateRecovery`.
    pub async fn validate_recovery(&mut self, body: &Value) -> SpandaResult<Value> {
        let resp = self
            .inner
            .validate_recovery(JsonBodyRequest {
                body_json: Self::recovery_body(body.clone()),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// List recovery playbooks via `ListRecoveryPlaybooks`.
    pub async fn list_recovery_playbooks(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .list_recovery_playbooks(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Get recovery metrics via `GetRecoveryMetrics`.
    pub async fn get_recovery_metrics(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_recovery_metrics(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Get recovery graph via `GetRecoveryGraph`.
    pub async fn get_recovery_graph(&mut self, entity_id: Option<&str>) -> SpandaResult<Value> {
        let query = entity_id
            .map(|id| format!("entity_id={id}"))
            .unwrap_or_default();
        let resp = self
            .inner
            .get_recovery_graph(QueryRequest { query })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// List recovery policies via `ListRecoveryPolicies`.
    pub async fn list_recovery_policies(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .list_recovery_policies(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Explain recovery via `ExplainRecovery`.
    pub async fn explain_recovery(&mut self, body: &Value) -> SpandaResult<Value> {
        let resp = self
            .inner
            .explain_recovery(JsonBodyRequest {
                body_json: Self::recovery_body(body.clone()),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// List API keys via `ListAdminApiKeys`.
    pub async fn list_admin_api_keys(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .list_admin_api_keys(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Create API key via `CreateAdminApiKey`.
    pub async fn create_admin_api_key(&mut self, body: &Value) -> SpandaResult<Value> {
        let resp = self
            .inner
            .create_admin_api_key(JsonBodyRequest {
                body_json: body.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// List users via `ListAdminUsers`.
    pub async fn list_admin_users(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .list_admin_users(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Get alert channels via `GetAlertChannels`.
    pub async fn get_alert_channels(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_alert_channels(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Update alert channels via `UpdateAlertChannels`.
    pub async fn update_alert_channels(&mut self, body: &Value) -> SpandaResult<Value> {
        let resp = self
            .inner
            .update_alert_channels(JsonBodyRequest {
                body_json: body.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// List operator missions via `ListOperatorMissions`.
    pub async fn list_operator_missions(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .list_operator_missions(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// List program traces via `ListProgramTraces`.
    pub async fn list_program_traces(&mut self, query: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .list_program_traces(QueryRequest {
                query: query.into(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// List Twin Cloud snapshots via `ListTwins`.
    pub async fn list_twins(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .list_twins(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Latest Twin Cloud snapshot via `GetTwin`.
    pub async fn get_twin(&mut self, twin_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_twin(EntityIdRequest {
                entity_id: twin_id.into(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Twin snapshot history via `GetTwinHistory`.
    pub async fn get_twin_history(&mut self, twin_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_twin_history(EntityIdRequest {
                entity_id: twin_id.into(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Sync loaded program twin via `SyncTwin`.
    pub async fn sync_twin(&mut self, query: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .sync_twin(QueryRequest {
                query: query.into(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Push Twin Cloud snapshot via `PushTwinSnapshot`.
    pub async fn push_twin_snapshot(&mut self, twin_id: &str, body: &Value) -> SpandaResult<Value> {
        let resp = self
            .inner
            .push_twin_snapshot(EntityBodyRequest {
                entity_id: twin_id.into(),
                body_json: body.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Import legacy replay JSON via `ImportTwinReplay`.
    pub async fn import_twin_replay(&mut self, body: &Value) -> SpandaResult<Value> {
        let resp = self
            .inner
            .import_twin_replay(JsonBodyRequest {
                body_json: body.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }
}
