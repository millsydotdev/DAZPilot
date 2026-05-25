use crate::agents::{AgentAction, AgentRequest, AgentResponse};

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    let mut actions = vec![];
    let mut messages: Vec<String> = vec![];

    // Helper: extract first number from string
    fn extract_number(s: &str) -> Option<i32> {
        let num_str: String = s
            .chars()
            .skip_while(|c| !c.is_ascii_digit())
            .take_while(|c| c.is_ascii_digit())
            .collect();
        num_str.parse().ok()
    }

    // Play
    if input.contains("play") && !input.contains("pose") {
        actions.push(AgentAction {
            action_type: "timeline_play".to_string(),
            command: "play_timeline".to_string(),
            args: vec![],
        });
        messages.push("Playing animation timeline.".to_string());
    }

    // Pause
    if input.contains("pause") || input.contains("freeze") || input.contains("hold") {
        actions.push(AgentAction {
            action_type: "timeline_pause".to_string(),
            command: "pause_timeline".to_string(),
            args: vec![],
        });
        messages.push("Pausing animation timeline.".to_string());
    }

    // Stop
    if input.contains("stop") && !input.contains("pose") {
        actions.push(AgentAction {
            action_type: "timeline_stop".to_string(),
            command: "stop_timeline".to_string(),
            args: vec![],
        });
        messages.push("Stopping animation timeline and resetting to frame 0.".to_string());
    }

    // Go to frame N
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

    // Set range N to M
    if (input.contains("set range")
        || input.contains("set timeline")
        || input.contains("play range"))
        && input.contains("to")
    {
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
                "Setting timeline range from frame {} to {}.",
                numbers[0], numbers[1]
            ));
        }
    }

    // Apply pose [name] to [figure]
    if input.contains("apply pose") || input.contains("pose ") {
        let words: Vec<&str> = input.split_whitespace().collect();
        // Try to extract pose name and figure from the request
        let pose_arg = words
            .iter()
            .position(|w| *w == "pose")
            .and_then(|i| words.get(i + 1))
            .map(|s| s.to_string())
            .unwrap_or_else(|| "requested".to_string());
        let figure_arg = words
            .iter()
            .position(|w| *w == "to" || *w == "on")
            .and_then(|i| words.get(i + 1))
            .map(|s| s.to_string());

        let mut args = vec![pose_arg];
        if let Some(fig) = figure_arg {
            args.push(fig);
        }
        actions.push(AgentAction {
            action_type: "apply_pose".to_string(),
            command: "apply_pose".to_string(),
            args,
        });
        messages.push("Applying pose to figure.".to_string());
    }

    // Run dForce on [node]
    if (input.contains("dforce") || input.contains("d-force") || input.contains("physics sim"))
        && (input.contains("on") || input.contains("run") || input.contains("simulate"))
    {
        let node_arg = input
            .split_whitespace()
            .rfind(|w| {
                *w != "on"
                    && *w != "run"
                    && *w != "simulate"
                    && *w != "dforce"
                    && *w != "d-force"
                    && *w != "physics"
                    && *w != "sim"
            })
            .map(|s| s.to_string())
            .unwrap_or_else(|| "selected".to_string());

        // Try to extract frame numbers
        let numbers: Vec<i32> = input
            .split_whitespace()
            .filter_map(|w| w.parse::<i32>().ok())
            .collect();
        let start = numbers.first().copied().unwrap_or(1);
        let end = numbers.get(1).copied().unwrap_or(30);

        actions.push(AgentAction {
            action_type: "run_dforce".to_string(),
            command: "run_dforce_simulation".to_string(),
            args: vec![node_arg.clone(), start.to_string(), end.to_string()],
        });
        messages.push(format!(
            "Running dForce simulation on {} (frames {}–{}).",
            node_arg, start, end
        ));
    }

    // Set keyframe
    if input.contains("set keyframe") || input.contains("keyframe") {
        let numbers: Vec<f32> = input
            .split_whitespace()
            .filter_map(|w| w.parse::<f32>().ok())
            .collect();
        let frame = numbers.first().copied().unwrap_or(0.0);
        let value = numbers.get(1).copied().unwrap_or(0.0);

        // Try to extract property name
        let prop = if input.contains("rotation") || input.contains("rot") {
            "yRot"
        } else if input.contains("position") || input.contains("pos") || input.contains("translate")
        {
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
        messages.push(format!(
            "Setting keyframe for {} at frame {} to {}.",
            prop, frame, value
        ));
    }

    if actions.is_empty() {
        return AgentResponse {
            success: false,
            result: None,
            error: Some("No animation intents identified.".to_string()),
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
