use dazpilot_lib::animation;
use serial_test::serial;

#[test]
fn test_play_does_not_panic_with_mock_bridge() {
    std::env::set_var("DAZPILOT_DEV_MOCK_BRIDGE", "1");
    animation::play();
    std::env::remove_var("DAZPILOT_DEV_MOCK_BRIDGE");
}

#[test]
fn test_pause_does_not_panic_with_mock_bridge() {
    std::env::set_var("DAZPILOT_DEV_MOCK_BRIDGE", "1");
    animation::pause();
    std::env::remove_var("DAZPILOT_DEV_MOCK_BRIDGE");
}

#[test]
fn test_stop_does_not_panic_with_mock_bridge() {
    std::env::set_var("DAZPILOT_DEV_MOCK_BRIDGE", "1");
    animation::stop();
    std::env::remove_var("DAZPILOT_DEV_MOCK_BRIDGE");
}

#[test]
#[serial]
fn test_get_timeline_state_with_mock_bridge() {
    std::env::set_var("DAZPILOT_DEV_MOCK_BRIDGE", "1");
    let state = animation::get_timeline_state();
    assert_eq!(state.current_frame, 0.0);
    assert_eq!(state.total_frames, 300.0);
    assert!(!state.is_playing);
    std::env::remove_var("DAZPILOT_DEV_MOCK_BRIDGE");
}

#[test]
fn test_get_playback_state_defaults() {
    // Without bridge, playback state returns sensible defaults
    let state = animation::get_playback_state();
    assert!(!state.playing);
    assert_eq!(state.speed, 1.0);
}

#[test]
fn test_pose_library_lookup() {
    let poses = animation::get_pose_library();
    assert!(!poses.is_empty(), "Pose library should not be empty.");
    assert!(
        poses.iter().any(|p| p.name == "Standing"),
        "Standing pose should be registered in default poses library."
    );
}

#[test]
fn test_active_figure_assignment() {
    let res = animation::set_active_figure("Genesis 8 Female");
    assert!(res.success, "Setting active figure should report success.");
}

#[test]
fn test_set_playback_speed_no_panic() {
    animation::set_playback_speed(15.0);
    animation::set_playback_speed(0.01);
}

#[test]
fn test_toggle_loop_no_panic() {
    animation::toggle_loop();
}
