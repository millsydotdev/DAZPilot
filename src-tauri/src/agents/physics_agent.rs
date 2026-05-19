//! Physics Agent: configures and runs dForce simulations.

use crate::agents::{AgentRequest, AgentResponse, AgentAction};

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    let mut actions = vec![];
    let mut messages = vec![];

    if input.contains("simulate") || input.contains("dforce") || input.contains("physics") {
        let mut start_frame = "0".to_string();
        let mut end_frame = "30".to_string();
        
        // Very basic frame extraction
        let words: Vec<&str> = input.split_whitespace().collect();
        for i in 0..words.len() {
            if words[i] == "from" && i + 1 < words.len() {
                start_frame = words[i+1].trim_matches(|c: char| !c.is_numeric()).to_string();
            }
            if words[i] == "to" && i + 1 < words.len() {
                end_frame = words[i+1].trim_matches(|c: char| !c.is_numeric()).to_string();
            }
        }

        actions.push(AgentAction {
            action_type: "simulation".to_string(),
            command: "run_dforce_simulation".to_string(),
            args: vec!["selected".to_string(), start_frame, end_frame],
        });
        messages.push("Configuring and starting dForce physics simulation.");
    }

    if actions.is_empty() {
        return AgentResponse {
            success: false,
            result: None,
            error: Some("No physics/simulation intents identified.".to_string()),
            actions: vec![],
        };
    }

    AgentResponse {
        success: true,
        result: Some(messages.join(" ")),
        error: None,
        actions,
    }
}
