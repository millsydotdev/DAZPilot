use super::registry;
use super::{AgentRequest, AgentResponse, SubAgentResult};

/// Delegate a request from a parent agent to a specific child agent.
/// Returns the child's response, or an error if delegation is not possible.
pub fn delegate_to_child(
    parent_type: &str,
    child_type: &str,
    mut request: AgentRequest,
) -> Result<AgentResponse, String> {
    let handler = registry::with_registry(|reg| {
        let child = reg
            .get(child_type)
            .ok_or_else(|| format!("Child agent '{}' not found in registry", child_type))?;

        let parent = reg
            .get(parent_type)
            .ok_or_else(|| format!("Parent agent '{}' not found in registry", parent_type))?;

        if !parent.children.contains(&child_type.to_string()) {
            return Err(format!(
                "'{}' is not a child of '{}'. Children: {:?}",
                child_type, parent_type, parent.children
            ));
        }

        if request.delegation_chain.contains(&child_type.to_string()) {
            return Err(format!(
                "Circular delegation detected: '{}' is already in the delegation chain",
                child_type
            ));
        }

        if request.delegation_chain.len() as u32 >= request.max_delegation_depth {
            return Err(format!(
                "Max delegation depth ({}) reached",
                request.max_delegation_depth
            ));
        }

        request.delegation_chain.push(parent_type.to_string());

        Ok(child.handler)
    })?;

    Ok(handler(request))
}

/// Delegate to all children whose capabilities match the input.
/// Returns responses from all matching children.
pub fn delegate_by_capability(
    parent_type: &str,
    input: &str,
    request: AgentRequest,
) -> Vec<(String, AgentResponse)> {
    let handlers = registry::with_registry(|reg| {
        let children = reg.get_children(parent_type);
        let mut handlers = Vec::new();

        for child in children {
            let matched = child
                .capabilities
                .iter()
                .any(|cap| registry::input_matches_capability(input, cap));

            if matched {
                let child_req = AgentRequest {
                    delegation_chain: {
                        let mut chain = request.delegation_chain.clone();
                        chain.push(parent_type.to_string());
                        chain
                    },
                    ..request.clone()
                };

                handlers.push((child.agent_type.clone(), child.handler, child_req));
            }
        }

        handlers
    });

    handlers
        .into_iter()
        .map(|(agent_type, handler, child_req)| (agent_type, handler(child_req)))
        .collect()
}

/// Delegate with a combined result from all matching children.
/// Collects all successful actions into a single response.
pub fn delegate_and_aggregate(
    parent_type: &str,
    input: &str,
    request: AgentRequest,
) -> AgentResponse {
    let sub_results = delegate_by_capability(parent_type, input, request);

    let mut all_actions = Vec::new();
    let mut all_messages = Vec::new();
    let mut total_success = true;

    for (agent_type, resp) in &sub_results {
        if resp.success {
            all_actions.extend(resp.actions.clone());
            if let Some(ref msg) = resp.result {
                all_messages.push(format_agent_message(agent_type, msg));
            }
        } else {
            total_success = false;
            if let Some(ref err) = resp.error {
                all_messages.push(format_agent_message(agent_type, format!("Error: {}", err)));
            }
        }
    }

    let sub_agent_results: Vec<SubAgentResult> = sub_results
        .into_iter()
        .map(|(agent_type, resp)| SubAgentResult::from((agent_type, resp)))
        .collect();

    if all_actions.is_empty() && !total_success {
        return AgentResponse {
            success: false,
            result: None,
            error: Some(format!("No agents matched '{}' in delegation scope", input)),
            actions: vec![],
            sub_results: sub_agent_results,
        };
    }

    AgentResponse {
        success: total_success,
        result: Some(all_messages.join("\n")),
        error: if total_success {
            None
        } else {
            Some("Some sub-agents reported errors".to_string())
        },
        actions: all_actions,
        sub_results: sub_agent_results,
    }
}

pub fn format_agent_message(agent_type: &str, message: impl AsRef<str>) -> String {
    format!("{}: {}", agent_type, message.as_ref())
}
