//! Pluggable signing backends for Ed25519 decision and policy signatures.

use crate::crypto::sign as software_sign;
use std::sync::OnceLock;

/// Supported signing backend kinds selected via `SPANDA_CRYPTO_BACKEND`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigningBackendKind {
    Software,
    MockHsm,
}

/// Trait for signing UTF-8 payloads with a key reference (material or HSM key id).
pub trait SigningBackend: Send + Sync {
    fn sign_utf8(&self, data: &str, key_ref: &str) -> Result<String, String>;
    fn backend_label(&self) -> &'static str;
}

struct SoftwareSigningBackend;

impl SigningBackend for SoftwareSigningBackend {
    fn sign_utf8(&self, data: &str, key_ref: &str) -> Result<String, String> {
        Ok(software_sign(data, key_ref))
    }

    fn backend_label(&self) -> &'static str {
        "software"
    }
}

/// Mock HSM backend: key material stays in env; signing uses software crypto under a key id gate.
struct MockHsmSigningBackend {
    key_id: String,
    key_material: String,
}

impl MockHsmSigningBackend {
    fn from_env() -> Result<Self, String> {
        let key_id = std::env::var("SPANDA_DECISION_SIGNING_KEY_ID")
            .map_err(|_| "SPANDA_DECISION_SIGNING_KEY_ID required for mock_hsm backend".to_string())?;
        let key_material = std::env::var("SPANDA_DECISION_POLICY_SIGNING_KEY")
            .or_else(|_| std::env::var("SPANDA_POLICY_SIGNING_KEY"))
            .map_err(|_| {
                "SPANDA_DECISION_POLICY_SIGNING_KEY required for mock_hsm backend".to_string()
            })?;
        Ok(Self {
            key_id,
            key_material,
        })
    }
}

impl SigningBackend for MockHsmSigningBackend {
    fn sign_utf8(&self, data: &str, key_ref: &str) -> Result<String, String> {
        if key_ref != self.key_id && key_ref != self.key_material.as_str() {
            return Err(format!(
                "mock HSM key id mismatch: expected '{}', got '{}'",
                self.key_id, key_ref
            ));
        }
        Ok(software_sign(data, &self.key_material))
    }

    fn backend_label(&self) -> &'static str {
        "mock_hsm"
    }
}

fn signing_backend_kind_from_env() -> SigningBackendKind {
    match std::env::var("SPANDA_CRYPTO_BACKEND")
        .unwrap_or_else(|_| "software".into())
        .to_ascii_lowercase()
        .as_str()
    {
        "hsm" | "mock_hsm" | "pkcs11" => SigningBackendKind::MockHsm,
        _ => SigningBackendKind::Software,
    }
}

fn active_backend() -> &'static dyn SigningBackend {
    static BACKEND: OnceLock<Box<dyn SigningBackend>> = OnceLock::new();
    BACKEND.get_or_init(|| match signing_backend_kind_from_env() {
        SigningBackendKind::MockHsm => match MockHsmSigningBackend::from_env() {
            Ok(backend) => Box::new(backend),
            Err(error) => {
                eprintln!("mock HSM backend init failed ({error}); falling back to software");
                Box::new(SoftwareSigningBackend)
            }
        },
        SigningBackendKind::Software => Box::new(SoftwareSigningBackend),
    })
    .as_ref()
}

/// Resolve the signing key reference from env or caller material.
pub fn resolve_signing_key_ref(fallback_material: &str) -> String {
    std::env::var("SPANDA_DECISION_SIGNING_KEY_ID").unwrap_or_else(|_| fallback_material.to_string())
}

/// Sign UTF-8 data using the active backend (`SPANDA_CRYPTO_BACKEND`).
pub fn sign_with_backend(data: &str, key_material: &str) -> String {
    let key_ref = resolve_signing_key_ref(key_material);
    active_backend()
        .sign_utf8(data, &key_ref)
        .or_else(|_| active_backend().sign_utf8(data, key_material))
        .unwrap_or_else(|_| software_sign(data, key_material))
}

/// Label of the active signing backend (for traces and diagnostics).
pub fn active_signing_backend_label() -> &'static str {
    active_backend().backend_label()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::verify_signature;

    #[test]
    fn software_backend_roundtrip() {
        let sig = sign_with_backend("payload", "test-signing-material");
        assert!(verify_signature("payload", &sig, "test-signing-material"));
    }
}
