use dazpilot_lib::animation;

#[test]
fn test_playback_controls_state_transitions() {
    // 1. Initial State
    animation::stop();
    let state = animation::get_playback_state();
    assert_eq!(state.playing, false, "Should not be playing initially.");
    assert_eq!(state.current_time, 0.0, "Should seek to 0.0 on stop.");

    // 2. Play
    animation::play();
    let state = animation::get_playback_state();
    assert_eq!(state.playing, true, "Should transition to playing: true.");

    // 3. Pause
    animation::pause();
    let state = animation::get_playback_state();
    assert_eq!(state.playing, false, "Should pause playback.");
}

#[test]
fn test_playback_speed_clamps() {
    animation::set_playback_speed(15.0);
    let state = animation::get_playback_state();
    assert_eq!(state.speed, 10.0, "Playback speed should be clamped to a maximum of 10.0.");

    animation::set_playback_speed(0.01);
    let state = animation::get_playback_state();
    assert_eq!(state.speed, 0.1, "Playback speed should be clamped to a minimum of 0.1.");
}

#[test]
fn test_loop_toggling() {
    animation::stop();
    let initial = animation::get_playback_state().loop_enabled;
    animation::toggle_loop();
    let toggled = animation::get_playback_state().loop_enabled;
    assert_ne!(initial, toggled, "Toggling loop should invert the loop_enabled boolean state.");
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
    let state = animation::get_timeline_state();
    assert_eq!(
        state.active_figure,
        Some("Genesis 8 Female".to_string()),
        "Active figure should update in timeline state."
    );
}
