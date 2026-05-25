use crate::agents::{AgentAction, AgentRequest, AgentResponse};

fn extract_number(s: &str) -> Option<i32> {
    let num_str: String = s
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    num_str.parse().ok()
}

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    let mut actions = vec![];
    let mut messages = vec![];

    if input.contains("play") && !input.contains("pose") {
        actions.push(AgentAction {
            action_type: "timeline_play".to_string(),
            command: "play_timeline".to_string(),
            args: vec![],
        });
        messages.push("Playing animation timeline.".to_string());
    }

    if input.contains("pause") || input.contains("freeze") {
        actions.push(AgentAction {
            action_type: "timeline_pause".to_string(),
            command: "pause_timeline".to_string(),
            args: vec![],
        });
        messages.push("Pausing animation timeline.".to_string());
    }

    if input.contains("stop") && !input.contains("pose") {
        actions.push(AgentAction {
            action_type: "timeline_stop".to_string(),
            command: "stop_timeline".to_string(),
            args: vec![],
        });
        messages.push("Stopping animation timeline.".to_string());
    }

    if (input.contains("go to") || input.contains("jump to") || input.contains("seek"))
        && input.contains("frame")
    {
        if let Some(frame) = extract_number(&input) {
            actions.push(AgentAction {
                action_type: "seek_frame".to_string(),
                command: "seek_to_frame".to_string(),
                args: vec![frame.to_string()],
            });
            messages.push(format!("Seeking to frame {}.", frame));
        }
    }

    if (input.contains("set range") || input.contains("play range")) && input.contains("to") {
        let numbers: Vec<i32> = input
            .split_whitespace()
            .filter_map(|w| w.parse::<i32>().ok())
            .collect();
        if numbers.len() >= 2 {
            actions.push(AgentAction {
                action_type: "set_range".to_string(),
                command: "set_timeline_range".to_string(),
                args: vec![numbers[0].to_string(), numbers[1].to_string()],
            });
            messages.push(format!(
                "Setting timeline range to {}–{}.",
                numbers[0], numbers[1]
            ));
        }
    }

    if input.contains("set keyframe") || input.contains("keyframe") {
        let numbers: Vec<f32> = input
            .split_whitespace()
            .filter_map(|w| w.parse::<f32>().ok())
            .collect();
        let frame = numbers.first().copied().unwrap_or(0.0);
        let value = numbers.get(1).copied().unwrap_or(0.0);
        let prop = if input.contains("rotation") || input.contains("rot") {
            "yRot"
        } else if input.contains("position") || input.contains("translate") {
            "yTranslate"
        } else {
            "value"
        };
        actions.push(AgentAction {
            action_type: "set_keyframe".to_string(),
            command: "set_keyframe".to_string(),
            args: vec![
                "selected".to_string(),
                prop.to_string(),
                frame.to_string(),
                value.to_string(),
                "linear".to_string(),
            ],
        });
        messages.push(format!("Setting keyframe for {} at frame {}.", prop, frame));
    }

    if input.contains("loop") {
        let should_loop =
            !input.contains("stop") && !input.contains("off") && !input.contains("disable");
        actions.push(AgentAction {
            action_type: "toggle_loop".to_string(),
            command: "toggle_loop".to_string(),
            args: vec![if should_loop { "true" } else { "false" }.to_string()],
        });
        messages.push(if should_loop {
            "Enabling timeline loop.".to_string()
        } else {
            "Disabling timeline loop.".to_string()
        });
    }

    if actions.is_empty() {
        return AgentResponse {
            success: false,
            result: None,
            error: Some("No timeline intents identified.".to_string()),
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
