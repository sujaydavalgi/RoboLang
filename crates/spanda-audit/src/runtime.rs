//! runtime support for Spanda.
//!
use crate::backend::{AuditBackend, LocalAuditBackend};
use crate::crypto::{sha256, sign, verify_signature};
use crate::error::{AuditError, AuditResult};
use crate::record::{AuditRecord, DeviceIdentity, Hash, ProvenanceRecord, RecordId};
use chrono::Utc;

/// High-level audit runtime used by the Spanda interpreter.
#[derive(Debug)]
pub struct AuditRuntime {
    backend: LocalAuditBackend,
    pub identity: Option<DeviceIdentity>,
    pub audit_name: String,
    pub watched_fields: Vec<String>,
    pub hash_algo: String,
    pub signed_by: Option<String>,
    next_id: u64,
}

impl AuditRuntime {
    pub fn new(audit_name: impl Into<String>, watched_fields: Vec<String>) -> Self {
        // Description:
        //     Construct a new instance.
        //
        // Inputs:
        //     audit_name: impl Into<String>
        //         Caller-supplied audit name.
        //     watched_fields: Vec<String>
        //         Caller-supplied watched fields.
        //
        // Outputs:
        //     result: Self
        //         Return value from `new`.
        //
        // Example:
        //     let value = spanda_audit::runtime::new(audit_name, watched_fields);

        // Assemble the struct fields and return it.
        Self {
            backend: LocalAuditBackend::new(),
            identity: None,
            audit_name: audit_name.into(),
            watched_fields,
            hash_algo: "sha256".into(),
            signed_by: None,
            next_id: 1,
        }
    }

    pub fn with_identity(mut self, identity: DeviceIdentity) -> Self {
        // Description:
        //     With identity.
        //
        // Inputs:
        //     mut self: input value
        //         Caller-supplied mut self.
        //     identity: DeviceIdentity
        //         Caller-supplied identity.
        //
        // Outputs:
        //     result: Self
        //         Return value from `with_identity`.
        //
        // Example:
        //     let result = spanda_audit::runtime::with_identity(mut self, identity);

        // Call identity = Some on the current instance.
        self.identity = Some(identity);
        self
    }

    pub fn with_provenance(
        mut self,
        hash_algo: impl Into<String>,
        signed_by: impl Into<String>,
    ) -> Self {
        // Description:
        //     With provenance.
        //
        // Inputs:
        //     mut self: input value
        //         Caller-supplied mut self.
        //     hash_algo: impl Into<String>
        //         Caller-supplied hash algo.
        //     signed_by: impl Into<String>
        //         Caller-supplied signed by.
        //
        // Outputs:
        //     result: Self
        //         Return value from `with_provenance`.
        //
        // Example:
        //     let result = spanda_audit::runtime::with_provenance(mut self, hash_algo, signed_by);

        // Call into on the current instance.
        self.hash_algo = hash_algo.into();
        self.signed_by = Some(signed_by.into());
        self
    }

    pub fn record_event(&mut self, event_type: &str, payload: &str) -> AuditResult<RecordId> {
        // Description:
        //     Record event.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     event_type: &str
        //         Caller-supplied event type.
        //     payload: &str
        //         Caller-supplied payload.
        //
        // Outputs:
        //     result: AuditResult<RecordId>
        //         Return value from `record_event`.
        //
        // Example:
        //     let result = spanda_audit::runtime::record_event(&mut self, event_type, payload);

        // Compute id for the following logic.
        let id = RecordId(format!("audit-{}", self.next_id));
        self.next_id += 1;
        let previous_hash = self.backend.last_hash();
        let timestamp = Utc::now();
        let body = format!(
            "{}|{}|{}|{}",
            timestamp.to_rfc3339(),
            event_type,
            payload,
            previous_hash.as_ref().map(|h| h.0.as_str()).unwrap_or("")
        );
        let hash = sha256(&body);
        let (signature, signer_id, signing_key) = if let Some(identity) = &self.identity {
            let material = identity.signing_material();
            (
                Some(sign(&body, &material)),
                Some(identity.id.clone()),
                Some(identity.verifying_key_hex()),
            )
        } else {
            (None, None, None)
        };
        let record = AuditRecord {
            id: id.clone(),
            timestamp,
            event_type: event_type.to_string(),
            payload: payload.to_string(),
            hash,
            signature,
            signer_id,
            signing_key,
            previous_hash,
        };
        self.backend.append(record)?;
        Ok(id)
    }

    pub fn record_platform_event(
        &mut self,
        event: &crate::platform_event::PlatformEvent,
    ) -> AuditResult<RecordId> {
        let payload = event.to_json_string()?;
        self.record_event(event.event_type.as_str(), &payload)
    }

    pub fn verify_record(&self, record_id: &RecordId) -> AuditResult<bool> {
        // Description:
        //     Verify record.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //     record_id: &RecordId
        //         Caller-supplied record id.
        //
        // Outputs:
        //     result: AuditResult<bool>
        //         Return value from `verify_record`.
        //
        // Example:
        //     let result = spanda_audit::runtime::verify_record(&self, record_id);

        // Call verify on the current instance.
        self.backend.verify(record_id)
    }

    pub fn export_json(&self) -> AuditResult<String> {
        // Description:
        //     Export json.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: AuditResult<String>
        //         Return value from `export_json`.
        //
        // Example:
        //     let result = spanda_audit::runtime::export_json(&self);

        // Compute export for the following logic.
        let export = self.backend.export()?;
        serde_json::to_string_pretty(&export).map_err(|e| AuditError::Serialization(e.to_string()))
    }

    pub fn record_count(&self) -> usize {
        // Description:
        //     Record count.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: usize
        //         Return value from `record_count`.
        //
        // Example:
        //     let result = spanda_audit::runtime::record_count(&self);

        // Call record count on the current instance.
        self.backend.record_count()
    }

    pub fn create_provenance(
        &self,
        name: &str,
        record_id: &RecordId,
    ) -> AuditResult<ProvenanceRecord> {
        // Description:
        //     Create provenance.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //     name: &str
        //         Caller-supplied name.
        //     record_id: &RecordId
        //         Caller-supplied record id.
        //
        // Outputs:
        //     result: AuditResult<ProvenanceRecord>
        //         Return value from `create_provenance`.
        //
        // Example:
        //     let result = spanda_audit::runtime::create_provenance(&self, name, record_id);

        // Compute record for the following logic.
        let record = self
            .backend
            .records()
            .iter()
            .find(|r| r.id == *record_id)
            .ok_or_else(|| AuditError::NotFound(record_id.0.clone()))?;
        let signed_by = self
            .signed_by
            .clone()
            .or_else(|| self.identity.as_ref().map(|i| i.id.clone()))
            .unwrap_or_else(|| "unknown".into());
        let material = self
            .identity
            .as_ref()
            .map(|i| i.signing_material())
            .unwrap_or_else(|| signed_by.clone());
        let sig = sign(&record.hash.0, &material);
        Ok(ProvenanceRecord {
            name: name.to_string(),
            record_id: record_id.clone(),
            hash: record.hash.clone(),
            signed_by,
            signature: sig,
            anchored: false,
            anchor_tx: None,
        })
    }

    pub fn verify_provenance_signature(&self, prov: &ProvenanceRecord) -> bool {
        // Description:
        //     Verify provenance signature.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //     prov: &ProvenanceRecord
        //         Caller-supplied prov.
        //
        // Outputs:
        //     result: bool
        //         Return value from `verify_provenance_signature`.
        //
        // Example:
        //     let result = spanda_audit::runtime::verify_provenance_signature(&self, prov);

        // Compute verify key for the following logic.
        let verify_key = self
            .identity
            .as_ref()
            .map(|i| i.verifying_key_hex())
            .unwrap_or_else(|| prov.signed_by.clone());
        verify_signature(&prov.hash.0, &prov.signature, &verify_key)
    }

    pub fn root_hash(&self) -> Option<Hash> {
        // Description:
        //     Root hash.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: Option<Hash>
        //         Return value from `root_hash`.
        //
        // Example:
        //     let result = spanda_audit::runtime::root_hash(&self);

        // Call last hash on the current instance.
        self.backend.last_hash()
    }
}
