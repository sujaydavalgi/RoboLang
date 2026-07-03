//! Digital mission twin cloud sync — HTTP client and in-memory store for Twin Cloud SaaS.

mod client;
mod config;
mod snapshot;
mod store;

pub use client::TwinCloudClient;
pub use config::TwinCloudConfig;
pub use snapshot::{
    build_snapshot_from_program, default_twin_id, TwinCloudHistoryResponse, TwinCloudListResponse,
    TwinCloudSnapshot, TwinCloudSummary, TwinCloudSyncResponse, TWIN_CLOUD_API_VERSION,
};
pub use store::TwinCloudStore;

#[cfg(test)]
mod tests {
    use super::{build_snapshot_from_program, TwinCloudStore};
    use std::path::PathBuf;

    fn patrol_program() -> (spanda_ast::nodes::Program, String) {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/showcase/mission_twin/patrol.sd");
        let source = std::fs::read_to_string(&path).expect("read patrol.sd");
        let tokens = spanda_lexer::tokenize(&source).expect("tokenize");
        let program = spanda_parser::parse(tokens).expect("parse");
        (program, path.display().to_string())
    }

    #[test]
    fn store_upsert_and_list() {
        let (program, label) = patrol_program();
        let mut store = TwinCloudStore::new();
        let snapshot = build_snapshot_from_program(&program, &label, None, "default");
        store.upsert(snapshot);
        let listed = store.list(Some("default"));
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].twin_id, "patrol");
    }
}
