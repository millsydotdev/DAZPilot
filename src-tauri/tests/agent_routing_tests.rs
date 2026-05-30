use dazpilot_lib::agents::registry;
use dazpilot_lib::agents::{execute_agent, register_default_agents, AgentRequest};
use std::sync::Once;

static REGISTER_AGENTS: Once = Once::new();

fn ensure_agents() {
    REGISTER_AGENTS.call_once(register_default_agents);
}

#[test]
fn capability_matching_uses_word_boundaries() {
    assert!(registry::input_matches_capability(
        "add a warm rim light",
        "light"
    ));
    assert!(registry::input_matches_capability(
        "apply pose",
        "apply pose"
    ));
    assert!(!registry::input_matches_capability(
        "make a slight adjustment",
        "light"
    ));
    assert!(!registry::input_matches_capability(
        "this is supposedly done",
        "pose"
    ));
}

#[test]
fn agents_register_without_hanging() {
    ensure_agents();
    let count = registry::with_registry(|reg| reg.len());
    assert!(
        count >= 14,
        "Expected default agents to register, got {count}"
    );
}

#[test]
fn material_agent_handles_prompt_directly() {
    ensure_agents();
    let response = execute_agent(AgentRequest::new(
        "material",
        "set the skin material roughness",
    ));
    assert!(response.success, "{response:?}");
}

#[test]
fn asset_selection_routes_material_prompt_to_child() {
    ensure_agents();
    let response = execute_agent(AgentRequest::new(
        "asset_selection",
        "set the skin material roughness",
    ));
    assert!(response.success, "{response:?}");
}

#[test]
fn task_planner_routes_material_prompts_to_material_sub_agent() {
    ensure_agents();

    let response = execute_agent(AgentRequest::new(
        "task_planner",
        "set the skin material roughness",
    ));

    assert!(response.success, "{response:?}");
    assert!(
        response
            .actions
            .iter()
            .any(|action| action.command == "set_material_property"),
        "Expected material property action, got {:?}",
        response.actions
    );
    assert!(
        response
            .result
            .as_deref()
            .unwrap_or_default()
            .contains("material:"),
        "Expected formatted material routing result, got {:?}",
        response.result
    );
}

#[test]
fn task_planner_routes_camera_prompts_through_render_parent() {
    ensure_agents();

    let response = execute_agent(AgentRequest::new(
        "task_planner",
        "set a portrait camera view",
    ));

    assert!(response.success, "{response:?}");
    assert!(
        response
            .actions
            .iter()
            .any(|action| action.command == "set_camera"),
        "Expected camera action, got {:?}",
        response.actions
    );
    assert!(
        response
            .result
            .as_deref()
            .unwrap_or_default()
            .contains("camera:"),
        "Expected formatted camera routing result, got {:?}",
        response.result
    );
}

#[test]
fn capability_matching_handles_synonyms_and_plurals() {
    assert!(registry::input_matches_capability(
        "configure the lighting setup",
        "light"
    ));
    assert!(registry::input_matches_capability(
        "start rendering a preview",
        "render"
    ));
    assert!(registry::input_matches_capability(
        "apply a facial expression",
        "morph"
    ));
}

#[test]
fn unrelated_prompts_do_not_trigger_render_from_substrings() {
    ensure_agents();

    let response = execute_agent(AgentRequest::new(
        "task_planner",
        "make a slight adjustment to the selected node",
    ));

    assert!(response.success, "{response:?}");
    assert!(
        response
            .actions
            .iter()
            .all(|action| action.command != "set_light"),
        "Substring match routed to lighting unexpectedly: {:?}",
        response.actions
    );
}
