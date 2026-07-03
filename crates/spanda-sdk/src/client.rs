//! SpandaClient — thin REST v1 client for Control Center.
//!
use crate::error::{SpandaError, SpandaResult};
use crate::types::{
    AssuranceReport, DiagnosisReport, Entity, HealthReport, PackageTrustReport, ReadinessReport,
    RecoveryReport, ReplayResult, SimulationResult, TrustReport,
};
use serde_json::{json, Value};
use std::env;
use std::time::Duration;

/// Authentication configuration for SDK clients.
#[derive(Debug, Clone, Default)]
pub struct AuthConfig {
    pub api_key: Option<String>,
}

/// Builder for [`SpandaClient`].
#[derive(Debug, Clone)]
pub struct SpandaClientBuilder {
    base_url: String,
    auth: AuthConfig,
    timeout: Duration,
}

impl Default for SpandaClientBuilder {
    fn default() -> Self {
        Self {
            base_url: env::var("SPANDA_CONTROL_CENTER_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8080".into()),
            auth: AuthConfig {
                api_key: env::var("SPANDA_API_KEY").ok(),
            },
            timeout: Duration::from_secs(30),
        }
    }
}

impl SpandaClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.auth.api_key = Some(key.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn build(self) -> SpandaClient {
        SpandaClient {
            base_url: self.base_url.trim_end_matches('/').to_string(),
            auth: self.auth,
            timeout: self.timeout,
        }
    }
}

/// Official Spanda SDK client — delegates to Control Center REST API v1.
#[derive(Debug, Clone)]
pub struct SpandaClient {
    base_url: String,
    auth: AuthConfig,
    timeout: Duration,
}

impl SpandaClient {
    /// Connect to the local Control Center (`http://127.0.0.1:8080`).
    pub fn local() -> Self {
        Self::builder().build()
    }

    pub fn builder() -> SpandaClientBuilder {
        SpandaClientBuilder::new()
    }

    pub fn with_url(base_url: impl Into<String>) -> Self {
        Self::builder().base_url(base_url).build()
    }

    fn correlation_id() -> String {
        format!(
            "rust-sdk-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..12]
        )
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
        auth: bool,
    ) -> SpandaResult<Value> {
        let url = format!("{}{}", self.base_url, path);
        let agent = ureq::AgentBuilder::new().timeout(self.timeout).build();
        let mut req = match method {
            "GET" => agent.get(&url),
            "POST" => agent.post(&url),
            "PATCH" => agent.patch(&url),
            "PUT" => agent.put(&url),
            "DELETE" => agent.delete(&url),
            _ => {
                return Err(SpandaError::validation(format!(
                    "unsupported method {method}"
                )))
            }
        };
        req = req.set("Accept", "application/json");
        req = req.set("X-Correlation-ID", &Self::correlation_id());
        if auth {
            if let Some(key) = &self.auth.api_key {
                req = req.set("Authorization", &format!("Bearer {key}"));
            }
        }
        if let Some(payload) = body {
            req = req.set("Content-Type", "application/json");
            let resp = req
                .send_json(payload)
                .map_err(|e| SpandaError::connection(e.to_string()))?;
            return Self::parse_response(resp);
        }
        let resp = req
            .call()
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_response(resp)
    }

    fn parse_response(resp: ureq::Response) -> SpandaResult<Value> {
        let status = resp.status();
        let body: Value = resp.into_json().unwrap_or(json!({}));
        if (200..300).contains(&status) {
            return Ok(body);
        }
        let message = body
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("request failed")
            .to_string();
        Err(SpandaError::from_status(status, message))
    }

    fn program_body(file: &str) -> Value {
        json!({ "file": file })
    }

    fn percent_encode_query(value: &str) -> String {
        let mut out = String::new();
        for byte in value.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(byte as char);
                }
                _ => out.push_str(&format!("%{byte:02X}")),
            }
        }
        out
    }

    /// Evaluate operational readiness for a program file.
    pub fn readiness(&self, file_or_project: &str) -> SpandaResult<ReadinessReport> {
        let body = Self::program_body(file_or_project);
        let value = self.request("POST", "/v1/programs/readiness", Some(&body), false)?;
        Ok(ReadinessReport::from_api(value))
    }

    /// Run mission assurance for a program file.
    pub fn assure(&self, file_or_project: &str) -> SpandaResult<AssuranceReport> {
        let body = Self::program_body(file_or_project);
        let value = self.request("POST", "/v1/programs/assure", Some(&body), false)?;
        Ok(AssuranceReport::from_api(value))
    }

    /// Diagnose a program or `.trace` file.
    pub fn diagnose(&self, trace_or_file: &str) -> SpandaResult<DiagnosisReport> {
        let body = Self::program_body(trace_or_file);
        let value = self.request("POST", "/v1/programs/diagnose", Some(&body), false)?;
        Ok(DiagnosisReport::from_api(value))
    }

    /// Evaluate recovery options for a program.
    pub fn heal(&self, target: &str) -> SpandaResult<RecoveryReport> {
        let body = Self::program_body(target);
        let value = self.request("POST", "/v1/programs/recovery/heal", Some(&body), false)?;
        Ok(RecoveryReport { raw: value })
    }

    /// Plan recovery via the Recovery Orchestrator.
    pub fn plan_recovery(&self, body: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/recovery/plan", Some(body), false)
    }

    /// Simulate recovery without affecting runtime state.
    pub fn simulate_recovery(&self, body: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/recovery/simulate", Some(body), false)
    }

    /// Execute recovery through the orchestrator.
    pub fn execute_recovery(&self, body: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/recovery/execute", Some(body), false)
    }

    /// Validate a recovery plan (dry-run with gates).
    pub fn validate_recovery(&self, body: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/recovery/validate", Some(body), false)
    }

    /// List recovery policies from config.
    pub fn list_recovery_policies(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/recovery/policies", None, false)
    }

    /// List recovery playbooks.
    pub fn list_recovery_playbooks(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/recovery/playbooks", None, false)
    }

    /// Get recovery history evidence records.
    pub fn get_recovery_history(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/recovery/history", None, false)
    }

    /// Get aggregated recovery metrics.
    pub fn get_recovery_metrics(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/recovery/metrics", None, false)
    }

    /// Get recovery graph (optional entity_id query param).
    pub fn get_recovery_graph(&self, entity_id: Option<&str>) -> SpandaResult<Value> {
        let path = match entity_id {
            Some(id) => format!(
                "/v1/recovery/graph?entity_id={}",
                Self::percent_encode_query(id)
            ),
            None => "/v1/recovery/graph".into(),
        };
        self.request("GET", &path, None, false)
    }

    /// Explain recovery decision for entity failure.
    pub fn explain_recovery(&self, body: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/recovery/explain", Some(body), false)
    }

    /// List active recovery plans.
    pub fn list_recovery_plans(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/recovery/plans", None, false)
    }

    /// List API keys (administrator only).
    pub fn list_admin_api_keys(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/admin/api-keys", None, true)
    }

    /// Create an API key (administrator only); token returned once.
    pub fn create_admin_api_key(&self, body: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/admin/api-keys", Some(body), true)
    }

    /// Update API key metadata (administrator only).
    pub fn patch_admin_api_key(&self, key_id: &str, body: &Value) -> SpandaResult<Value> {
        self.request(
            "PATCH",
            &format!(
                "/v1/admin/api-keys/{}",
                Self::encode_query_component(key_id)
            ),
            Some(body),
            true,
        )
    }

    /// Revoke an API key (administrator only).
    pub fn delete_admin_api_key(&self, key_id: &str) -> SpandaResult<Value> {
        self.request(
            "DELETE",
            &format!(
                "/v1/admin/api-keys/{}",
                Self::encode_query_component(key_id)
            ),
            None,
            true,
        )
    }

    /// List Control Center users (administrator only).
    pub fn list_admin_users(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/admin/users", None, true)
    }

    /// Create a Control Center user (administrator only).
    pub fn create_admin_user(&self, body: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/admin/users", Some(body), true)
    }

    /// Update a Control Center user (administrator only).
    pub fn patch_admin_user(&self, user_id: &str, body: &Value) -> SpandaResult<Value> {
        self.request(
            "PATCH",
            &format!(
                "/v1/admin/users/{}",
                Self::encode_query_component(user_id)
            ),
            Some(body),
            true,
        )
    }

    /// Delete a Control Center user (administrator only).
    pub fn delete_admin_user(&self, user_id: &str) -> SpandaResult<Value> {
        self.request(
            "DELETE",
            &format!(
                "/v1/admin/users/{}",
                Self::encode_query_component(user_id)
            ),
            None,
            true,
        )
    }

    /// Admin integrations summary (administrator only).
    pub fn get_admin_integrations(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/admin/integrations", None, true)
    }

    /// Get alert channel configuration (administrator only).
    pub fn get_alert_channels(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/admin/alert-channels", None, true)
    }

    /// Replace alert channel configuration (administrator only).
    pub fn update_alert_channels(&self, body: &Value) -> SpandaResult<Value> {
        self.request("PUT", "/v1/admin/alert-channels", Some(body), true)
    }

    /// List missions with runtime state.
    pub fn list_operator_missions(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/operator/missions", None, false)
    }

    /// Pause a running mission.
    pub fn operator_mission_pause(&self, body: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/operator/mission/pause", Some(body), true)
    }

    /// Resume a paused mission.
    pub fn operator_mission_resume(&self, body: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/operator/mission/resume", Some(body), true)
    }

    /// Cancel a mission.
    pub fn operator_mission_cancel(&self, body: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/operator/mission/cancel", Some(body), true)
    }

    /// List mission trace files discovered in the project tree.
    pub fn list_program_traces(&self, limit: Option<u32>) -> SpandaResult<Value> {
        let path = match limit {
            Some(n) => format!("/v1/programs/traces?limit={n}"),
            None => "/v1/programs/traces".into(),
        };
        self.request("GET", &path, None, false)
    }

    /// Verify hardware compatibility for a program.
    pub fn verify_hardware(&self, project: &str) -> SpandaResult<Value> {
        let body = Self::program_body(project);
        self.request("POST", "/v1/programs/verify/hardware", Some(&body), false)
    }

    /// Verify robot capabilities for a program.
    pub fn verify_capabilities(&self, project: &str) -> SpandaResult<Value> {
        let body = json!({ "file": project, "traceability": true });
        self.request(
            "POST",
            "/v1/programs/verify/capabilities",
            Some(&body),
            false,
        )
    }

    /// List unified entities (all platform objects).
    pub fn list_entities(&self) -> SpandaResult<Vec<Entity>> {
        let value = self.request("GET", "/v1/entities", None, false)?;
        Self::parse_entity_list(value)
    }

    /// Query entities with filter parameters.
    pub fn query_entities(&self, query: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/entities/query", Some(query), false)
    }

    /// Fetch the full entity graph for traversal and visualization.
    pub fn entity_graph(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/entities/graph", None, false)
    }

    /// Unified traceability across entity registry, program graph, and digital thread.
    pub fn entity_traceability(
        &self,
        entity_id: Option<&str>,
        capability: Option<&str>,
        device_id: Option<&str>,
    ) -> SpandaResult<Value> {
        let mut query = String::from("/v1/entities/traceability");
        let mut parts = Vec::new();
        if let Some(id) = entity_id {
            parts.push(format!("entity_id={}", Self::percent_encode_query(id)));
        }
        if let Some(cap) = capability {
            parts.push(format!("capability={}", Self::percent_encode_query(cap)));
        }
        if let Some(dev) = device_id {
            parts.push(format!("device_id={}", Self::percent_encode_query(dev)));
        }
        if !parts.is_empty() {
            query.push('?');
            query.push_str(&parts.join("&"));
        }
        self.request("GET", &query, None, false)
    }

    /// Get a single entity by id.
    pub fn get_entity(&self, id: &str) -> SpandaResult<Entity> {
        let value = self.request("GET", &format!("/v1/entities/{id}"), None, false)?;
        let raw = value.get("entity").cloned().unwrap_or(value);
        Ok(Self::parse_entity(raw, id))
    }

    /// Relationship edges, impact set, and dependency chain for an entity.
    pub fn entity_relationships(&self, id: &str) -> SpandaResult<Value> {
        self.request(
            "GET",
            &format!("/v1/entities/{id}/relationships"),
            None,
            false,
        )
    }

    /// Health snapshot for any entity kind.
    pub fn entity_health(&self, id: &str) -> SpandaResult<HealthReport> {
        let value = self.request("GET", &format!("/v1/entities/{id}/health"), None, false)?;
        Ok(HealthReport { raw: value })
    }

    /// Readiness snapshot for any entity kind.
    pub fn entity_readiness(&self, id: &str) -> SpandaResult<Value> {
        self.request("GET", &format!("/v1/entities/{id}/readiness"), None, false)
    }

    /// Trust evaluation for any entity kind.
    pub fn entity_trust(&self, id: &str) -> SpandaResult<TrustReport> {
        let value = self.request("GET", &format!("/v1/entities/{id}/trust"), None, false)?;
        Ok(TrustReport { raw: value })
    }

    /// Distributed decision report for an entity.
    pub fn get_entity_decisions(&self, id: &str) -> SpandaResult<Value> {
        self.request("GET", &format!("/v1/entities/{id}/decisions"), None, false)
    }

    /// List decision architecture (authorities, trees, offline policies).
    pub fn list_decisions(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/decisions", None, false)
    }

    /// Simulate distributed decisions under failure scenarios.
    pub fn simulate_decision(&self, body: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/decisions/simulate", Some(body), false)
    }

    /// Approve a pending decision escalation.
    pub fn approve_escalation(&self, body: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/decisions/escalate", Some(body), true)
    }

    /// List decision policies from the loaded program.
    pub fn list_decision_policies(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/decision-policies", None, false)
    }

    /// List v3 decision trace frames from a mission trace file.
    pub fn list_decision_traces(
        &self,
        file: Option<&str>,
        trace: Option<&str>,
    ) -> SpandaResult<Value> {
        let mut query = Vec::new();
        if let Some(f) = file {
            query.push(format!("file={f}"));
        }
        if let Some(t) = trace {
            query.push(format!("trace={t}"));
        }
        let path = if query.is_empty() {
            "/v1/decisions/traces".to_string()
        } else {
            format!("/v1/decisions/traces?{}", query.join("&"))
        };
        self.request("GET", &path, None, false)
    }

    /// List persisted signed offline policy cache entries on disk.
    pub fn list_decision_policy_cache(&self, cache: Option<&str>) -> SpandaResult<Value> {
        let path = if let Some(c) = cache {
            format!("/v1/decision-policy-cache?cache={c}")
        } else {
            "/v1/decision-policy-cache".to_string()
        };
        self.request("GET", &path, None, false)
    }

    /// Unified verification for any entity kind (hardware, mission, fleet, device pool).
    pub fn entity_verify(&self, id: &str, body: Option<&Value>) -> SpandaResult<Value> {
        self.request("POST", &format!("/v1/entities/{id}/verify"), body, false)
    }

    /// Register or update an entity in the runtime mutation overlay.
    pub fn register_entity(&self, body: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/entities/register", Some(body), true)
    }

    /// Add or remove tags on an entity overlay record.
    pub fn tag_entity(&self, id: &str, body: &Value) -> SpandaResult<Value> {
        self.request("POST", &format!("/v1/entities/{id}/tags"), Some(body), true)
    }

    /// Relate two entities in the mutation overlay.
    pub fn relate_entities(&self, body: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/entities/relationships", Some(body), true)
    }

    /// Sync mutation overlay entities back to TOML fragments.
    pub fn sync_entities(&self) -> SpandaResult<Value> {
        self.request("POST", "/v1/entities/sync", None, true)
    }

    fn parse_entity_list(value: Value) -> SpandaResult<Vec<Entity>> {
        let entities = value
            .get("entities")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(entities
            .into_iter()
            .filter_map(|raw| {
                let id = raw.get("id")?.as_str()?.to_string();
                Some(Self::parse_entity(raw, &id))
            })
            .collect())
    }

    fn parse_entity(raw: Value, fallback_id: &str) -> Entity {
        let entity_id = raw
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or(fallback_id)
            .to_string();
        Entity {
            id: entity_id,
            kind: raw.get("kind").and_then(|v| v.as_str()).map(String::from),
            entity_type: raw
                .get("entity_type")
                .and_then(|v| v.as_str())
                .map(String::from),
            display_name: raw
                .get("display_name")
                .and_then(|v| v.as_str())
                .map(String::from),
            health_status: raw
                .get("health_status")
                .and_then(|v| v.as_str())
                .map(String::from),
            readiness_status: raw
                .get("readiness_status")
                .and_then(|v| v.as_str())
                .map(String::from),
            trust_status: raw
                .get("trust_status")
                .and_then(|v| v.as_str())
                .map(String::from),
            lifecycle_state: raw
                .get("lifecycle_state")
                .and_then(|v| v.as_str())
                .map(String::from),
            raw,
        }
    }

    /// List devices in the device pool.
    pub fn list_devices(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/devices", None, true)
    }

    /// Smart Spaces blueprint summary.
    pub fn smart_spaces_summary(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/smart-spaces/summary", None, false)
    }

    /// List facilities, gateways, and zones.
    pub fn list_facilities(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/facilities", None, false)
    }

    /// Facility readiness rollup with weighted factor chart.
    pub fn facility_readiness(&self, facility_id: &str) -> SpandaResult<Value> {
        self.request(
            "GET",
            &format!("/v1/facilities/{facility_id}/readiness"),
            None,
            false,
        )
    }

    /// Zone occupancy twin snapshot.
    pub fn zone_occupancy(&self, zone_id: &str) -> SpandaResult<Value> {
        self.request(
            "GET",
            &format!("/v1/zones/{zone_id}/occupancy"),
            None,
            false,
        )
    }

    /// Energy systems inventory.
    pub fn list_energy_systems(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/energy/systems", None, false)
    }

    /// Emergency and continuity status.
    pub fn emergency_status(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/emergency/status", None, false)
    }

    /// Smart Spaces device inventory.
    pub fn smart_spaces_devices(&self, facility_id: Option<&str>) -> SpandaResult<Value> {
        let path = match facility_id {
            Some(id) => format!("/v1/smart-spaces/devices?facility_id={id}"),
            None => "/v1/smart-spaces/devices".into(),
        };
        self.request("GET", &path, None, false)
    }

    /// Facility device pool health.
    pub fn facility_health(&self, facility_id: &str) -> SpandaResult<Value> {
        self.request(
            "GET",
            &format!("/v1/facilities/{facility_id}/health"),
            None,
            false,
        )
    }

    /// Facility security status.
    pub fn facility_security(&self, facility_id: &str) -> SpandaResult<Value> {
        self.request(
            "GET",
            &format!("/v1/facilities/{facility_id}/security"),
            None,
            false,
        )
    }

    /// Zone environmental readings.
    pub fn zone_environment(&self, zone_id: &str) -> SpandaResult<Value> {
        self.request(
            "GET",
            &format!("/v1/zones/{zone_id}/environment"),
            None,
            false,
        )
    }

    /// Energy system detail.
    pub fn energy_system(&self, system_id: &str) -> SpandaResult<Value> {
        self.request(
            "GET",
            &format!("/v1/energy/systems/{system_id}"),
            None,
            false,
        )
    }

    /// Facility floor map zone tree.
    pub fn facility_floor_map(&self, facility_id: &str) -> SpandaResult<Value> {
        self.request(
            "GET",
            &format!("/v1/facilities/{facility_id}/floor-map"),
            None,
            false,
        )
    }

    /// Provision a device (requires auth).
    pub fn provision_device(&self, device_id: &str, body: &Value) -> SpandaResult<Value> {
        self.request(
            "POST",
            &format!("/v1/devices/{device_id}/provision"),
            Some(body),
            true,
        )
    }

    /// Plan or execute simulation for a program (`execute: true` runs the driver).
    pub fn run_simulation(&self, project: &str, execute: bool) -> SpandaResult<SimulationResult> {
        let body = json!({ "file": project, "execute": execute });
        let value = self.request("POST", "/v1/programs/simulation", Some(&body), false)?;
        Ok(SimulationResult { raw: value })
    }

    /// Load or verify mission trace replay (`deterministic` / `playback` flags).
    pub fn replay_with_options(
        &self,
        trace: &str,
        deterministic: bool,
        playback: bool,
    ) -> SpandaResult<ReplayResult> {
        let body = json!({
            "file": trace,
            "deterministic": deterministic,
            "playback": playback,
        });
        let value = self.request("POST", "/v1/programs/replay", Some(&body), false)?;
        Ok(ReplayResult { raw: value })
    }

    /// Load mission trace replay metadata (inspect only).
    pub fn replay(&self, trace: &str) -> SpandaResult<ReplayResult> {
        self.replay_with_options(trace, false, false)
    }

    /// Get health for an entity.
    pub fn get_health(&self, entity_id: &str) -> SpandaResult<HealthReport> {
        let value = self.request(
            "GET",
            &format!("/v1/entities/{entity_id}/health"),
            None,
            false,
        )?;
        Ok(HealthReport { raw: value })
    }

    /// Get trust metadata for an entity.
    pub fn get_trust(&self, entity_id: &str) -> SpandaResult<TrustReport> {
        let value = self.request(
            "GET",
            &format!("/v1/entities/{entity_id}/trust"),
            None,
            false,
        )?;
        Ok(TrustReport { raw: value })
    }

    /// Evaluate package trust.
    pub fn get_package_trust(
        &self,
        package: &str,
        version: Option<&str>,
    ) -> SpandaResult<PackageTrustReport> {
        let mut path = format!("/v1/trust/package?name={package}");
        if let Some(v) = version {
            path.push_str(&format!("&version={v}"));
        }
        let value = self.request("GET", &path, None, false)?;
        Ok(PackageTrustReport { raw: value })
    }

    /// Service liveness check.
    pub fn health_check(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/health", None, false)
    }

    /// Readiness trends for the loaded Control Center program (`GET /v1/analytics/readiness`).
    pub fn analytics_readiness(&self, query: Option<&str>) -> SpandaResult<Value> {
        let path = match query {
            Some(q) if !q.is_empty() => format!("/v1/analytics/readiness?{q}"),
            _ => "/v1/analytics/readiness".into(),
        };
        self.request("GET", &path, None, false)
    }

    /// What-if failure scenario analysis (`GET /v1/analytics/what-if`).
    pub fn analytics_what_if(&self, scenario: Option<&str>, all: bool) -> SpandaResult<Value> {
        let path = Self::analytics_path(
            "/v1/analytics/what-if",
            scenario.map(|s| ("scenario", s)),
            all,
        );
        self.request("GET", &path, None, false)
    }

    /// Mission deployment risk score (`GET /v1/analytics/mission-risk`).
    pub fn analytics_mission_risk(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/analytics/mission-risk", None, false)
    }

    /// Readiness degradation forecast (`GET /v1/analytics/readiness-forecast`).
    pub fn analytics_readiness_forecast(
        &self,
        horizon: Option<&str>,
        all: bool,
    ) -> SpandaResult<Value> {
        let path = Self::analytics_path(
            "/v1/analytics/readiness-forecast",
            horizon.map(|h| ("horizon", h)),
            all,
        );
        self.request("GET", &path, None, false)
    }

    /// Trust-weighted dependency graph (`GET /v1/analytics/trust-graph`).
    pub fn analytics_trust_graph(&self, format: Option<&str>) -> SpandaResult<Value> {
        let path = match format {
            Some(value) if !value.is_empty() => {
                format!("/v1/analytics/trust-graph?format={value}")
            }
            _ => "/v1/analytics/trust-graph".into(),
        };
        self.request("GET", &path, None, false)
    }

    /// Digital mission twin report (`GET /v1/analytics/mission-twin`).
    pub fn analytics_mission_twin(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/analytics/mission-twin", None, false)
    }

    /// Certification evidence pack (`GET /v1/analytics/certification-pack`).
    pub fn analytics_certification_pack(&self, strict: bool) -> SpandaResult<Value> {
        let path = if strict {
            "/v1/analytics/certification-pack?strict=1".to_string()
        } else {
            "/v1/analytics/certification-pack".to_string()
        };
        self.request("GET", &path, None, false)
    }

    /// Mission time travel trace inspection (`GET /v1/analytics/time-travel`).
    pub fn analytics_time_travel(
        &self,
        at: &str,
        inspect: Option<&str>,
        trace: Option<&str>,
    ) -> SpandaResult<Value> {
        let mut params = vec![("at", at)];
        if let Some(value) = inspect.filter(|s| !s.is_empty()) {
            params.push(("inspect", value));
        }
        if let Some(value) = trace.filter(|s| !s.is_empty()) {
            params.push(("trace", value));
        }
        let path = Self::analytics_query_path("/v1/analytics/time-travel", &params);
        self.request("GET", &path, None, false)
    }

    /// Human/robot teaming verification (`GET /v1/analytics/human-teaming`).
    pub fn analytics_human_teaming(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/analytics/human-teaming", None, false)
    }

    /// Autonomous governance policy evaluation (`GET /v1/analytics/governance`).
    pub fn analytics_governance(&self, policy: Option<&str>) -> SpandaResult<Value> {
        let path = match policy.filter(|s| !s.is_empty()) {
            Some(name) => {
                Self::analytics_query_path("/v1/analytics/governance", &[("policy", name)])
            }
            None => "/v1/analytics/governance".to_string(),
        };
        self.request("GET", &path, None, false)
    }

    /// List Twin Cloud mission twin snapshots (`GET /v1/twins`).
    pub fn list_twins(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/twins", None, false)
    }

    /// Fetch latest Twin Cloud snapshot (`GET /v1/twins/{id}`).
    pub fn get_twin(&self, twin_id: &str) -> SpandaResult<Value> {
        self.request(
            "GET",
            &format!("/v1/twins/{}", Self::encode_query_component(twin_id)),
            None,
            false,
        )
    }

    /// Sync mission twin from loaded program (`POST /v1/twins/sync`).
    pub fn sync_twin(&self, twin_id: Option<&str>) -> SpandaResult<Value> {
        let path = match twin_id.filter(|value| !value.is_empty()) {
            Some(id) => format!(
                "/v1/twins/sync?twin_id={}",
                Self::encode_query_component(id)
            ),
            None => "/v1/twins/sync".to_string(),
        };
        self.request("POST", &path, Some(&serde_json::json!({})), true)
    }

    /// Push a Twin Cloud snapshot envelope (`POST /v1/twins/{id}/snapshots`).
    pub fn push_twin_snapshot(&self, twin_id: &str, snapshot: &Value) -> SpandaResult<Value> {
        self.request(
            "POST",
            &format!(
                "/v1/twins/{}/snapshots",
                Self::encode_query_component(twin_id)
            ),
            Some(snapshot),
            true,
        )
    }

    /// Twin snapshot history (`GET /v1/twins/{id}/history`).
    pub fn get_twin_history(&self, twin_id: &str) -> SpandaResult<Value> {
        self.request(
            "GET",
            &format!(
                "/v1/twins/{}/history",
                Self::encode_query_component(twin_id)
            ),
            None,
            false,
        )
    }

    /// Import legacy replay JSON (`POST /v1/twins/import-replay`).
    pub fn import_twin_replay(&self, program: &str, twin_id: Option<&str>) -> SpandaResult<Value> {
        let mut body = serde_json::json!({ "program": program });
        if let Some(id) = twin_id.filter(|value| !value.is_empty()) {
            body["twin_id"] = serde_json::Value::String(id.to_string());
        }
        self.request("POST", "/v1/twins/import-replay", Some(&body), true)
    }

    fn analytics_path(base: &str, named: Option<(&str, &str)>, all: bool) -> String {
        let mut params = Vec::new();
        if all {
            params.push("all=1".to_string());
        }
        if let Some((key, value)) = named {
            params.push(format!("{key}={value}"));
        }
        if params.is_empty() {
            base.to_string()
        } else {
            format!("{base}?{}", params.join("&"))
        }
    }

    fn analytics_query_path(base: &str, params: &[(&str, &str)]) -> String {
        if params.is_empty() {
            return base.to_string();
        }
        let query: Vec<String> = params
            .iter()
            .map(|(key, value)| {
                format!(
                    "{}={}",
                    Self::encode_query_component(key),
                    Self::encode_query_component(value)
                )
            })
            .collect();
        format!("{base}?{}", query.join("&"))
    }

    fn encode_query_component(raw: &str) -> String {
        raw.chars()
            .map(|ch| match ch {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => ch.to_string(),
                ' ' => "+".into(),
                _ => format!("%{:02X}", ch as u32),
            })
            .collect()
    }

    /// Call JSON-RPC gateway (`POST /v1/rpc`) with a gRPC-style method name.
    pub fn rpc(&self, method: &str, params: Option<&Value>) -> SpandaResult<Value> {
        let body = json!({
            "method": method,
            "params": params.unwrap_or(&json!({})),
        });
        self.request("POST", "/v1/rpc", Some(&body), false)
            .and_then(|value| {
                value
                    .get("result")
                    .cloned()
                    .ok_or_else(|| SpandaError::validation("rpc response missing result"))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_client_uses_default_url() {
        let client = SpandaClient::local();
        assert!(client.base_url.contains("127.0.0.1"));
    }

    #[test]
    fn program_body_includes_file() {
        let body = SpandaClient::program_body("rover.sd");
        assert_eq!(body["file"], "rover.sd");
    }

    #[test]
    fn analytics_what_if_path_includes_query() {
        let path = SpandaClient::analytics_path(
            "/v1/analytics/what-if",
            Some(("scenario", "gps_failure")),
            true,
        );
        assert_eq!(path, "/v1/analytics/what-if?all=1&scenario=gps_failure");
    }

    #[test]
    fn analytics_time_travel_path_encodes_timestamp() {
        let path = SpandaClient::analytics_query_path(
            "/v1/analytics/time-travel",
            &[("at", "T+00:01"), ("inspect", "decisions")],
        );
        assert_eq!(
            path,
            "/v1/analytics/time-travel?at=T%2B00%3A01&inspect=decisions"
        );
    }
}
