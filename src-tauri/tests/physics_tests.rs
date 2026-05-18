use dazpilot_lib::physics;

#[test]
fn test_default_physics_parameters() {
    let settings = physics::get_default_physics_settings();
    assert_eq!(settings.enabled, false, "Physics should be disabled by default.");
    assert_eq!(settings.substeps, 3, "Default substeps count must be 3.");
    assert_eq!(settings.gravity, -9.81, "Standard gravity must be -9.81 m/s^2.");
}

#[test]
fn test_simulation_quality_substeps() {
    let preview = physics::set_simulation_quality(physics::SimulationQuality::Preview);
    assert_eq!(preview.success, true);
    assert_eq!(preview.data.unwrap().get("substeps").and_then(|v| v.as_i64()), Some(1));

    let ultra = physics::set_simulation_quality(physics::SimulationQuality::Ultra);
    assert_eq!(ultra.success, true);
    assert_eq!(ultra.data.unwrap().get("substeps").and_then(|v| v.as_i64()), Some(10));
}

#[test]
fn test_frame_calculations() {
    let sim = physics::run_simulation(10, 50);
    assert_eq!(sim.frames_computed, 40, "Simulation computed frames should equal difference.");

    let bake = physics::bake_to_keyframes(0, 100);
    assert_eq!(bake.keyframe_count, 100, "Baked keyframes count must equal frames difference.");
    assert_eq!(bake.start_frame, 0);
    assert_eq!(bake.end_frame, 100);
}

#[test]
fn test_collision_zone_addition() {
    let zone = physics::CollisionZone {
        zone_name: "test_shield".to_string(),
        body_part: physics::BodyPart::Head,
        enabled: true,
        collision_type: physics::CollisionType::Capsule,
        restitution: 0.1,
        friction: 0.8,
    };
    
    let res = physics::add_collision_zone(zone);
    assert_eq!(res.success, true);
    assert!(res.message.contains("test_shield"), "Message should echo the custom zone name.");
}

#[test]
fn test_particle_emitter_creation() {
    let system = physics::create_particle_system("Fire", "Torch");
    assert_eq!(system.name, "Fire");
    assert_eq!(system.emitter_node, "Torch");
    assert_eq!(system.particle_count, 500);
    assert_eq!(system.gravity_influence, 0.5);
}
