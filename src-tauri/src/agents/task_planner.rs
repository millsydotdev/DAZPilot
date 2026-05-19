use crate::agents::{self, AgentRequest, AgentResponse, AgentAction};

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    
    // 1. Check for conflicts first as a proactive step
    let conflict_resp = agents::conflict_resolution::execute(request.clone());
    
    let mut actions = vec![];
    let mut results = vec![];

    if let Some(res) = conflict_resp.result {
        if !res.contains("No conflicts") {
            results.push(format!("Pre-task conflict check: {}", res));
            actions.extend(conflict_resp.actions);
        }
    }

    // 2. Delegate to specialized agents based on intent
    if input.contains("simulate") || input.contains("dforce") || input.contains("physics") {
        let resp = agents::physics_agent::execute(request.clone());
        if resp.success {
            results.push(resp.result.unwrap_or_default());
            actions.extend(resp.actions);
        }
    }

    if input.contains("load") || input.contains("apply") || input.contains("find") || input.contains("search") {
        let resp = agents::asset_selection::execute(request.clone());
        if resp.success {
            results.push(resp.result.unwrap_or_default());
            actions.extend(resp.actions);
        }
    }

    if input.contains("render") || input.contains("light") || input.contains("lighting") {
        let resp = agents::render_agent::execute(request.clone());
        if resp.success {
            results.push(resp.result.unwrap_or_default());
            actions.extend(resp.actions);
        }
    }

    if input.contains("pose") || input.contains("animation") || input.contains("timeline") {
        let resp = agents::animation_agent::execute(request.clone());
        if resp.success {
            results.push(resp.result.unwrap_or_default());
            actions.extend(resp.actions);
        }
    }

    // 3. Fallback to general parsing if no specialized agent took over or for other intents
    let general_actions = parse_and_create_actions(&input);
    for action in general_actions {
        // Avoid duplicates if a specialized agent already handled it
        if !actions.iter().any(|a| a.command == action.command) {
            actions.push(action);
        }
    }

    if results.is_empty() && !actions.is_empty() {
        results.push(format!("Decomposed into {} action(s)", actions.len()));
    }
    
    AgentResponse {
        success: true,
        result: Some(results.join("\n")),
        error: None,
        actions,
    }
}

fn parse_and_create_actions(input: &str) -> Vec<AgentAction> {
    let mut actions = vec![];
    
    if input.contains("select") && (input.contains("figure") || input.contains("node")) {
        actions.push(AgentAction {
            action_type: "select_node".to_string(),
            command: "select_node".to_string(),
            args: vec!["genesis_8_female".to_string()],
        });
    }
    
    if input.contains("create") {
        if input.contains("light") {
            actions.push(AgentAction {
                action_type: "create_light".to_string(),
                command: "create_light".to_string(),
                args: vec!["directional".to_string()],
            });
        } else if input.contains("camera") {
            actions.push(AgentAction {
                action_type: "create_camera".to_string(),
                command: "create_camera".to_string(),
                args: vec![],
            });
        }
    }
    
    if actions.is_empty() {
        actions.push(AgentAction {
            action_type: "chat".to_string(),
            command: "chat".to_string(),
            args: vec![input.to_string()],
        });
    }
    
    actions
}