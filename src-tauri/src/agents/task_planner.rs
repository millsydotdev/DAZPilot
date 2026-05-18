use crate::agents::{AgentRequest, AgentResponse, AgentAction};

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    let actions = parse_and_create_actions(&input);
    
    AgentResponse {
        success: true,
        result: Some(format!("Decomposed into {} action(s)", actions.len())),
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
    
    if input.contains("load") || input.contains("apply") {
        if input.contains("pose") {
            actions.push(AgentAction {
                action_type: "apply_pose".to_string(),
                command: "apply_pose".to_string(),
                args: vec![],
            });
        } else if input.contains("asset") || input.contains("cloth") || input.contains("clothes") {
            actions.push(AgentAction {
                action_type: "load_asset".to_string(),
                command: "load_asset".to_string(),
                args: vec![],
            });
        }
    }
    
    if input.contains("render") {
        actions.push(AgentAction {
            action_type: "render".to_string(),
            command: "render_preview".to_string(),
            args: vec![],
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