use crate::capability::{capability_for_operation, CapabilitySet};
use crate::error::{SecurityError, SecurityResult};
use crate::identity::RobotIdentity;
use crate::permissions::PackagePermissions;
use crate::secrets::SecretStore;
use crate::secure_comm::{SecureEndpointRegistry, SecurePolicy};
use crate::signed::SignedMessage;
use crate::trust::TrustLevel;
use serde::{Deserialize, Serialize};
use spanda_audit::AuditRuntime;

/// Unified security context for the Spanda interpreter.
#[derive(Debug)]
pub struct SecurityContext {
    pub identity: Option<RobotIdentity>,
    pub trust: TrustLevel,
    pub secrets: SecretStore,
    pub capabilities: CapabilitySet,
    pub secure_endpoints: SecureEndpointRegistry,
    pub audit_security_events: bool,
    /// When true, block-level capability auto-grants are disabled.
    pub strict_permissions: bool,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityContext {
    pub fn new() -> Self {
        Self {
            identity: None,
            trust: TrustLevel::Trusted,
            secrets: SecretStore::new(),
            capabilities: CapabilitySet::new(),
            secure_endpoints: SecureEndpointRegistry::new(),
            audit_security_events: true,
            strict_permissions: false,
        }
    }

    pub fn enable_strict_permissions(&mut self) {
        self.strict_permissions = true;
    }

    pub fn grant_if_not_strict(&mut self, capability: impl Into<String>) {
        if !self.strict_permissions {
            self.capabilities.grant(capability);
        }
    }

    pub fn with_permissions(perms: &PackagePermissions) -> Self {
        Self {
            capabilities: perms.capabilities.clone(),
            ..Self::new()
        }
    }

    pub fn set_identity(&mut self, identity: RobotIdentity) {
        self.trust = identity.trust;
        self.identity = Some(identity);
    }

    pub fn require_operation(&self, operation: &str) -> SecurityResult<()> {
        if let Some(cap) = capability_for_operation(operation) {
            self.capabilities.require(cap)?;
        }
        Ok(())
    }

    pub fn register_secure_endpoint(&mut self, path: impl Into<String>, policy: SecurePolicy) {
        self.secure_endpoints.register(path, policy);
    }

    pub fn sign_outbound(
        &self,
        path: &str,
        payload: &str,
    ) -> SecurityResult<Option<SignedMessage>> {
        let policy = self.secure_endpoints.policy_or_open(path);
        policy.prepare_outbound(
            payload,
            self.identity.as_ref(),
            &self.capabilities,
            path,
        )
    }

    pub fn verify_inbound(
        &self,
        path: &str,
        signed: Option<&SignedMessage>,
    ) -> SecurityResult<()> {
        let policy = self.secure_endpoints.policy_or_open(path);
        policy.verify_inbound(signed, self.identity.as_ref(), &self.capabilities, path)
    }

    /// Record security-relevant events into the audit log when configured.
    pub fn audit_event(
        &self,
        audit: &mut AuditRuntime,
        event_type: &str,
        detail: &str,
    ) -> SecurityResult<()> {
        if !self.audit_security_events {
            return Ok(());
        }
        self.require_operation("audit.record")?;
        let redacted = if detail.contains("secret:") {
            detail.to_string()
        } else {
            detail.to_string()
        };
        audit
            .record_event(&format!("security.{event_type}"), &redacted)
            .map_err(|e| SecurityError::Other(format!("audit failed: {e}")))?;
        Ok(())
    }
}

/// Serializable snapshot of security state for export/debugging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySnapshot {
    pub identity_id: Option<String>,
    pub trust: TrustLevel,
    pub granted_capabilities: Vec<String>,
    pub secret_names: Vec<String>,
    pub secure_endpoint_count: usize,
}

impl SecurityContext {
    pub fn snapshot(&self) -> SecuritySnapshot {
        SecuritySnapshot {
            identity_id: self.identity.as_ref().map(|i| i.id().to_string()),
            trust: self.trust,
            granted_capabilities: self.capabilities.granted().map(str::to_string).collect(),
            secret_names: self.secrets.names().map(str::to_string).collect(),
            secure_endpoint_count: self.secure_endpoints.len(),
        }
    }
}
