#![allow(dead_code)]

pub mod animation_tools;
pub mod asset_discovery;
pub mod camera_tools;
pub mod character_tools;
pub mod clothing_tools;
pub mod environment_tools;
pub mod export_tools;
pub mod figure_tools;
pub mod hair_tools;
pub mod knowledge_tools;
pub mod lighting_tools;
pub mod material_tools;
pub mod meta_tools;
pub mod morph_tools;
pub mod physics_tools;
pub mod pipeline_tools;
pub mod pose_tools;
pub mod props_tools;
pub mod render_tools;
pub mod rigging_tools;
pub mod scene_composition;
pub mod scene_observation;
pub mod scene_tools;
pub mod selection_tools;
pub mod transform_tools;
pub mod utility_tools;
pub mod viewport_tools;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub param_type: ToolParamType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolParamType {
    String,
    Number,
    Integer,
    Boolean,
    Color,
    FloatArray,
    StringArray,
    Object,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub category: ToolCategory,
    pub parameters: Vec<ToolParameter>,
    pub return_description: String,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolCategory {
    SceneObservation,
    AssetDiscovery,
    CharacterCustomization,
    Animation,
    Materials,
    Lighting,
    Camera,
    SceneComposition,
    Knowledge,
    Pipeline,
    Meta,
    Environment,
    Rendering,
    Selection,
    Morphs,
    Rigging,
    Utility,
    Export,
    Figure,
    Transform,
    Scene,
    Viewport,
    Clothing,
    Hair,
    Props,
    Pose,
    Physics,
}

impl ToolCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolCategory::SceneObservation => "Scene Observation",
            ToolCategory::AssetDiscovery => "Asset Discovery",
            ToolCategory::CharacterCustomization => "Character Customization",
            ToolCategory::Animation => "Animation",
            ToolCategory::Materials => "Materials",
            ToolCategory::Lighting => "Lighting",
            ToolCategory::Camera => "Camera",
            ToolCategory::SceneComposition => "Scene Composition",
            ToolCategory::Knowledge => "Knowledge & Guidance",
            ToolCategory::Pipeline => "Pipeline & Automation",
            ToolCategory::Meta => "Meta",
            ToolCategory::Environment => "Environment",
            ToolCategory::Rendering => "Rendering",
            ToolCategory::Selection => "Selection",
            ToolCategory::Morphs => "Morphs",
            ToolCategory::Rigging => "Rigging",
            ToolCategory::Utility => "Utility",
            ToolCategory::Export => "Export",
            ToolCategory::Figure => "Figure",
            ToolCategory::Transform => "Transform",
            ToolCategory::Scene => "Scene",
            ToolCategory::Viewport => "Viewport",
            ToolCategory::Clothing => "Clothing",
            ToolCategory::Hair => "Hair",
            ToolCategory::Props => "Props",
            ToolCategory::Pose => "Pose",
            ToolCategory::Physics => "Physics",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub message: String,
    pub tool_name: String,
}

impl ToolResponse {
    pub fn ok(tool_name: impl Into<String>, data: impl Into<serde_json::Value>) -> Self {
        Self {
            success: true,
            data: data.into(),
            message: String::new(),
            tool_name: tool_name.into(),
        }
    }

    pub fn ok_with_message(
        tool_name: impl Into<String>,
        data: impl Into<serde_json::Value>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            success: true,
            data: data.into(),
            message: message.into(),
            tool_name: tool_name.into(),
        }
    }

    pub fn err(tool_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: serde_json::Value::Null,
            message: message.into(),
            tool_name: tool_name.into(),
        }
    }
}

pub type ToolHandler = fn(request: ToolRequest) -> ToolResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequest {
    pub tool_name: String,
    pub args: HashMap<String, serde_json::Value>,
}

impl ToolRequest {
    pub fn get_str(&self, key: &str) -> Option<String> {
        self.args
            .get(key)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.args.get(key).and_then(|v| v.as_f64())
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.args.get(key).and_then(|v| v.as_i64())
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.args.get(key).and_then(|v| v.as_bool())
    }

    pub fn get_array(&self, key: &str) -> Vec<serde_json::Value> {
        self.args
            .get(key)
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
    }

    pub fn get_object(&self, key: &str) -> Option<HashMap<String, serde_json::Value>> {
        self.args.get(key).and_then(|v| {
            v.as_object()
                .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        })
    }

    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.get_str(key).unwrap_or_else(|| default.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetrics {
    pub tool_name: String,
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub average_duration_ms: f64,
}

static TOOL_REGISTRY: Mutex<Option<HashMap<String, RegisteredTool>>> = Mutex::new(None);

#[derive(Clone)]
struct RegisteredTool {
    definition: ToolDefinition,
    handler: ToolHandler,
    metrics: ToolMetrics,
}

fn ensure_registry() {
    let mut guard = TOOL_REGISTRY.lock().unwrap();
    if guard.is_none() {
        *guard = Some(HashMap::new());
    }
}

pub fn register_tool(definition: ToolDefinition, handler: ToolHandler) {
    ensure_registry();
    let mut guard = TOOL_REGISTRY.lock().unwrap();
    if let Some(ref mut registry) = *guard {
        let name = definition.name.clone();
        registry.insert(
            name.clone(),
            RegisteredTool {
                definition,
                handler,
                metrics: ToolMetrics {
                    tool_name: name,
                    total_calls: 0,
                    successful_calls: 0,
                    failed_calls: 0,
                    average_duration_ms: 0.0,
                },
            },
        );
    }
}

pub fn execute_tool(tool_name: &str, args: HashMap<String, serde_json::Value>) -> ToolResponse {
    ensure_registry();
    let start = std::time::Instant::now();
    let guard = TOOL_REGISTRY.lock().unwrap();
    if let Some(ref registry) = *guard {
        if let Some(registered) = registry.get(tool_name) {
            let handler = registered.handler;
            let request = ToolRequest {
                tool_name: tool_name.to_string(),
                args,
            };
            drop(guard);
            let response = handler(request);
            let duration = start.elapsed();
            let mut guard2 = TOOL_REGISTRY.lock().unwrap();
            if let Some(ref mut reg) = *guard2 {
                if let Some(m) = reg.get_mut(tool_name) {
                    m.metrics.total_calls += 1;
                    if response.success {
                        m.metrics.successful_calls += 1;
                    } else {
                        m.metrics.failed_calls += 1;
                    }
                    let d = duration.as_secs_f64() * 1000.0;
                    let count = m.metrics.total_calls as f64;
                    m.metrics.average_duration_ms =
                        m.metrics.average_duration_ms * ((count - 1.0) / count) + d / count;
                }
            }
            response
        } else {
            drop(guard);
            ToolResponse::err(
                tool_name,
                format!(
                    "Unknown tool: '{}'. Use list_tools to see available tools.",
                    tool_name
                ),
            )
        }
    } else {
        ToolResponse::err(tool_name, "Tool registry not initialized")
    }
}

pub fn get_tool_definition(name: &str) -> Option<ToolDefinition> {
    ensure_registry();
    let guard = TOOL_REGISTRY.lock().unwrap();
    guard
        .as_ref()
        .and_then(|r| r.get(name))
        .map(|t| t.definition.clone())
}

pub fn list_tools() -> Vec<ToolDefinition> {
    ensure_registry();
    let guard = TOOL_REGISTRY.lock().unwrap();
    guard
        .as_ref()
        .map(|r| {
            let mut tools: Vec<_> = r.values().map(|t| t.definition.clone()).collect();
            tools.sort_by(|a, b| a.name.cmp(&b.name));
            tools
        })
        .unwrap_or_default()
}

pub fn list_tools_by_category(category: &ToolCategory) -> Vec<ToolDefinition> {
    list_tools()
        .into_iter()
        .filter(|t| t.category == *category)
        .collect()
}

pub fn get_tool_metrics() -> Vec<ToolMetrics> {
    ensure_registry();
    let guard = TOOL_REGISTRY.lock().unwrap();
    guard
        .as_ref()
        .map(|r| {
            let mut metrics: Vec<_> = r.values().map(|t| t.metrics.clone()).collect();
            metrics.sort_by(|a, b| a.tool_name.cmp(&b.tool_name));
            metrics
        })
        .unwrap_or_default()
}

pub fn build_tool_catalog_prompt() -> String {
    let tools = list_tools();
    let mut lines = Vec::new();
    for tool in &tools {
        let params: Vec<String> = tool
            .parameters
            .iter()
            .map(|p| {
                format!(
                    "{}: {} ({})",
                    p.name,
                    p.description,
                    if p.required { "required" } else { "optional" }
                )
            })
            .collect();
        let param_str = if params.is_empty() {
            "none".to_string()
        } else {
            params.join(", ")
        };
        lines.push(format!(
            "- {} [{}]: {} | params: {}",
            tool.name,
            tool.category.as_str(),
            tool.description,
            param_str
        ));
    }
    lines.join("\n")
}

pub fn register_tool_with_params(
    name: &str,
    description: &str,
    category: ToolCategory,
    parameters: &[ToolParameter],
    return_description: &str,
    examples: &[&str],
    handler: ToolHandler,
) {
    register_tool(
        ToolDefinition {
            name: name.to_string(),
            description: description.to_string(),
            category,
            parameters: parameters.to_vec(),
            return_description: return_description.to_string(),
            examples: examples.iter().map(|s| s.to_string()).collect(),
        },
        handler,
    );
}

// Convenience macro: define_tool!(name, desc, cat, [params...], return, [examples...], handler)
#[macro_export]
macro_rules! define_tool {
    ($name:expr, $description:expr, $category:expr, [$($param:expr),* $(,)?], $return_desc:expr, [$($example:expr),* $(,)?], $handler:path) => {
        $crate::tools::register_tool_with_params(
            $name,
            $description,
            $category,
            &[$($param),*],
            $return_desc,
            &[$($example),*],
            $handler,
        );
    };
}

pub fn tool_param(
    name: &str,
    description: &str,
    required: bool,
    param_type: ToolParamType,
) -> ToolParameter {
    ToolParameter {
        name: name.to_string(),
        description: description.to_string(),
        required,
        param_type,
    }
}

pub fn init_all_tools() {
    scene_observation::register_tools();
    asset_discovery::register_tools();
    character_tools::register_tools();
    animation_tools::register_tools();
    material_tools::register_tools();
    lighting_tools::register_tools();
    camera_tools::register_tools();
    scene_composition::register_tools();
    knowledge_tools::register_tools();
    pipeline_tools::register_tools();
    meta_tools::register_tools();
    environment_tools::register_tools();
    render_tools::register_tools();
    selection_tools::register_tools();
    morph_tools::register_tools();
    export_tools::register_tools();
    rigging_tools::register_tools();
    utility_tools::register_tools();
    figure_tools::register_tools();
    transform_tools::register_tools();
    clothing_tools::register_tools();
    hair_tools::register_tools();
    scene_tools::register_tools();
    viewport_tools::register_tools();
    props_tools::register_tools();
    pose_tools::register_tools();
    physics_tools::register_tools();
}
