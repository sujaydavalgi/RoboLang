//! AES-256-GCM encryption for configuration snapshots at rest.
//!
use crate::error::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};
use spanda_security::WireCryptoSession;

const ENVELOPE_FORMAT: &str = "spanda-config-snapshot-v1";

/// On-disk envelope for an encrypted configuration snapshot file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncryptedSnapshotEnvelope {
    pub format: String,
    pub cipher: String,
    pub ciphertext: String,
}

/// Whether snapshot encryption is enabled for this save operation.
pub fn snapshot_encryption_requested(explicit: Option<bool>) -> bool {
    explicit.unwrap_or_else(|| {
        std::env::var("SPANDA_CONFIG_SNAPSHOT_ENCRYPT")
            .ok()
            .map(|value| {
                value == "1"
                    || value.eq_ignore_ascii_case("true")
                    || value.eq_ignore_ascii_case("yes")
            })
            .unwrap_or(false)
    })
}

/// Encryption key material from `SPANDA_CONFIG_SNAPSHOT_KEY`.
pub fn snapshot_encryption_key() -> Option<String> {
    std::env::var("SPANDA_CONFIG_SNAPSHOT_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn encryption_key_or_error() -> ConfigResult<String> {
    snapshot_encryption_key().ok_or_else(|| ConfigError::SnapshotEncryption {
        detail: "SPANDA_CONFIG_SNAPSHOT_KEY is required when encryption is enabled".into(),
    })
}

/// Encrypt snapshot JSON bytes for persistence.
pub fn encrypt_snapshot_bytes(plaintext: &[u8]) -> ConfigResult<EncryptedSnapshotEnvelope> {
    let key = encryption_key_or_error()?;
    let session = WireCryptoSession::from_material(&key);
    let encrypted = session
        .encrypt(plaintext)
        .map_err(|error| ConfigError::SnapshotEncryption { detail: error })?;
    Ok(EncryptedSnapshotEnvelope {
        format: ENVELOPE_FORMAT.into(),
        cipher: session.cipher_suite,
        ciphertext: hex::encode(encrypted),
    })
}

/// Decrypt an on-disk encrypted snapshot envelope.
pub fn decrypt_snapshot_envelope(envelope: &EncryptedSnapshotEnvelope) -> ConfigResult<Vec<u8>> {
    if envelope.format != ENVELOPE_FORMAT {
        return Err(ConfigError::SnapshotEncryption {
            detail: format!("unsupported snapshot envelope format '{}'", envelope.format),
        });
    }
    let key = encryption_key_or_error()?;
    let session = WireCryptoSession::from_material(&key);
    let ciphertext =
        hex::decode(&envelope.ciphertext).map_err(|error| ConfigError::SnapshotEncryption {
            detail: error.to_string(),
        })?;
    session
        .decrypt(&ciphertext)
        .map_err(|error| ConfigError::SnapshotEncryption { detail: error })
}

#[cfg(test)]
pub(crate) mod snapshot_env_test {
    use std::sync::Mutex;

    static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

    /// Serialize tests that mutate `SPANDA_CONFIG_SNAPSHOT_KEY` and related env vars.
    pub fn lock() -> std::sync::MutexGuard<'static, ()> {
        ENV_TEST_LOCK.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::snapshot_env_test::lock;

    #[test]
    fn encrypted_snapshot_roundtrip() {
        let _guard = lock();
        std::env::set_var("SPANDA_CONFIG_SNAPSHOT_KEY", "test-snapshot-key-material");
        let plaintext = br#"{"meta":{"id":"cfg-1"},"resolved":{}}"#;
        let envelope = encrypt_snapshot_bytes(plaintext).expect("encrypt");
        let decrypted = decrypt_snapshot_envelope(&envelope).expect("decrypt");
        assert_eq!(decrypted, plaintext);
    }
}
