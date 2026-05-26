use crate::agents::orchestrator;
use crate::agents::{AgentAction, AgentRequest, AgentResponse};

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    let mut actions = vec![];
    let mut messages: Vec<String> = vec![];

    let child_response = orchestrator::delegate_and_aggregate("render", &input, request.clone());
    if !child_response.actions.is_empty() {
        actions.extend(child_response.actions);
        if let Some(result) = child_response.result {
            messages.push(result);
        }
    }

    if input.contains("render") {
        if input.contains("4k") || input.contains("high res") {
            actions.push(AgentAction {
                action_type: "render_config".to_string(),
                command: "set_render_settings".to_string(),
                args: vec!["3840".to_string(), "2160".to_string()],
            });
            messages.push("Setting render resolution to 4K.".to_string());
        } else if input.contains("1080") || input.contains("full hd") {
            actions.push(AgentAction {
                action_type: "render_config".to_string(),
                command: "set_render_settings".to_string(),
                args: vec!["1920".to_string(), "1080".to_string()],
            });
            messages.push("Setting render resolution to 1080p.".to_string());
        }

        actions.push(AgentAction {
            action_type: "render_preview".to_string(),
            command: "render_preview".to_string(),
            args: vec![],
        });
        messages.push("Triggering render preview.".to_string());
    }

    if input.contains("light") || input.contains("lighting") {
        if input.contains("bright") || input.contains("intense") {
            actions.push(AgentAction {
                action_type: "lighting_adj".to_string(),
                command: "set_light".to_string(),
                args: vec![
                    "selected".to_string(),
                    "Intensity".to_string(),
                    "2.0".to_string(),
                ],
            });
            messages.push("Increasing light intensity.".to_string());
        } else if input.contains("dim") || input.contains("soft") {
            actions.push(AgentAction {
                action_type: "lighting_adj".to_string(),
                command: "set_light".to_string(),
                args: vec![
                    "selected".to_string(),
                    "Intensity".to_string(),
                    "0.5".to_string(),
                ],
            });
            messages.push("Dimming light intensity.".to_string());
        }

        if input.contains("warm") {
            actions.push(AgentAction {
                action_type: "lighting_color".to_string(),
                command: "set_light".to_string(),
                args: vec![
                    "selected".to_string(),
                    "Color".to_string(),
                    "255,200,150".to_string(),
                ],
            });
            messages.push("Setting warm light color.".to_string());
        } else if input.contains("cool") || input.contains("blue") {
            actions.push(AgentAction {
                action_type: "lighting_color".to_string(),
                command: "set_light".to_string(),
                args: vec![
                    "selected".to_string(),
                    "Color".to_string(),
                    "150,200,255".to_string(),
                ],
            });
            messages.push("Setting cool light color.".to_string());
        }
    }

    if messages.is_empty() {
        return AgentResponse {
            success: false,
            result: None,
            error: Some("No render or lighting intents identified.".to_string()),
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
