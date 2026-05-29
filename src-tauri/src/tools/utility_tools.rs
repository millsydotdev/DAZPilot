use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "get_scene_statistics",
        "Returns comprehensive statistics about the current scene: node counts by type, polygon counts, memory usage, and texture resolution summary.",
        ToolCategory::Utility,
        [
            tool_param("detailed", "Whether to include per-node breakdown (default false)", false, ToolParamType::Boolean),
        ],
        "Result with scene statistics including counts, memory, and texture info",
        [
            "How big is my current scene?",
            "Show me scene statistics",
            "What's the polygon count of my scene?",
        ],
        handle_get_scene_statistics
    );
    define_tool!(
        "duplicate_nodes",
        "Duplicates selected or specified nodes in the scene. Can create single or multiple copies with optional offsets.",
        ToolCategory::Utility,
        [
            tool_param("node_ids", "Array of node IDs to duplicate, or omit to duplicate current selection", false, ToolParamType::StringArray),
            tool_param("copies", "Number of copies to create (default 1)", false, ToolParamType::Integer),
            tool_param("offset_x", "X offset between copies", false, ToolParamType::Number),
            tool_param("offset_y", "Y offset between copies", false, ToolParamType::Number),
            tool_param("offset_z", "Z offset between copies", false, ToolParamType::Number),
        ],
        "Result with list of newly created node IDs",
        [
            "Duplicate the selected prop",
            "Create 5 copies with spacing along X axis",
            "Duplicate my character 3 times in a row",
        ],
        handle_duplicate_nodes
    );
    define_tool!(
        "delete_nodes",
        "Deletes specified nodes from the scene. Supports undo via confirmation.",
        ToolCategory::Utility,
        [
            tool_param(
                "node_ids",
                "Array of node IDs to delete",
                true,
                ToolParamType::StringArray
            ),
            tool_param(
                "confirm",
                "Set to true to confirm deletion (safety measure)",
                true,
                ToolParamType::Boolean
            ),
        ],
        "Result with count of deleted nodes",
        [
            "Delete the selected lights",
            "Remove props 'Chair' and 'Table' from the scene",
        ],
        handle_delete_nodes
    );
    define_tool!(
        "rename_node",
        "Renames a node in the scene. Useful for organizing the scene hierarchy with meaningful names.",
        ToolCategory::Utility,
        [
            tool_param("node_id", "Current node ID or name to rename", true, ToolParamType::String),
            tool_param("new_name", "New name for the node", true, ToolParamType::String),
        ],
        "Result confirming the rename",
        [
            "Rename 'Genesis9' to 'Hero Character'",
            "Rename the selected light to 'Key Light'",
        ],
        handle_rename_node
    );
    define_tool!(
        "group_nodes",
        "Groups specified nodes under a new parent null node. Useful for scene organization and batch transforms.",
        ToolCategory::Utility,
        [
            tool_param("node_ids", "Array of node IDs to group together", true, ToolParamType::StringArray),
            tool_param("group_name", "Name for the new group", true, ToolParamType::String),
        ],
        "Result with the new group node ID",
        [
            "Group the background props under 'Background'",
            "Group the selected lights into 'Lighting Rig'",
        ],
        handle_group_nodes
    );
    define_tool!(
        "center_on_node",
        "Centers the viewport focus on a specific node. Useful for quickly navigating to objects in complex scenes.",
        ToolCategory::Utility,
        [
            tool_param("node_id", "Node ID to center the view on", true, ToolParamType::String),
            tool_param("animate", "Animate the camera transition (default true)", false, ToolParamType::Boolean),
        ],
        "Result confirming the viewport focus changed",
        [
            "Focus the view on Genesis 9",
            "Center on the main prop",
        ],
        handle_center_on_node
    );
}
fn handle_get_scene_statistics(request: ToolRequest) -> ToolResponse {
    let detailed = request.get_bool("detailed").unwrap_or(false);
    let result = crate::mcp_client::send_mcp_request(
        "get_scene_stats",
        serde_json::json!({ "detailed": detailed }),
    );
    match result {
        Ok(r) => {
            let data = r.data.unwrap_or(serde_json::json!({}));
            ToolResponse::ok_with_message(
                "get_scene_statistics",
                data.clone(),
                format!(
                    "Scene statistics retrieved{}",
                    if detailed { " (detailed)" } else { "" }
                ),
            )
        },
        Err(e) => ToolResponse::err("get_scene_statistics", e),
    }
}
fn handle_duplicate_nodes(request: ToolRequest) -> ToolResponse {
    let node_ids = request.get_array("node_ids");
    let copies = request.get_i64("copies").unwrap_or(1).max(1);
    let offset_x = request.get_f64("offset_x").unwrap_or(0.0);
    let offset_y = request.get_f64("offset_y").unwrap_or(0.0);
    let offset_z = request.get_f64("offset_z").unwrap_or(0.0);
    let result = crate::mcp_client::send_mcp_request(
        "duplicate_nodes",
        serde_json::json!({
            "node_ids": node_ids,
            "copies": copies,
            "offset": { "x": offset_x, "y": offset_y, "z": offset_z },
        }),
    );
    match result {
        Ok(r) => {
            let created = r
                .data
                .as_ref()
                .and_then(|d| d.get("created"))
                .cloned()
                .unwrap_or(serde_json::json!([]));
            ToolResponse::ok_with_message(
                "duplicate_nodes",
                serde_json::json!({ "copies": copies, "created": created }),
                format!("Created {} duplicate(s)", copies),
            )
        },
        Err(e) => ToolResponse::err("duplicate_nodes", e),
    }
}
fn handle_delete_nodes(request: ToolRequest) -> ToolResponse {
    let node_ids = request.get_array("node_ids");
    let confirm = request.get_bool("confirm").unwrap_or(false);
    if !confirm {
        return ToolResponse::err(
            "delete_nodes",
            "Deletion requires confirm=true as a safety measure",
        );
    }
    let result = crate::mcp_client::send_mcp_request(
        "delete_nodes",
        serde_json::json!({ "node_ids": node_ids }),
    );
    match result {
        Ok(_) => {
            let count = node_ids.len();
            ToolResponse::ok_with_message(
                "delete_nodes",
                serde_json::json!({ "deleted": count }),
                format!("Deleted {} node(s)", count),
            )
        },
        Err(e) => ToolResponse::err("delete_nodes", e),
    }
}
fn handle_rename_node(request: ToolRequest) -> ToolResponse {
    let node_id = request.get_str("node_id").unwrap_or_default();
    let new_name = request.get_str("new_name").unwrap_or_default();
    if node_id.is_empty() || new_name.is_empty() {
        return ToolResponse::err("rename_node", "node_id and new_name are required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "rename_node",
        serde_json::json!({ "node_id": node_id, "new_name": new_name }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "rename_node",
            serde_json::json!({ "old_name": node_id, "new_name": new_name }),
            format!("Renamed '{}' to '{}'", node_id, new_name),
        ),
        Err(e) => ToolResponse::err("rename_node", e),
    }
}
fn handle_group_nodes(request: ToolRequest) -> ToolResponse {
    let node_ids = request.get_array("node_ids");
    let group_name = request.get_str("group_name").unwrap_or_default();
    if group_name.is_empty() {
        return ToolResponse::err("group_nodes", "group_name is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "group_nodes",
        serde_json::json!({ "node_ids": node_ids, "group_name": group_name }),
    );
    match result {
        Ok(r) => {
            let group_id = r.data.as_ref().and_then(|d| d.get("group_id")).cloned();
            ToolResponse::ok_with_message(
                "group_nodes",
                serde_json::json!({ "group_name": group_name, "group_id": group_id }),
                format!("Grouped nodes under '{}'", group_name),
            )
        },
        Err(e) => ToolResponse::err("group_nodes", e),
    }
}
fn handle_center_on_node(request: ToolRequest) -> ToolResponse {
    let node_id = request.get_str("node_id").unwrap_or_default();
    let animate = request.get_bool("animate").unwrap_or(true);
    if node_id.is_empty() {
        return ToolResponse::err("center_on_node", "node_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "center_view",
        serde_json::json!({ "node_id": node_id, "animate": animate }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "center_on_node",
            serde_json::json!({ "node_id": node_id }),
            format!("Centered view on '{}'", node_id),
        ),
        Err(e) => ToolResponse::err("center_on_node", e),
    }
}
