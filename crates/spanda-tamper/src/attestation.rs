//! Optional live hardware attestation via HTTP endpoint or TPM backend.

use crate::remote_attestation::{attestation_trust_store_dir, validate_ak_cert_chain};
use crate::tpm::query_tpm_attestation;
use serde::{Deserialize, Serialize};

/// Live attestation result from an external verifier or device agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveAttestationResult {
    pub attested: bool,
    pub boot_state: String,
    pub score: u32,
    pub detail: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ak_chain_verified: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ak_chain_detail: Option<String>,
}

/// Apply AK certificate chain validation policy to a live attestation result.
pub fn apply_ak_chain_policy(
    mut result: LiveAttestationResult,
    ak_cert_chain: Vec<String>,
) -> LiveAttestationResult {
    // Merge remote AK chain validation into a live attestation result.
    //
    // Parameters:
    // - `result` — base attestation outcome
    // - `ak_cert_chain` — optional PEM chain from verifier or TPM vendor SDK
    //
    // Returns:
    // Updated result with `ak_chain_verified` and adjusted pass/fail posture.
    //
    // Options:
    // `SPANDA_ATTESTATION_AK_CHAIN_OPTIONAL` — do not fail attestation when chain invalid.
    //
    // Example:
    // let live = apply_ak_chain_policy(base, chain);

    if ak_cert_chain.is_empty() {
        return result;
    }

    let validation =
        validate_ak_cert_chain(&ak_cert_chain, attestation_trust_store_dir().as_deref());
    result.ak_chain_verified = Some(validation.verified);
    result.ak_chain_detail = Some(validation.detail.clone());

    let optional = std::env::var("SPANDA_ATTESTATION_AK_CHAIN_OPTIONAL")
        .ok()
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if !validation.verified && !optional {
        result.attested = false;
        result.boot_state = "failed".into();
        result.score = result.score.min(40);
        result.detail = format!("{}; ak chain: {}", result.detail, validation.detail);
    } else if validation.verified {
        result.detail = format!("{}; ak chain: {}", result.detail, validation.detail);
        result.score = result.score.max(90);
    }
    result
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
    // `SPANDA_TPM_BACKEND` — optional TPM stub (`mock`, `jetson`, `pi`, `vendor`, `tpm2`, `file`, `script`).
    // `SPANDA_ATTESTATION_TRUST_STORE` — trusted CA PEM directory for remote AK chain validation.
    //
    // Example:
    // let live = query_live_attestation("trust.jetson", "spanda-trust-jetson", Some("rover.sd"));

    query_http_attestation(contract, package, program_label)
        .or_else(|| query_tpm_attestation(contract, package, program_label))
}

fn query_http_attestation(
    contract: &str,
    package: &str,
    program_label: Option<&str>,
) -> Option<LiveAttestationResult> {
    let endpoint = std::env::var("SPANDA_ATTESTATION_ENDPOINT")
        .ok()
        .filter(|value| !value.trim().is_empty())?;
    let body = serde_json::json!({
        "contract": contract,
        "package": package,
        "program": program_label,
    });
    #[cfg(not(feature = "http"))]
    {
        let _ = body;
        return None;
    }
    #[cfg(feature = "http")]
    {
        let response =
            spanda_deploy_http::http_request("POST", &endpoint, Some(&body.to_string()), None)
                .ok()?;
        if !(200..300).contains(&response.status) {
            return None;
        }
        let payload: AttestationResponse = serde_json::from_str(&response.body).ok()?;
        Some(parse_attestation_response(payload))
    }
}

fn parse_attestation_response(payload: AttestationResponse) -> LiveAttestationResult {
    let base = LiveAttestationResult {
        attested: payload.attested,
        boot_state: if payload.boot_state.is_empty() {
            if payload.attested {
                "verified".into()
            } else {
                "unknown".into()
            }
        } else {
            payload.boot_state
        },
        score: payload
            .score
            .unwrap_or(if payload.attested { 100 } else { 0 }),
        detail: payload.detail.unwrap_or_else(|| {
            if payload.attested {
                "live attestation verified".into()
            } else {
                "live attestation failed".into()
            }
        }),
        ak_chain_verified: None,
        ak_chain_detail: None,
    };
    apply_ak_chain_policy(base, payload.ak_cert_chain)
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
    #[serde(default)]
    ak_cert_chain: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attestation_sync::attestation_env_lock;

    const SAMPLE_PEM: &str = "-----BEGIN CERTIFICATE-----\nQUJDRA==\n-----END CERTIFICATE-----";

    #[test]
    fn attestation_response_deserializes() {
        let json = r#"{"attested":true,"boot_state":"verified","score":95,"detail":"tpm ok"}"#;
        let payload: AttestationResponse = serde_json::from_str(json).unwrap();
        assert!(payload.attested);
        assert_eq!(payload.boot_state, "verified");
        assert_eq!(payload.score, Some(95));
    }

    #[test]
    fn query_is_noop_without_backend() {
        std::env::remove_var("SPANDA_ATTESTATION_ENDPOINT");
        std::env::remove_var("SPANDA_TPM_BACKEND");
        let result =
            query_live_attestation("trust.jetson", "spanda-trust-jetson", Some("rover.sd"));
        assert!(result.is_none());
    }

    #[test]
    fn ak_chain_policy_marks_failed_without_trust_store() {
        let _guard = attestation_env_lock();
        std::env::remove_var("SPANDA_ATTESTATION_AK_CHAIN_OPTIONAL");
        std::env::remove_var("SPANDA_ATTESTATION_TRUST_STORE");
        std::env::set_var("SPANDA_ATTESTATION_OPENSSL_VERIFY", "0");
        std::env::set_var(
            "SPANDA_ATTESTATION_AK_EXPECT_FINGERPRINT",
            "00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00",
        );
        let base = LiveAttestationResult {
            attested: true,
            boot_state: "verified".into(),
            score: 95,
            detail: "remote attestation ok".into(),
            ak_chain_verified: None,
            ak_chain_detail: None,
        };
        let result = apply_ak_chain_policy(base, vec![SAMPLE_PEM.into()]);
        assert!(!result.attested);
        assert_eq!(result.ak_chain_verified, Some(false));
        std::env::remove_var("SPANDA_ATTESTATION_AK_EXPECT_FINGERPRINT");
        std::env::remove_var("SPANDA_ATTESTATION_OPENSSL_VERIFY");
    }
}
