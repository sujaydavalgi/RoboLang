//! Security foundation for Spanda robotics programs.
//!
//! Provides identity, secrets, capability enforcement, signed messages,
//! trust levels, package permissions, and secure communication policies.

pub mod capability;
pub mod error;
pub mod identity;
pub mod permissions;
pub mod runtime;
pub mod secrets;
pub mod secure_comm;
pub mod signed;
pub mod trust;

pub use capability::{
    capability_for_operation, is_known_capability, known_capabilities, CapabilitySet, Permission,
};
pub use error::{SecurityError, SecurityResult};
pub use identity::RobotIdentity;
pub use permissions::PackagePermissions;
pub use runtime::{SecurityContext, SecuritySnapshot};
pub use secrets::{SecretHandle, SecretSource, SecretStore};
pub use secure_comm::{SecureEndpointRegistry, SecurePolicy};
pub use signed::{Signature, SignedMessage};
pub use trust::TrustLevel;
