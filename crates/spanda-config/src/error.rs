//! Configuration loading and validation errors.
//!
use std::path::PathBuf;
use thiserror::Error;

/// Errors produced while loading, merging, or validating Spanda configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse TOML at {path}: {source}")]
    TomlParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("failed to parse JSON at {path}: {source}")]
    JsonParse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("manifest not found: expected {path}")]
    ManifestNotFound { path: PathBuf },

    #[error("referenced config file not found: {path}")]
    ConfigFileNotFound { path: PathBuf },

    #[error("circular config layer reference: {cycle}")]
    CircularLayer { cycle: String },

    #[error("merge conflict at {path}: {detail}")]
    MergeConflict { path: String, detail: String },

    #[error("invalid manifest: {detail}")]
    InvalidManifest { detail: String },

    #[error("config approval error: {detail}")]
    Approval { detail: String },

    #[error("package manifest error: {0}")]
    Package(#[from] spanda_package::PackageError),
}

pub type ConfigResult<T> = Result<T, ConfigError>;
