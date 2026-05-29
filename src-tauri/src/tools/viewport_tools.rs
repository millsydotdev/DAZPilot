use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "set_viewport_display",
        "Changes the viewport display mode. Controls how the scene is rendered in the interactive viewport.",
        ToolCategory::Viewport,
        [
            tool_param("mode", "Display mode: textured, solid, wireframe, hidden_line, shaded, smooth_shaded (default textured)", false, ToolParamType::String),
            tool_param("viewport_index", "Viewport index to change (0 = main viewport, default 0)", false, ToolParamType::Integer),
        ],
        "Result confirming the display mode change",
        [
            "Switch to wireframe mode",
            "Set viewport to textured display",
            "Show the scene in smooth shaded mode",
        ],
        handle_set_viewport_display
    );
    define_tool!(
        "set_viewport_quality",
        "Sets the viewport rendering quality. Controls texture resolution, anti-aliasing, and shadow quality.",
        ToolCategory::Viewport,
        [
            tool_param("quality", "Quality level: full, half, quarter, eighth, user (default full)", false, ToolParamType::String),
            tool_param("texture_resolution", "Max texture resolution: full, half, quarter (default full)", false, ToolParamType::String),
            tool_param("anti_aliasing", "Anti-aliasing: none, 2x, 4x, 8x (default 4x)", false, ToolParamType::String),
        ],
        "Result confirming quality settings",
        [
            "Set viewport to half quality for better performance",
            "Maximize viewport quality with 8x anti-aliasing",
        ],
        handle_set_viewport_quality
    );
    define_tool!(
        "toggle_viewport_guides",
        "Shows or hides viewport guides: grid, ground plane, axes, safe frames, and center marker.",
        ToolCategory::Viewport,
        [
            tool_param(
                "guide",
                "Guide to toggle: grid, ground, axes, safe_frame, center, all",
                true,
                ToolParamType::String
            ),
            tool_param(
                "show",
                "Show (true) or hide (false) the guide",
                true,
                ToolParamType::Boolean
            ),
        ],
        "Result confirming guide visibility change",
        [
            "Show the grid in the viewport",
            "Hide the ground plane",
            "Toggle axes display on",
        ],
        handle_toggle_viewport_guides
    );
    define_tool!(
        "set_viewport_camera",
        "Sets the active viewport camera. Can switch between perspective, orthographic, front, side, top, or a specific camera node.",
        ToolCategory::Viewport,
        [
            tool_param("camera", "Camera view: perspective, front, back, left, right, top, bottom, or a camera node name", true, ToolParamType::String),
            tool_param("viewport_index", "Viewport index (0 = main, default 0)", false, ToolParamType::Integer),
        ],
        "Result confirming camera change",
        [
            "Switch to top view",
            "Change viewport to front orthographic",
            "View through the main camera",
        ],
        handle_set_viewport_camera
    );
    define_tool!(
        "set_viewport_lighting",
        "Controls the viewport lighting mode: scene lights, default lights, directional, or ambient only.",
        ToolCategory::Viewport,
        [
            tool_param("lighting", "Lighting mode: scene_lights, default_lights, directional, ambient, none (default scene_lights)", false, ToolParamType::String),
            tool_param("ambient_intensity", "Ambient light brightness 0.0-1.0 (default 0.5)", false, ToolParamType::Number),
        ],
        "Result confirming lighting mode change",
        [
            "Use scene lights in viewport",
            "Switch to default viewport lighting",
            "Set ambient light to 30%",
        ],
        handle_set_viewport_lighting
    );
}
fn handle_set_viewport_display(request: ToolRequest) -> ToolResponse {
    let mode = request
        .get_str("mode")
        .unwrap_or_else(|| "textured".to_string());
    let viewport_index = request.get_i64("viewport_index").unwrap_or(0);
    let result = crate::mcp_client::send_mcp_request(
        "set_display_mode",
        serde_json::json!({ "mode": mode, "viewport": viewport_index }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_viewport_display",
            serde_json::json!({ "mode": mode }),
            format!("Viewport display set to '{}'", mode),
        ),
        Err(e) => ToolResponse::err("set_viewport_display", e),
    }
}
fn handle_set_viewport_quality(request: ToolRequest) -> ToolResponse {
    let quality = request
        .get_str("quality")
        .unwrap_or_else(|| "full".to_string());
    let texture_resolution = request
        .get_str("texture_resolution")
        .unwrap_or_else(|| "full".to_string());
    let anti_aliasing = request
        .get_str("anti_aliasing")
        .unwrap_or_else(|| "4x".to_string());
    let result = crate::mcp_client::send_mcp_request(
        "set_viewport_quality",
        serde_json::json!({
            "quality": quality,
            "texture_resolution": texture_resolution,
            "anti_aliasing": anti_aliasing,
        }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_viewport_quality",
            serde_json::json!({ "quality": quality }),
            format!("Viewport quality set to '{}'", quality),
        ),
        Err(e) => ToolResponse::err("set_viewport_quality", e),
    }
}
fn handle_toggle_viewport_guides(request: ToolRequest) -> ToolResponse {
    let guide = request.get_str("guide").unwrap_or_default();
    let show = request.get_bool("show").unwrap_or(true);
    if guide.is_empty() {
        return ToolResponse::err("toggle_viewport_guides", "guide is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "toggle_guide",
        serde_json::json!({ "guide": guide, "show": show }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "toggle_viewport_guides",
            serde_json::json!({ "guide": guide, "visible": show }),
            if show {
                format!("Showing '{}' guide", guide)
            } else {
                format!("Hiding '{}' guide", guide)
            },
        ),
        Err(e) => ToolResponse::err("toggle_viewport_guides", e),
    }
}
fn handle_set_viewport_camera(request: ToolRequest) -> ToolResponse {
    let camera = request.get_str("camera").unwrap_or_default();
    let viewport_index = request.get_i64("viewport_index").unwrap_or(0);
    if camera.is_empty() {
        return ToolResponse::err("set_viewport_camera", "camera is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "set_viewport_camera",
        serde_json::json!({ "camera": camera, "viewport": viewport_index }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_viewport_camera",
            serde_json::json!({ "camera": camera }),
            format!("Viewport camera set to '{}'", camera),
        ),
        Err(e) => ToolResponse::err("set_viewport_camera", e),
    }
}
fn handle_set_viewport_lighting(request: ToolRequest) -> ToolResponse {
    let lighting = request
        .get_str("lighting")
        .unwrap_or_else(|| "scene_lights".to_string());
    let ambient_intensity = request.get_f64("ambient_intensity").unwrap_or(0.5);
    let result = crate::mcp_client::send_mcp_request(
        "set_viewport_lighting",
        serde_json::json!({ "lighting": lighting, "ambient_intensity": ambient_intensity }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_viewport_lighting",
            serde_json::json!({ "lighting": lighting }),
            format!("Viewport lighting set to '{}'", lighting),
        ),
        Err(e) => ToolResponse::err("set_viewport_lighting", e),
    }
}
