//! Native rclrs loader tests for spanda-transport-ros2.

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use spanda_transport_ros2::native;

    #[test]
    fn missing_library_does_not_panic() {
        let available = native::sdk_available();
        let _ = native::publish("/x", "y");
        let _ = native::subscribe("/x");
        if std::env::var("SPANDA_ROS2_RCLRS_LIB").is_err() {
            assert!(!available);
        }
    }
}
