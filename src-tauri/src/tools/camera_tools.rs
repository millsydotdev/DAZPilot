use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "analyze_camera_setup",
        "Describes the current active camera: position, focal length, field of view, and composition characteristics.",
        ToolCategory::Camera,
        [],
        "Camera analysis with position, focal length, FOV, and composition notes",
        [
            "What camera settings am I using?",
            "Analyze the current camera setup",
        ],
        handle_analyze_camera_setup
    );
    define_tool!(
        "suggest_camera_angle",
        "Suggests optimal camera angles, focal lengths, and positions for the best composition given scene type.",
        ToolCategory::Camera,
        [
            tool_param("scene_type", "Shot type: portrait, full_body, wide, action, close_up, product, dutch, overhead", true, ToolParamType::String),
            tool_param("mood", "Desired mood for composition guidance", false, ToolParamType::String),
        ],
        "Camera angle suggestions with positioning and lens recommendations",
        [
            "What's the best camera angle for a portrait?",
            "Suggest a dramatic camera angle for action scene",
        ],
        handle_suggest_camera_angle
    );
    define_tool!(
        "apply_camera_preset",
        "Applies a camera preset: portrait, full_body, close_up, wide_angle, overhead, dutch_angle, tracking, low_angle.",
        ToolCategory::Camera,
        [
            tool_param("preset_name", "Camera preset name", true, ToolParamType::String),
        ],
        "Confirms preset applied",
        [
            "Apply a portrait camera preset",
            "Set up a Dutch angle camera shot",
        ],
        handle_apply_camera_preset
    );
    define_tool!(
        "list_camera_presets",
        "Lists available camera presets with descriptions and typical usage scenarios.",
        ToolCategory::Camera,
        [],
        "List of camera presets with descriptions",
        [
            "What camera presets are available?",
            "Show me all camera angle presets",
        ],
        handle_list_camera_presets
    );
    define_tool!(
        "create_camera",
        "Creates a new camera node in the scene at a specified position and focal length.",
        ToolCategory::Camera,
        [
            tool_param(
                "camera_name",
                "Name for the new camera",
                false,
                ToolParamType::String
            ),
            tool_param(
                "position",
                "Position as [x, y, z]",
                false,
                ToolParamType::FloatArray
            ),
            tool_param(
                "focal_length",
                "Focal length in mm (35-200, default 50)",
                false,
                ToolParamType::Number
            ),
        ],
        "Result with new camera node ID",
        [
            "Create a new camera at eye level",
            "Add a close-up camera with 85mm lens",
        ],
        handle_create_camera
    );
    define_tool!(
        "set_camera_transform",
        "Positions a camera to specific world coordinates and optionally sets a look-at target.",
        ToolCategory::Camera,
        [
            tool_param(
                "camera_id",
                "Camera node ID (defaults to active camera)",
                false,
                ToolParamType::String
            ),
            tool_param(
                "position",
                "Target position as [x, y, z]",
                true,
                ToolParamType::FloatArray
            ),
            tool_param(
                "target",
                "Look-at point [x, y, z]",
                false,
                ToolParamType::FloatArray
            ),
        ],
        "Result confirming camera transform",
        [
            "Move the camera to [50, 30, -200]",
            "Position camera to look at [0, 100, 0]",
        ],
        handle_set_camera_transform
    );
    define_tool!(
        "set_camera_focal_length",
        "Sets the focal length of a camera. Lower = wider, higher = more zoomed.",
        ToolCategory::Camera,
        [
            tool_param(
                "camera_id",
                "Camera node ID (defaults to active camera)",
                false,
                ToolParamType::String
            ),
            tool_param(
                "focal_length",
                "Focal length in mm (15-300)",
                true,
                ToolParamType::Number
            ),
        ],
        "Result confirming focal length",
        ["Zoom in with 135mm focal length", "Go wide with 24mm lens",],
        handle_set_camera_focal_length
    );
    define_tool!(
        "set_camera_aperture",
        "Sets camera aperture and depth of field settings for cinematic blur.",
        ToolCategory::Camera,
        [
            tool_param(
                "camera_id",
                "Camera node ID (defaults to active)",
                false,
                ToolParamType::String
            ),
            tool_param(
                "f_stop",
                "f-stop value (1.0-22.0, default 8.0)",
                false,
                ToolParamType::Number
            ),
            tool_param(
                "enable_dof",
                "Enable depth of field (default false)",
                false,
                ToolParamType::Boolean
            ),
            tool_param(
                "focus_distance",
                "Focus distance in Daz units",
                false,
                ToolParamType::Number
            ),
        ],
        "Result confirming aperture settings",
        [
            "Set shallow depth of field at f/1.4",
            "Enable deep focus at f/16",
        ],
        handle_set_camera_aperture
    );
    define_tool!(
        "focus_on_object",
        "Aims a camera to look directly at a specific scene object.",
        ToolCategory::Camera,
        [
            tool_param(
                "camera_id",
                "Camera node ID (defaults to active)",
                false,
                ToolParamType::String
            ),
            tool_param(
                "target_node",
                "Node ID or name of the object to focus on",
                true,
                ToolParamType::String
            ),
            tool_param(
                "offset",
                "Optional framing offset [x, y, z]",
                false,
                ToolParamType::FloatArray
            ),
        ],
        "Result confirming focus target",
        [
            "Point the camera at the main character",
            "Focus on the prop and frame it",
        ],
        handle_focus_on_object
    );
}
fn handle_analyze_camera_setup(_request: ToolRequest) -> ToolResponse {
    let result = crate::mcp_client::send_mcp_request("get_cameras", serde_json::json!({}));
    match result {
        Ok(r) => {
            let data = r.data.unwrap_or(serde_json::json!([]));
            let cam_array = data.as_array().cloned().unwrap_or_default();
            ToolResponse::ok_with_message(
                "analyze_camera_setup",
                serde_json::json!({ "cameras": cam_array, "total_cameras": cam_array.len() }),
                format!("Found {} camera(s) in scene", cam_array.len()),
            )
        },
        Err(e) => ToolResponse::err("analyze_camera_setup", e),
    }
}
fn handle_suggest_camera_angle(request: ToolRequest) -> ToolResponse {
    let scene_type = request.get_str("scene_type").unwrap_or_default();
    if scene_type.is_empty() {
        return ToolResponse::err("suggest_camera_angle", "scene_type is required");
    }
    let suggestions: Vec<serde_json::Value> = match scene_type.to_lowercase().as_str() {
        "portrait" | "headshot" => vec![
            serde_json::json!({"name": "Classic Portrait", "focal_length": 85, "position": [0, 10, -100], "description": "85mm for flattering compression"}),
            serde_json::json!({"name": "Three-quarter Portrait", "focal_length": 70, "position": [30, 5, -90], "description": "Subject angled 45°, natural and engaging"}),
            serde_json::json!({"name": "Close Portrait", "focal_length": 105, "position": [0, 8, -70], "description": "Tighter framing for detail"}),
        ],
        "full_body" | "fullbody" => vec![
            serde_json::json!({"name": "Full Body Straight", "focal_length": 50, "position": [0, 30, -200], "description": "50mm for natural proportions"}),
            serde_json::json!({"name": "Full Body Low Angle", "focal_length": 35, "position": [0, -10, -180], "description": "Low angle for powerful appearance"}),
        ],
        "close_up" | "closeup" | "detail" => vec![
            serde_json::json!({"name": "Extreme Close-up", "focal_length": 135, "position": [0, 8, -40], "description": "Emphasizing expression or detail"}),
            serde_json::json!({"name": "Detail Shot", "focal_length": 100, "position": [15, 5, -50], "description": "Showing texture and detail"}),
        ],
        "action" | "dynamic" => vec![
            serde_json::json!({"name": "Action Low Angle", "focal_length": 24, "position": [0, -20, -150], "description": "Dramatic low angle with wide lens"}),
            serde_json::json!({"name": "Dutch Angle", "focal_length": 35, "position": [25, 15, -160], "description": "Tilted camera for tension"}),
        ],
        "wide" | "wideangle" => vec![
            serde_json::json!({"name": "Establishing Wide", "focal_length": 24, "position": [0, 40, -400], "description": "Full scene and environment"}),
            serde_json::json!({"name": "Bird's Eye", "focal_length": 35, "position": [0, 200, -10], "description": "Overhead view"}),
        ],
        "overhead" | "top" | "birdseye" => vec![
            serde_json::json!({"name": "Top Down", "focal_length": 50, "position": [0, 200, 0], "description": "Directly overhead"}),
        ],
        "product" | "object" => vec![
            serde_json::json!({"name": "Product Front", "focal_length": 85, "position": [0, 5, -60], "description": "Clean front-facing product shot"}),
        ],
        _ => vec![
            serde_json::json!({"name": "Standard View", "focal_length": 50, "position": [0, 20, -150], "description": "Balanced eye-level view"}),
        ],
    };
    ToolResponse::ok_with_message(
        "suggest_camera_angle",
        serde_json::json!({ "scene_type": scene_type, "suggestions": suggestions }),
        format!(
            "Found {} suggestions for '{}'",
            suggestions.len(),
            scene_type
        ),
    )
}
fn handle_apply_camera_preset(_request: ToolRequest) -> ToolResponse {
    ToolResponse::ok_with_message(
        "apply_camera_preset",
        serde_json::json!({ "available_presets": ["portrait", "full_body", "close_up", "wide_angle", "dutch_angle", "overhead"] }),
        "Use suggest_camera_angle for position recommendations, then set_camera_transform to apply.",
    )
}
fn handle_list_camera_presets(_request: ToolRequest) -> ToolResponse {
    ToolResponse::ok_with_message(
        "list_camera_presets",
        serde_json::json!({
            "presets": [
                {"name": "portrait", "description": "Eye-level portrait at 85mm", "best_for": "Headshots"},
                {"name": "full_body", "description": "Full body at 50mm", "best_for": "Showing complete outfit"},
                {"name": "close_up", "description": "Tight framing at 105mm", "best_for": "Facial detail"},
                {"name": "wide_angle", "description": "24mm wide shot", "best_for": "Establishing shots"},
                {"name": "dutch_angle", "description": "Tilted camera", "best_for": "Action and thriller"},
                {"name": "overhead", "description": "Top-down view", "best_for": "Scene layouts"},
                {"name": "low_angle", "description": "Looking up", "best_for": "Heroic subjects"},
                {"name": "tracking", "description": "Side view", "best_for": "Walk cycles and motion"},
            ]
        }),
        "Found 8 camera presets",
    )
}
fn handle_create_camera(request: ToolRequest) -> ToolResponse {
    let camera_name = request.get_str("camera_name").unwrap_or_default();
    let position = request.get_array("position");
    let focal_length = request.get_f64("focal_length").unwrap_or(50.0);
    let result = crate::mcp_client::send_mcp_request(
        "create_camera",
        serde_json::json!({ "name": camera_name, "position": position, "focal_length": focal_length }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "create_camera",
            serde_json::json!({ "result": r.data }),
            if camera_name.is_empty() {
                "Created new camera".to_string()
            } else {
                format!("Created camera '{}'", camera_name)
            },
        ),
        Err(e) => ToolResponse::err("create_camera", e),
    }
}
fn handle_set_camera_transform(request: ToolRequest) -> ToolResponse {
    let camera_id = request.get_str("camera_id");
    let position = request.get_array("position");
    let target = request.get_array("target");
    if position.is_empty() {
        return ToolResponse::err("set_camera_transform", "position is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "set_camera_transform",
        serde_json::json!({ "camera_id": camera_id, "position": position, "target": target }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_camera_transform",
            serde_json::json!({ "position": position }),
            "Camera repositioned",
        ),
        Err(e) => ToolResponse::err("set_camera_transform", e),
    }
}
fn handle_set_camera_focal_length(request: ToolRequest) -> ToolResponse {
    let camera_id = request.get_str("camera_id");
    let focal_length = request.get_f64("focal_length").unwrap_or(50.0);
    let result = crate::mcp_client::send_mcp_request(
        "set_focal_length",
        serde_json::json!({ "camera_id": camera_id, "focal_length": focal_length }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_camera_focal_length",
            serde_json::json!({ "focal_length": focal_length }),
            format!("Focal length set to {:.0}mm", focal_length),
        ),
        Err(e) => ToolResponse::err("set_camera_focal_length", e),
    }
}
fn handle_set_camera_aperture(request: ToolRequest) -> ToolResponse {
    let camera_id = request.get_str("camera_id");
    let f_stop = request.get_f64("f_stop").unwrap_or(8.0);
    let enable_dof = request.get_bool("enable_dof").unwrap_or(false);
    let focus_distance = request.get_f64("focus_distance");
    let result = crate::mcp_client::send_mcp_request(
        "set_aperture",
        serde_json::json!({ "camera_id": camera_id, "f_stop": f_stop, "enable_dof": enable_dof, "focus_distance": focus_distance }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_camera_aperture",
            serde_json::json!({ "f_stop": f_stop, "depth_of_field": enable_dof }),
            format!(
                "Aperture f/{}, DOF {}",
                f_stop,
                if enable_dof { "enabled" } else { "disabled" }
            ),
        ),
        Err(e) => ToolResponse::err("set_camera_aperture", e),
    }
}
fn handle_focus_on_object(request: ToolRequest) -> ToolResponse {
    let camera_id = request.get_str("camera_id");
    let target_node = request.get_str("target_node").unwrap_or_default();
    let offset = request.get_array("offset");
    if target_node.is_empty() {
        return ToolResponse::err("focus_on_object", "target_node is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "focus_camera",
        serde_json::json!({ "camera_id": camera_id, "target": target_node, "offset": offset }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "focus_on_object",
            serde_json::json!({ "target": target_node }),
            format!("Camera focused on '{}'", target_node),
        ),
        Err(e) => ToolResponse::err("focus_on_object", e),
    }
}
