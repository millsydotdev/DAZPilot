use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "set_transform",
        "Sets the position, rotation, and/or scale of a node. Only provided parameters are modified — omit to leave unchanged.",
        ToolCategory::Transform,
        [
            tool_param("node_id", "Node ID to transform", true, ToolParamType::String),
            tool_param("position", "New position as [x, y, z] in Daz units", false, ToolParamType::FloatArray),
            tool_param("rotation", "New rotation as [x, y, z] Euler angles in degrees", false, ToolParamType::FloatArray),
            tool_param("scale", "New scale as [x, y, z] multiplier (1.0 = 100%)", false, ToolParamType::FloatArray),
            tool_param("space", "Coordinate space: world, local, parent (default world)", false, ToolParamType::String),
        ],
        "Result with the new transform values",
        [
            "Move Genesis 9 to position [0, 0, 0]",
            "Rotate the prop 90 degrees on Y axis",
            "Scale the selected node to 150%",
        ],
        handle_set_transform
    );
    define_tool!(
        "align_nodes",
        "Aligns selected nodes to a target node's position. Supports aligning individual axes.",
        ToolCategory::Transform,
        [
            tool_param(
                "target_node",
                "Node ID to align to",
                true,
                ToolParamType::String
            ),
            tool_param(
                "node_ids",
                "Array of node IDs to align, or omit for current selection",
                false,
                ToolParamType::StringArray
            ),
            tool_param(
                "axes",
                "Axes to align: x, y, z, xy, xz, yz, xyz (default xyz)",
                false,
                ToolParamType::String
            ),
            tool_param(
                "alignment",
                "Alignment type: min, center, max (default center)",
                false,
                ToolParamType::String
            ),
        ],
        "Result with count of aligned nodes",
        [
            "Align all selected props to the center of the ground plane",
            "Align the Y positions of all lights to match the key light",
        ],
        handle_align_nodes
    );
    define_tool!(
        "distribute_nodes",
        "Distributes selected nodes evenly along a specified axis. Useful for arranging props, lights, or clones.",
        ToolCategory::Transform,
        [
            tool_param("node_ids", "Array of node IDs to distribute, or omit for current selection", false, ToolParamType::StringArray),
            tool_param("axis", "Axis to distribute along: x, y, z (default x)", false, ToolParamType::String),
            tool_param("spacing", "Fixed spacing between nodes, or 'auto' for even distribution between endpoints (default 'auto')", false, ToolParamType::String),
            tool_param("bounds", "Total span for distribution as [min, max]. Auto-calculated if not set", false, ToolParamType::FloatArray),
        ],
        "Result with final positions of distributed nodes",
        [
            "Distribute the 5 lights evenly along X axis",
            "Space the props evenly with 50-unit gaps",
        ],
        handle_distribute_nodes
    );
    define_tool!(
        "snap_to_ground",
        "Snaps a node to the ground plane (Y=0) or to the top of another node. Useful for placing props on surfaces.",
        ToolCategory::Transform,
        [
            tool_param("node_id", "Node ID to snap to ground", true, ToolParamType::String),
            tool_param("offset_y", "Y offset from ground (e.g., raise slightly above surface)", false, ToolParamType::Number),
            tool_param("snap_to_node", "Optional node ID to snap onto instead of ground", false, ToolParamType::String),
        ],
        "Result with the node's new position",
        [
            "Snap the prop to the ground",
            "Place the cup on the table surface",
        ],
        handle_snap_to_ground
    );
    define_tool!(
        "reset_transform",
        "Resets position, rotation, or scale of a node back to default values. Selectively resets only specified properties.",
        ToolCategory::Transform,
        [
            tool_param("node_id", "Node ID to reset", true, ToolParamType::String),
            tool_param("reset_position", "Reset position to (0,0,0) (default true)", false, ToolParamType::Boolean),
            tool_param("reset_rotation", "Reset rotation to (0,0,0) (default true)", false, ToolParamType::Boolean),
            tool_param("reset_scale", "Reset scale to (1,1,1) (default true)", false, ToolParamType::Boolean),
        ],
        "Result confirming transform was reset",
        [
            "Reset the position of Genesis 9",
            "Reset all transforms on the selected prop",
            "Reset only rotation on the camera",
        ],
        handle_reset_transform
    );
}
fn handle_set_transform(request: ToolRequest) -> ToolResponse {
    let node_id = request.get_str("node_id").unwrap_or_default();
    let position = request.get_array("position");
    let rotation = request.get_array("rotation");
    let scale = request.get_array("scale");
    let space = request
        .get_str("space")
        .unwrap_or_else(|| "world".to_string());
    if node_id.is_empty() {
        return ToolResponse::err("set_transform", "node_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "set_transform",
        serde_json::json!({
            "node_id": node_id,
            "position": if position.is_empty() { serde_json::Value::Null } else { serde_json::json!(position) },
            "rotation": if rotation.is_empty() { serde_json::Value::Null } else { serde_json::json!(rotation) },
            "scale": if scale.is_empty() { serde_json::Value::Null } else { serde_json::json!(scale) },
            "space": space,
        }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_transform",
            serde_json::json!({ "node_id": node_id }),
            format!("Updated transform on '{}'", node_id),
        ),
        Err(e) => ToolResponse::err("set_transform", e),
    }
}
fn handle_align_nodes(request: ToolRequest) -> ToolResponse {
    let target_node = request.get_str("target_node").unwrap_or_default();
    let node_ids = request.get_array("node_ids");
    let axes = request.get_str("axes").unwrap_or_else(|| "xyz".to_string());
    let alignment = request
        .get_str("alignment")
        .unwrap_or_else(|| "center".to_string());
    if target_node.is_empty() {
        return ToolResponse::err("align_nodes", "target_node is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "align_nodes",
        serde_json::json!({
            "target_node": target_node,
            "node_ids": if node_ids.is_empty() { serde_json::Value::Null } else { serde_json::json!(node_ids) },
            "axes": axes,
            "alignment": alignment,
        }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "align_nodes",
            serde_json::json!({ "target": target_node }),
            format!("Aligned nodes to '{}'", target_node),
        ),
        Err(e) => ToolResponse::err("align_nodes", e),
    }
}
fn handle_distribute_nodes(request: ToolRequest) -> ToolResponse {
    let node_ids = request.get_array("node_ids");
    let axis = request.get_str("axis").unwrap_or_else(|| "x".to_string());
    let spacing = request
        .get_str("spacing")
        .unwrap_or_else(|| "auto".to_string());
    let bounds = request.get_array("bounds");
    let count = if node_ids.is_empty() {
        0
    } else {
        node_ids.len()
    };
    let result = crate::mcp_client::send_mcp_request(
        "distribute_nodes",
        serde_json::json!({
            "node_ids": if node_ids.is_empty() { serde_json::Value::Null } else { serde_json::json!(node_ids) },
            "axis": axis,
            "spacing": spacing,
            "bounds": if bounds.is_empty() { serde_json::Value::Null } else { serde_json::json!(bounds) },
        }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "distribute_nodes",
            serde_json::json!({ "distributed": count, "result": r.data }),
            format!("Distributed {} nodes along {} axis", count, axis),
        ),
        Err(e) => ToolResponse::err("distribute_nodes", e),
    }
}
fn handle_snap_to_ground(request: ToolRequest) -> ToolResponse {
    let node_id = request.get_str("node_id").unwrap_or_default();
    let offset_y = request.get_f64("offset_y").unwrap_or(0.0);
    let snap_to_node = request.get_str("snap_to_node");
    if node_id.is_empty() {
        return ToolResponse::err("snap_to_ground", "node_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "snap_to_ground",
        serde_json::json!({
            "node_id": node_id,
            "offset_y": offset_y,
            "snap_to_node": snap_to_node,
        }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "snap_to_ground",
            serde_json::json!({ "node_id": node_id, "result": r.data }),
            format!("Snapped '{}' to ground", node_id),
        ),
        Err(e) => ToolResponse::err("snap_to_ground", e),
    }
}
fn handle_reset_transform(request: ToolRequest) -> ToolResponse {
    let node_id = request.get_str("node_id").unwrap_or_default();
    let reset_position = request.get_bool("reset_position").unwrap_or(true);
    let reset_rotation = request.get_bool("reset_rotation").unwrap_or(true);
    let reset_scale = request.get_bool("reset_scale").unwrap_or(true);
    if node_id.is_empty() {
        return ToolResponse::err("reset_transform", "node_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "reset_transform",
        serde_json::json!({
            "node_id": node_id,
            "position": reset_position,
            "rotation": reset_rotation,
            "scale": reset_scale,
        }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "reset_transform",
            serde_json::json!({ "node_id": node_id }),
            format!("Reset transform on '{}'", node_id),
        ),
        Err(e) => ToolResponse::err("reset_transform", e),
    }
}
