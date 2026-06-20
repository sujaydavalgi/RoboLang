use crate::capability::CapabilitySet;
use crate::error::{SecurityError, SecurityResult};
use crate::identity::RobotIdentity;
use crate::signed::SignedMessage;
use crate::trust::TrustLevel;
use serde::{Deserialize, Serialize};

/// Security policy attached to a topic, service, or action endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SecurePolicy {
    pub signed: bool,
    pub min_trust: Option<TrustLevel>,
    pub requires: Vec<String>,
}

impl SecurePolicy {
    pub fn open() -> Self {
        Self::default()
    }

    pub fn signed_trusted() -> Self {
        Self {
            signed: true,
            min_trust: Some(TrustLevel::Trusted),
            requires: vec!["identity.verify".into()],
        }
    }

    pub fn check_trust(&self, trust: TrustLevel) -> SecurityResult<()> {
        if let Some(required) = self.min_trust {
            if !trust.satisfies(required) {
                return Err(SecurityError::TrustInsufficient {
                    required: required.as_str().into(),
                    actual: trust.as_str().into(),
                });
            }
        }
        Ok(())
    }

    pub fn check_capabilities(&self, caps: &CapabilitySet) -> SecurityResult<()> {
        for cap in &self.requires {
            caps.require(cap)?;
        }
        Ok(())
    }

    pub fn prepare_outbound(
        &self,
        payload: &str,
        identity: Option<&RobotIdentity>,
        caps: &CapabilitySet,
        endpoint: &str,
    ) -> SecurityResult<Option<SignedMessage>> {
        if self.signed || self.min_trust.is_some() || !self.requires.is_empty() {
            self.check_capabilities(caps)?;
            if let Some(id) = identity {
                self.check_trust(id.trust)?;
                if self.signed {
                    caps.require("identity.sign")?;
                    return Ok(Some(SignedMessage::sign(payload, id)));
                }
                return Ok(None);
            }
            return Err(SecurityError::IdentityRequired {
                operation: endpoint.to_string(),
            });
        }
        Ok(None)
    }

    pub fn verify_inbound(
        &self,
        signed: Option<&SignedMessage>,
        identity: Option<&RobotIdentity>,
        caps: &CapabilitySet,
        endpoint: &str,
    ) -> SecurityResult<()> {
        if self.signed || self.min_trust.is_some() || !self.requires.is_empty() {
            self.check_capabilities(caps)?;
            let id = identity.ok_or_else(|| SecurityError::IdentityRequired {
                operation: endpoint.to_string(),
            })?;
            self.check_trust(id.trust)?;
            if self.signed {
                let msg = signed.ok_or_else(|| SecurityError::SecureEndpoint {
                    endpoint: endpoint.to_string(),
                    reason: "missing signature".into(),
                })?;
                if !msg.verify(id)? {
                    return Err(SecurityError::SignatureInvalid);
                }
            }
        }
        Ok(())
    }
}

/// Registry of secure policies keyed by endpoint path.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecureEndpointRegistry {
    policies: std::collections::HashMap<String, SecurePolicy>,
}

impl SecureEndpointRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, path: impl Into<String>, policy: SecurePolicy) {
        self.policies.insert(path.into(), policy);
    }

    pub fn get(&self, path: &str) -> Option<&SecurePolicy> {
        self.policies.get(path)
    }

    pub fn policy_or_open(&self, path: &str) -> SecurePolicy {
        self.get(path).cloned().unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        self.policies.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilitySet;

    #[test]
    fn secure_topic_requires_identity() {
        let policy = SecurePolicy::signed_trusted();
        let mut caps = CapabilitySet::new();
        caps.grant("identity.sign");
        caps.grant("identity.verify");
        let err = policy
            .prepare_outbound("data", None, &caps, "/cmd")
            .unwrap_err();
        assert!(matches!(err, SecurityError::IdentityRequired { .. }));
    }
}
