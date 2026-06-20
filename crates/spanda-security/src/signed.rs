use crate::error::{SecurityError, SecurityResult};
use crate::identity::RobotIdentity;
use serde::{Deserialize, Serialize};
use spanda_audit::{sign, verify_signature};

/// Cryptographic signature over a payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    pub value: String,
    pub signer_id: String,
}

/// Signed message envelope for secure communication.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignedMessage {
    pub payload: String,
    pub signature: Signature,
    pub hash: String,
}

impl SignedMessage {
    pub fn sign(payload: impl Into<String>, identity: &RobotIdentity) -> Self {
        let payload = payload.into();
        let hash = spanda_audit::sha256(&payload);
        let sig_value = sign(&payload, &identity.signing_key());
        Self {
            payload,
            signature: Signature {
                value: sig_value,
                signer_id: identity.id().to_string(),
            },
            hash: hash.0,
        }
    }

    pub fn verify(&self, identity: &RobotIdentity) -> SecurityResult<bool> {
        if self.signature.signer_id != identity.id() {
            return Err(SecurityError::SignatureInvalid);
        }
        Ok(verify_signature(
            &self.payload,
            &self.signature.value,
            &identity.signing_key(),
        ))
    }

    pub fn to_json(&self) -> SecurityResult<String> {
        serde_json::to_string(self)
            .map_err(|e| SecurityError::Other(format!("serialization failed: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_message_roundtrip() {
        let id = RobotIdentity::new("rover-1", "key-abc");
        let msg = SignedMessage::sign("telemetry", &id);
        assert!(msg.verify(&id).unwrap());
    }
}
