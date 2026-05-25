use crate::agents::{AgentAction, AgentRequest, AgentResponse};

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    let mut actions = vec![];
    let mut messages = vec![];

    if input.contains("apply") && input.contains("pose") {
        let words: Vec<&str> = input.split_whitespace().collect();
        let pose_name = words
            .iter()
            .position(|w| *w == "pose")
            .and_then(|i| words.get(i + 1))
            .map(|s| s.to_string())
            .unwrap_or_else(|| "requested".to_string());
        let figure = words
            .iter()
            .position(|w| *w == "to" || *w == "on")
            .and_then(|i| words.get(i + 1))
            .map(|s| s.to_string());

        let mut args = vec![pose_name];
        if let Some(fig) = figure {
            args.push(fig);
        }
        actions.push(AgentAction {
            action_type: "apply_pose".to_string(),
            command: "apply_pose".to_string(),
            args,
        });
        messages.push("Applying pose to figure.");
    }

    if input.contains("search") && input.contains("pose") {
        let query: String = input
            .replace("search", "")
            .replace("pose", "")
            .replace("for", "")
            .trim()
            .to_string();
        let search_term = if query.is_empty() {
            "pose".to_string()
        } else {
            query
        };
        actions.push(AgentAction {
            action_type: "search_poses".to_string(),
            command: "search_content".to_string(),
            args: vec![search_term, "pose".to_string(), "10".to_string()],
        });
        messages.push("Searching pose library.");
    }

    if input.contains("list") && (input.contains("pose") || input.contains("poses")) {
        actions.push(AgentAction {
            action_type: "list_poses".to_string(),
            command: "get_pose_library".to_string(),
            args: vec![],
        });
        messages.push("Listing available poses.");
    }

    if actions.is_empty() {
        return AgentResponse {
            success: false,
            result: None,
            error: Some("No pose intents identified.".to_string()),
            actions: vec![],
            sub_results: vec![],
        };
    }

    AgentResponse {
        success: true,
        result: Some(messages.join(" ")),
        error: None,
        actions,
        sub_results: vec![],
    }
}
