//! Animation Agent: handles posing and timeline sequences.

use crate::agents::{AgentRequest, AgentResponse, AgentAction};

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    let mut actions = vec![];
    let mut messages = vec![];

    if input.contains("pose") || input.contains("animation") {
        if input.contains("sequence") || input.contains("play") {
            actions.push(AgentAction {
                action_type: "timeline_play".to_string(),
                command: "play_timeline".to_string(),
                args: vec![],
            });
            messages.push("Playing animation timeline.");
        } else if input.contains("reset") {
            actions.push(AgentAction {
                action_type: "timeline_reset".to_string(),
                command: "seek_to_frame".to_string(),
                args: vec!["0".to_string()],
            });
            messages.push("Resetting timeline to frame 0.");
        }
    }

    if actions.is_empty() {
        return AgentResponse {
            success: false,
            result: None,
            error: Some("No animation intents identified.".to_string()),
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
