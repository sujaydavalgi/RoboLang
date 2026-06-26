//! TLS policy helpers for production discovery transports.
//!
use std::path::Path;

/// TLS requirements for discovery transports in production fleets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryTlsPolicy {
    pub require_tls: bool,
    pub ca_bundle_path: Option<String>,
}

/// Load discovery TLS policy from environment.
pub fn discovery_tls_policy() -> DiscoveryTlsPolicy {
    let require_tls = std::env::var("SPANDA_DISCOVERY_REQUIRE_TLS")
        .ok()
        .is_some_and(|value| {
            value == "1" || value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("yes")
        })
        || std::env::var("SPANDA_PRODUCTION_POLICY")
            .ok()
            .is_some_and(|value| value.eq_ignore_ascii_case("production"));
    let ca_bundle_path = std::env::var("SPANDA_DISCOVERY_TLS_CA_BUNDLE")
        .ok()
        .filter(|value| !value.trim().is_empty());
    DiscoveryTlsPolicy {
        require_tls,
        ca_bundle_path,
    }
}

/// Validate an endpoint URL against discovery TLS policy.
pub fn validate_discovery_endpoint(url: &str, policy: &DiscoveryTlsPolicy) -> Result<(), String> {
    let lower = url.to_ascii_lowercase();
    if policy.require_tls && (lower.starts_with("http://") || lower.starts_with("mqtt://")) {
        return Err(format!(
            "insecure discovery endpoint blocked by SPANDA_DISCOVERY_REQUIRE_TLS: {url}"
        ));
    }
    if let Some(bundle) = &policy.ca_bundle_path {
        if !Path::new(bundle).exists() {
            return Err(format!(
                "SPANDA_DISCOVERY_TLS_CA_BUNDLE not found: {bundle}"
            ));
        }
    }
    Ok(())
}

/// Summary for discovery API responses.
pub fn discovery_tls_summary() -> serde_json::Value {
    let policy = discovery_tls_policy();
    serde_json::json!({
        "require_tls": policy.require_tls,
        "ca_bundle_path": policy.ca_bundle_path,
        "vendor_cert_docs": [
            "spanda-discovery-mdns",
            "spanda-discovery-wifi",
            "spanda-discovery-cellular",
            "spanda-discovery-tls"
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_insecure_http_when_tls_required() {
        let policy = DiscoveryTlsPolicy {
            require_tls: true,
            ca_bundle_path: None,
        };
        assert!(validate_discovery_endpoint("http://robot.local", &policy).is_err());
        assert!(validate_discovery_endpoint("https://robot.local", &policy).is_ok());
    }
}
