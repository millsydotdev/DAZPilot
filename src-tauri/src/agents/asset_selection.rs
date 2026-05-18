use crate::agents::{AgentRequest, AgentResponse, AgentAction};

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    
    let category = detect_category(&input);
    let actions = find_matching_assets(&input, &category);
    
    AgentResponse {
        success: true,
        result: Some(format!("Found assets in category: {}", category)),
        error: None,
        actions,
    }
}

fn detect_category(input: &str) -> String {
    if input.contains("shirt") || input.contains("top") || input.contains("jacket") {
        "body_upper".to_string()
    } else if input.contains("pants") || input.contains("skirt") || input.contains("bottom") {
        "body_lower".to_string()
    } else if input.contains("shoe") || input.contains("boot") || input.contains("footwear") {
        "feet".to_string()
    } else if input.contains("hair") || input.contains("style") {
        "head_hair".to_string()
    } else if input.contains("pose") {
        "poses".to_string()
    } else if input.contains("morph") || input.contains("shape") {
        "shapes".to_string()
    } else if input.contains("skin") || input.contains("material") || input.contains("texture") {
        "materials".to_string()
    } else {
        "uncategorized".to_string()
    }
}

fn find_matching_assets(input: &str, category: &str) -> Vec<AgentAction> {
    let mut actions = vec![];
    
    if !input.contains("find") && !input.contains("list") && !input.contains("show") {
        actions.push(AgentAction {
            action_type: "search_assets".to_string(),
            command: "search_assets".to_string(),
            args: vec![category.to_string()],
        });
    }
    
    actions.push(AgentAction {
        action_type: "list_category".to_string(),
        command: "list_category".to_string(),
        args: vec![category.to_string()],
    });
    
    actions
}