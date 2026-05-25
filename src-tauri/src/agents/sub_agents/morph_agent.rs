use crate::agents::{AgentAction, AgentRequest, AgentResponse};

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    let mut actions = vec![];
    let mut messages = vec![];

    if input.contains("morph") || input.contains("shape") || input.contains("adjust") {
        let morph_names = vec![
            "Head_Height",
            "Head_Width",
            "Head_Depth",
            "Waist_Width",
            "Waist_Height",
            "Chest_Width",
            "Chest_Depth",
            "Hip_Width",
            "Hip_Depth",
            "Shoulder_Width",
            "Neck_Length",
            "Neck_Width",
            "Arm_Length",
            "Forearm_Length",
            "Leg_Length",
            "Calf_Length",
            "Eye_Open",
            "Eye_Close",
            "Mouth_Smile",
            "Mouth_Frown",
            "Nose_Tip",
            "Nose_Width",
            "Ear_Tip",
            "Ear_Rotate",
        ];

        let mut found_morph = String::new();
        let mut found_value = 0.5;

        for m in &morph_names {
            let key = m.to_lowercase().replace("_", " ");
            if input.contains(&key) {
                found_morph = m.to_string();
                break;
            }
        }
        if found_morph.is_empty() {
            for m in &morph_names {
                let parts: Vec<&str> = m.split('_').collect();
                if parts.len() >= 2 && input.contains(&parts[0].to_lowercase()) {
                    found_morph = m.to_string();
                    break;
                }
            }
        }

        let numbers: Vec<f32> = input
            .split_whitespace()
            .filter_map(|w| w.parse::<f32>().ok())
            .collect();
        if !numbers.is_empty() {
            found_value = numbers[0].clamp(0.0, 1.0);
        }

        if !found_morph.is_empty() {
            actions.push(AgentAction {
                action_type: "set_morph".to_string(),
                command: "set_morph".to_string(),
                args: vec![
                    "selected".to_string(),
                    found_morph.clone(),
                    found_value.to_string(),
                ],
            });
            messages.push(format!("Setting {} to {:.2}.", found_morph, found_value));
        }
    }

    if input.contains("expression") || input.contains("express") || input.contains("facial") {
        let expressions = vec![
            "Smile", "Frown", "Surprise", "Angry", "Sad", "Happy", "Neutral", "Blink", "Squint",
            "Kiss",
        ];
        let mut found_expr = String::new();
        let mut expr_value = 0.8;

        for e in &expressions {
            if input.contains(&e.to_lowercase()) {
                found_expr = e.to_string();
                break;
            }
        }

        let numbers: Vec<f32> = input
            .split_whitespace()
            .filter_map(|w| w.parse::<f32>().ok())
            .collect();
        if !numbers.is_empty() {
            expr_value = numbers[0].clamp(0.0, 1.0);
        }

        if !found_expr.is_empty() {
            actions.push(AgentAction {
                action_type: "apply_expression".to_string(),
                command: "apply_expression".to_string(),
                args: vec![
                    "selected".to_string(),
                    found_expr.to_string(),
                    expr_value.to_string(),
                ],
            });
            messages.push(format!(
                "Applying {} expression at {:.2}.",
                found_expr, expr_value
            ));
        } else if input.contains("expression") {
            actions.push(AgentAction {
                action_type: "list_expressions".to_string(),
                command: "bridge_get_active_expressions".to_string(),
                args: vec![],
            });
            messages.push("Listing available expressions.".to_string());
        }
    }

    if actions.is_empty() {
        return AgentResponse {
            success: false,
            result: None,
            error: Some("No morph or expression intents identified.".to_string()),
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
