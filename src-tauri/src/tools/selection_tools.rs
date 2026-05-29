use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "select_nodes_by_type",
        "Selects all nodes in the scene matching the given type(s). Useful for bulk operations on lights, cameras, figures, props, or bones.",
        ToolCategory::Selection,
        [
            tool_param("node_type", "Node type to select: figure, prop, light, camera, bone, all (default all)", false, ToolParamType::String),
            tool_param("mode", "Selection mode: replace, add, remove, toggle (default replace)", false, ToolParamType::String),
            tool_param("name_contains", "Optional substring filter to match node names", false, ToolParamType::String),
        ],
        "Result with count of selected nodes and their names",
        [
            "Select all lights in the scene",
            "Select all figures for batch posing",
            "Add all cameras to the current selection",
        ],
        handle_select_nodes_by_type
    );
    define_tool!(
        "save_selection",
        "Saves the current node selection with a name for later recall. Selections persist for the session.",
        ToolCategory::Selection,
        [
            tool_param("name", "Name to save the selection set as", true, ToolParamType::String),
        ],
        "Result confirming selection saved with node count",
        [
            "Save the current selection as 'hero_group'",
            "Name this selection 'background_props'",
        ],
        handle_save_selection
    );
    define_tool!(
        "load_selection",
        "Restores a previously saved selection by name. Replaces the current selection with the saved nodes.",
        ToolCategory::Selection,
        [
            tool_param("name", "Name of the saved selection to restore", true, ToolParamType::String),
        ],
        "Result with restored selection count and node names",
        [
            "Restore the 'hero_group' selection",
            "Load my 'background_props' selection",
        ],
        handle_load_selection
    );
    define_tool!(
        "select_hierarchy",
        "Selects a node and all its children recursively. Useful for selecting entire figure hierarchies or grouped props.",
        ToolCategory::Selection,
        [
            tool_param("node_id", "Root node ID to select along with its hierarchy", true, ToolParamType::String),
            tool_param("include_parent", "Also include the parent node (default true)", false, ToolParamType::Boolean),
        ],
        "Result with total node count in hierarchy and selected names",
        [
            "Select Genesis 9 and all its children",
            "Select the floor group with all sub-objects",
        ],
        handle_select_hierarchy
    );
    define_tool!(
        "invert_selection",
        "Inverts the current selection — unselects selected nodes and selects unselected ones. Only affects nodes of the specified type(s).",
        ToolCategory::Selection,
        [
            tool_param("node_type", "Restrict inversion to this type: figure, prop, light, camera, bone, all (default all)", false, ToolParamType::String),
        ],
        "Result with new selection count",
        [
            "Invert the current selection",
            "Select everything except the currently selected figures",
        ],
        handle_invert_selection
    );
}
fn handle_select_nodes_by_type(request: ToolRequest) -> ToolResponse {
    let node_type = request
        .get_str("node_type")
        .unwrap_or_else(|| "all".to_string());
    let mode = request
        .get_str("mode")
        .unwrap_or_else(|| "replace".to_string());
    let name_contains = request.get_str("name_contains");
    let result = crate::mcp_client::send_mcp_request(
        "select_by_type",
        serde_json::json!({
            "type": node_type,
            "mode": mode,
            "name_contains": name_contains,
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
                "select_nodes_by_type",
                serde_json::json!({ "selected": count, "type": node_type, "mode": mode }),
                format!("Selected {} {} nodes", count, node_type),
            )
        },
        Err(e) => ToolResponse::err("select_nodes_by_type", e),
    }
}
fn handle_save_selection(request: ToolRequest) -> ToolResponse {
    let name = request.get_str("name").unwrap_or_default();
    if name.is_empty() {
        return ToolResponse::err("save_selection", "name is required");
    }
    let result =
        crate::mcp_client::send_mcp_request("save_selection", serde_json::json!({ "name": name }));
    match result {
        Ok(r) => {
            let count = r
                .data
                .as_ref()
                .and_then(|d| d.get("count").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            ToolResponse::ok_with_message(
                "save_selection",
                serde_json::json!({ "name": name, "count": count }),
                format!("Saved selection '{}' with {} nodes", name, count),
            )
        },
        Err(e) => ToolResponse::err("save_selection", e),
    }
}
fn handle_load_selection(request: ToolRequest) -> ToolResponse {
    let name = request.get_str("name").unwrap_or_default();
    if name.is_empty() {
        return ToolResponse::err("load_selection", "name is required");
    }
    let result =
        crate::mcp_client::send_mcp_request("load_selection", serde_json::json!({ "name": name }));
    match result {
        Ok(r) => {
            let count = r
                .data
                .as_ref()
                .and_then(|d| d.get("count").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            ToolResponse::ok_with_message(
                "load_selection",
                serde_json::json!({ "name": name, "count": count }),
                format!("Restored selection '{}' ({} nodes)", name, count),
            )
        },
        Err(e) => ToolResponse::err("load_selection", e),
    }
}
fn handle_select_hierarchy(request: ToolRequest) -> ToolResponse {
    let node_id = request.get_str("node_id").unwrap_or_default();
    let include_parent = request.get_bool("include_parent").unwrap_or(true);
    if node_id.is_empty() {
        return ToolResponse::err("select_hierarchy", "node_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "select_hierarchy",
        serde_json::json!({
            "node_id": node_id,
            "include_parent": include_parent,
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
                "select_hierarchy",
                serde_json::json!({ "root": node_id, "count": count }),
                format!("Selected '{}' hierarchy ({} nodes)", node_id, count),
            )
        },
        Err(e) => ToolResponse::err("select_hierarchy", e),
    }
}
fn handle_invert_selection(request: ToolRequest) -> ToolResponse {
    let node_type = request
        .get_str("node_type")
        .unwrap_or_else(|| "all".to_string());
    let result = crate::mcp_client::send_mcp_request(
        "invert_selection",
        serde_json::json!({ "type": node_type }),
    );
    match result {
        Ok(r) => {
            let count = r
                .data
                .as_ref()
                .and_then(|d| d.get("count").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            ToolResponse::ok_with_message(
                "invert_selection",
                serde_json::json!({ "count": count }),
                format!("Inverted selection, {} nodes now selected", count),
            )
        },
        Err(e) => ToolResponse::err("invert_selection", e),
    }
}
