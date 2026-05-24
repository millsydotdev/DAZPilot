use dazpilot_lib::advanced;
use std::collections::HashMap;

#[test]
fn test_default_import_export_settings() {
    let import_set = advanced::get_default_import_settings();
    assert_eq!(import_set.scale, 1.0);
    assert_eq!(import_set.import_materials, true);

    let export_set = advanced::get_default_export_settings();
    assert_eq!(export_set.include_materials, true);
    assert_eq!(export_set.include_animations, true);
}

#[test]
fn test_supported_formats() {
    let imports = advanced::get_supported_import_formats();
    assert!(imports.contains(&"OBJ".to_string()));
    assert!(imports.contains(&"FBX".to_string()));

    let exports = advanced::get_supported_export_formats();
    assert!(exports.contains(&"OBJ".to_string()));
    assert!(exports.contains(&"glTF".to_string()));
    assert!(exports.contains(&"MP4".to_string()));
}

#[test]
fn test_sequence_action_durations() {
    let mut seq = advanced::create_scene_sequence("TestSequence");
    assert_eq!(seq.total_duration, 0.0);
    assert_eq!(seq.actions.len(), 0);

    let action = advanced::SceneAction {
        action_type: "Move".to_string(),
        target: "Figure".to_string(),
        parameters: HashMap::new(),
        delay_frames: Some(60), // 60 frames / 30 FPS = 2.0 seconds
    };

    advanced::add_action_to_sequence(&mut seq, action);
    assert_eq!(
        seq.total_duration, 2.0,
        "Total duration should calculate using frame delay."
    );
    assert_eq!(seq.actions.len(), 1);
}

#[test]
fn test_export_presets_registry() {
    let presets = advanced::get_export_presets();
    assert!(presets.contains_key("web"));
    assert!(presets.contains_key("archival"));
    assert!(presets.contains_key("web3d"));
}
