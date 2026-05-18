use dazpilot_lib::ai_action;

#[test]
fn test_query_noise_word_extractor() {
    let clean = ai_action::extract_asset_search_query("could you please load the Genesis 8 Female figure into the scene?");
    assert_eq!(clean, Some("Genesis 8 Female".to_string()), "Query extractor should remove verbs, pronouns, and polite particles on word boundaries.");
}

#[test]
fn test_timeline_planning_rules() {
    // Test that a timeline seeking query parses correctly into seek_to_frame structured command
    let plan = ai_action::plan_validated_action("please seek to frame 120");
    assert!(plan.is_some(), "Should successfully parse seeking request.");
    let action = plan.unwrap();
    assert_eq!(action.command, "seek_to_frame");
    assert_eq!(action.args.get("frame").and_then(|v| v.as_f64()), Some(120.0));
}

#[test]
fn test_dforce_physics_simulation_planning() {
    // Test that dforce request parses into run_dforce_simulation structured command
    let plan = ai_action::plan_validated_action("run a physics simulation from frame 0 to 90");
    assert!(plan.is_some(), "Should parse dforce physics request.");
    let action = plan.unwrap();
    assert_eq!(action.command, "run_dforce_simulation");
    assert_eq!(action.args.get("start_frame").and_then(|v| v.as_i64()), Some(0));
    assert_eq!(action.args.get("end_frame").and_then(|v| v.as_i64()), Some(90));
}
