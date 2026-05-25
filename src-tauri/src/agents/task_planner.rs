use crate::agents::orchestrator;
use crate::agents::registry;
use crate::agents::{AgentAction, AgentRequest, AgentResponse};

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();

    let mut all_actions: Vec<AgentAction> = Vec::new();
    let mut all_messages: Vec<String> = Vec::new();
    let mut had_match = false;

    let children = registry::with_registry(|reg| {
        reg.get_children("task_planner")
            .into_iter()
            .map(|n| n.agent_type.clone())
            .collect::<Vec<_>>()
    });

    for child_type in &children {
        let child_req = AgentRequest {
            delegation_chain: {
                let mut chain = request.delegation_chain.clone();
                chain.push("task_planner".to_string());
                chain
            },
            ..request.clone()
        };

        let resp = match orchestrator::delegate_to_child("task_planner", child_type, child_req) {
            Ok(r) => r,
            Err(_) => continue,
        };

        if resp.success && !resp.actions.is_empty() {
            had_match = true;
            if let Some(msg) = resp.result {
                all_messages.push(format!("[{}] {}", child_type, msg));
            }
            for action in resp.actions {
                if !all_actions.iter().any(|a| a.command == action.command) {
                    all_actions.push(action);
                }
            }
        }
    }

    let general_actions = parse_and_create_actions(&input);
    for action in general_actions {
        if !all_actions.iter().any(|a| a.command == action.command) {
            all_actions.push(action);
        }
    }

    if !had_match && all_actions.is_empty() {
        return AgentResponse {
            success: false,
            result: None,
            error: Some(format!(
                "No agent could handle '{}'. Try rephrasing your request.",
                request.input
            )),
            actions: vec![],
            sub_results: vec![],
        };
    }

    if all_messages.is_empty() && !all_actions.is_empty() {
        all_messages.push(format!("Decomposed into {} action(s)", all_actions.len()));
    }

    AgentResponse {
        success: true,
        result: Some(all_messages.join("\n")),
        error: None,
        actions: all_actions,
        sub_results: vec![],
    }
}

fn parse_and_create_actions(input: &str) -> Vec<AgentAction> {
    let mut actions = vec![];

    if input.contains("select") && (input.contains("figure") || input.contains("node")) {
        actions.push(AgentAction {
            action_type: "select_node".to_string(),
            command: "select_node".to_string(),
            args: vec!["genesis_8_female".to_string()],
        });
    }

    if input.contains("create") {
        if input.contains("light") {
            actions.push(AgentAction {
                action_type: "create_light".to_string(),
                command: "create_light".to_string(),
                args: vec!["directional".to_string()],
            });
        } else if input.contains("camera") {
            actions.push(AgentAction {
                action_type: "create_camera".to_string(),
                command: "create_camera".to_string(),
                args: vec![],
            });
        }
    }

    if actions.is_empty() {
        actions.push(AgentAction {
            action_type: "chat".to_string(),
            command: "chat".to_string(),
            args: vec![input.to_string()],
        });
    }

    actions
}
