//! Security error types and result alias for the Spanda security crate.

use thiserror::Error;

/// Convenience alias for security operations that may fail with [`SecurityError`].
pub type SecurityResult<T> = Result<T, SecurityError>;

#[derive(Debug, Error, PartialEq)]
pub enum SecurityError {
    #[error("capability denied: {0}")]
    CapabilityDenied(String),
    #[error("trust level insufficient: required {required}, have {actual}")]
    TrustInsufficient { required: String, actual: String },
    #[error("secret not found: {0}")]
    SecretNotFound(String),
    #[error("signature verification failed")]
    SignatureInvalid,
    #[error("secure endpoint violation on {endpoint}: {reason}")]
    SecureEndpoint { endpoint: String, reason: String },
    #[error("identity required for {operation}")]
    IdentityRequired { operation: String },
    #[error("authentication failed: {reason}")]
    AuthenticationFailed { reason: String },
    #[error("replay attack detected on {endpoint}")]
    ReplayDetected { endpoint: String },
    #[error("certificate expired: {subject}")]
    CertificateExpired { subject: String },
    #[error("untrusted source rejected: {0}")]
    UntrustedSource(String),
    #[error("encryption required but not configured for {endpoint}")]
    EncryptionNotConfigured { endpoint: String },
    #[error("{0}")]
    Other(String),
}
