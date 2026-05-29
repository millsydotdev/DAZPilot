use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "load_hair",
        "Loads a hair asset from the content library and optionally applies it to a figure.",
        ToolCategory::Hair,
        [
            tool_param(
                "hair_name",
                "Name or search term for the hair asset",
                true,
                ToolParamType::String
            ),
            tool_param(
                "figure_id",
                "Figure node ID to apply hair to",
                false,
                ToolParamType::String
            ),
            tool_param(
                "color",
                "Hair color: blonde, brunette, red, black, white, custom",
                false,
                ToolParamType::String
            ),
        ],
        "Result with loaded hair node ID and style name",
        [
            "Load long flowing hair for Genesis 9",
            "Find and load a short ponytail hairstyle",
            "Load red curly hair for my character",
        ],
        handle_load_hair
    );
    define_tool!(
        "style_hair",
        "Applies a styling preset: straight, wavy, curly, voluminous, sleek, messy, updo, ponytail.",
        ToolCategory::Hair,
        [
            tool_param("hair_id", "Node ID of the hair asset", true, ToolParamType::String),
            tool_param("preset", "Style preset: straight, wavy, curly, voluminous, sleek, messy, updo, ponytail", true, ToolParamType::String),
        ],
        "Result confirming style applied",
        [
            "Style the hair as wavy",
            "Make the hair curly and voluminous",
            "Style into a ponytail",
        ],
        handle_style_hair
    );
    define_tool!(
        "set_hair_color",
        "Changes the hair color with optional highlights and root color for multi-tonal effects.",
        ToolCategory::Hair,
        [
            tool_param(
                "hair_id",
                "Node ID of the hair asset",
                true,
                ToolParamType::String
            ),
            tool_param(
                "color",
                "Color: blonde, brunette, red, black, white, or hex #RRGGBB",
                true,
                ToolParamType::String
            ),
            tool_param(
                "highlights",
                "Optional highlight color",
                false,
                ToolParamType::String
            ),
            tool_param(
                "root_color",
                "Root color for ombre/grown-out effects",
                false,
                ToolParamType::String
            ),
        ],
        "Result confirming color change",
        [
            "Change hair color to blonde with highlights",
            "Set hair to deep brunette",
            "Make hair bright red with dark roots",
        ],
        handle_set_hair_color
    );
    define_tool!(
        "apply_hair_physics",
        "Configures dForce physics on a hair asset for natural movement.",
        ToolCategory::Hair,
        [
            tool_param(
                "hair_id",
                "Node ID of the hair asset",
                true,
                ToolParamType::String
            ),
            tool_param(
                "enable",
                "Enable hair physics (default true)",
                false,
                ToolParamType::Boolean
            ),
            tool_param(
                "stiffness",
                "Stiffness 0.0-1.0 (default 0.3)",
                false,
                ToolParamType::Number
            ),
            tool_param(
                "gravity_scale",
                "Gravity 0.0-2.0 (default 1.0)",
                false,
                ToolParamType::Number
            ),
            tool_param(
                "wind",
                "Wind strength 0.0-1.0 (default 0.0)",
                false,
                ToolParamType::Number
            ),
        ],
        "Result confirming physics settings",
        [
            "Add physics to hair for natural movement",
            "Make the hair stiffer to hold shape",
            "Add wind to the hair simulation",
        ],
        handle_apply_hair_physics
    );
    define_tool!(
        "set_hair_length",
        "Adjusts the length of a hair asset. Supports presets for common lengths and custom adjustments.",
        ToolCategory::Hair,
        [
            tool_param("hair_id", "Node ID of the hair asset", true, ToolParamType::String),
            tool_param("length", "Length preset: very_short, short, medium, long, very_long, floor_length", true, ToolParamType::String),
            tool_param("scale_factor", "Custom scale multiplier 0.5-2.0 (overrides preset if set)", false, ToolParamType::Number),
        ],
        "Result confirming hair length change",
        [
            "Make the hair longer",
            "Cut the hair to short length",
            "Set hair to medium length",
        ],
        handle_set_hair_length
    );
    define_tool!(
        "set_hair_volume",
        "Controls the volume/thickness of a hair asset. Adjusts bulk, density, and spread.",
        ToolCategory::Hair,
        [
            tool_param(
                "hair_id",
                "Node ID of the hair asset",
                true,
                ToolParamType::String
            ),
            tool_param(
                "volume",
                "Volume level: flat, thin, normal, full, very_full (default normal)",
                true,
                ToolParamType::String
            ),
        ],
        "Result confirming volume change",
        [
            "Add more volume to the hair",
            "Make the hair thinner and flatter",
            "Give the hair maximum fullness",
        ],
        handle_set_hair_volume
    );
    define_tool!(
        "list_hair_presets",
        "Lists available hair style presets, colors, and length options that can be applied.",
        ToolCategory::Hair,
        [tool_param(
            "figure_type",
            "Optional figure type filter: genesis_8, genesis_9, etc.",
            false,
            ToolParamType::String
        ),],
        "Result with available hair presets and options",
        [
            "What hair presets are available?",
            "Show me Genesis 9 compatible hair",
        ],
        handle_list_hair_presets
    );
    define_tool!(
        "remove_hair",
        "Removes a hair asset from the scene or from a specific figure.",
        ToolCategory::Hair,
        [
            tool_param(
                "hair_id",
                "Node ID of the hair to remove",
                true,
                ToolParamType::String
            ),
            tool_param(
                "keep_textures",
                "Keep texture files (default false)",
                false,
                ToolParamType::Boolean
            ),
        ],
        "Result confirming hair removal",
        ["Remove the hair from the scene", "Delete the hair asset",],
        handle_remove_hair
    );
    define_tool!(
        "set_hair_shader",
        "Changes the hair shader settings including gloss, specular, and translucency.",
        ToolCategory::Hair,
        [
            tool_param(
                "hair_id",
                "Node ID of the hair asset",
                true,
                ToolParamType::String
            ),
            tool_param(
                "shader_type",
                "Shader type: glossy, matte, wet, natural, fantasy (default natural)",
                false,
                ToolParamType::String
            ),
            tool_param(
                "gloss",
                "Glossiness 0.0-1.0 (default 0.5)",
                false,
                ToolParamType::Number
            ),
            tool_param(
                "specular",
                "Specular strength 0.0-1.0 (default 0.3)",
                false,
                ToolParamType::Number
            ),
        ],
        "Result confirming shader change",
        [
            "Make the hair glossy and shiny",
            "Give the hair a matte finish",
            "Make hair look wet",
        ],
        handle_set_hair_shader
    );
    define_tool!(
        "apply_hair_preset",
        "Applies a complete hair style preset including length, volume, color, and shader settings.",
        ToolCategory::Hair,
        [
            tool_param("hair_id", "Node ID of the hair asset", true, ToolParamType::String),
            tool_param("preset_name", "Name of the full style preset to apply", true, ToolParamType::String),
        ],
        "Result confirming preset applied",
        [
            "Apply the 'beachy waves' hair preset",
            "Apply a formal updo preset to the hair",
        ],
        handle_apply_hair_preset
    );
}
fn handle_load_hair(request: ToolRequest) -> ToolResponse {
    let hair_name = request.get_str("hair_name").unwrap_or_default();
    let figure_id = request.get_str("figure_id");
    let color = request.get_str("color");
    if hair_name.is_empty() {
        return ToolResponse::err("load_hair", "hair_name is required");
    }
    let mut params = serde_json::json!({ "name": hair_name });
    if let Some(fid) = figure_id {
        params["figure_id"] = serde_json::json!(fid);
    }
    if let Some(c) = color {
        params["color"] = serde_json::json!(c);
    }
    let result = crate::mcp_client::send_mcp_request("load_hair", params);
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "load_hair",
            serde_json::json!({ "result": r.data }),
            format!("Loaded hair '{}'", hair_name),
        ),
        Err(e) => ToolResponse::err("load_hair", e),
    }
}
fn handle_style_hair(request: ToolRequest) -> ToolResponse {
    let hair_id = request.get_str("hair_id").unwrap_or_default();
    let preset = request.get_str("preset").unwrap_or_default();
    if hair_id.is_empty() || preset.is_empty() {
        return ToolResponse::err("style_hair", "hair_id and preset are required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "style_hair",
        serde_json::json!({ "hair_id": hair_id, "preset": preset }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "style_hair",
            serde_json::json!({ "hair_id": hair_id, "style": preset }),
            format!("Styled hair as {}", preset),
        ),
        Err(e) => ToolResponse::err("style_hair", e),
    }
}
fn handle_set_hair_color(request: ToolRequest) -> ToolResponse {
    let hair_id = request.get_str("hair_id").unwrap_or_default();
    let color = request.get_str("color").unwrap_or_default();
    let highlights = request.get_str("highlights");
    let root_color = request.get_str("root_color");
    if hair_id.is_empty() || color.is_empty() {
        return ToolResponse::err("set_hair_color", "hair_id and color are required");
    }
    let mut params = serde_json::json!({ "hair_id": hair_id, "color": color });
    if let Some(h) = highlights {
        params["highlights"] = serde_json::json!(h);
    }
    if let Some(r) = root_color {
        params["root_color"] = serde_json::json!(r);
    }
    let result = crate::mcp_client::send_mcp_request("set_hair_color", params);
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_hair_color",
            serde_json::json!({ "hair_id": hair_id, "color": color }),
            format!("Hair color changed to {}", color),
        ),
        Err(e) => ToolResponse::err("set_hair_color", e),
    }
}
fn handle_apply_hair_physics(request: ToolRequest) -> ToolResponse {
    let hair_id = request.get_str("hair_id").unwrap_or_default();
    let enable = request.get_bool("enable").unwrap_or(true);
    let stiffness = request.get_f64("stiffness").unwrap_or(0.3);
    let gravity = request.get_f64("gravity_scale").unwrap_or(1.0);
    let wind = request.get_f64("wind").unwrap_or(0.0);
    if hair_id.is_empty() {
        return ToolResponse::err("apply_hair_physics", "hair_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "apply_hair_physics",
        serde_json::json!({ "hair_id": hair_id, "enable": enable, "stiffness": stiffness, "gravity_scale": gravity, "wind": wind }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "apply_hair_physics",
            serde_json::json!({ "hair_id": hair_id, "physics_enabled": enable }),
            if enable {
                format!("Hair physics on (stiffness: {:.1})", stiffness)
            } else {
                "Hair physics disabled".into()
            },
        ),
        Err(e) => ToolResponse::err("apply_hair_physics", e),
    }
}
fn handle_set_hair_length(request: ToolRequest) -> ToolResponse {
    let hair_id = request.get_str("hair_id").unwrap_or_default();
    let length = request.get_str("length").unwrap_or_default();
    let scale = request.get_f64("scale_factor");
    if hair_id.is_empty() || length.is_empty() {
        return ToolResponse::err("set_hair_length", "hair_id and length are required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "set_hair_length",
        serde_json::json!({ "hair_id": hair_id, "length": length, "scale_factor": scale }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_hair_length",
            serde_json::json!({ "hair_id": hair_id, "length": length }),
            format!("Hair length set to {}", length),
        ),
        Err(e) => ToolResponse::err("set_hair_length", e),
    }
}
fn handle_set_hair_volume(request: ToolRequest) -> ToolResponse {
    let hair_id = request.get_str("hair_id").unwrap_or_default();
    let volume = request.get_str("volume").unwrap_or_default();
    if hair_id.is_empty() || volume.is_empty() {
        return ToolResponse::err("set_hair_volume", "hair_id and volume are required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "set_hair_volume",
        serde_json::json!({ "hair_id": hair_id, "volume": volume }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_hair_volume",
            serde_json::json!({ "hair_id": hair_id, "volume": volume }),
            format!("Hair volume set to {}", volume),
        ),
        Err(e) => ToolResponse::err("set_hair_volume", e),
    }
}
fn handle_list_hair_presets(request: ToolRequest) -> ToolResponse {
    let figure_type = request.get_str("figure_type");
    let result = crate::mcp_client::send_mcp_request(
        "list_hair_presets",
        serde_json::json!({ "figure_type": figure_type }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "list_hair_presets",
            serde_json::json!({ "presets": r.data, "options": { "styles": ["straight", "wavy", "curly", "voluminous", "sleek", "messy", "updo", "ponytail"], "lengths": ["very_short", "short", "medium", "long", "very_long", "floor_length"], "colors": ["blonde", "brunette", "red", "black", "white"] } }),
            "Hair presets listed",
        ),
        Err(e) => ToolResponse::err("list_hair_presets", e),
    }
}
fn handle_remove_hair(request: ToolRequest) -> ToolResponse {
    let hair_id = request.get_str("hair_id").unwrap_or_default();
    let keep = request.get_bool("keep_textures").unwrap_or(false);
    if hair_id.is_empty() {
        return ToolResponse::err("remove_hair", "hair_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "remove_hair",
        serde_json::json!({ "hair_id": hair_id, "keep_textures": keep }),
    );
    match result {
        Ok(_) => ToolResponse::ok("remove_hair", serde_json::json!({ "removed": hair_id })),
        Err(e) => ToolResponse::err("remove_hair", e),
    }
}
fn handle_set_hair_shader(request: ToolRequest) -> ToolResponse {
    let hair_id = request.get_str("hair_id").unwrap_or_default();
    let shader_type = request
        .get_str("shader_type")
        .unwrap_or_else(|| "natural".to_string());
    let gloss = request.get_f64("gloss").unwrap_or(0.5);
    let specular = request.get_f64("specular").unwrap_or(0.3);
    if hair_id.is_empty() {
        return ToolResponse::err("set_hair_shader", "hair_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "set_hair_shader",
        serde_json::json!({ "hair_id": hair_id, "shader_type": shader_type, "gloss": gloss, "specular": specular }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_hair_shader",
            serde_json::json!({ "hair_id": hair_id, "shader_type": shader_type, "gloss": gloss }),
            format!("Hair shader: {} (gloss: {:.1})", shader_type, gloss),
        ),
        Err(e) => ToolResponse::err("set_hair_shader", e),
    }
}
fn handle_apply_hair_preset(request: ToolRequest) -> ToolResponse {
    let hair_id = request.get_str("hair_id").unwrap_or_default();
    let preset_name = request.get_str("preset_name").unwrap_or_default();
    if hair_id.is_empty() || preset_name.is_empty() {
        return ToolResponse::err("apply_hair_preset", "hair_id and preset_name are required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "apply_hair_preset",
        serde_json::json!({ "hair_id": hair_id, "preset": preset_name }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "apply_hair_preset",
            serde_json::json!({ "hair_id": hair_id, "preset": preset_name }),
            format!("Applied hair preset '{}'", preset_name),
        ),
        Err(e) => ToolResponse::err("apply_hair_preset", e),
    }
}
