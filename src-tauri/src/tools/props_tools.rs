use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "load_prop",
        "Loads a prop into the scene from the content library. Searches by name with optional category filter.",
        ToolCategory::Props,
        [
            tool_param("prop_name", "Name or search term for the prop", true, ToolParamType::String),
            tool_param("category", "Category filter: furniture, decor, nature, vehicles, architecture, food, weapons, custom (optional)", false, ToolParamType::String),
            tool_param("position", "Placement position as [x, y, z] (default [0, 0, 0])", false, ToolParamType::FloatArray),
        ],
        "Result with the loaded prop's node ID and name",
        [
            "Load a wooden chair prop",
            "Search for and load a fantasy sword",
            "Load a palm tree at position [100, 0, 50]",
        ],
        handle_load_prop
    );
    define_tool!(
        "position_prop",
        "Moves a prop to a specific position in the scene. Supports absolute positioning or relative offset.",
        ToolCategory::Props,
        [
            tool_param("prop_id", "Node ID of the prop to move", true, ToolParamType::String),
            tool_param("position", "Target position as [x, y, z]", true, ToolParamType::FloatArray),
            tool_param("relative", "If true, position is added as offset from current (default false)", false, ToolParamType::Boolean),
        ],
        "Result with the prop's new position",
        [
            "Move the chair to position [50, 0, 30]",
            "Move the sword 10 units to the right",
        ],
        handle_position_prop
    );
    define_tool!(
        "rotate_prop",
        "Rotates a prop by specified Euler angles. Supports absolute or relative rotation.",
        ToolCategory::Props,
        [
            tool_param(
                "prop_id",
                "Node ID of the prop to rotate",
                true,
                ToolParamType::String
            ),
            tool_param(
                "rotation",
                "Rotation as [x, y, z] Euler angles in degrees",
                true,
                ToolParamType::FloatArray
            ),
            tool_param(
                "relative",
                "If true, rotation is added to current (default false)",
                false,
                ToolParamType::Boolean
            ),
        ],
        "Result with the prop's new rotation",
        [
            "Rotate the chair 90 degrees on Y axis",
            "Tilt the picture frame 15 degrees on X",
        ],
        handle_rotate_prop
    );
    define_tool!(
        "scale_prop",
        "Scales a prop uniformly or per-axis. Useful for resizing props to fit the scene scale.",
        ToolCategory::Props,
        [
            tool_param(
                "prop_id",
                "Node ID of the prop to scale",
                true,
                ToolParamType::String
            ),
            tool_param(
                "scale",
                "Scale value: single number for uniform, or [x, y, z] for per-axis. 1.0 = 100%",
                true,
                ToolParamType::FloatArray
            ),
            tool_param(
                "relative",
                "If true, scale multiplies current scale (default false)",
                false,
                ToolParamType::Boolean
            ),
        ],
        "Result with the prop's new scale",
        [
            "Scale the chair to 120%",
            "Make the prop twice as tall",
            "Flatten the prop on Y axis to 50%",
        ],
        handle_scale_prop
    );
    define_tool!(
        "hide_show_prop",
        "Shows or hides a prop in the viewport and render. Can also isolate a single prop by hiding all others.",
        ToolCategory::Props,
        [
            tool_param("prop_id", "Node ID of the prop", true, ToolParamType::String),
            tool_param("visible", "Show (true) or hide (false) the prop", true, ToolParamType::Boolean),
            tool_param("isolate", "Hide all other props when showing this one (default false)", false, ToolParamType::Boolean),
        ],
        "Result confirming visibility change",
        [
            "Hide the chair from view",
            "Show only the main prop and hide everything else",
        ],
        handle_hide_show_prop
    );
    define_tool!(
        "list_props",
        "Lists all props in the current scene with their node IDs, types, and current transform values.",
        ToolCategory::Props,
        [
            tool_param("category", "Optional category filter to narrow results", false, ToolParamType::String),
        ],
        "Result with array of prop objects",
        [
            "List all props in the scene",
            "Show me all furniture props",
        ],
        handle_list_props
    );
}
fn handle_load_prop(request: ToolRequest) -> ToolResponse {
    let prop_name = request.get_str("prop_name").unwrap_or_default();
    let category = request.get_str("category");
    let position = request.get_array("position");
    if prop_name.is_empty() {
        return ToolResponse::err("load_prop", "prop_name is required");
    }
    let mut params = serde_json::json!({ "name": prop_name });
    if let Some(c) = category {
        params["category"] = serde_json::json!(c);
    }
    if !position.is_empty() {
        params["position"] = serde_json::json!(position);
    }
    let result = crate::mcp_client::send_mcp_request("load_prop", params);
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "load_prop",
            serde_json::json!({ "result": r.data }),
            format!("Loaded prop '{}'", prop_name),
        ),
        Err(e) => ToolResponse::err("load_prop", e),
    }
}
fn handle_position_prop(request: ToolRequest) -> ToolResponse {
    let prop_id = request.get_str("prop_id").unwrap_or_default();
    let position = request.get_array("position");
    let relative = request.get_bool("relative").unwrap_or(false);
    if prop_id.is_empty() || position.is_empty() {
        return ToolResponse::err("position_prop", "prop_id and position are required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "position_prop",
        serde_json::json!({ "prop_id": prop_id, "position": position, "relative": relative }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "position_prop",
            serde_json::json!({ "prop_id": prop_id }),
            format!("Positioned prop '{}'", prop_id),
        ),
        Err(e) => ToolResponse::err("position_prop", e),
    }
}
fn handle_rotate_prop(request: ToolRequest) -> ToolResponse {
    let prop_id = request.get_str("prop_id").unwrap_or_default();
    let rotation = request.get_array("rotation");
    let relative = request.get_bool("relative").unwrap_or(false);
    if prop_id.is_empty() || rotation.is_empty() {
        return ToolResponse::err("rotate_prop", "prop_id and rotation are required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "rotate_prop",
        serde_json::json!({ "prop_id": prop_id, "rotation": rotation, "relative": relative }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "rotate_prop",
            serde_json::json!({ "prop_id": prop_id }),
            format!("Rotated prop '{}'", prop_id),
        ),
        Err(e) => ToolResponse::err("rotate_prop", e),
    }
}
fn handle_scale_prop(request: ToolRequest) -> ToolResponse {
    let prop_id = request.get_str("prop_id").unwrap_or_default();
    let scale = request.get_array("scale");
    let relative = request.get_bool("relative").unwrap_or(false);
    if prop_id.is_empty() || scale.is_empty() {
        return ToolResponse::err("scale_prop", "prop_id and scale are required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "scale_prop",
        serde_json::json!({ "prop_id": prop_id, "scale": scale, "relative": relative }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "scale_prop",
            serde_json::json!({ "prop_id": prop_id }),
            format!("Scaled prop '{}'", prop_id),
        ),
        Err(e) => ToolResponse::err("scale_prop", e),
    }
}
fn handle_hide_show_prop(request: ToolRequest) -> ToolResponse {
    let prop_id = request.get_str("prop_id").unwrap_or_default();
    let visible = request.get_bool("visible").unwrap_or(true);
    let isolate = request.get_bool("isolate").unwrap_or(false);
    if prop_id.is_empty() {
        return ToolResponse::err("hide_show_prop", "prop_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "set_visibility",
        serde_json::json!({ "node_id": prop_id, "visible": visible, "isolate": isolate }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "hide_show_prop",
            serde_json::json!({ "prop_id": prop_id, "visible": visible }),
            if visible {
                format!("Showed prop '{}'", prop_id)
            } else {
                format!("Hid prop '{}'", prop_id)
            },
        ),
        Err(e) => ToolResponse::err("hide_show_prop", e),
    }
}
fn handle_list_props(request: ToolRequest) -> ToolResponse {
    let category = request.get_str("category");
    let result = crate::mcp_client::send_mcp_request(
        "list_props",
        serde_json::json!({ "category": category }),
    );
    match result {
        Ok(r) => {
            let props = r.data.unwrap_or(serde_json::json!([]));
            ToolResponse::ok_with_message(
                "list_props",
                serde_json::json!({ "props": props }),
                format!(
                    "Found {} props",
                    props.as_array().map(|a| a.len()).unwrap_or(0)
                ),
            )
        },
        Err(e) => ToolResponse::err("list_props", e),
    }
}
