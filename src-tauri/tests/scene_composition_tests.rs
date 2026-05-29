use dazpilot_lib::agents::register_default_agents;
use dazpilot_lib::agents::{AgentAction, AgentRequest};
use dazpilot_lib::tools::ToolResponse;
use serial_test::serial;
use std::sync::Once;

static REGISTER_AGENTS: Once = Once::new();

fn ensure_agents() {
    REGISTER_AGENTS.call_once(register_default_agents);
}

#[test]
fn scene_composer_plans_fantasy_scene() {
    ensure_agents();
    let response = dazpilot_lib::agents::execute_agent(AgentRequest::new(
        "scene_composer",
        "create a fantasy wizard tower scene",
    ));
    assert!(response.success, "Agent failed: {:?}", response.error);
    assert!(!response.actions.is_empty(), "Expected at least one action");
    let commands: Vec<&str> = response
        .actions
        .iter()
        .map(|a| a.command.as_str())
        .collect();
    assert!(
        commands.contains(&"begin_undo_batch"),
        "Expected undo batch start, got {:?}",
        commands
    );
    assert!(
        commands.contains(&"add_figure") || commands.contains(&"load_asset"),
        "Expected figure/asset loading, got {:?}",
        commands
    );
    assert!(
        commands.contains(&"accept_undo_batch"),
        "Expected undo batch end, got {:?}",
        commands
    );
}

#[test]
fn scene_composer_plans_sci_fi_scene() {
    ensure_agents();
    let response = dazpilot_lib::agents::execute_agent(AgentRequest::new(
        "scene_composer",
        "build a sci-fi spaceship interior scene",
    ));
    assert!(response.success, "Agent failed: {:?}", response.error);
    assert!(!response.actions.is_empty(), "Expected at least one action");
    let commands: Vec<&str> = response
        .actions
        .iter()
        .map(|a| a.command.as_str())
        .collect();
    assert!(
        commands.contains(&"begin_undo_batch"),
        "Expected undo batch start, got {:?}",
        commands
    );
    assert!(
        commands.contains(&"add_node"),
        "Expected lighting setup, got {:?}",
        commands
    );
}

#[test]
fn scene_composer_handles_empty_input_gracefully() {
    ensure_agents();
    let response = dazpilot_lib::agents::execute_agent(AgentRequest::new("scene_composer", ""));
    // Agent should either succeed with a plan or return a helpful error
    if !response.success {
        assert!(
            response.error.is_some(),
            "Expected error message on empty input"
        );
    }
}

#[test]
fn scene_composer_generates_at_least_fourteen_steps() {
    ensure_agents();
    let response =
        dazpilot_lib::agents::execute_agent(AgentRequest::new("scene_composer", "create a scene"));
    assert!(response.success, "{:?}", response.error);
    assert!(
        response.actions.len() >= 14,
        "Create Scene workflow should have 14+ steps, got {}: {:?}",
        response.actions.len(),
        response
            .actions
            .iter()
            .map(|a| a.command.as_str())
            .collect::<Vec<&str>>()
    );
}

#[test]
fn convert_agent_action_to_structured_simple() {
    let action = AgentAction {
        action_type: "AddFigure".to_string(),
        command: "add_figure".to_string(),
        args: vec!["genesis9".to_string()],
    };
    let result = dazpilot_lib::tools::scene_composition::test_helpers::convert_action(&action);
    assert!(result.is_some(), "Conversion should succeed");
    let sa = result.unwrap();
    assert_eq!(sa.command, "add_figure");
}

#[test]
fn convert_agent_action_to_structured_with_args() {
    let action = AgentAction {
        action_type: "SetLight".to_string(),
        command: "set_light".to_string(),
        args: vec![
            "Spot".to_string(),
            "Intensity".to_string(),
            "200%".to_string(),
        ],
    };
    let result = dazpilot_lib::tools::scene_composition::test_helpers::convert_action(&action);
    assert!(result.is_some(), "Conversion should succeed");
    let sa = result.unwrap();
    assert_eq!(sa.command, "set_light");
}

#[test]
fn generate_refinement_actions_with_lighting_suggestion() {
    let analysis = ToolResponse::ok_with_message(
        "analyze_scene_balance",
        serde_json::json!({
            "score": 4.5,
            "score_label": "Needs work",
            "suggestions": [
                "Add more lighting to brighten the scene",
                "Consider repositioning the main subject"
            ],
        }),
        "Score: 4.5/10",
    );
    let result =
        dazpilot_lib::tools::scene_composition::test_helpers::generate_refinements(&analysis);
    assert!(
        result.is_some(),
        "Should generate refinement for lighting suggestion"
    );
    let actions = result.unwrap();
    assert_eq!(actions.len(), 1, "Should generate exactly one action");
    assert_eq!(actions[0].action.command, "set_light");
}

#[test]
fn generate_refinement_actions_no_suggestions() {
    let analysis = ToolResponse::ok_with_message(
        "analyze_scene_balance",
        serde_json::json!({
            "score": 8.5,
            "score_label": "Good",
            "suggestions": [],
        }),
        "Score: 8.5/10",
    );
    let result =
        dazpilot_lib::tools::scene_composition::test_helpers::generate_refinements(&analysis);
    assert!(result.is_none(), "Should not refine if no suggestions");
}

#[test]
fn generate_refinement_actions_non_lighting_suggestion() {
    let analysis = ToolResponse::ok_with_message(
        "analyze_scene_balance",
        serde_json::json!({
            "score": 5.0,
            "suggestions": [
                "Position the subject off-center following rule of thirds"
            ],
        }),
        "Score: 5.0/10",
    );
    let result =
        dazpilot_lib::tools::scene_composition::test_helpers::generate_refinements(&analysis);
    assert!(
        result.is_none(),
        "Should not refine for non-lighting suggestions"
    );
}

// --- Session tests ---

#[test]
#[serial]
fn session_creates_and_stores_state() {
    dazpilot_lib::tools::scene_composition::test_helpers::clear_session();

    let steps = vec![
        dazpilot_lib::agents::scene_composer::CompositionStep {
            description: "Add figure".to_string(),
            action: AgentAction {
                action_type: "AddFigure".to_string(),
                command: "add_figure".to_string(),
                args: vec!["genesis9".to_string()],
            },
        },
        dazpilot_lib::agents::scene_composer::CompositionStep {
            description: "Set lighting".to_string(),
            action: AgentAction {
                action_type: "SetLight".to_string(),
                command: "set_light".to_string(),
                args: vec![
                    "Spot".to_string(),
                    "Intensity".to_string(),
                    "100%".to_string(),
                ],
            },
        },
    ];

    let session = dazpilot_lib::tools::scene_composition::test_helpers::create_test_session(
        "test scene",
        steps,
    );

    assert_eq!(session.session_id, "test_session");
    assert_eq!(session.description, "test scene");
    assert_eq!(session.steps.len(), 2);
    assert_eq!(session.current_index, 0);
    assert!(!session.completed);

    // Verify it was stored globally
    let stored = dazpilot_lib::tools::scene_composition::test_helpers::get_active_session();
    assert!(stored.is_some());
    assert_eq!(stored.unwrap().session_id, "test_session");

    dazpilot_lib::tools::scene_composition::test_helpers::clear_session();
}

#[test]
#[serial]
fn session_clear_removes_state() {
    dazpilot_lib::tools::scene_composition::test_helpers::clear_session();

    let steps = vec![dazpilot_lib::agents::scene_composer::CompositionStep {
        description: "Test step".to_string(),
        action: AgentAction {
            action_type: "Test".to_string(),
            command: "chat".to_string(),
            args: vec![],
        },
    }];

    dazpilot_lib::tools::scene_composition::test_helpers::create_test_session("test", steps);
    assert!(dazpilot_lib::tools::scene_composition::test_helpers::get_active_session().is_some());

    dazpilot_lib::tools::scene_composition::test_helpers::clear_session();
    assert!(dazpilot_lib::tools::scene_composition::test_helpers::get_active_session().is_none());
}

#[test]
#[serial]
fn handle_continue_composition_rejects_missing_session_id() {
    use dazpilot_lib::tools::ToolRequest;
    use std::collections::HashMap;

    let mut args = HashMap::new();
    args.insert("session_id".to_string(), serde_json::json!(""));
    let request = ToolRequest {
        tool_name: "continue_composition".to_string(),
        args,
    };
    let response =
        dazpilot_lib::tools::scene_composition::handle_execute_scene_composition_internal(request);
    assert!(!response.success);
    assert!(response.message.contains("session_id is required"));
}

#[test]
#[serial]
fn handle_continue_composition_rejects_missing_session() {
    dazpilot_lib::tools::scene_composition::test_helpers::clear_session();

    use dazpilot_lib::tools::ToolRequest;
    use std::collections::HashMap;

    let mut args = HashMap::new();
    args.insert("session_id".to_string(), serde_json::json!("nonexistent"));
    let request = ToolRequest {
        tool_name: "continue_composition".to_string(),
        args,
    };
    let response =
        dazpilot_lib::tools::scene_composition::handle_execute_scene_composition_internal(request);
    assert!(!response.success);
    assert!(response.message.contains("No composition session found"));
}

#[test]
#[serial]
fn handle_continue_composition_rejects_wrong_session_id() {
    dazpilot_lib::tools::scene_composition::test_helpers::clear_session();

    let steps = vec![dazpilot_lib::agents::scene_composer::CompositionStep {
        description: "Test".to_string(),
        action: AgentAction {
            action_type: "Test".to_string(),
            command: "chat".to_string(),
            args: vec![],
        },
    }];
    dazpilot_lib::tools::scene_composition::test_helpers::create_test_session("test", steps);

    use dazpilot_lib::tools::ToolRequest;
    use std::collections::HashMap;

    let mut args = HashMap::new();
    args.insert("session_id".to_string(), serde_json::json!("wrong_id"));
    let request = ToolRequest {
        tool_name: "continue_composition".to_string(),
        args,
    };
    let response =
        dazpilot_lib::tools::scene_composition::handle_execute_scene_composition_internal(request);
    assert!(!response.success);
    assert!(response.message.contains("does not match"));

    dazpilot_lib::tools::scene_composition::test_helpers::clear_session();
}

#[test]
#[serial]
fn handle_continue_composition_finishes_chat_session() {
    dazpilot_lib::tools::scene_composition::test_helpers::clear_session();

    let step = dazpilot_lib::agents::scene_composer::CompositionStep {
        description: "Test".to_string(),
        action: AgentAction {
            action_type: "Test".to_string(),
            command: "chat".to_string(),
            args: vec!["hello".to_string()],
        },
    };

    dazpilot_lib::tools::scene_composition::test_helpers::create_test_session(
        "test chat finish",
        vec![step],
    );

    use dazpilot_lib::tools::ToolRequest;
    use std::collections::HashMap;
    let mut args = HashMap::new();
    args.insert("session_id".to_string(), serde_json::json!("test_session"));
    let request = ToolRequest {
        tool_name: "continue_composition".to_string(),
        args,
    };
    let response =
        dazpilot_lib::tools::scene_composition::handle_execute_scene_composition_internal(request);
    assert!(
        response.success,
        "Continue should handle chat-only sessions: {:?}",
        response.message
    );

    dazpilot_lib::tools::scene_composition::test_helpers::clear_session();
}

#[test]
#[serial]
fn session_round_trip_via_create_and_retrieve() {
    dazpilot_lib::tools::scene_composition::test_helpers::clear_session();

    let steps = vec![
        dazpilot_lib::agents::scene_composer::CompositionStep {
            description: "Step 1".to_string(),
            action: AgentAction {
                action_type: "Chat".to_string(),
                command: "chat".to_string(),
                args: vec!["hello".to_string()],
            },
        },
        dazpilot_lib::agents::scene_composer::CompositionStep {
            description: "Step 2".to_string(),
            action: AgentAction {
                action_type: "Chat".to_string(),
                command: "chat".to_string(),
                args: vec!["world".to_string()],
            },
        },
    ];

    let session = dazpilot_lib::tools::scene_composition::test_helpers::create_test_session(
        "round trip",
        steps,
    );
    assert_eq!(session.steps.len(), 2);
    assert_eq!(session.current_index, 0);

    // Retrieve from global store
    let retrieved = dazpilot_lib::tools::scene_composition::test_helpers::get_active_session()
        .expect("Session should exist");
    assert_eq!(retrieved.description, "round trip");
    assert_eq!(retrieved.session_id, session.session_id);

    dazpilot_lib::tools::scene_composition::test_helpers::clear_session();
}
