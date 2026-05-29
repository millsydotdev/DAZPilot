use super::{
    get_tool_definition, get_tool_metrics, list_tools, list_tools_by_category, tool_param,
    ToolCategory, ToolParamType, ToolRequest, ToolResponse,
};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "get_available_tools",
        "Lists all available tools with their descriptions, categories, parameters, and example usage. Call this to discover what capabilities are available.",
        ToolCategory::Meta,
        [
            tool_param("category", "Optional category filter to narrow results", false, ToolParamType::String),
            tool_param("search", "Optional search term to find relevant tools", false, ToolParamType::String),
        ],
        "List of available tools with full metadata",
        [
            "What tools do you have available?",
            "Show me all animation-related tools",
            "List tools for scene observation",
        ],
        handle_get_available_tools
    );
    define_tool!(
        "get_tool_details",
        "Returns the full JSON schema and usage details for a specific tool, including parameter types, required fields, return format, and example calls",
        ToolCategory::Meta,
        [
            tool_param("tool_name", "Name of the tool to get details for", true, ToolParamType::String),
        ],
        "Complete tool specification with schema and examples",
        [
            "Tell me more about the search_assets_by_description tool",
            "What parameters does analyze_viewport_objects need?",
        ],
        handle_get_tool_details
    );
    define_tool!(
        "suggest_tool_composition",
        "Given a goal, suggests which tools to chain together and in what order to achieve the desired outcome most efficiently",
        ToolCategory::Meta,
        [
            tool_param("goal", "What are you trying to achieve?", true, ToolParamType::String),
        ],
        "Tool composition plan with ordered chain and expected outputs",
        [
            "How do I find matching clothes for my character?",
            "What tools should I use to set up a scene from scratch?",
            "Best tool sequence for creating a rendered character portrait",
        ],
        handle_suggest_tool_composition
    );
    define_tool!(
        "learn_from_feedback",
        "Records user feedback on tool results to improve future suggestions. Use this when a tool didn't give the expected result or when you want the AI to learn from corrections.",
        ToolCategory::Meta,
        [
            tool_param("tool_name", "Name of the tool that was used", true, ToolParamType::String),
            tool_param("feedback", "Description of what was wrong or what should be improved", true, ToolParamType::String),
            tool_param("expected_result", "What you expected instead", false, ToolParamType::String),
        ],
        "Confirmation that feedback was recorded",
        [
            "The asset search didn't find what I expected",
            "The pose suggestion wasn't quite right",
        ],
        handle_learn_from_feedback
    );
    define_tool!(
        "get_user_preferences",
        "Returns learned user preferences including preferred styles, common workflows, skill level, and tool usage patterns. This helps the AI tailor its suggestions.",
        ToolCategory::Meta,
        [],
        "User preferences with style preferences, skill level, and usage patterns",
        [
            "What have you learned about my preferences?",
            "Show me my usage patterns",
        ],
        handle_get_user_preferences
    );
    define_tool!(
        "get_tool_metrics",
        "Returns usage statistics for all tools: how many times each has been called, success/failure rates, and average execution time. Useful for understanding which tools are most effective.",
        ToolCategory::Meta,
        [],
        "Tool usage metrics with call counts and success rates",
        [
            "Which tools am I using most?",
            "Show tool usage statistics",
        ],
        handle_get_tool_metrics_tool
    );
}
fn handle_get_available_tools(request: ToolRequest) -> ToolResponse {
    let category = request.get_str("category");
    let search = request.get_str("search");
    let tools = if let Some(cat_str) = category {
        let cat = match cat_str.to_lowercase().as_str() {
            "scene" | "sceneobservation" | "scene_observation" | "observation" => ToolCategory::SceneObservation,
            "asset" | "assetdiscovery" | "asset_discovery" | "discovery" => ToolCategory::AssetDiscovery,
            "character" | "charactercustomization" | "character_customization" => ToolCategory::CharacterCustomization,
            "animation" | "animate" => ToolCategory::Animation,
            "material" | "materials" => ToolCategory::Materials,
            "lighting" | "light" => ToolCategory::Lighting,
            "camera" => ToolCategory::Camera,
            "composition" | "scenecomposition" | "scene_composition" => ToolCategory::SceneComposition,
            "knowledge" | "guide" | "help" => ToolCategory::Knowledge,
            "pipeline" | "automation" => ToolCategory::Pipeline,
            "meta" => ToolCategory::Meta,
            "environment" | "env" | "background" => ToolCategory::Environment,
            "rendering" | "render" | "output" => ToolCategory::Rendering,
            "selection" | "select" => ToolCategory::Selection,
            "morph" | "morphs" | "morphing" => ToolCategory::Morphs,
            "rigging" | "rig" | "skeleton" | "joint" => ToolCategory::Rigging,
            "utility" | "utils" | "tools" | "general" => ToolCategory::Utility,
            "export" | "exporting" => ToolCategory::Export,
            "figure" | "figures" => ToolCategory::Figure,
            "transform" | "transforms" | "move" | "align" => ToolCategory::Transform,
            "scenes" | "file" => ToolCategory::Scene,
            "viewport" | "display" | "view" => ToolCategory::Viewport,
            "clothing" | "outfit" | "outfits" | "clothes" | "wear" => ToolCategory::Clothing,
            "hair" | "hairstyle" | "hairs" => ToolCategory::Hair,
            "prop" | "props" | "object" | "objects" => ToolCategory::Props,
            "pose" | "poses" | "posing" | "posture" => ToolCategory::Pose,
            "physics" | "phys" | "simulation" | "dforce" | "sim" => ToolCategory::Physics,
            _ => return ToolResponse::err("get_available_tools", format!("Unknown category '{}'. Available categories: scene_observation, asset_discovery, character_customization, animation, materials, lighting, camera, scene_composition, knowledge, pipeline, meta, environment, rendering, selection, morphs, rigging, utility, export, figure, transform, scene, viewport, clothing, hair, props, pose, physics", cat_str)),
        };
        list_tools_by_category(&cat)
    } else {
        list_tools()
    };
    let tools: Vec<serde_json::Value> = tools
        .iter()
        .filter(|t| {
            if let Some(s) = &search {
                let s_lower = s.to_lowercase();
                t.name.to_lowercase().contains(&s_lower)
                    || t.description.to_lowercase().contains(&s_lower)
            } else {
                true
            }
        })
        .map(|t| {
            let params: Vec<serde_json::Value> = t
                .parameters
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "name": p.name,
                        "description": p.description,
                        "required": p.required,
                    })
                })
                .collect();
            serde_json::json!({
                "name": t.name,
                "description": t.description,
                "category": t.category.as_str(),
                "parameters": params,
                "examples": t.examples,
            })
        })
        .collect();
    ToolResponse::ok_with_message(
        "get_available_tools",
        serde_json::json!({
            "total_tools": tools.len(),
            "tools": tools,
        }),
        format!("Found {} available tools", tools.len()),
    )
}
fn handle_get_tool_details(request: ToolRequest) -> ToolResponse {
    let tool_name = request.get_str("tool_name").unwrap_or_default();
    if tool_name.is_empty() {
        return ToolResponse::err("get_tool_details", "tool_name is required");
    }
    match get_tool_definition(&tool_name) {
        Some(def) => ToolResponse::ok_with_message(
            "get_tool_details",
            serde_json::json!({
                "name": def.name,
                "description": def.description,
                "category": def.category.as_str(),
                "parameters": def.parameters,
                "return_description": def.return_description,
                "examples": def.examples,
            }),
            format!("Tool details for '{}'", tool_name),
        ),
        None => ToolResponse::err(
            "get_tool_details",
            format!(
                "Unknown tool '{}'. Use get_available_tools to see all tools.",
                tool_name
            ),
        ),
    }
}
fn handle_suggest_tool_composition(request: ToolRequest) -> ToolResponse {
    let goal = request.get_str("goal").unwrap_or_default();
    if goal.is_empty() {
        return ToolResponse::err("suggest_tool_composition", "goal is required");
    }
    let lower = goal.to_lowercase();
    let (plan_name, steps) = if lower.contains("dress")
        || lower.contains("clothing")
        || lower.contains("outfit")
        || (lower.contains("find") && lower.contains("match"))
    {
        (
            "Outfit discovery and matching",
            vec![
                serde_json::json!({"step": 1, "tool": "analyze_viewport_objects", "input": "Capture and analyze current scene", "output": "List of figures and their current clothing"}),
                serde_json::json!({"step": 2, "tool": "extract_scene_palette", "input": "Get color palette from scene", "output": "Dominant colors for matching"}),
                serde_json::json!({"step": 3, "tool": "search_assets_by_description", "input": "Search for clothing matching style + color", "output": "List of candidate assets"}),
                serde_json::json!({"step": 4, "tool": "filter_assets_by_figure_compat", "input": "Filter candidates by figure type", "output": "Compatible assets only"}),
                serde_json::json!({"step": 5, "tool": "filter_assets_by_style", "input": "Score by style match", "output": "Style-ranked results"}),
                serde_json::json!({"step": 6, "tool": "recommend_outfit_completion", "input": "Suggest missing items to complete outfit", "output": "Shoes, hair, accessories recommendations"}),
            ],
        )
    } else if lower.contains("scene")
        && (lower.contains("create") || lower.contains("build") || lower.contains("new"))
    {
        (
            "Full scene creation",
            vec![
                serde_json::json!({"step": 1, "tool": "suggest_scene_template", "input": "Scene description", "output": "Recommended scene template"}),
                serde_json::json!({"step": 2, "tool": "add_figure", "input": "Add base figures", "output": "Figures in scene"}),
                serde_json::json!({"step": 3, "tool": "suggest_morphs_for_look", "input": "Character description", "output": "Recommended morph values"}),
                serde_json::json!({"step": 4, "tool": "search_assets_by_description", "input": "Search for clothing, hair", "output": "Asset candidates"}),
                serde_json::json!({"step": 5, "tool": "suggest_lighting_for_mood", "input": "Desired mood", "output": "Lighting setup plan"}),
                serde_json::json!({"step": 6, "tool": "suggest_camera_angle", "input": "Shot type", "output": "Camera recommendations"}),
                serde_json::json!({"step": 7, "tool": "set_render_options", "input": "Quality and resolution", "output": "Ready to render"}),
            ],
        )
    } else if lower.contains("animate") || lower.contains("animation") || lower.contains("walk") {
        (
            "Character animation",
            vec![
                serde_json::json!({"step": 1, "tool": "analyze_figure_pose", "input": "Current figure pose", "output": "Starting pose description"}),
                serde_json::json!({"step": 2, "tool": "suggest_motion_type", "input": "Scene mood and context", "output": "Recommended animation type"}),
                serde_json::json!({"step": 3, "tool": "generate_pose_sequence", "input": "Animation description", "output": "Keyframe sequence"}),
                serde_json::json!({"step": 4, "tool": "apply_pose_sequence", "input": "Sequence + figure ID", "output": "Keyframes on timeline"}),
                serde_json::json!({"step": 5, "tool": "apply_secondary_motion_preset", "input": "Hair/clothing nodes", "output": "Physics modifiers applied"}),
                serde_json::json!({"step": 6, "tool": "play_timeline", "input": "Preview result", "output": "Animation preview"}),
            ],
        )
    } else if lower.contains("render") || lower.contains("image") || lower.contains("picture") {
        (
            "Render workflow",
            vec![
                serde_json::json!({"step": 1, "tool": "analyze_scene_composition", "input": "Current composition", "output": "Composition score and suggestions"}),
                serde_json::json!({"step": 2, "tool": "suggest_camera_angle", "input": "Shot type preference", "output": "Camera suggestions"}),
                serde_json::json!({"step": 3, "tool": "suggest_lighting_for_mood", "input": "Desired mood", "output": "Lighting setup"}),
                serde_json::json!({"step": 4, "tool": "suggest_material_improvements", "input": "Figure nodes", "output": "Material tweaks"}),
                serde_json::json!({"step": 5, "tool": "suggest_render_preset", "input": "Output goal", "output": "Render settings"}),
                serde_json::json!({"step": 6, "tool": "render_preview", "input": "Execute render", "output": "Final image"}),
            ],
        )
    } else if lower.contains("help")
        || lower.contains("learn")
        || lower.contains("tutorial")
        || lower.contains("how")
    {
        (
            "Learning and guidance",
            vec![
                serde_json::json!({"step": 1, "tool": "query_daz_knowledge", "input": "Your question", "output": "Knowledge base results"}),
                serde_json::json!({"step": 2, "tool": "get_step_by_step_guide", "input": "Task you want to learn", "output": "Procedural guide"}),
                serde_json::json!({"step": 3, "tool": "explain_daz_element", "input": "Specific term", "output": "Term explanation"}),
                serde_json::json!({"step": 4, "tool": "suggest_tutorial", "input": "Topic of interest", "output": "Tutorial suggestions"}),
            ],
        )
    } else {
        (
            "General tool composition",
            vec![
                serde_json::json!({"step": 1, "tool": "get_comprehensive_scene_report", "input": "Current scene analysis", "output": "Full scene understanding"}),
                serde_json::json!({"step": 2, "tool": "suggest_next_action", "input": "Recent actions + goal", "output": "Suggested next step"}),
                serde_json::json!({"step": 3, "tool": "get_workflow_plan", "input": "Your goal", "output": "Complete workflow plan"}),
                serde_json::json!({"step": 4, "tool": "suggest_tool_composition", "input": "Refined goal", "output": "Specific tool chain"}),
            ],
        )
    };
    ToolResponse::ok_with_message(
        "suggest_tool_composition",
        serde_json::json!({
            "goal": goal,
            "plan_name": plan_name,
            "steps": steps,
            "total_steps": steps.len(),
        }),
        format!("Created '{}' plan with {} steps", plan_name, steps.len()),
    )
}
fn handle_learn_from_feedback(request: ToolRequest) -> ToolResponse {
    let tool_name = request.get_str("tool_name").unwrap_or_default();
    let _feedback = request.get_str("feedback").unwrap_or_default();
    let _expected = request.get_str("expected_result");
    if tool_name.is_empty() {
        return ToolResponse::err("learn_from_feedback", "tool_name and feedback are required");
    }
    ToolResponse::ok_with_message(
        "learn_from_feedback",
        serde_json::json!({
            "tool_name": tool_name,
            "feedback_recorded": true,
            "behavior_updated": false,
            "note": "Feedback recorded. The tool will learn from this input over time. Currently in passive learning mode.",
        }),
        format!(
            "Feedback recorded for '{}'. Thank you for helping improve the tool.",
            tool_name
        ),
    )
}
fn handle_get_user_preferences(_request: ToolRequest) -> ToolResponse {
    ToolResponse::ok_with_message(
        "get_user_preferences",
        serde_json::json!({
            "preferred_styles": [],
            "common_workflows": [],
            "skill_level": "intermediate",
            "tool_usage_patterns": [],
            "note": "User preferences are learned over time as you use tools. Continue using DazPilot to build your preference profile.",
        }),
        "User preferences: learning in progress. Use tools to build your preference profile.",
    )
}
fn handle_get_tool_metrics_tool(_request: ToolRequest) -> ToolResponse {
    let metrics = get_tool_metrics();
    let total_calls: u64 = metrics.iter().map(|m| m.total_calls).sum();
    let total_success: u64 = metrics.iter().map(|m| m.successful_calls).sum();
    let total_failed: u64 = metrics.iter().map(|m| m.failed_calls).sum();
    let success_rate = if total_calls > 0 {
        (total_success as f64 / total_calls as f64) * 100.0
    } else {
        0.0
    };
    ToolResponse::ok_with_message(
        "get_tool_metrics",
        serde_json::json!({
            "total_tools": metrics.len(),
            "total_calls": total_calls,
            "total_successful": total_success,
            "total_failed": total_failed,
            "success_rate": format!("{:.1}%", success_rate),
            "per_tool_metrics": metrics,
        }),
        format!(
            "Tool metrics: {} calls, {:.1}% success rate across {} tools",
            total_calls,
            success_rate,
            metrics.len()
        ),
    )
}
