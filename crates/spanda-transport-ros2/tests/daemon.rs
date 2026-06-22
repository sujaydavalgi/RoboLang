//! ROS2 daemon bridge tests.

use spanda_transport_ros2::daemon_script_path;

#[test]
fn daemon_script_resolves_in_repo() {
    if std::env::var("CARGO_MANIFEST_DIR").is_ok() {
        assert!(daemon_script_path().is_ok());
    }
}

#[test]
fn rclrs_disabled_by_default() {
    std::env::remove_var("SPANDA_ROS2_RCLRS");
    assert!(!spanda_transport_ros2::rclrs_enabled());
}
