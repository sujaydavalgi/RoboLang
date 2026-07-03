//! HTTP client for remote Twin Cloud SaaS APIs.

use crate::config::TwinCloudConfig;
use crate::snapshot::{
    TwinCloudListResponse, TwinCloudSnapshot, TwinCloudSyncResponse, TWIN_CLOUD_API_VERSION,
};
use thiserror::Error;

/// Remote Twin Cloud HTTP client.
#[derive(Debug, Clone)]
pub struct TwinCloudClient {
    config: TwinCloudConfig,
}

#[derive(Debug, Error)]
pub enum TwinCloudError {
    #[error("twin cloud request failed: {0}")]
    Request(String),
    #[error("twin cloud response parse failed: {0}")]
    Parse(String),
    #[error("twin cloud not found: {0}")]
    NotFound(String),
}

impl TwinCloudClient {
    pub fn new(config: TwinCloudConfig) -> Self {
        Self { config }
    }

    pub fn from_env() -> Result<Self, TwinCloudError> {
        TwinCloudConfig::from_env()
            .map(Self::new)
            .ok_or_else(|| TwinCloudError::Request("SPANDA_TWIN_CLOUD_URL not set".into()))
    }

    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    pub fn push_snapshot(
        &self,
        snapshot: &TwinCloudSnapshot,
    ) -> Result<TwinCloudSyncResponse, TwinCloudError> {
        let path = format!("/v1/twins/{}/snapshots", snapshot.twin_id);
        let body =
            serde_json::to_value(snapshot).map_err(|err| TwinCloudError::Parse(err.to_string()))?;
        let value: serde_json::Value = self.request_json("POST", &path, Some(body))?;
        if let Ok(response) = serde_json::from_value::<TwinCloudSyncResponse>(value.clone()) {
            return Ok(response);
        }
        Ok(TwinCloudSyncResponse {
            version: TWIN_CLOUD_API_VERSION.into(),
            twin_id: snapshot.twin_id.clone(),
            captured_at_ms: snapshot.captured_at_ms,
            snapshot: snapshot.clone(),
        })
    }

    pub fn latest_snapshot(&self, twin_id: &str) -> Result<TwinCloudSnapshot, TwinCloudError> {
        let path = format!("/v1/twins/{twin_id}");
        let value = self.request_json("GET", &path, None)?;
        if let Ok(snapshot) = serde_json::from_value::<TwinCloudSnapshot>(value.clone()) {
            return Ok(snapshot);
        }
        value
            .get("snapshot")
            .cloned()
            .and_then(|nested| serde_json::from_value(nested).ok())
            .ok_or_else(|| TwinCloudError::Parse("missing snapshot payload".into()))
    }

    pub fn list_twins(&self) -> Result<TwinCloudListResponse, TwinCloudError> {
        let value = self.request_json("GET", "/v1/twins", None)?;
        serde_json::from_value(value).map_err(|err| TwinCloudError::Parse(err.to_string()))
    }

    pub fn sync_program_snapshot(
        &self,
        twin_id: Option<&str>,
    ) -> Result<TwinCloudSyncResponse, TwinCloudError> {
        let mut query = String::new();
        if let Some(id) = twin_id {
            query = format!("?twin_id={id}");
        }
        let path = format!("/v1/twins/sync{query}");
        let value = self.request_json("POST", &path, Some(serde_json::json!({})))?;
        serde_json::from_value(value).map_err(|err| TwinCloudError::Parse(err.to_string()))
    }

    pub fn import_replay(
        &self,
        program: &str,
        twin_id: Option<&str>,
    ) -> Result<serde_json::Value, TwinCloudError> {
        let mut body = serde_json::json!({ "program": program });
        if let Some(id) = twin_id {
            body["twin_id"] = serde_json::Value::String(id.to_string());
        }
        self.request_json("POST", "/v1/twins/import-replay", Some(body))
    }

    pub fn twin_history(&self, twin_id: &str) -> Result<serde_json::Value, TwinCloudError> {
        let path = format!("/v1/twins/{twin_id}/history");
        self.request_json("GET", &path, None)
    }

    fn request_json(
        &self,
        method: &str,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, TwinCloudError> {
        let url = format!("{}{}", self.config.base_url, path);
        let mut request = match method {
            "GET" => ureq::get(&url),
            "POST" => ureq::post(&url),
            other => {
                return Err(TwinCloudError::Request(format!(
                    "unsupported method {other}"
                )))
            }
        };
        request = request.set("Content-Type", "application/json");
        request = request.set("X-Spanda-Api-Version", TWIN_CLOUD_API_VERSION);
        if let Some(key) = &self.config.api_key {
            request = request.set("Authorization", &format!("Bearer {key}"));
        }
        let response = match body {
            Some(payload) => request.send_json(payload),
            None => request.call(),
        }
        .map_err(|err| TwinCloudError::Request(err.to_string()))?;
        let status = response.status();
        let text = response
            .into_string()
            .map_err(|err| TwinCloudError::Parse(err.to_string()))?;
        if status == 404 {
            return Err(TwinCloudError::NotFound(text));
        }
        if !(200..300).contains(&status) {
            return Err(TwinCloudError::Request(format!(
                "{url}: HTTP {status}: {text}"
            )));
        }
        serde_json::from_str(&text).map_err(|err| TwinCloudError::Parse(err.to_string()))
    }
}
