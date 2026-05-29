use dazpilot_lib::ai_action;

#[test]
fn test_query_noise_word_extractor() {
    let clean = ai_action::extract_asset_search_query(
        "could you please load the Genesis 8 Female figure into the scene?",
    );
    assert_eq!(
        clean,
        Some("Genesis 8 Female".to_string()),
        "Query extractor should remove verbs, pronouns, and polite particles on word boundaries."
    );
}

#[test]
fn test_timeline_planning_rules() {
    // Test that a timeline seeking query parses correctly into seek_to_frame structured command
    let plan = ai_action::plan_validated_action("please seek to frame 120");
    assert!(plan.is_some(), "Should successfully parse seeking request.");
    let action = plan.unwrap();
    assert_eq!(action.command, "seek_to_frame");
    assert_eq!(
        action.args.get("frame").and_then(|v| v.as_f64()),
        Some(120.0)
    );
}

#[test]
fn test_dforce_physics_simulation_planning() {
    // Test that dforce request parses into run_dforce_simulation structured command
    let plan = ai_action::plan_validated_action("run a physics simulation from frame 0 to 90");
    assert!(plan.is_some(), "Should parse dforce physics request.");
    let action = plan.unwrap();
    assert_eq!(action.command, "run_dforce_simulation");
    assert_eq!(
        action.args.get("start_frame").and_then(|v| v.as_i64()),
        Some(0)
    );
    assert_eq!(
        action.args.get("end_frame").and_then(|v| v.as_i64()),
        Some(90)
    );
}

#[test]
fn test_body_opacity_planning() {
    let action = ai_action::plan_validated_action("make the body transparent at 15 percent")
        .expect("Should plan body opacity request.");
    assert_eq!(action.command, "set_body_opacity");
    assert_eq!(
        action.args.get("node_id").and_then(|v| v.as_str()),
        Some("selected")
    );
    let value = action.args.get("value").and_then(|v| v.as_f64()).unwrap();
    assert!((value - 0.15).abs() < 0.0001);
}

#[test]
fn test_surface_opacity_planning() {
    let action = ai_action::plan_validated_action("make torso opacity 0.05")
        .expect("Should plan surface opacity request.");
    assert_eq!(action.command, "set_surface_opacity");
    assert_eq!(
        action.args.get("surface_pattern").and_then(|v| v.as_str()),
        Some("torso")
    );
    let value = action.args.get("value").and_then(|v| v.as_f64()).unwrap();
    assert!((value - 0.05).abs() < 0.0001);
}

#[test]
fn test_internal_surface_and_anatomy_planning() {
    let discover = ai_action::plan_validated_action("list internal surfaces")
        .expect("Should plan internal surface discovery.");
    assert_eq!(discover.command, "get_internal_surfaces");

    let show = ai_action::plan_validated_action("show anatomy on the selected figure")
        .expect("Should plan show anatomy.");
    assert_eq!(show.command, "show_anatomy");
}

#[test]
fn test_place_asset_inside_planning() {
    let action =
        ai_action::plan_validated_action("place asset \"C:/Content/Alien.duf\" inside the figure")
            .expect("Should plan inside asset placement.");
    assert_eq!(action.command, "place_asset_inside");
    assert_eq!(
        action.args.get("asset_path").and_then(|v| v.as_str()),
        Some("C:/Content/Alien.duf")
    );
}
