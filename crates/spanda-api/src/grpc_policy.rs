//! gRPC protobuf semver and compatibility policy for Control Center.
//!
use serde_json::{json, Value};

/// Protobuf package name (`package` directive in `.proto`).
pub const PROTO_PACKAGE: &str = "spanda.v1";

/// Semver for the published `control_center.proto` contract (independent of crate version).
pub const PROTO_SEMVER: &str = "1.0.2";

/// Relative path to the proto file from the `spanda-api` crate root.
pub const PROTO_FILE: &str = "proto/spanda/v1/control_center.proto";

/// gRPC server reflection is not enabled; clients must pin the published proto file.
pub const REFLECTION_ENABLED: bool = false;

/// Count `rpc` methods declared in `control_center.proto`.
pub fn control_center_rpc_count() -> usize {
    include_str!("../proto/spanda/v1/control_center.proto")
        .lines()
        .filter(|line| line.trim().starts_with("rpc "))
        .count()
}

/// JSON policy block returned by `GET /v1/version` and embedded in gRPC health metadata docs.
pub fn policy_json() -> Value {
    json!({
        "package": PROTO_PACKAGE,
        "proto_semver": PROTO_SEMVER,
        "proto_file": PROTO_FILE,
        "rpc_count": control_center_rpc_count(),
        "reflection_enabled": REFLECTION_ENABLED,
        "api_version_header": "x-spanda-api-version",
        "supported_api_versions": [crate::versioning::SUPPORTED_API_VERSION],
        "compatibility": "Additive RPCs and optional message fields ship in patch proto semver bumps. Breaking RPC or field semantic changes require a new package (for example spanda.v2) and a new tonic service.",
        "discovery": "Download control_center.proto from the repository or call GET /v1/version for the pinned proto_semver; grpcurl requires --proto flags (reflection disabled).",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proto_declares_at_least_fifty_nine_rpcs() {
        assert!(
            control_center_rpc_count() >= 59,
            "unexpected rpc count: {}",
            control_center_rpc_count()
        );
    }

    #[test]
    fn proto_file_documents_semver_package() {
        let proto = include_str!("../proto/spanda/v1/control_center.proto");
        assert!(proto.contains("package spanda.v1"));
        assert!(proto.contains("proto semver"));
    }
}
