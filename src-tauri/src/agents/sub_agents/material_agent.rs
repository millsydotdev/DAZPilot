use crate::agents::{AgentAction, AgentRequest, AgentResponse};

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    let mut actions = vec![];
    let mut messages = vec![];

    if input.contains("material")
        || input.contains("texture")
        || input.contains("shader")
        || input.contains("surface")
    {
        let channels = vec![
            ("base color", "Base Color"),
            ("diffuse", "Base Color"),
            ("albedo", "Base Color"),
            ("roughness", "Roughness"),
            ("smoothness", "Roughness"),
            ("metallic", "Metallic"),
            ("metalness", "Metallic"),
            ("opacity", "Opacity"),
            ("transparency", "Opacity"),
            ("normal", "Normal"),
            ("bump", "Normal"),
            ("displacement", "Displacement"),
            ("height", "Displacement"),
            ("ambient occlusion", "Ambient Occlusion"),
            ("ao", "Ambient Occlusion"),
            ("emission", "Emission"),
            ("glow", "Emission"),
            ("specular", "Specular"),
            ("reflection", "Specular"),
        ];

        let mut found_channel = String::new();
        for (keyword, channel) in &channels {
            if input.contains(keyword) {
                found_channel = channel.to_string();
                break;
            }
        }

        if !found_channel.is_empty() && input.contains("set") {
            if input.contains("texture")
                || input.contains("map")
                || input.contains("file")
                || input.contains("image")
            {
                actions.push(AgentAction {
                    action_type: "set_material_texture".to_string(),
                    command: "set_material_texture".to_string(),
                    args: vec!["selected".to_string(), found_channel.clone(), String::new()],
                });
                messages.push(format!("Setting {} texture channel.", found_channel));
            } else {
                let value = if input.contains("white") || input.contains("light") {
                    "1.0".to_string()
                } else if input.contains("black") || input.contains("dark") {
                    "0.0".to_string()
                } else if input.contains("red") {
                    "255,0,0".to_string()
                } else if input.contains("green") {
                    "0,255,0".to_string()
                } else if input.contains("blue") {
                    "0,0,255".to_string()
                } else {
                    "0.5".to_string()
                };
                actions.push(AgentAction {
                    action_type: "set_material_property".to_string(),
                    command: "set_material_property".to_string(),
                    args: vec!["selected".to_string(), found_channel.clone(), value.clone()],
                });
                messages.push(format!("Setting {} to {}.", found_channel, value));
            }
        }

        if input.contains("reset") || input.contains("default") || input.contains("clear") {
            actions.push(AgentAction {
                action_type: "reset_material".to_string(),
                command: "set_material_property".to_string(),
                args: vec![
                    "selected".to_string(),
                    "Base Color".to_string(),
                    "255,255,255".to_string(),
                ],
            });
            messages.push("Resetting material to defaults.".to_string());
        }
    }

    if actions.is_empty() {
        return AgentResponse {
            success: false,
            result: None,
            error: Some("No material or texture intents identified.".to_string()),
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
