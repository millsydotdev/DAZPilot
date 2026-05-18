use crate::agents::{AgentRequest, AgentResponse, AgentAction};

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    let actions = parse_animation_commands(&input);
    
    AgentResponse {
        success: true,
        result: Some(format!("Animation: {} action(s) planned", actions.len())),
        error: None,
        actions,
    }
}

fn parse_animation_commands(input: &str) -> Vec<AgentAction> {
    let mut actions = vec![];
    
    if input.contains("pose") {
        actions.push(AgentAction {
            action_type: "apply_pose".to_string(),
            command: "apply_pose".to_string(),
            args: vec![],
        });
    }
    
    if input.contains("key") || input.contains("animate") || input.contains("animation") {
        actions.push(AgentAction {
            action_type: "set_keyframe".to_string(),
            command: "set_keyframe".to_string(),
            args: vec!["current".to_string()],
        });
    }
    
    if input.contains("play") || input.contains("timeline") {
        actions.push(AgentAction {
            action_type: "play_timeline".to_string(),
            command: "play_timeline".to_string(),
            args: vec![],
        });
    }
    
    if input.contains("bounce") || input.contains("spring") || input.contains("physics") {
        actions.push(AgentAction {
            action_type: "enable_physics".to_string(),
            command: "enable_physics".to_string(),
            args: vec![],
        });
    }
    
    actions
}