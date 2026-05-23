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
            "- {} ({}): {} | params: {}",
            schema.name, schema.category, schema.description, params
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
    format!(
        "You are DazPilot's action planner. Choose ONE bridge command to satisfy the user request.\n\
         Reply with ONLY a JSON object (no markdown prose) in this exact shape:\n\
         {{\"command\":\"<name>\",\"args\":{{}},\"confidence\":0.0-1.0,\"sdk_refs\":[]}}\n\
         Use command \"none\" with empty args if no command fits.\n\
         Do NOT use run_script. Prefer safe read-only commands when unsure.\n\n\
         Available commands:\n{}\n\n\
         Scene context:\n{}\n\n\
         User request:\n{}",
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
    let response = crate::ai_providers::run_chat(
        provider,
        model,
        prompt,
        api_key,
        base_url,
        0.1,
        512,
        None,
    )
    .await
    .ok()?;
    parse_llm_tool_plan(&response)
}

/// Merge heuristic and LLM plans using a confidence-informed strategy:
/// - heuristic >= 0.85 → prefer heuristic (high-confidence keyword match)
/// - heuristic < 0.70  → prefer LLM (heuristic uncertain, LLM may generalize better)
/// - otherwise         → pick whichever has higher confidence
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
        }
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
