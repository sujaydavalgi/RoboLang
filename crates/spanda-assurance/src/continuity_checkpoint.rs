//! Durable mission checkpoint persistence for continuity handoffs across restarts.

pub use spanda_runtime::continuity_primitives::{
    default_checkpoint_store_path, load_checkpoint, load_checkpoint_store, record_checkpoint,
    save_checkpoint_store,
};
pub use spanda_runtime::continuity_types::ContinuityCheckpointStore;
