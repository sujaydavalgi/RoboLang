use thiserror::Error;

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("manifest error: {0}")]
    Manifest(String),

    #[error("dependency error: {0}")]
    Dependency(String),

    #[error("lockfile error: {0}")]
    Lockfile(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("semver error: {0}")]
    Semver(#[from] semver::Error),
}

pub type PackageResult<T> = Result<T, PackageError>;
