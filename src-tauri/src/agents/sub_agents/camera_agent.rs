use crate::agents::{AgentAction, AgentRequest, AgentResponse};

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    let mut actions = vec![];
    let mut messages = vec![];

    if input.contains("camera") || input.contains("view") || input.contains("shot") {
        let camera_name =
            if input.contains("front") || input.contains("face") || input.contains("portrait") {
                "Front View"
            } else if input.contains("side") || input.contains("profile") {
                "Side View"
            } else if input.contains("top") || input.contains("above") || input.contains("overhead")
            {
                "Top View"
            } else if input.contains("three quarter")
                || input.contains("3 quarter")
                || input.contains("3/4")
            {
                "Perspective View"
            } else if input.contains("back") || input.contains("rear") {
                "Back View"
            } else if input.contains("left") {
                "Left View"
            } else if input.contains("right") {
                "Right View"
            } else {
                "Perspective View"
            };

        let focal_length = if input.contains("wide")
            || input.contains("landscape")
            || input.contains("environment")
        {
            "35"
        } else if input.contains("portrait") || input.contains("face") || input.contains("headshot")
        {
            "85"
        } else if input.contains("macro") || input.contains("close") || input.contains("detail") {
            "100"
        } else if input.contains("tele") || input.contains("zoom") || input.contains("far") {
            "200"
        } else {
            "50"
        };

        let focal_distance = if input.contains("close") || input.contains("near") {
            "100"
        } else if input.contains("far") || input.contains("distant") {
            "500"
        } else {
            "200"
        };

        actions.push(AgentAction {
            action_type: "set_camera".to_string(),
            command: "set_camera".to_string(),
            args: vec![
                camera_name.to_string(),
                focal_length.to_string(),
                focal_distance.to_string(),
            ],
        });
        messages.push(format!(
            "Setting camera to {} with {}mm lens.",
            camera_name, focal_length
        ));
    }

    if actions.is_empty() {
        return AgentResponse {
            success: false,
            result: None,
            error: Some("No camera intents identified.".to_string()),
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
