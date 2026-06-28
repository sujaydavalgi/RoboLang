//! Humans and wearables REST handlers.
use spanda_api::state::ControlCenterState;
use spanda_config::ConfigResolver;
use std::path::PathBuf;

#[test]
fn humans_and_wearables_list_from_spatial_blueprint() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/solutions/spatial-computing");
    let resolved = ConfigResolver::new()
        .with_validation(false)
        .resolve_from_dir(&root)
        .expect("resolve spatial blueprint");
    let mut state = ControlCenterState::new();
    state.resolved = Some(resolved);
    let humans = spanda_api::humans::humans_list(&state);
    assert_eq!(humans.status, 200);
    assert!(humans.body.contains("operator-001"));
    let wearables = spanda_api::humans::wearables_list(&state);
    assert_eq!(wearables.status, 200);
    assert!(wearables.body.contains("watch-001"));
    let policy = spanda_api::humans::human_health_policy(&state);
    assert_eq!(policy.status, 200);
    assert!(policy.body.contains("human_health"));
}
