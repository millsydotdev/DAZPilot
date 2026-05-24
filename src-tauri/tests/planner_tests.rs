use dazpilot_lib::ai_system::{Entity, EntityType, Intent};
use dazpilot_lib::knowledge::command_knowledge::CommandKnowledgeBase;
use dazpilot_lib::knowledge::workflow_knowledge::{action_to_command_map, ActionType};
use dazpilot_lib::library_scanner::AssetInfo;
use dazpilot_lib::reasoning::planner::{Goal, GoalPriority, PlanStep, Planner, PlanningContext};
use std::collections::HashMap;

fn make_goal(description: &str, intent: Intent, entities: Vec<Entity>) -> Goal {
    Goal {
        id: "test_goal".into(),
        description: description.into(),
        intent,
        entities,
        priority: GoalPriority::Medium,
        constraints: vec![],
    }
}

fn empty_context() -> PlanningContext {
    PlanningContext {
        scene_state: None,
        recent_actions: vec![],
        user_preferences: None,
        available_assets: vec![],
        constraints: vec![],
    }
}

#[test]
fn test_create_scene_workflow_detected() {
    let planner = Planner::new();
    let goal = make_goal(
        "Build a complete scene from scratch",
        Intent::CreateScene,
        vec![],
    );
    let plan = planner.plan_for_goal(&goal, &empty_context());
    assert!(
        plan.is_some(),
        "Planner should return a plan for scene creation"
    );
    let plan = plan.unwrap();
    assert!(
        plan.description.contains("Scene") || plan.steps.len() >= 10,
        "Plan description should reference scene or have 10+ steps, got: {} ({} steps)",
        plan.description,
        plan.steps.len()
    );
    assert!(!plan.steps.is_empty(), "Scene workflow should have steps");
    assert_eq!(
        plan.steps[0].action.command, "begin_undo_batch",
        "First step should begin undo batch"
    );
}

#[test]
fn test_create_scene_workflow_full_steps() {
    let planner = Planner::new();
    let goal = make_goal(
        "Create a full scene from scratch",
        Intent::CreateScene,
        vec![],
    );
    let plan = planner.plan_for_goal(&goal, &empty_context());
    assert!(plan.is_some());
    let plan = plan.unwrap();

    let commands: Vec<&str> = plan
        .steps
        .iter()
        .map(|s| s.action.command.as_str())
        .collect();
    assert!(commands.contains(&"add_figure"), "Should add a figure");
    assert!(
        commands.contains(&"add_node"),
        "Should create lights via add_node"
    );
    assert!(commands.contains(&"render_preview"), "Should render");
    assert!(
        commands.contains(&"begin_undo_batch"),
        "Should use undo batching"
    );
    assert!(
        commands.contains(&"accept_undo_batch"),
        "Should close undo batch"
    );
}

#[test]
fn test_create_scene_workflow_has_resolved_prerequisites() {
    let planner = Planner::new();
    let goal = make_goal("Create a full scene", Intent::CreateScene, vec![]);
    let plan = planner.plan_for_goal(&goal, &empty_context()).unwrap();
    let ids: std::collections::HashSet<&str> = plan.steps.iter().map(|s| s.id.as_str()).collect();

    for step in &plan.steps {
        for prereq in &step.prerequisites {
            assert!(
                ids.contains(prereq.as_str()),
                "Step '{}' has unresolved prerequisite '{}'",
                step.description,
                prereq
            );
        }
    }
}

fn asset(path: &str, name: &str, category: &str, tags: Vec<&str>) -> AssetInfo {
    AssetInfo {
        path: path.into(),
        name: name.into(),
        file_type: "duf".into(),
        size: 1024,
        category: category.into(),
        subcategory: None,
        metadata: HashMap::new(),
        thumbnail_path: None,
        compatibility_base: vec![],
        dforce_enabled: false,
        asset_type_detail: None,
        vendor: None,
        tags: tags.into_iter().map(String::from).collect(),
        visual_properties: None,
        visual_description: None,
    }
}

#[test]
fn test_create_scene_workflow_has_no_placeholder_asset_paths() {
    let planner = Planner::new();
    let goal = make_goal("Create a full scene", Intent::CreateScene, vec![]);
    let plan = planner.plan_for_goal(&goal, &empty_context()).unwrap();

    for step in &plan.steps {
        if let Some(path) = step.action.args.get("path").and_then(|v| v.as_str()) {
            assert!(
                !path.is_empty(),
                "Step '{}' has empty path",
                step.description
            );
            assert!(
                !path.contains("TODO"),
                "Step '{}' has placeholder path: {}",
                step.description,
                path
            );
            assert!(
                !path.contains("unknown"),
                "Step '{}' has unknown path: {}",
                step.description,
                path
            );
        }
    }
}

#[test]
fn test_create_scene_workflow_uses_available_assets() {
    let planner = Planner::new();
    let goal = make_goal(
        "Create a full scene with clothing",
        Intent::CreateScene,
        vec![],
    );
    let mut context = empty_context();
    context.available_assets.push(asset(
        "/library/clothing/sleek_jacket.duf",
        "Sleek Jacket",
        "clothing",
        vec!["clothing", "outfit"],
    ));

    let plan = planner.plan_for_goal(&goal, &context).unwrap();
    assert!(
        plan.steps.iter().any(|step| {
            step.action.command == "load_asset"
                && step.action.args.get("path").and_then(|v| v.as_str())
                    == Some("/library/clothing/sleek_jacket.duf")
        }),
        "Scene workflow should load matching assets from planning context"
    );
}

#[test]
fn test_create_character_workflow() {
    let planner = Planner::new();
    let goal = make_goal(
        "create a genesis figure with clothing",
        Intent::LoadAsset,
        vec![Entity {
            entity_type: EntityType::Figure,
            value: "genesis9".into(),
            confidence: 0.9,
        }],
    );
    let plan = planner.plan_for_goal(&goal, &empty_context());
    assert!(plan.is_some(), "Should create character workflow");
    let plan = plan.unwrap();
    let cmds: Vec<&str> = plan
        .steps
        .iter()
        .map(|s| s.action.command.as_str())
        .collect();
    assert_eq!(
        cmds[0], "add_figure",
        "Character workflow starts with loading the figure"
    );
}

#[test]
fn test_setup_lighting_workflow() {
    let planner = Planner::new();
    let goal = make_goal("Setup lighting for the scene", Intent::CreateLight, vec![]);
    let plan = planner.plan_for_goal(&goal, &empty_context());
    assert!(plan.is_some(), "Should return a lighting plan");
    let plan = plan.unwrap();
    let light_steps: Vec<&PlanStep> = plan
        .steps
        .iter()
        .filter(|s| s.action.command == "add_node")
        .collect();
    assert!(
        light_steps.len() >= 3,
        "Should create at least 3 lights (key/fill/rim), got {}",
        light_steps.len()
    );
}

#[test]
fn test_pose_character_workflow() {
    let planner = Planner::new();
    let goal = make_goal("Pose the character naturally", Intent::ApplyPose, vec![]);
    let plan = planner.plan_for_goal(&goal, &empty_context());
    assert!(plan.is_some(), "Should return a posing plan");
    let plan = plan.unwrap();
    let pose_commands: Vec<&str> = plan
        .steps
        .iter()
        .map(|s| s.action.command.as_str())
        .collect();
    assert!(pose_commands.contains(&"apply_pose"), "Should apply a pose");
}

#[test]
fn test_render_still_workflow() {
    let planner = Planner::new();
    let goal = make_goal("Render the current scene", Intent::Render, vec![]);
    let plan = planner.plan_for_goal(&goal, &empty_context());
    assert!(plan.is_some(), "Should return a render plan");
    let plan = plan.unwrap();
    let cmds: Vec<&str> = plan
        .steps
        .iter()
        .map(|s| s.action.command.as_str())
        .collect();
    assert!(
        cmds.contains(&"render_preview"),
        "Should call render_preview"
    );
    assert!(
        cmds.contains(&"begin_undo_batch"),
        "Should wrap in undo batch"
    );
}

#[test]
fn test_animate_character_workflow() {
    let planner = Planner::new();
    let goal = make_goal("Animate the character walking", Intent::Animate, vec![]);
    let plan = planner.plan_for_goal(&goal, &empty_context());
    assert!(plan.is_some(), "Should return an animation plan");
    let plan = plan.unwrap();
    let cmds: Vec<&str> = plan
        .steps
        .iter()
        .map(|s| s.action.command.as_str())
        .collect();
    assert!(
        cmds.contains(&"set_timeline_range"),
        "Should set timeline range"
    );
}

#[test]
fn test_workflow_step_confidence() {
    let planner = Planner::new();
    let goal = make_goal("Create a scene with lighting", Intent::CreateScene, vec![]);
    let plan = planner.plan_for_goal(&goal, &empty_context());
    assert!(plan.is_some());
    let plan = plan.unwrap();
    for step in &plan.steps {
        assert!(
            step.confidence > 0.0,
            "Step '{}' should have confidence > 0",
            step.description
        );
        assert!(
            step.estimated_time_seconds > 0,
            "Step '{}' should have estimated time > 0",
            step.description
        );
    }
}

#[test]
fn test_goal_without_known_keywords_returns_plan() {
    let planner = Planner::new();
    let goal = make_goal("Do something interesting", Intent::Unknown, vec![]);
    let plan = planner.plan_for_goal(&goal, &empty_context());
    assert!(
        plan.is_none() || !plan.as_ref().unwrap().steps.is_empty(),
        "Should either return None or a plan with steps"
    );
}

#[test]
fn test_action_to_command_map_covers_all_action_types() {
    use std::collections::HashSet;
    let map = action_to_command_map();
    let mapped: HashSet<ActionType> = map.keys().cloned().collect();

    let all_types: HashSet<ActionType> = vec![
        ActionType::LoadAsset,
        ActionType::ApplyPose,
        ActionType::AdjustProperty,
        ActionType::CreateLight,
        ActionType::CreateCamera,
        ActionType::Render,
        ActionType::ChangeMaterial,
        ActionType::ExportScene,
        ActionType::RunSimulation,
        ActionType::UndoBatchBegin,
        ActionType::UndoBatchEnd,
        ActionType::AddFigure,
        ActionType::SetMorph,
        ActionType::SetExpression,
        ActionType::SetCamera,
        ActionType::SetRenderOptions,
        ActionType::SetMaterialTexture,
        ActionType::Animate,
        ActionType::RunScript,
        ActionType::SearchContent,
        ActionType::ListBones,
        ActionType::SetBoneTransform,
    ]
    .into_iter()
    .collect();

    let missing: Vec<&ActionType> = all_types.difference(&mapped).collect();
    assert!(
        missing.is_empty(),
        "Action types missing from command map: {:?}",
        missing
    );
}

#[test]
fn test_command_knowledge_base_has_all_commands() {
    use dazpilot_lib::mcp_client::get_command_schemas;
    let kb = CommandKnowledgeBase::new();
    let schemas = get_command_schemas();
    for schema in &schemas {
        let cmd = kb.get_command(schema.name);
        assert!(
            cmd.is_some(),
            "Command '{}' present in MCP schemas but missing from command knowledge base",
            schema.name
        );
    }
}

#[test]
fn test_command_knowledge_parameter_accuracy() {
    use dazpilot_lib::mcp_client::get_command_schemas;
    let kb = CommandKnowledgeBase::new();
    let schemas = get_command_schemas();
    for schema in &schemas {
        let cmd = kb
            .get_command(schema.name)
            .unwrap_or_else(|| panic!("Missing from knowledge base: {}", schema.name));
        let expected_params: std::collections::BTreeSet<&str> =
            schema.parameters.iter().map(|s| *s).collect();
        let actual_params: std::collections::BTreeSet<&str> =
            cmd.parameters.iter().map(|s| *s).collect();
        assert_eq!(
            actual_params, expected_params,
            "Parameter mismatch for '{}': knowledge has {:?}, schema has {:?}",
            schema.name, actual_params, expected_params
        );
    }
}

#[test]
fn test_high_risk_consistency() {
    use dazpilot_lib::mcp_client::get_command_schemas;
    let kb = CommandKnowledgeBase::new();
    let schemas = get_command_schemas();
    for schema in &schemas {
        let cmd = kb
            .get_command(schema.name)
            .unwrap_or_else(|| panic!("Missing: {}", schema.name));
        assert_eq!(
            cmd.high_risk, schema.high_risk,
            "High risk mismatch for '{}': knowledge says {}, schema says {}",
            schema.name, cmd.high_risk, schema.high_risk
        );
    }
}

#[test]
fn test_scene_workflow_commands_exist() {
    let kb = CommandKnowledgeBase::new();
    let cmds = kb.get_scene_workflow_commands();
    assert!(!cmds.is_empty(), "Should return scene workflow commands");
    let names: Vec<&str> = cmds.iter().map(|c| c.name).collect();
    assert!(names.contains(&"add_figure"), "Should include add_figure");
    assert!(names.contains(&"set_light"), "Should include set_light");
    assert!(
        names.contains(&"render_preview"),
        "Should include render_preview"
    );
}

#[test]
fn test_find_commands_for_purpose() {
    let kb = CommandKnowledgeBase::new();
    let results = kb.find_commands_for_purpose("light");
    assert!(!results.is_empty(), "Should find lighting-related commands");
    let names: Vec<&str> = results.iter().map(|c| c.name).collect();
    assert!(names.contains(&"set_light"), "Should find set_light");
    assert!(
        names.contains(&"add_node"),
        "Should find add_node (for creating lights)"
    );
}

#[test]
fn test_build_command_catalog_prompt_succeeds() {
    let kb = CommandKnowledgeBase::new();
    let prompt = kb.build_command_catalog_prompt();
    assert!(
        prompt.contains("Available Daz Bridge commands:"),
        "Prompt should have header"
    );
    assert!(
        prompt.contains("add_figure"),
        "Prompt should include add_figure"
    );
    assert!(
        prompt.contains("set_morph"),
        "Prompt should include set_morph"
    );
    assert!(
        prompt.contains("render_preview"),
        "Prompt should include render_preview"
    );
    assert!(
        prompt.contains("[HIGH RISK]"),
        "Prompt should mark high-risk commands"
    );
}
