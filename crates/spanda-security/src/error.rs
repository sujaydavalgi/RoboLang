use thiserror::Error;

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
    #[error("{0}")]
    Other(String),
}
