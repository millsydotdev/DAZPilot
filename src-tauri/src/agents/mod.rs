pub mod animation_agent;
pub mod asset_selection;
pub mod conflict_resolution;
pub mod orchestrator;
pub mod physics_agent;
pub mod registry;
pub mod render_agent;
pub mod scene_composer;
pub mod sub_agents;
pub mod task_planner;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    pub agent_type: String,
    pub input: String,
    pub context: Option<AgentContext>,
    #[serde(default)]
    pub delegation_chain: Vec<String>,
    #[serde(default = "default_max_depth")]
    pub max_delegation_depth: u32,
}

fn default_max_depth() -> u32 {
    5
}

impl AgentRequest {
    pub fn new(agent_type: impl Into<String>, input: impl Into<String>) -> Self {
        Self {
            agent_type: agent_type.into(),
            input: input.into(),
            context: None,
            delegation_chain: Vec::new(),
            max_delegation_depth: 5,
        }
    }

    pub fn with_context(mut self, context: AgentContext) -> Self {
        self.context = Some(context);
        self
    }
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
    #[serde(default)]
    pub sub_results: Vec<SubAgentResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentResult {
    pub agent_type: String,
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
    pub actions: Vec<AgentAction>,
}

impl From<(String, AgentResponse)> for SubAgentResult {
    fn from((agent_type, resp): (String, AgentResponse)) -> Self {
        SubAgentResult {
            agent_type,
            success: resp.success,
            result: resp.result,
            error: resp.error,
            actions: resp.actions,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    pub action_type: String,
    pub command: String,
    pub args: Vec<String>,
}

pub fn execute_agent(request: AgentRequest) -> AgentResponse {
    registry::with_registry(|reg| match reg.get(&request.agent_type) {
        Some(node) => (node.handler)(request),
        None => AgentResponse {
            success: false,
            result: None,
            error: Some(format!(
                "Unknown agent type: '{}'. Use list_agents to see available agents.",
                request.agent_type
            )),
            actions: vec![],
            sub_results: vec![],
        },
    })
}

pub fn register_default_agents() {
    registry::init_registry();
    registry::with_registry_mut(|reg| {
        register(
            reg,
            "task_planner",
            "Primary orchestrator: parses intent and delegates to specialized agents",
            None,
            vec!["orchestrator", "planner", "delegate", "coordinate"],
        );

        register(
            reg,
            "asset_selection",
            "Searches and selects assets using semantic and multi-strategy matching",
            Some("task_planner"),
            vec!["load", "apply", "find", "search", "asset", "content"],
        );
        register(
            reg,
            "animation",
            "Controls timeline, poses, keyframes, and dForce simulation",
            Some("task_planner"),
            vec![
                "pose",
                "animation",
                "timeline",
                "keyframe",
                "dforce",
                "play",
                "pause",
            ],
        );
        register(
            reg,
            "conflict_resolution",
            "Detects and resolves asset conflicts (shell zones, UV, morph IDs)",
            Some("task_planner"),
            vec!["conflict", "fix", "issue", "shell", "uv"],
        );
        register(
            reg,
            "render",
            "Configures render settings, lights, and triggers preview renders",
            Some("task_planner"),
            vec!["render", "light", "lighting", "preview", "4k", "hd"],
        );
        register(
            reg,
            "physics",
            "Configures and runs dForce physics simulations",
            Some("task_planner"),
            vec!["simulate", "dforce", "physics", "simulation"],
        );
        register(
            reg,
            "scene_composer",
            "Composes multi-step scene creation from high-level descriptions",
            Some("task_planner"),
            vec!["compose", "scene", "environment", "setup"],
        );

        register(
            reg,
            "pose",
            "Handles pose application, searching, and listing",
            Some("animation"),
            vec!["pose", "apply pose", "posing"],
        );
        register(
            reg,
            "timeline",
            "Controls animation timeline: play, pause, stop, seek, keyframes, loop",
            Some("animation"),
            vec![
                "play", "pause", "stop", "timeline", "keyframe", "seek", "loop",
            ],
        );

        register(
            reg,
            "lighting",
            "Manages scene lights: add, remove, adjust intensity and color",
            Some("render"),
            vec!["light", "lighting", "bright", "dim", "warm", "cool"],
        );
        register(
            reg,
            "camera",
            "Controls camera views, focal length, and framing",
            Some("render"),
            vec!["camera", "view", "shot", "angle", "zoom"],
        );

        register(
            reg,
            "morph",
            "Applies figure morphs and facial expressions",
            Some("asset_selection"),
            vec!["morph", "expression", "shape", "facial", "adjust"],
        );
        register(
            reg,
            "material",
            "Manages material properties, textures, and shader channels",
            Some("asset_selection"),
            vec!["material", "texture", "shader", "surface", "color"],
        );

        register(
            reg,
            "export",
            "Handles scene export, batch export, and render output",
            Some("scene_composer"),
            vec!["export", "save", "output", "batch"],
        );
    });
}

fn register(
    reg: &mut registry::AgentRegistry,
    agent_type: &str,
    description: &str,
    parent: Option<&str>,
    capabilities: Vec<&str>,
) {
    let handler = match agent_type {
        "task_planner" => task_planner::execute as registry::AgentHandler,
        "asset_selection" => asset_selection::execute as registry::AgentHandler,
        "animation" => animation_agent::execute as registry::AgentHandler,
        "conflict_resolution" => conflict_resolution::execute as registry::AgentHandler,
        "render" => render_agent::execute as registry::AgentHandler,
        "physics" => physics_agent::execute as registry::AgentHandler,
        "scene_composer" => scene_composer::execute as registry::AgentHandler,
        "pose" => sub_agents::pose_agent::execute as registry::AgentHandler,
        "timeline" => sub_agents::timeline_agent::execute as registry::AgentHandler,
        "lighting" => sub_agents::lighting_agent::execute as registry::AgentHandler,
        "camera" => sub_agents::camera_agent::execute as registry::AgentHandler,
        "morph" => sub_agents::morph_agent::execute as registry::AgentHandler,
        "material" => sub_agents::material_agent::execute as registry::AgentHandler,
        "export" => sub_agents::export_agent::execute as registry::AgentHandler,
        _ => panic!("Unknown agent type: {}", agent_type),
    };
    let _ = reg.register(registry::AgentNode {
        agent_type: agent_type.to_string(),
        description: description.to_string(),
        parent: parent.map(|p| p.to_string()),
        children: Vec::new(),
        capabilities: capabilities.into_iter().map(|s| s.to_string()).collect(),
        handler,
    });
}
