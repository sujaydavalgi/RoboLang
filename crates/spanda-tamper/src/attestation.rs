//! Optional live hardware attestation via HTTP endpoint.

use serde::{Deserialize, Serialize};

/// Live attestation result from an external verifier or device agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveAttestationResult {
    pub attested: bool,
    pub boot_state: String,
    pub score: u32,
    pub detail: String,
}

/// Query optional live attestation for a secure-boot contract import.
pub fn query_live_attestation(
    contract: &str,
    package: &str,
    program_label: Option<&str>,
) -> Option<LiveAttestationResult> {
    // POST contract metadata to SPANDA_ATTESTATION_ENDPOINT when configured.
    //
    // Parameters:
    // - `contract` — import path (e.g. trust.jetson)
    // - `package` — registry package name
    // - `program_label` — optional program file label
    //
    // Returns:
    // Live attestation result when endpoint responds successfully.
    //
    // Options:
    // `SPANDA_ATTESTATION_ENDPOINT` — HTTP URL accepting attestation JSON.
    //
    // Example:
    // let live = query_live_attestation("trust.jetson", "spanda-trust-jetson", Some("rover.sd"));

    let endpoint = std::env::var("SPANDA_ATTESTATION_ENDPOINT")
        .ok()
        .filter(|value| !value.trim().is_empty())?;
    let body = serde_json::json!({
        "contract": contract,
        "package": package,
        "program": program_label,
    });
    let response = spanda_deploy_http::http_request(
        "POST",
        &endpoint,
        Some(&body.to_string()),
        None,
    )
    .ok()?;
    if !(200..300).contains(&response.status) {
        return None;
    }
    let payload: AttestationResponse = serde_json::from_str(&response.body).ok()?;
    Some(LiveAttestationResult {
        attested: payload.attested,
        boot_state: payload.boot_state,
        score: payload.score.unwrap_or(if payload.attested { 100 } else { 0 }),
        detail: payload.detail.unwrap_or_else(|| {
            if payload.attested {
                "live attestation verified".into()
            } else {
                "live attestation failed".into()
            }
        }),
    })
}

#[derive(Debug, Deserialize)]
struct AttestationResponse {
    attested: bool,
    #[serde(default)]
    boot_state: String,
    #[serde(default)]
    score: Option<u32>,
    #[serde(default)]
    detail: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attestation_response_deserializes() {
        let json = r#"{"attested":true,"boot_state":"verified","score":95,"detail":"tpm ok"}"#;
        let payload: AttestationResponse = serde_json::from_str(json).unwrap();
        assert!(payload.attested);
        assert_eq!(payload.boot_state, "verified");
        assert_eq!(payload.score, Some(95));
    }

    #[test]
    fn query_is_noop_without_endpoint() {
        std::env::remove_var("SPANDA_ATTESTATION_ENDPOINT");
        let result = query_live_attestation("trust.jetson", "spanda-trust-jetson", Some("rover.sd"));
        assert!(result.is_none());
    }
}
