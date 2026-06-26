//! API versioning policy for Control Center REST and gRPC.
//!
use spanda_deploy_http::HttpResponse;

pub const SUPPORTED_API_VERSION: &str = "v1";

/// Parse `X-Spanda-Api-Version` from raw HTTP headers.
pub fn api_version_from_headers(raw_headers: &str) -> Option<String> {
    for line in raw_headers.lines() {
        let lower = line.to_ascii_lowercase();
        if let Some(value) = lower.strip_prefix("x-spanda-api-version:") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

/// Reject unsupported explicit API version headers.
pub fn enforce_api_version(version: Option<&str>) -> Option<HttpResponse> {
    let Some(requested) = version else {
        return None;
    };
    if requested == SUPPORTED_API_VERSION {
        return None;
    }
    Some(HttpResponse {
        status: 400,
        body: serde_json::json!({
            "ok": false,
            "error": "unsupported api version",
            "requested": requested,
            "supported": [SUPPORTED_API_VERSION],
            "policy": "Breaking changes ship under a new /v2/ path prefix. Send X-Spanda-Api-Version: v1 or omit the header.",
        })
        .to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_version_header_is_allowed() {
        assert!(enforce_api_version(None).is_none());
    }

    #[test]
    fn v1_version_header_is_allowed() {
        assert!(enforce_api_version(Some("v1")).is_none());
    }

    #[test]
    fn v2_version_header_is_rejected() {
        let response = enforce_api_version(Some("v2")).expect("rejected");
        assert_eq!(response.status, 400);
        assert!(response.body.contains("unsupported api version"));
    }
}
