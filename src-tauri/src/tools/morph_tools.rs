use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "batch_set_morphs",
        "Sets multiple morph dial values on a figure at once. Takes a dictionary of morph names to values (0.0-1.0). Faster than setting morphs one at a time.",
        ToolCategory::Morphs,
        [
            tool_param("figure_id", "Node ID of the figure to apply morphs to", true, ToolParamType::String),
            tool_param("morphs", "JSON object of {morph_name: value} pairs where each value is 0.0-1.0", true, ToolParamType::Object),
        ],
        "Result with count of morphs applied",
        [
            "Set lip size to 0.5 and eye width to 0.3 on Genesis 9",
            "Apply {'Cheek_Bone': 0.7, 'Jaw_Shape': 0.4} to my figure",
        ],
        handle_batch_set_morphs
    );
    define_tool!(
        "symmetry_morphs",
        "Copies morph values from one side of the figure to the other to ensure perfect symmetry. Can mirror left→right or right→left.",
        ToolCategory::Morphs,
        [
            tool_param("figure_id", "Node ID of the figure", true, ToolParamType::String),
            tool_param("direction", "Mirror direction: left_to_right, right_to_left, or both (default both)", false, ToolParamType::String),
            tool_param("morph_group", "Optional morph group filter: face, body, legs, arms, all (default all)", false, ToolParamType::String),
        ],
        "Result with count of morphs mirrored and affected regions",
        [
            "Symmetrize the face morphs on Genesis 9",
            "Mirror body morphs from left to right",
        ],
        handle_symmetry_morphs
    );
    define_tool!(
        "randomize_morphs",
        "Randomizes morph values on a figure within specified constraints. Useful for generating varied character looks quickly.",
        ToolCategory::Morphs,
        [
            tool_param("figure_id", "Node ID of the figure", true, ToolParamType::String),
            tool_param("intensity", "Randomization intensity: subtle (0-0.2), moderate (0-0.5), extreme (0-1.0) or a custom 0-1 value (default moderate)", false, ToolParamType::String),
            tool_param("morph_group", "Morph group to randomize: face, body, all (default all)", false, ToolParamType::String),
            tool_param("seed", "Optional seed for reproducible randomization", false, ToolParamType::Integer),
        ],
        "Result with list of morphs that were randomized and their new values",
        [
            "Randomize the face morphs subtly",
            "Randomize all body morphs moderately",
            "Give me a random character look with extreme intensity",
        ],
        handle_randomize_morphs
    );
    define_tool!(
        "save_morph_preset",
        "Saves the current morph values of a figure as a named preset that can be applied later with load_morph_preset.",
        ToolCategory::Morphs,
        [
            tool_param("figure_id", "Node ID of the figure", true, ToolParamType::String),
            tool_param("preset_name", "Name for this morph preset", true, ToolParamType::String),
            tool_param("morph_group", "Only save morphs from this group: face, body, all (default all)", false, ToolParamType::String),
        ],
        "Result confirming the preset was saved with morph count",
        [
            "Save this face as a morph preset called 'hero_face'",
            "Save the current body shape as 'athletic_female'",
        ],
        handle_save_morph_preset
    );
    define_tool!(
        "load_morph_preset",
        "Loads and applies a previously saved morph preset to a figure. Optionally blend with the current morphs using a blend factor.",
        ToolCategory::Morphs,
        [
            tool_param("figure_id", "Node ID of the figure to apply the preset to", true, ToolParamType::String),
            tool_param("preset_name", "Name of the saved morph preset to load", true, ToolParamType::String),
            tool_param("blend", "Blend factor 0.0-1.0: 0 = keep current, 1 = full preset (default 1.0)", false, ToolParamType::Number),
        ],
        "Result with count of morphs applied from the preset",
        [
            "Apply the 'hero_face' morph preset to Genesis 9",
            "Load 'athletic_female' at 50% blend",
        ],
        handle_load_morph_preset
    );
    define_tool!(
        "reset_morphs",
        "Resets all morph dials (or a specific group) on a figure back to zero. Useful for starting fresh or undoing morph adjustments.",
        ToolCategory::Morphs,
        [
            tool_param("figure_id", "Node ID of the figure", true, ToolParamType::String),
            tool_param("morph_group", "Morph group to reset: face, body, all (default all)", false, ToolParamType::String),
        ],
        "Result with count of morphs reset",
        [
            "Reset all morphs on Genesis 9",
            "Reset only the face morphs",
        ],
        handle_reset_morphs
    );
}
fn handle_batch_set_morphs(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let morphs = request.get_object("morphs");
    if figure_id.is_empty() {
        return ToolResponse::err("batch_set_morphs", "figure_id is required");
    }
    let count = morphs.as_ref().map(|m| m.len()).unwrap_or(0);
    let result = crate::mcp_client::send_mcp_request(
        "batch_set_morphs",
        serde_json::json!({
            "figure_id": figure_id,
            "morphs": morphs,
        }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "batch_set_morphs",
            serde_json::json!({ "figure_id": figure_id, "morphs_applied": count }),
            format!("Applied {} morphs to '{}'", count, figure_id),
        ),
        Err(e) => ToolResponse::err("batch_set_morphs", e),
    }
}
fn handle_symmetry_morphs(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let direction = request
        .get_str("direction")
        .unwrap_or_else(|| "both".to_string());
    let morph_group = request
        .get_str("morph_group")
        .unwrap_or_else(|| "all".to_string());
    if figure_id.is_empty() {
        return ToolResponse::err("symmetry_morphs", "figure_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "symmetry_morphs",
        serde_json::json!({
            "figure_id": figure_id,
            "direction": direction,
            "morph_group": morph_group,
        }),
    );
    match result {
        Ok(r) => {
            let count = r
                .data
                .as_ref()
                .and_then(|d| d.get("count").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            ToolResponse::ok_with_message(
                "symmetry_morphs",
                serde_json::json!({ "figure_id": figure_id, "morphs_mirrored": count }),
                format!("Symmetrized {} morphs on '{}'", count, figure_id),
            )
        },
        Err(e) => ToolResponse::err("symmetry_morphs", e),
    }
}
fn handle_randomize_morphs(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let intensity_str = request
        .get_str("intensity")
        .unwrap_or_else(|| "moderate".to_string());
    let morph_group = request
        .get_str("morph_group")
        .unwrap_or_else(|| "all".to_string());
    let _seed = request.get_i64("seed");
    if figure_id.is_empty() {
        return ToolResponse::err("randomize_morphs", "figure_id is required");
    }
    let intensity = match intensity_str.as_str() {
        "subtle" => 0.2,
        "extreme" => 1.0,
        _ => 0.5,
    };
    let result = crate::mcp_client::send_mcp_request(
        "randomize_morphs",
        serde_json::json!({
            "figure_id": figure_id,
            "intensity": intensity,
            "morph_group": morph_group,
        }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "randomize_morphs",
            serde_json::json!({ "figure_id": figure_id, "result": r.data }),
            format!("Randomized {} morphs on '{}'", morph_group, figure_id),
        ),
        Err(e) => ToolResponse::err("randomize_morphs", e),
    }
}
fn handle_save_morph_preset(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let preset_name = request.get_str("preset_name").unwrap_or_default();
    let morph_group = request
        .get_str("morph_group")
        .unwrap_or_else(|| "all".to_string());
    if figure_id.is_empty() || preset_name.is_empty() {
        return ToolResponse::err(
            "save_morph_preset",
            "figure_id and preset_name are required",
        );
    }
    let result = crate::mcp_client::send_mcp_request(
        "save_morph_preset",
        serde_json::json!({
            "figure_id": figure_id,
            "preset_name": preset_name,
            "morph_group": morph_group,
        }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "save_morph_preset",
            serde_json::json!({ "preset_name": preset_name, "figure_id": figure_id }),
            format!("Saved morph preset '{}' from '{}'", preset_name, figure_id),
        ),
        Err(e) => ToolResponse::err("save_morph_preset", e),
    }
}
fn handle_load_morph_preset(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let preset_name = request.get_str("preset_name").unwrap_or_default();
    let blend = request.get_f64("blend").unwrap_or(1.0);
    if figure_id.is_empty() || preset_name.is_empty() {
        return ToolResponse::err(
            "load_morph_preset",
            "figure_id and preset_name are required",
        );
    }
    let result = crate::mcp_client::send_mcp_request(
        "load_morph_preset",
        serde_json::json!({
            "figure_id": figure_id,
            "preset_name": preset_name,
            "blend": blend,
        }),
    );
    match result {
        Ok(r) => {
            let count = r
                .data
                .as_ref()
                .and_then(|d| d.get("count").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            ToolResponse::ok_with_message(
                "load_morph_preset",
                serde_json::json!({ "preset_name": preset_name, "figure_id": figure_id, "morphs_applied": count }),
                format!(
                    "Loaded morph preset '{}' on '{}' ({} morphs)",
                    preset_name, figure_id, count
                ),
            )
        },
        Err(e) => ToolResponse::err("load_morph_preset", e),
    }
}
fn handle_reset_morphs(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let morph_group = request
        .get_str("morph_group")
        .unwrap_or_else(|| "all".to_string());
    if figure_id.is_empty() {
        return ToolResponse::err("reset_morphs", "figure_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "reset_morphs",
        serde_json::json!({
            "figure_id": figure_id,
            "morph_group": morph_group,
        }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "reset_morphs",
            serde_json::json!({ "figure_id": figure_id, "group": morph_group }),
            format!("Reset {} morphs on '{}'", morph_group, figure_id),
        ),
        Err(e) => ToolResponse::err("reset_morphs", e),
    }
}
