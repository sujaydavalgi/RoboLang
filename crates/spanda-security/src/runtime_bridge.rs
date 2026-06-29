//! Security-backed implementation of the runtime security boundary.
//!
use spanda_audit::AuditRuntime;
use spanda_runtime::security_runtime::SecurityRuntime;
use spanda_runtime::security_types::{
    AuthenticationMode, EncryptionMode, IntegrityMode, RobotIdentity, SecretHandle, SecretSource,
    SecurePolicy, TrustBoundaryKind, TrustLevel,
};

use crate::error::SecurityError;
use crate::identity::RobotIdentity as SecurityRobotIdentity;
use crate::policy::{
    AuthenticationMode as SecAuth, EncryptionMode as SecEnc, IntegrityMode as SecInt,
};
use crate::runtime::SecurityContext;
use crate::secrets::{SecretHandle as SecSecretHandle, SecretSource as SecSecretSource};
use crate::secure_comm::SecurePolicy as SecSecurePolicy;
use crate::signed::SignedMessage;
use crate::trust::TrustLevel as SecTrustLevel;
use crate::trust_boundary::TrustBoundaryKind as SecTrustBoundaryKind;

/// Full security runtime delegating to `spanda-security::SecurityContext`.
pub struct SecurityBackedRuntime {
    inner: SecurityContext,
}

impl Default for SecurityBackedRuntime {
    fn default() -> Self {
        Self {
            inner: SecurityContext::new(),
        }
    }
}

impl SecurityBackedRuntime {
    pub fn new() -> Self {
        Self::default()
    }
}

fn map_trust(level: TrustLevel) -> SecTrustLevel {
    match level {
        TrustLevel::Untrusted => SecTrustLevel::Untrusted,
        TrustLevel::Restricted => SecTrustLevel::Restricted,
        TrustLevel::Trusted => SecTrustLevel::Trusted,
        TrustLevel::Certified => SecTrustLevel::Certified,
    }
}

fn map_trust_back(level: SecTrustLevel) -> TrustLevel {
    match level {
        SecTrustLevel::Untrusted => TrustLevel::Untrusted,
        SecTrustLevel::Restricted => TrustLevel::Restricted,
        SecTrustLevel::Trusted => TrustLevel::Trusted,
        SecTrustLevel::Certified => TrustLevel::Certified,
    }
}

fn map_enc(mode: EncryptionMode) -> SecEnc {
    match mode {
        EncryptionMode::None => SecEnc::None,
        EncryptionMode::Optional => SecEnc::Optional,
        EncryptionMode::Required => SecEnc::Required,
    }
}

fn map_auth(mode: AuthenticationMode) -> SecAuth {
    match mode {
        AuthenticationMode::None => SecAuth::None,
        AuthenticationMode::Signed => SecAuth::Signed,
        AuthenticationMode::Mutual => SecAuth::Mutual,
    }
}

fn map_int(mode: IntegrityMode) -> SecInt {
    match mode {
        IntegrityMode::None => SecInt::None,
        IntegrityMode::Required => SecInt::Required,
    }
}

fn map_boundary(boundary: TrustBoundaryKind) -> SecTrustBoundaryKind {
    match boundary {
        TrustBoundaryKind::RobotInternal => SecTrustBoundaryKind::RobotInternal,
        TrustBoundaryKind::RobotToRobot => SecTrustBoundaryKind::RobotToRobot,
        TrustBoundaryKind::RobotToCloud => SecTrustBoundaryKind::RobotToCloud,
        TrustBoundaryKind::OperatorToRobot => SecTrustBoundaryKind::OperatorToRobot,
    }
}

fn map_boundary_back(boundary: SecTrustBoundaryKind) -> TrustBoundaryKind {
    match boundary {
        SecTrustBoundaryKind::RobotInternal => TrustBoundaryKind::RobotInternal,
        SecTrustBoundaryKind::RobotToRobot => TrustBoundaryKind::RobotToRobot,
        SecTrustBoundaryKind::RobotToCloud => TrustBoundaryKind::RobotToCloud,
        SecTrustBoundaryKind::OperatorToRobot => TrustBoundaryKind::OperatorToRobot,
    }
}

fn map_policy(policy: &SecurePolicy) -> SecSecurePolicy {
    SecSecurePolicy {
        signed: policy.signed,
        min_trust: policy.min_trust.map(map_trust),
        requires: policy.requires.clone(),
        encryption: map_enc(policy.encryption),
        authentication: map_auth(policy.authentication),
        integrity: map_int(policy.integrity),
        trusted_sources: policy.trusted_sources.clone(),
        reject_untrusted: policy.reject_untrusted,
    }
}

fn map_secret(handle: SecretHandle) -> SecSecretHandle {
    let source = match handle.source {
        SecretSource::Env { var } => SecSecretSource::Env { var },
        SecretSource::File { path } => SecSecretSource::File { path },
        SecretSource::Literal { value } => SecSecretSource::Literal { value },
    };
    SecSecretHandle {
        name: handle.name,
        source,
    }
}

fn map_identity(identity: RobotIdentity) -> SecurityRobotIdentity {
    SecurityRobotIdentity::new(identity.id, identity.public_key).with_trust(map_trust(identity.trust))
}

fn security_err(err: SecurityError) -> String {
    err.to_string()
}

impl SecurityRuntime for SecurityBackedRuntime {
    fn enable_strict_permissions(&mut self) {
        self.inner.enable_strict_permissions();
    }

    fn inject_security_fault(&mut self, fault: &str) {
        self.inner.inject_security_fault(fault);
    }

    fn declare_trust_boundary(&mut self, boundary: TrustBoundaryKind) {
        self.inner.trust_boundaries.declare(map_boundary(boundary));
    }

    fn configure_wire_session(&mut self, cert_path: Option<String>, key_secret: Option<String>) {
        self.inner.configure_wire_session(cert_path, key_secret);
    }

    fn set_transport_context(
        &mut self,
        boundary: Option<TrustBoundaryKind>,
        encryption: EncryptionMode,
        authentication: AuthenticationMode,
        integrity: IntegrityMode,
    ) {
        self.inner.set_transport_context(
            boundary.map(map_boundary),
            map_enc(encryption),
            map_auth(authentication),
            map_int(integrity),
        );
    }

    fn grant_capability(&mut self, capability: &str) {
        self.inner.capabilities.grant(capability);
    }

    fn grant_capabilities(&mut self, capabilities: &[&str]) {
        self.inner.capabilities.grant_all(capabilities.iter().copied());
    }

    fn granted_capability_count(&self) -> usize {
        self.inner.capabilities.granted().count()
    }

    fn set_trust_level(&mut self, trust: TrustLevel) {
        self.inner.trust = map_trust(trust);
    }

    fn trust_level(&self) -> TrustLevel {
        map_trust_back(self.inner.trust)
    }

    fn grant_if_not_strict(&mut self, capability: &str) {
        self.inner.grant_if_not_strict(capability);
    }

    fn require_operation(&self, operation: &str) -> Result<(), String> {
        self.inner.require_operation(operation).map_err(security_err)
    }

    fn register_secret(&mut self, handle: SecretHandle) {
        self.inner.secrets.register(map_secret(handle));
    }

    fn secret_exists(&self, name: &str) -> bool {
        self.inner.secrets.get(name).is_ok()
    }

    fn resolve_secret(&self, name: &str) -> Result<String, String> {
        self.inner.secrets.resolve(name).map_err(security_err)
    }

    fn set_identity(&mut self, identity: RobotIdentity) {
        self.inner.set_identity(map_identity(identity));
    }

    fn identity_id(&self) -> Option<String> {
        self.inner.identity.as_ref().map(|id| id.id().to_string())
    }

    fn register_secure_endpoint(&mut self, path: &str, policy: SecurePolicy) {
        self.inner
            .register_secure_endpoint(path, map_policy(&policy));
    }

    fn audit_event(
        &self,
        audit: &mut AuditRuntime,
        event_type: &str,
        detail: &str,
    ) -> Result<(), String> {
        self.inner
            .audit_event(audit, event_type, detail)
            .map_err(security_err)
    }

    fn audit_security_event(
        &self,
        audit: &mut AuditRuntime,
        event_type: &str,
        detail: &str,
    ) -> Result<(), String> {
        self.inner
            .audit_security_event(audit, event_type, detail)
            .map_err(security_err)
    }

    fn prepare_publish(
        &mut self,
        path: &str,
        payload: &str,
        source_id: &str,
        message_type: &str,
    ) -> Result<(), String> {
        self.inner
            .prepare_publish(path, payload, source_id, message_type)
            .map(|_| ())
            .map_err(security_err)
    }

    fn verify_inbound(
        &self,
        path: &str,
        signed_json: Option<&str>,
        source_id: Option<&str>,
    ) -> Result<(), String> {
        let signed = signed_json
            .map(serde_json::from_str::<SignedMessage>)
            .transpose()
            .map_err(|e| format!("invalid signed message JSON: {e}"))?;
        self.inner
            .verify_inbound(path, signed.as_ref(), source_id)
            .map_err(security_err)
    }

    fn sign_outbound(&self, path: &str, payload: &str) -> Result<(), String> {
        self.inner
            .sign_outbound(path, payload)
            .map(|_| ())
            .map_err(security_err)
    }

    fn authorize_subscribe(&self, path: &str) -> Result<(), String> {
        self.inner.authorize_subscribe(path).map_err(security_err)
    }

    fn verify_inbound_message(
        &mut self,
        path: &str,
        payload: &str,
        source_id: Option<&str>,
        signed_json: Option<&str>,
        message_type: &str,
    ) -> Result<(), String> {
        let signed = signed_json
            .map(serde_json::from_str::<SignedMessage>)
            .transpose()
            .map_err(|e| format!("invalid signed message JSON: {e}"))?;
        self.inner
            .verify_inbound_message(path, payload, source_id, signed.as_ref(), message_type)
            .map_err(security_err)
    }

    fn verify_remote_signature(&self, signature_json: &str) -> Result<(), String> {
        let signed: SignedMessage = serde_json::from_str(signature_json)
            .map_err(|e| format!("invalid kill switch signature JSON: {e}"))?;
        let identity = self
            .inner
            .identity
            .as_ref()
            .ok_or_else(|| "robot identity required for signature verification".to_string())?;
        if !signed.verify(identity).unwrap_or(false) {
            return Err("kill switch signature verification failed".into());
        }
        Ok(())
    }
}
