use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "set_environment_hdri",
        "Sets an HDRI environment map for scene lighting and background. Supports built-in presets or custom HDRI paths.",
        ToolCategory::Environment,
        [
            tool_param("preset", "HDRI preset name: studio, outdoor, sunset, night, forest, desert, arctic, custom", true, ToolParamType::String),
            tool_param("intensity", "HDRI brightness multiplier (0.1-5.0, default 1.0)", false, ToolParamType::Number),
            tool_param("rotation", "HDRI rotation in degrees (0-360, default 0)", false, ToolParamType::Number),
            tool_param("custom_path", "File path to a custom HDRI image (only used when preset=custom)", false, ToolParamType::String),
        ],
        "Result confirming the applied HDRI environment with preset name, intensity, and rotation",
        [
            "Set the environment to a sunny outdoor HDRI",
            "Load a studio HDRI with reduced intensity",
            "Use a custom sunset HDRI from my library",
        ],
        handle_set_environment_hdri
    );
    define_tool!(
        "add_ground_plane",
        "Adds a ground plane or reflection surface to the scene. Useful for casting shadows and creating realistic reflections.",
        ToolCategory::Environment,
        [
            tool_param("type", "Ground type: shadow_catcher, reflection, solid_color, grid (default shadow_catcher)", false, ToolParamType::String),
            tool_param("color", "Base color for solid_color type (hex or named color, default #888888)", false, ToolParamType::String),
            tool_param("size", "Ground plane size in Daz units (default 500)", false, ToolParamType::Number),
        ],
        "Result with ground plane node ID and type",
        [
            "Add a shadow-catching ground plane",
            "Add a reflective surface to the scene",
            "Add a solid color ground for product renders",
        ],
        handle_add_ground_plane
    );
    define_tool!(
        "set_environment_fog",
        "Adds or removes atmospheric fog/mist effects in the scene for depth and atmosphere.",
        ToolCategory::Environment,
        [
            tool_param(
                "enabled",
                "Whether to enable fog (true) or disable it (false)",
                true,
                ToolParamType::Boolean
            ),
            tool_param(
                "density",
                "Fog density from 0.0 to 1.0 (default 0.05)",
                false,
                ToolParamType::Number
            ),
            tool_param(
                "color",
                "Fog color as hex or named color (default #CCCCCC)",
                false,
                ToolParamType::String
            ),
            tool_param(
                "distance",
                "Fog start distance in Daz units (default 100)",
                false,
                ToolParamType::Number
            ),
        ],
        "Result confirming fog settings applied",
        [
            "Add subtle atmospheric fog to my scene",
            "Remove all fog from the scene",
            "Add dense blue-tinted fog for a moody atmosphere",
        ],
        handle_set_environment_fog
    );
    define_tool!(
        "set_sun_position",
        "Controls the sun position in the scene by direction vector for shadow placement and mood.",
        ToolCategory::Environment,
        [
            tool_param(
                "direction",
                "Sun direction as [x, y, z] vector (default [0.5, 1.0, 0.3])",
                false,
                ToolParamType::FloatArray
            ),
            tool_param(
                "intensity",
                "Sun brightness multiplier (0.1-5.0, default 1.0)",
                false,
                ToolParamType::Number
            ),
            tool_param(
                "color",
                "Sun light color as hex or named (default #FFFFFF)",
                false,
                ToolParamType::String
            ),
        ],
        "Result confirming sun position applied",
        [
            "Position the sun for dramatic side lighting",
            "Set the sun overhead for midday lighting",
            "Create a warm sunset light with orange tint",
        ],
        handle_set_sun_position
    );
    define_tool!(
        "set_time_of_day",
        "Sets a scene time of day, automatically adjusting sun position, color temperature, and intensity.",
        ToolCategory::Environment,
        [
            tool_param("time", "Time of day: dawn, morning, midday, afternoon, golden_hour, sunset, dusk, night", true, ToolParamType::String),
            tool_param("adjust_env", "Also adjust the environment HDRI to match (default true)", false, ToolParamType::Boolean),
        ],
        "Result with time of day applied and lighting adjustments",
        [
            "Set the scene to golden hour lighting",
            "Switch to night time with moonlight",
            "Make it midday for bright even lighting",
        ],
        handle_set_time_of_day
    );
    define_tool!(
        "add_environment_light",
        "Adds an environment fill light. Useful for adding ambient fill or rim lighting to the scene.",
        ToolCategory::Environment,
        [
            tool_param("type", "Light type: fill, rim, bounce, ambient (default fill)", false, ToolParamType::String),
            tool_param("intensity", "Light intensity (0.0-5.0, default 0.5)", false, ToolParamType::Number),
            tool_param("color", "Light color (default white)", false, ToolParamType::String),
            tool_param("direction", "Light direction as [x, y, z]", false, ToolParamType::FloatArray),
        ],
        "Result confirming light added",
        [
            "Add a warm fill light from the left",
            "Add a blue rim light for edge definition",
            "Add ambient fill light at low intensity",
        ],
        handle_add_environment_light
    );
    define_tool!(
        "set_environment_rotation",
        "Rotates the entire environment sphere, changing reflections and background appearance.",
        ToolCategory::Environment,
        [
            tool_param(
                "rotation",
                "Environment rotation in degrees (0-360)",
                true,
                ToolParamType::Number
            ),
            tool_param(
                "hdri_only",
                "Only rotate the HDRI, not other environment elements (default false)",
                false,
                ToolParamType::Boolean
            ),
        ],
        "Result confirming rotation applied",
        [
            "Rotate the environment 90 degrees",
            "Spin the HDRI to face the subject better",
        ],
        handle_set_environment_rotation
    );
    define_tool!(
        "list_environments",
        "Lists available environment presets and the current environment settings on the scene.",
        ToolCategory::Environment,
        [],
        "Result with available presets and current env config",
        [
            "What environments are available?",
            "Show me my current environment setup",
        ],
        handle_list_environments
    );
    define_tool!(
        "remove_environment",
        "Removes all environment effects: HDRI, fog, and ground plane. Resets to default scene environment.",
        ToolCategory::Environment,
        [
            tool_param("remove_hdri", "Remove HDRI environment (default true)", false, ToolParamType::Boolean),
            tool_param("remove_ground", "Remove ground plane (default true)", false, ToolParamType::Boolean),
            tool_param("remove_fog", "Remove fog effects (default true)", false, ToolParamType::Boolean),
        ],
        "Result confirming environment cleared",
        [
            "Clear all environment effects",
            "Remove the HDRI but keep the ground plane",
        ],
        handle_remove_environment
    );
}
fn handle_set_environment_hdri(request: ToolRequest) -> ToolResponse {
    let preset = request.get_str("preset").unwrap_or_default();
    let intensity = request.get_f64("intensity").unwrap_or(1.0);
    let rotation = request.get_f64("rotation").unwrap_or(0.0);
    let _custom_path = request.get_str("custom_path");
    if preset.is_empty() {
        return ToolResponse::err("set_environment_hdri", "preset is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "set_environment",
        serde_json::json!({ "type": "hdri", "preset": preset, "intensity": intensity, "rotation": rotation }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_environment_hdri",
            serde_json::json!({ "preset": preset, "intensity": intensity, "rotation": rotation }),
            format!(
                "Applied '{}' HDRI environment at {:.1}x intensity",
                preset, intensity
            ),
        ),
        Err(e) => ToolResponse::err("set_environment_hdri", e),
    }
}
fn handle_add_ground_plane(request: ToolRequest) -> ToolResponse {
    let ground_type = request
        .get_str("type")
        .unwrap_or_else(|| "shadow_catcher".to_string());
    let color = request
        .get_str("color")
        .unwrap_or_else(|| "#888888".to_string());
    let size = request.get_f64("size").unwrap_or(500.0);
    let result = crate::mcp_client::send_mcp_request(
        "add_ground",
        serde_json::json!({ "type": ground_type, "color": color, "size": size }),
    );
    match result {
        Ok(r) => {
            let node_id = r
                .data
                .and_then(|d| d.get("node_id").and_then(|v| v.as_str().map(String::from)));
            ToolResponse::ok_with_message(
                "add_ground_plane",
                serde_json::json!({ "type": ground_type, "node_id": node_id, "size": size }),
                format!("Added {} ground plane (size: {:.0})", ground_type, size),
            )
        },
        Err(e) => ToolResponse::err("add_ground_plane", e),
    }
}
fn handle_set_environment_fog(request: ToolRequest) -> ToolResponse {
    let enabled = request.get_bool("enabled").unwrap_or(false);
    let density = request.get_f64("density").unwrap_or(0.05);
    let color = request
        .get_str("color")
        .unwrap_or_else(|| "#CCCCCC".to_string());
    let distance = request.get_f64("distance").unwrap_or(100.0);
    let result = crate::mcp_client::send_mcp_request(
        "set_fog",
        serde_json::json!({ "enabled": enabled, "density": density, "color": color, "distance": distance }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_environment_fog",
            serde_json::json!({ "enabled": enabled, "density": density, "color": color, "distance": distance }),
            if enabled {
                format!(
                    "Fog enabled: density {:.3}, color {}, start distance {:.0}",
                    density, color, distance
                )
            } else {
                "Fog disabled".to_string()
            },
        ),
        Err(e) => ToolResponse::err("set_environment_fog", e),
    }
}
fn handle_set_sun_position(request: ToolRequest) -> ToolResponse {
    let direction = request.get_array("direction");
    let intensity = request.get_f64("intensity").unwrap_or(1.0);
    let color = request
        .get_str("color")
        .unwrap_or_else(|| "#FFFFFF".to_string());
    if direction.len() < 3 {
        return ToolResponse::err("set_sun_position", "direction must be [x, y, z] array");
    }
    let result = crate::mcp_client::send_mcp_request(
        "set_sun",
        serde_json::json!({ "direction": direction, "intensity": intensity, "color": color }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_sun_position",
            serde_json::json!({ "direction": direction }),
            format!(
                "Sun positioned at [{}, {}, {}]",
                direction[0], direction[1], direction[2]
            ),
        ),
        Err(e) => ToolResponse::err("set_sun_position", e),
    }
}
fn handle_set_time_of_day(request: ToolRequest) -> ToolResponse {
    let time = request.get_str("time").unwrap_or_default();
    let adjust_env = request.get_bool("adjust_env").unwrap_or(true);
    if time.is_empty() {
        return ToolResponse::err("set_time_of_day", "time is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "set_time_of_day",
        serde_json::json!({ "time": time, "adjust_env": adjust_env }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_time_of_day",
            serde_json::json!({ "time": time }),
            format!("Scene set to {}", time),
        ),
        Err(e) => ToolResponse::err("set_time_of_day", e),
    }
}
fn handle_add_environment_light(request: ToolRequest) -> ToolResponse {
    let light_type = request
        .get_str("type")
        .unwrap_or_else(|| "fill".to_string());
    let intensity = request.get_f64("intensity").unwrap_or(0.5);
    let color = request
        .get_str("color")
        .unwrap_or_else(|| "#FFFFFF".to_string());
    let direction = request.get_array("direction");
    let result = crate::mcp_client::send_mcp_request(
        "add_env_light",
        serde_json::json!({ "type": light_type, "intensity": intensity, "color": color, "direction": direction }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "add_environment_light",
            serde_json::json!({ "type": light_type, "intensity": intensity }),
            format!("Added {} light at {:.1}x intensity", light_type, intensity),
        ),
        Err(e) => ToolResponse::err("add_environment_light", e),
    }
}
fn handle_set_environment_rotation(request: ToolRequest) -> ToolResponse {
    let rotation = request.get_f64("rotation").unwrap_or(0.0);
    let hdri_only = request.get_bool("hdri_only").unwrap_or(false);
    let result = crate::mcp_client::send_mcp_request(
        "rotate_environment",
        serde_json::json!({ "rotation": rotation, "hdri_only": hdri_only }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_environment_rotation",
            serde_json::json!({ "rotation": rotation }),
            format!("Environment rotated {:.0} degrees", rotation),
        ),
        Err(e) => ToolResponse::err("set_environment_rotation", e),
    }
}
fn handle_list_environments(_request: ToolRequest) -> ToolResponse {
    let result = crate::mcp_client::send_mcp_request("get_environment_info", serde_json::json!({}));
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "list_environments",
            serde_json::json!({
                "available_presets": ["studio", "outdoor", "sunset", "night", "forest", "desert", "arctic"],
                "current": r.data,
            }),
            "Environment presets listed",
        ),
        Err(e) => ToolResponse::err("list_environments", e),
    }
}
fn handle_remove_environment(request: ToolRequest) -> ToolResponse {
    let remove_hdri = request.get_bool("remove_hdri").unwrap_or(true);
    let remove_ground = request.get_bool("remove_ground").unwrap_or(true);
    let remove_fog = request.get_bool("remove_fog").unwrap_or(true);
    let result = crate::mcp_client::send_mcp_request(
        "clear_environment",
        serde_json::json!({ "hdri": remove_hdri, "ground": remove_ground, "fog": remove_fog }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "remove_environment",
            serde_json::json!({ "removed": { "hdri": remove_hdri, "ground": remove_ground, "fog": remove_fog } }),
            "Environment cleared",
        ),
        Err(e) => ToolResponse::err("remove_environment", e),
    }
}
