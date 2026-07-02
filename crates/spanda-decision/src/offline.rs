//! Offline decision policy support.

use crate::types::{DecisionLayer, DecisionPolicy};
use serde::{Deserialize, Serialize};
use spanda_ast::assurance_decl::OfflinePolicyDecl;
use spanda_ast::nodes::Program;
use spanda_audit::{sign, verify_signature};

/// Offline operation policy extracted from program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfflinePolicySpec {
    pub name: String,
    pub max_duration_minutes: u32,
    pub allowed_actions: Vec<String>,
    pub forbidden_actions: Vec<String>,
    pub policy_version: String,
    pub signature: Option<String>,
    pub expires_at_ms: Option<f64>,
}

/// Extract offline policies from a program.
pub fn extract_offline_policies(program: &Program) -> Vec<OfflinePolicySpec> {
    let Program::Program {
        offline_policies, ..
    } = program;
    offline_policies
        .iter()
        .map(|decl| {
            let OfflinePolicyDecl::OfflinePolicyDecl {
                name,
                max_duration_minutes,
                allowed_actions,
                forbidden_actions,
                policy_version,
                signature,
                expires_at_ms,
                ..
            } = decl;
            OfflinePolicySpec {
                name: name.clone(),
                max_duration_minutes: *max_duration_minutes,
                allowed_actions: allowed_actions.clone(),
                forbidden_actions: forbidden_actions.clone(),
                policy_version: policy_version.clone().unwrap_or_else(|| "1.0.0".into()),
                signature: signature.clone(),
                expires_at_ms: *expires_at_ms,
            }
        })
        .collect()
}

/// Resolve offline policies from program plus persisted signed cache.
pub fn resolve_offline_policies(program: &Program) -> Vec<OfflinePolicySpec> {
    let policies = extract_offline_policies(program);
    let cache = crate::policy_cache::load_persisted_policy_cache(None);
    crate::policy_cache::merge_offline_policies_with_cache(policies, &cache)
}

/// Canonical signing payload for an offline policy (stable field order).
pub fn offline_policy_signing_payload(spec: &OfflinePolicySpec) -> String {
    serde_json::json!({
        "name": spec.name,
        "max_duration_minutes": spec.max_duration_minutes,
        "allowed_actions": spec.allowed_actions,
        "forbidden_actions": spec.forbidden_actions,
        "policy_version": spec.policy_version,
    })
    .to_string()
}

/// Sign an offline policy with a trust key (for CI, control center, or tests).
pub fn sign_offline_policy(spec: &OfflinePolicySpec, signing_key: &str) -> String {
    sign(&offline_policy_signing_payload(spec), signing_key)
}

/// Verify an offline policy signature against a trust key (hex pubkey or key material).
pub fn verify_offline_policy_signature(spec: &OfflinePolicySpec, trust_key: &str) -> bool {
    let Some(signature) = spec.signature.as_ref().filter(|s| !s.is_empty()) else {
        return false;
    };
    verify_signature(&offline_policy_signing_payload(spec), signature, trust_key)
}

fn require_signed_offline_policies() -> bool {
    std::env::var("SPANDA_DECISION_REQUIRE_SIGNED_OFFLINE_POLICY")
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

fn offline_policy_trust_key() -> Option<String> {
    std::env::var("SPANDA_DECISION_POLICY_TRUST_KEY")
        .ok()
        .filter(|key| !key.is_empty())
}

fn policy_expired(spec: &OfflinePolicySpec) -> bool {
    let Some(expires_at_ms) = spec.expires_at_ms else {
        return false;
    };
    let now_ms = std::env::var("SPANDA_SIM_TIME_MS")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as f64)
                .unwrap_or(0.0)
        });
    now_ms > expires_at_ms
}

/// Validate offline policy trust (signature and optional expiry) before action checks.
pub fn validate_offline_policy_trust(spec: &OfflinePolicySpec) -> Result<(), String> {
    let require = require_signed_offline_policies();
    let has_signature = spec
        .signature
        .as_ref()
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    if !require && !has_signature {
        return Ok(());
    }

    if !has_signature {
        return Err(format!(
            "offline policy '{}' requires a signed policy cache entry",
            spec.name
        ));
    }

    let trust_key = offline_policy_trust_key().ok_or_else(|| {
        "SPANDA_DECISION_POLICY_TRUST_KEY not configured for offline policy verification"
            .to_string()
    })?;

    if !verify_offline_policy_signature(spec, &trust_key) {
        return Err(format!(
            "offline policy '{}' signature verification failed",
            spec.name
        ));
    }

    if policy_expired(spec) {
        return Err(format!("offline policy '{}' has expired", spec.name));
    }

    Ok(())
}

/// Convert offline policy to a decision policy for the local cache.
pub fn offline_to_decision_policy(spec: &OfflinePolicySpec, version: &str) -> DecisionPolicy {
    DecisionPolicy {
        name: spec.name.clone(),
        version: version.into(),
        layer: DecisionLayer::LocalEntity,
        allowed_actions: spec.allowed_actions.clone(),
        forbidden_actions: spec.forbidden_actions.clone(),
        signature: spec.signature.clone(),
        expires_at_ms: spec.expires_at_ms,
    }
}

/// Validate an offline action against policy and elapsed offline minutes.
pub fn validate_offline_action(
    spec: &OfflinePolicySpec,
    action: &str,
    offline_minutes: u32,
) -> Result<(), String> {
    if offline_minutes > spec.max_duration_minutes {
        return Err(format!(
            "offline duration {offline_minutes}m exceeds max {}m for policy '{}'",
            spec.max_duration_minutes, spec.name
        ));
    }
    if spec.forbidden_actions.iter().any(|a| a == action) {
        return Err(format!("action '{action}' forbidden while offline"));
    }
    if !spec.allowed_actions.is_empty() && !spec.allowed_actions.iter().any(|a| a == action) {
        return Err(format!("action '{action}' not in offline allowed list"));
    }
    Ok(())
}
