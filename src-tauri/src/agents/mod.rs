pub mod task_planner;
pub mod asset_selection;
pub mod animation_agent;
pub mod conflict_resolution;
pub mod render_agent;
pub mod physics_agent;
pub mod scene_composer;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    pub agent_type: String,
    pub input: String,
    pub context: Option<AgentContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub user_id: String,
    pub session_id: String,
    pub current_figure: Option<String>,
    pub selected_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
    pub actions: Vec<AgentAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    pub action_type: String,
    pub command: String,
    pub args: Vec<String>,
}

pub fn execute_agent(request: AgentRequest) -> AgentResponse {
    match request.agent_type.as_str() {
        "task_planner" => task_planner::execute(request),
        "asset_selection" => asset_selection::execute(request),
        "animation" => animation_agent::execute(request),
        "conflict_resolution" => conflict_resolution::execute(request),
        "render" => render_agent::execute(request),
        "physics" => physics_agent::execute(request),
        "scene_composer" => scene_composer::execute(request),
        _ => AgentResponse {
            success: false,
            result: None,
            error: Some(format!("Unknown agent type: {}", request.agent_type)),
            actions: vec![],
        },
    }
}
