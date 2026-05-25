use crate::agents::{AgentAction, AgentRequest, AgentResponse};

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    let mut actions = vec![];
    let mut messages = vec![];

    if input.contains("add") && input.contains("light") {
        let light_type = if input.contains("point") {
            "point_light"
        } else if input.contains("spot") {
            "spot_light"
        } else if input.contains("distant") || input.contains("sun") {
            "distant_light"
        } else if input.contains("area") || input.contains("panel") {
            "area_light"
        } else if input.contains("rim") || input.contains("back") {
            "distant_light"
        } else {
            "point_light"
        };
        let name = format!("AI_{}", light_type);
        actions.push(AgentAction {
            action_type: "add_light".to_string(),
            command: "add_node".to_string(),
            args: vec![light_type.to_string(), name],
        });
        messages.push(format!("Adding {} light.", light_type));
    }

    if input.contains("remove") && input.contains("light")
        || input.contains("delete") && input.contains("light")
    {
        let target = if input.contains("all") || input.contains("every") {
            "all".to_string()
        } else {
            "selected".to_string()
        };
        actions.push(AgentAction {
            action_type: "remove_light".to_string(),
            command: "delete_node".to_string(),
            args: vec![target],
        });
        messages.push("Removing light.".to_string());
    }

    if input.contains("bright") || input.contains("intensity") || input.contains("dim") {
        let is_bright = input.contains("bright") || input.contains("intense");
        let value = if is_bright { "2.0" } else { "0.5" };
        actions.push(AgentAction {
            action_type: "light_intensity".to_string(),
            command: "set_light".to_string(),
            args: vec![
                "selected".to_string(),
                "Intensity".to_string(),
                value.to_string(),
            ],
        });
        messages.push(if is_bright {
            "Increasing light intensity.".to_string()
        } else {
            "Dimming light intensity.".to_string()
        });
    }

    if input.contains("warm")
        || input.contains("cool")
        || input.contains("color")
        || input.contains("colour")
    {
        if input.contains("warm") {
            actions.push(AgentAction {
                action_type: "light_color".to_string(),
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
                action_type: "light_color".to_string(),
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

    if input.contains("enable") && input.contains("light")
        || input.contains("turn on") && input.contains("light")
    {
        actions.push(AgentAction {
            action_type: "light_enable".to_string(),
            command: "set_light".to_string(),
            args: vec![
                "selected".to_string(),
                "enable".to_string(),
                "true".to_string(),
            ],
        });
        messages.push("Enabling light.".to_string());
    }

    if input.contains("disable") && input.contains("light")
        || input.contains("turn off") && input.contains("light")
    {
        actions.push(AgentAction {
            action_type: "light_disable".to_string(),
            command: "set_light".to_string(),
            args: vec![
                "selected".to_string(),
                "enable".to_string(),
                "false".to_string(),
            ],
        });
        messages.push("Disabling light.".to_string());
    }

    if actions.is_empty() {
        return AgentResponse {
            success: false,
            result: None,
            error: Some("No lighting intents identified.".to_string()),
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
