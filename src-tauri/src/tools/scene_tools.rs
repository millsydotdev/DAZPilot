use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "save_current_scene",
        "Saves the current Daz scene to a file. Supports .duf (native), .obj (geometry), and .fbx (exchange) formats.",
        ToolCategory::Scene,
        [
            tool_param("filepath", "Full file path to save the scene to", true, ToolParamType::String),
            tool_param("format", "Format: duf, obj, fbx (default duf)", false, ToolParamType::String),
            tool_param("include_assets", "Include asset references (true) or embed minimal (false, default true)", false, ToolParamType::Boolean),
        ],
        "Result confirming the scene was saved with file path and size",
        [
            "Save the current scene as my_scene.duf",
            "Save the scene for export as FBX",
        ],
        handle_save_current_scene
    );
    define_tool!(
        "load_scene_file",
        "Loads a Daz scene file (.duf) into the current scene or replaces the current scene entirely.",
        ToolCategory::Scene,
        [
            tool_param("filepath", "Full path to the .duf scene file", true, ToolParamType::String),
            tool_param("merge", "Merge into current scene (true) or replace scene (false, default false)", false, ToolParamType::Boolean),
        ],
        "Result confirming the scene was loaded with node count",
        [
            "Load my_scene.duf",
            "Merge a scene file into the current scene",
        ],
        handle_load_scene_file
    );
    define_tool!(
        "clear_scene",
        "Clears all nodes from the current scene. Optionally keep the default camera and environment.",
        ToolCategory::Scene,
        [
            tool_param("confirm", "Set to true to confirm scene clearing (safety measure)", true, ToolParamType::Boolean),
            tool_param("keep_defaults", "Keep default camera and environment (default true)", false, ToolParamType::Boolean),
        ],
        "Result confirming the scene was cleared",
        [
            "Clear the scene and start fresh",
            "Reset the scene keeping defaults",
        ],
        handle_clear_scene
    );
    define_tool!(
        "merge_scene_file",
        "Merges a Daz scene file into the current scene. Useful for combining scenes or adding pre-built environments.",
        ToolCategory::Scene,
        [
            tool_param("filepath", "Full path to the .duf scene file to merge", true, ToolParamType::String),
            tool_param("import_location", "Position to place the merged content as [x, y, z] (default [0,0,0])", false, ToolParamType::FloatArray),
        ],
        "Result with count of nodes added from the merged scene",
        [
            "Merge a background environment into the scene",
            "Import props from another scene file",
        ],
        handle_merge_scene_file
    );
    define_tool!(
        "get_scene_info",
        "Returns metadata about the current scene: file path, modified status, node count, render settings summary, and content statistics.",
        ToolCategory::Scene,
        [
            tool_param("detailed", "Include detailed node breakdown (default false)", false, ToolParamType::Boolean),
        ],
        "Result with comprehensive scene metadata",
        [
            "Tell me about the current scene",
            "What's in my scene? Show me details",
        ],
        handle_get_scene_info
    );
}
fn handle_save_current_scene(request: ToolRequest) -> ToolResponse {
    let filepath = request.get_str("filepath").unwrap_or_default();
    let format = request
        .get_str("format")
        .unwrap_or_else(|| "duf".to_string());
    let include_assets = request.get_bool("include_assets").unwrap_or(true);
    if filepath.is_empty() {
        return ToolResponse::err("save_current_scene", "filepath is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "save_scene",
        serde_json::json!({ "filepath": filepath, "format": format, "include_assets": include_assets }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "save_current_scene",
            serde_json::json!({ "filepath": filepath, "result": r.data }),
            format!("Saved scene to '{}'", filepath),
        ),
        Err(e) => ToolResponse::err("save_current_scene", e),
    }
}
fn handle_load_scene_file(request: ToolRequest) -> ToolResponse {
    let filepath = request.get_str("filepath").unwrap_or_default();
    let merge = request.get_bool("merge").unwrap_or(false);
    if filepath.is_empty() {
        return ToolResponse::err("load_scene_file", "filepath is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "load_scene",
        serde_json::json!({ "filepath": filepath, "merge": merge }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "load_scene_file",
            serde_json::json!({ "filepath": filepath, "result": r.data }),
            format!("Loaded scene from '{}'", filepath),
        ),
        Err(e) => ToolResponse::err("load_scene_file", e),
    }
}
fn handle_clear_scene(request: ToolRequest) -> ToolResponse {
    let confirm = request.get_bool("confirm").unwrap_or(false);
    let keep_defaults = request.get_bool("keep_defaults").unwrap_or(true);
    if !confirm {
        return ToolResponse::err(
            "clear_scene",
            "Clearing requires confirm=true as a safety measure",
        );
    }
    let result = crate::mcp_client::send_mcp_request(
        "clear_scene",
        serde_json::json!({ "keep_defaults": keep_defaults }),
    );
    match result {
        Ok(_) => {
            ToolResponse::ok_with_message("clear_scene", serde_json::json!({}), "Scene cleared")
        },
        Err(e) => ToolResponse::err("clear_scene", e),
    }
}
fn handle_merge_scene_file(request: ToolRequest) -> ToolResponse {
    let filepath = request.get_str("filepath").unwrap_or_default();
    let import_location = request.get_array("import_location");
    if filepath.is_empty() {
        return ToolResponse::err("merge_scene_file", "filepath is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "merge_scene",
        serde_json::json!({
            "filepath": filepath,
            "import_location": if import_location.is_empty() { serde_json::json!([0,0,0]) } else { serde_json::json!(import_location) },
        }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "merge_scene_file",
            serde_json::json!({ "filepath": filepath, "result": r.data }),
            format!("Merged scene from '{}'", filepath),
        ),
        Err(e) => ToolResponse::err("merge_scene_file", e),
    }
}
fn handle_get_scene_info(request: ToolRequest) -> ToolResponse {
    let detailed = request.get_bool("detailed").unwrap_or(false);
    let result = crate::mcp_client::send_mcp_request(
        "get_scene_info",
        serde_json::json!({ "detailed": detailed }),
    );
    match result {
        Ok(r) => {
            let data = r.data.unwrap_or(serde_json::json!({}));
            ToolResponse::ok_with_message("get_scene_info", data.clone(), "Scene info retrieved")
        },
        Err(e) => ToolResponse::err("get_scene_info", e),
    }
}
