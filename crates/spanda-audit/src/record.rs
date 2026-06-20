use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Content hash (hex-encoded SHA-256).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hash(pub String);

/// Unique identifier for an audit record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordId(pub String);

/// Ledger transaction identifier (mock / future on-chain tx id).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionId(pub String);

/// Device identity for signing audit records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceIdentity {
    pub id: String,
    pub public_key: String,
}

impl DeviceIdentity {
    pub fn new(id: impl Into<String>, public_key: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            public_key: public_key.into(),
        }
    }

    /// Material used to derive the Ed25519 signing key.
    pub fn signing_material(&self) -> String {
        if self.public_key.is_empty() || crate::crypto::is_hex_public_key(&self.public_key) {
            format!("spanda-device-{}", self.id)
        } else {
            self.public_key.clone()
        }
    }

    /// Hex-encoded Ed25519 public key for signature verification.
    pub fn verifying_key_hex(&self) -> String {
        if crate::crypto::is_hex_public_key(&self.public_key) {
            self.public_key.clone()
        } else {
            crate::crypto::public_key_from_material(&self.signing_material())
        }
    }

    /// Backward-compatible alias for signing material.
    pub fn default_key(&self) -> String {
        self.signing_material()
    }
}

/// Append-only audit event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditRecord {
    pub id: RecordId,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub payload: String,
    pub hash: Hash,
    pub signature: Option<String>,
    pub signer_id: Option<String>,
    pub signing_key: Option<String>,
    pub previous_hash: Option<Hash>,
}

impl AuditRecord {
    pub fn canonical_body(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.timestamp.to_rfc3339(),
            self.event_type,
            self.payload,
            self.previous_hash
                .as_ref()
                .map(|h| h.0.as_str())
                .unwrap_or("")
        )
    }
}

/// Provenance metadata linking audit records to signed mission logs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceRecord {
    pub name: String,
    pub record_id: RecordId,
    pub hash: Hash,
    pub signed_by: String,
    pub signature: String,
    pub anchored: bool,
    pub anchor_tx: Option<TransactionId>,
}

/// Mission-level audit summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionRecord {
    pub mission_id: String,
    pub device_id: String,
    pub record_count: usize,
    pub root_hash: Hash,
    pub signed: bool,
}

/// Export bundle for JSON serialization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditExport {
    pub records: Vec<AuditRecord>,
    pub provenance: Vec<ProvenanceRecord>,
    pub mission: Option<MissionRecord>,
    pub exported_at: DateTime<Utc>,
}
