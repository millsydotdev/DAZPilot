//! LLM-based bridge command planner: maps natural language to validated COMMAND_SCHEMAS.

use crate::ai_action::StructuredAiAction;
use crate::mcp_client::{self, command_requires_confirmation};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
struct LlmToolPlan {
    command: String,
    #[serde(default)]
    args: Value,
    #[serde(default = "default_confidence")]
    confidence: f32,
    #[serde(default)]
    sdk_refs: Vec<String>,
}

fn default_confidence() -> f32 {
    0.75
}

pub fn build_command_catalog_prompt() -> String {
    let mut lines = Vec::new();

    macro_rules! notes {
        ($name:expr) => {{
            let s = match $name {
                "add_figure" => " Use 'genesis9' for latest, 'genesis8' for legacy.",
                "set_morph" => " value: 0.0-1.0. Morph names like 'Head_Height', 'Waist_Width'.",
                "apply_pose" => " pose_path to .duf file. Use search_content to find poses.",
                "add_node" => " type: point_light, spot_light, distant_light, camera, null.",
                "set_light" => " property: 'intensity'(float), 'color'('R,G,B'), 'enable'(bool).",
                "set_camera" => " camera name from get_cameras. focal_length: 35(wide)-200(tele).",
                "set_render_options" => " pixel_samples: 16(draft) to 4096(final).",
                "set_render_settings" => {
                    " Quick width/height presets: 1920x1080(HD), 3840x2160(4K)."
                },
                "set_material_property" => {
                    " Common: 'Base Color', 'Roughness', 'Metallic', 'Opacity'."
                },
                "set_material_texture" => {
                    " channel: 'Base Color', 'Roughness', 'Normal', 'Metallic'."
                },
                "search_content" => " type: figure, clothing, hair, pose, material, prop.",
                "apply_expression" => " expression_id from get_active_expressions. value: 0.0-1.0.",
                "set_bone_transform" => {
                    " bone_name from list_bones. rotation: [x,y,z] Euler degrees."
                },
                "run_dforce_simulation" => " Requires dForce clothing fitted to figure.",
                _ => "",
            };
            s
        }};
    }

    for schema in mcp_client::get_command_schemas() {
        if schema.name == "get_commands" || schema.name == "run_script" {
            continue;
        }
        let params = if schema.parameters.is_empty() {
            "none".to_string()
        } else {
            schema.parameters.join(", ")
        };
        lines.push(format!(
            "- {} ({}): {} | params: {}{}",
            schema.name,
            schema.category,
            schema.description,
            params,
            notes!(schema.name)
        ));
    }
    lines.join("\n")
}

pub fn parse_llm_tool_plan(raw: &str) -> Option<StructuredAiAction> {
    let json_str = extract_json_payload(raw)?;
    let plan: LlmToolPlan = serde_json::from_str(&json_str).ok()?;
    if plan.command.is_empty() || plan.command == "none" {
        return None;
    }
    mcp_client::validate_command(&plan.command, &plan.args).ok()?;
    let sdk_refs = if plan.sdk_refs.is_empty() {
        ai_action_sdk_refs(&plan.command)
    } else {
        plan.sdk_refs
    };
    let requires_confirmation = command_requires_confirmation(&plan.command);
    Some(StructuredAiAction {
        command: plan.command,
        args: plan.args,
        confidence: plan.confidence.clamp(0.0, 1.0),
        sdk_refs,
        requires_confirmation,
    })
}

fn extract_json_payload(text: &str) -> Option<String> {
    for marker in ["```json", "```JSON", "```"] {
        if let Some(start) = text.find(marker) {
            let content_start = start + marker.len();
            if let Some(end) = text[content_start..].find("```") {
                let block = text[content_start..content_start + end].trim();
                if block.starts_with('{') {
                    return Some(block.to_string());
                }
            }
        }
    }
    let trimmed = text.trim();
    if trimmed.starts_with('{') {
        if let Some(end) = trimmed.rfind('}') {
            return Some(trimmed[..=end].to_string());
        }
    }
    None
}

fn ai_action_sdk_refs(command: &str) -> Vec<String> {
    match command {
        "load_asset" | "import_model" => vec!["DzContentMgr".to_string()],
        "apply_pose" => vec!["DzPose".to_string()],
        "set_keyframe" | "seek_to_frame" | "set_timeline_range" => vec!["DzScene".to_string()],
        "set_property" | "set_morph" | "set_light" => vec!["DzProperty".to_string()],
        "set_material_property" => vec!["DzMaterial".to_string()],
        "add_node" | "add_figure" | "delete_node" => vec!["DzNode".to_string()],
        "render_preview" | "set_render_settings" => vec!["DzRenderMgr".to_string()],
        _ => vec!["DzScene".to_string()],
    }
}

pub fn build_tool_planning_prompt(user_message: &str, scene_summary: &str) -> String {
    let scenario_guidance = if user_message.to_lowercase().contains("scene")
        || (user_message.to_lowercase().contains("create")
            || user_message.to_lowercase().contains("make")
            || user_message.to_lowercase().contains("build"))
    {
        "SCENE CREATION GUIDANCE: Choose add_figure(figure_type='genesis9') as the first action.\n\
         Use 'genesis9' for latest figures (preferred) or 'genesis8' for legacy. The add_figure\n\
         action returns the node_id needed for future commands. After adding a figure, the user\n\
         will follow up with pose, clothing, lighting, and render requests."
    } else if user_message.to_lowercase().contains("render")
        || user_message.to_lowercase().contains("output")
        || user_message.to_lowercase().contains("image")
    {
        "RENDER GUIDANCE: Use set_render_options(pixel_samples=64) for quick preview or\n\
         set_render_options(pixel_samples=512) for final quality. Then use set_render_settings\n\
         for output dimensions (width=1920, height=1080)."
    } else if user_message.to_lowercase().contains("light")
        || user_message.to_lowercase().contains("illuminate")
    {
        "LIGHTING GUIDANCE: Use add_node with type='point_light' for fill, 'spot_light' for key,\n\
         'distant_light' for sun. Then set_light to configure intensity/color. 3-point lighting:\n\
         key light (bright, warm), fill light (half intensity, cool), rim light (from behind)."
    } else if user_message.to_lowercase().contains("pose")
        || user_message.to_lowercase().contains("position")
    {
        "POSE GUIDANCE: Use set_bone_transform for individual bone adjustments, or search_content\n\
         (type='pose') to find DUF pose files, then apply_pose with the full path."
    } else if user_message.to_lowercase().contains("morph")
        || user_message.to_lowercase().contains("shape")
        || user_message.to_lowercase().contains("face")
    {
        "MORPH GUIDANCE: Use set_morph with morph names like 'Head_Height', 'Waist_Width', \n\
          'Breast_Size', or 'Cheek_Bone'. Values: 0.0=neutral, 0.5=half, 1.0=full. Use\n\
          search_content(type='figure') first if the node_id is unknown."
    } else if user_message.to_lowercase().contains("preset")
        || user_message.to_lowercase().contains("save scene")
        || user_message.to_lowercase().contains("load scene")
    {
        "PRESET GUIDANCE: Use natural language to save and load scene configurations.\n\
           Examples: 'Save current lighting as a preset', 'Load my 3-point lighting preset', \n\
           'Save this camera angle as a portrait preset'. Presets can be for lighting, camera, figure poses, or full scenes."
    } else {
        ""
    };

    format!(
        "You are DazPilot's action planner. Choose ONE bridge command to satisfy the user request.\n\
         Reply with ONLY a JSON object (no markdown prose) in this exact shape:\n\
         {{\"command\":\"<name>\",\"args\":{{}},\"confidence\":0.0-1.0,\"sdk_refs\":[]}}\n\
         Use command \"none\" with empty args if no command fits.\n\
         Do NOT use run_script. Prefer safe read-only commands when unsure.\n\
         Set sdk_refs to relevant Daz SDK classes (e.g. [\"DzFigure\", \"DzScene\"]).\n\n\
         {}\n\n\
         Available commands:\n{}\n\n\
         Scene context:\n{}\n\n\
         User request:\n{}",
        scenario_guidance,
        build_command_catalog_prompt(),
        scene_summary,
        user_message
    )
}

pub async fn plan_with_llm_tools(
    user_message: &str,
    scene_summary: &str,
    provider: &str,
    model: &str,
    api_key: Option<String>,
    base_url: Option<String>,
) -> Option<StructuredAiAction> {
    let prompt = build_tool_planning_prompt(user_message, scene_summary);
    let response =
        crate::ai_providers::run_chat(provider, model, prompt, api_key, base_url, 0.1, 512, None)
            .await
            .ok()?;
    parse_llm_tool_plan(&response)
}

/// Merge heuristic and LLM plans using a confidence-informed strategy:
/// - heuristic >= 0.85 → prefer heuristic (high-confidence keyword match)
/// - heuristic < 0.70  → prefer LLM (heuristic uncertain, LLM may generalize better)
/// - otherwise         → pick whichever has higher confidence
pub async fn plan_next_action(
    user_message: &str,
    action_history: &[(String, String, bool)], // (command, output, success)
    scene_summary: &str,
    provider: Option<&str>,
    model: Option<&str>,
) -> Option<StructuredAiAction> {
    // Build history summary for the prompt
    let history_summary = if action_history.is_empty() {
        "No previous actions executed.".to_string()
    } else {
        action_history
            .iter()
            .enumerate()
            .map(|(i, (cmd, output, success))| {
                format!(
                    "Action {}: {}\nResult: {}\nStatus: {}\n",
                    i + 1,
                    cmd,
                    if output.len() > 200 {
                        format!("{}... (truncated)", &output[..200])
                    } else {
                        output.clone()
                    },
                    if *success { "SUCCESS" } else { "FAILED" }
                )
            })
            .collect::<Vec<String>>()
            .join("\n---\n")
    };

    // Check for potential loops (same command repeated 3+ times)
    let mut command_counts = std::collections::HashMap::new();
    for (cmd, _, _) in action_history.iter().rev().take(5) {
        // Check last 5 actions
        *command_counts.entry(cmd.clone()).or_insert(0) += 1;
    }

    let has_repetition = command_counts.values().any(|&count| count >= 3);

    let prompt = format!(
        "You are continuing a multi-step task.\n\n\
        Original request: {}\n\n\
        Action History:\n{}\n\n\
        Scene context: {}\n\n\
        Based on the history above, is another action needed to fully satisfy the original request?\n\
        If YES, output a single action as a JSON block:\n\
        ```json\n\
        {{\n\
          \"command\": \"command_name\",\n\
          \"args\": {{}},\n\
          \"confidence\": 0.8\n\
        }}\n\
        ```\n\
        If NO, meaning the task is complete, respond with just the text: DONE\n\n\
        Only output the JSON block or DONE - no extra explanation.\n\
        {}", 
        user_message,
        history_summary,
        scene_summary,
        if has_repetition {
            "WARNING: You have been repeating similar actions. Consider if a different approach is needed or if the task is actually complete."
        } else { "" }
    );

    let response = crate::ai_providers::run_chat(
        provider.unwrap_or("local-gguf"),
        model.unwrap_or("phi-2-q4"),
        prompt,
        None,
        None,
        0.1,
        512,
        None,
    )
    .await
    .ok()?;

    // Check if response is DONE (task complete)
    if response.trim().to_uppercase() == "DONE" {
        return None;
    }

    // Otherwise try to parse as JSON action
    parse_llm_tool_plan(&response)
}

pub fn merge_plans(
    heuristic: Option<StructuredAiAction>,
    llm: Option<StructuredAiAction>,
) -> Option<StructuredAiAction> {
    const HEURISTIC_HIGH_CONFIDENCE: f32 = 0.85;
    const HEURISTIC_LOW_CONFIDENCE: f32 = 0.70;

    match (heuristic, llm) {
        (Some(h), Some(l)) => {
            if h.confidence >= HEURISTIC_HIGH_CONFIDENCE {
                Some(h)
            } else if h.confidence < HEURISTIC_LOW_CONFIDENCE {
                Some(l)
            } else if h.confidence >= l.confidence {
                Some(h)
            } else {
                Some(l)
            }
        },
        (Some(h), None) => Some(h),
        (None, Some(l)) => Some(l),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_tool_block() {
        let raw = r#"Here is the plan:
```json
{"command":"list_nodes","args":{},"confidence":0.9}
```"#;
        let plan = parse_llm_tool_plan(raw).unwrap();
        assert_eq!(plan.command, "list_nodes");
    }

    #[test]
    fn rejects_invalid_command() {
        let raw = r#"{"command":"fake_cmd","args":{}}"#;
        assert!(parse_llm_tool_plan(raw).is_none());
    }
}
