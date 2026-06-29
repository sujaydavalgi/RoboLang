//! Audit, provenance, and ledger backend abstractions for Spanda.
//!
//! Blockchain is **not** part of the language core. This crate provides
//! trait-based backends that future packages (`spanda-ledger-ethereum`, etc.)
//! can implement for tamper-evident mission records and supply-chain traceability.

pub mod backend;
pub mod crypto;
pub mod error;
pub mod platform_event;
pub mod record;
pub mod runtime;

pub use backend::{
    AuditBackend, JsonAuditBackend, LedgerBackend, LocalAuditBackend, MockLedgerBackend,
};
pub use crypto::{public_key_from_material, sha256, sign, verify_signature};
pub use error::{AuditError, AuditResult};
pub use platform_event::{names, PlatformEvent, PlatformEventType};
pub use record::{
    AuditExport, AuditRecord, DeviceIdentity, Hash, MissionRecord, ProvenanceRecord, RecordId,
    TransactionId,
};
pub use runtime::AuditRuntime;
