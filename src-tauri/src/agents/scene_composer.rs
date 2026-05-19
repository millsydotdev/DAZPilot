//! Scene Composition Agent: orchestrates multi-step scene creation.

use crate::agents::{AgentRequest, AgentResponse, AgentAction};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionPlan {
    pub steps: Vec<CompositionStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionStep {
    pub description: String,
    pub action: AgentAction,
}

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    let mut steps = vec![];

    // Example high-level intent parsing
    if input.contains("street") || input.contains("cyberpunk") {
        steps.push(CompositionStep {
            description: "Load cyberpunk environment".to_string(),
            action: AgentAction {
                action_type: "load".to_string(),
                command: "load_asset".to_string(),
                args: vec!["/environments/cyberpunk_street.duf".to_string(), "Environment".to_string()],
            },
        });
        steps.push(CompositionStep {
            description: "Add ambient lighting".to_string(),
            action: AgentAction {
                action_type: "light".to_string(),
                command: "create_light".to_string(),
                args: vec!["ambient".to_string()],
            },
        });
    }

    if steps.is_empty() {
        return AgentResponse {
            success: false,
            result: None,
            error: Some("Unable to compose scene from intent.".to_string()),
            actions: vec![],
        };
    }

    AgentResponse {
        success: true,
        result: Some(format!("Composed scene with {} steps", steps.len())),
        error: None,
        actions: steps.into_iter().map(|s| s.action).collect(),
    }
}
