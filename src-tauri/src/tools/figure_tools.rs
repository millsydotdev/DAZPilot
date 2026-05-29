use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "add_figure",
        "Adds a new figure to the scene by type or preset. Supports Genesis 8, Genesis 9, and other base figures with optional preset morphs.",
        ToolCategory::Figure,
        [
            tool_param("figure_type", "Figure type: genesis9, genesis8, genesis81, victoria8, michael8, etc.", true, ToolParamType::String),
            tool_param("preset", "Optional figure preset name to apply after loading", false, ToolParamType::String),
            tool_param("name", "Custom name for the figure node (defaults to figure type)", false, ToolParamType::String),
            tool_param("add_to_scene", "Add to current scene (true) or open in new scene (default true)", false, ToolParamType::Boolean),
        ],
        "Result with the new figure's node ID and name",
        [
            "Add a Genesis 9 base figure to the scene",
            "Add Victoria 8 with a fantasy preset",
            "Load a Genesis 8 male character",
        ],
        handle_add_figure
    );
    define_tool!(
        "remove_figure",
        "Removes a figure and all its child nodes (joints, morphs, worn items) from the scene.",
        ToolCategory::Figure,
        [tool_param(
            "figure_id",
            "Node ID of the figure to remove",
            true,
            ToolParamType::String
        ),],
        "Result confirming the figure was removed",
        [
            "Remove Genesis 9 from the scene",
            "Delete the figure named 'Background Character'",
        ],
        handle_remove_figure
    );
    define_tool!(
        "list_figures",
        "Lists all figures in the current scene with their node IDs, type, and worn item counts.",
        ToolCategory::Figure,
        [tool_param(
            "include_details",
            "Include morph names, worn items, and material info (default false)",
            false,
            ToolParamType::Boolean
        ),],
        "Result with array of figure objects",
        [
            "What figures are in my scene?",
            "List all characters with details",
        ],
        handle_list_figures
    );
    define_tool!(
        "apply_figure_preset",
        "Applies a full-body figure preset (morphs + materials) to an existing figure. Presets can be from content library or previously saved.",
        ToolCategory::Figure,
        [
            tool_param("figure_id", "Node ID of the target figure", true, ToolParamType::String),
            tool_param("preset_path", "Path to the figure preset file or name of saved preset", true, ToolParamType::String),
            tool_param("apply_morphs", "Apply morph values from preset (default true)", false, ToolParamType::Boolean),
            tool_param("apply_materials", "Apply materials from preset (default true)", false, ToolParamType::Boolean),
        ],
        "Result with count of morphs and materials applied",
        [
            "Apply a character preset to Genesis 9",
            "Load a saved character preset onto my figure",
        ],
        handle_apply_figure_preset
    );
    define_tool!(
        "set_figure_visibility",
        "Shows or hides a figure in the viewport. Can also toggle visibility of specific figure parts (eyelashes, teeth, etc.).",
        ToolCategory::Figure,
        [
            tool_param("figure_id", "Node ID of the figure", true, ToolParamType::String),
            tool_param("visible", "Show (true) or hide (false) the figure", true, ToolParamType::Boolean),
            tool_param("include_worn_items", "Also toggle visibility of worn items (default true)", false, ToolParamType::Boolean),
        ],
        "Result confirming visibility change",
        [
            "Hide Genesis 9 in the viewport",
            "Show the background character",
        ],
        handle_set_figure_visibility
    );
}
fn handle_add_figure(request: ToolRequest) -> ToolResponse {
    let figure_type = request.get_str("figure_type").unwrap_or_default();
    let preset = request.get_str("preset");
    let name = request.get_str("name");
    let add_to_scene = request.get_bool("add_to_scene").unwrap_or(true);
    if figure_type.is_empty() {
        return ToolResponse::err("add_figure", "figure_type is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "add_figure",
        serde_json::json!({
            "figure_type": figure_type,
            "preset": preset,
            "name": name,
            "add_to_scene": add_to_scene,
        }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "add_figure",
            serde_json::json!({ "result": r.data }),
            format!("Added '{}' to the scene", figure_type),
        ),
        Err(e) => ToolResponse::err("add_figure", e),
    }
}
fn handle_remove_figure(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    if figure_id.is_empty() {
        return ToolResponse::err("remove_figure", "figure_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "remove_figure",
        serde_json::json!({ "figure_id": figure_id }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "remove_figure",
            serde_json::json!({ "figure_id": figure_id }),
            format!("Removed figure '{}'", figure_id),
        ),
        Err(e) => ToolResponse::err("remove_figure", e),
    }
}
fn handle_list_figures(request: ToolRequest) -> ToolResponse {
    let include_details = request.get_bool("include_details").unwrap_or(false);
    let result = crate::mcp_client::send_mcp_request(
        "list_figures",
        serde_json::json!({ "include_details": include_details }),
    );
    match result {
        Ok(r) => {
            let figures = r.data.unwrap_or(serde_json::json!([]));
            ToolResponse::ok_with_message(
                "list_figures",
                serde_json::json!({ "figures": figures }),
                "Retrieved figure list",
            )
        },
        Err(e) => ToolResponse::err("list_figures", e),
    }
}
fn handle_apply_figure_preset(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let preset_path = request.get_str("preset_path").unwrap_or_default();
    let apply_morphs = request.get_bool("apply_morphs").unwrap_or(true);
    let apply_materials = request.get_bool("apply_materials").unwrap_or(true);
    if figure_id.is_empty() || preset_path.is_empty() {
        return ToolResponse::err(
            "apply_figure_preset",
            "figure_id and preset_path are required",
        );
    }
    let result = crate::mcp_client::send_mcp_request(
        "apply_figure_preset",
        serde_json::json!({
            "figure_id": figure_id,
            "preset_path": preset_path,
            "apply_morphs": apply_morphs,
            "apply_materials": apply_materials,
        }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "apply_figure_preset",
            serde_json::json!({ "figure_id": figure_id, "result": r.data }),
            format!("Applied preset '{}' to '{}'", preset_path, figure_id),
        ),
        Err(e) => ToolResponse::err("apply_figure_preset", e),
    }
}
fn handle_set_figure_visibility(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let visible = request.get_bool("visible").unwrap_or(true);
    let include_worn_items = request.get_bool("include_worn_items").unwrap_or(true);
    if figure_id.is_empty() {
        return ToolResponse::err("set_figure_visibility", "figure_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "set_visibility",
        serde_json::json!({
            "node_id": figure_id,
            "visible": visible,
            "recursive": include_worn_items,
        }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_figure_visibility",
            serde_json::json!({ "figure_id": figure_id, "visible": visible }),
            if visible {
                format!("Showed '{}'", figure_id)
            } else {
                format!(" hid '{}'", figure_id)
            },
        ),
        Err(e) => ToolResponse::err("set_figure_visibility", e),
    }
}
