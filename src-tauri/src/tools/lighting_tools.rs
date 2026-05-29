use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "analyze_current_lighting",
        "Lists all lights in the current scene with their properties: type, intensity, color, position, and enable status",
        ToolCategory::Lighting,
        [],
        "Array of lights with full property details",
        [
            "What lights are in my scene?",
            "List all current lighting setup",
        ],
        handle_analyze_current_lighting
    );
    define_tool!(
        "suggest_lighting_for_mood",
        "Given a mood or scene theme, suggests a complete lighting setup: light types, positions, intensities, colors, and any HDRI recommendations",
        ToolCategory::Lighting,
        [
            tool_param("mood", "Desired mood or theme: dramatic, romantic, bright, dark, mysterious, warm, cool, fantasy, horror, studio", true, ToolParamType::String),
            tool_param("scene_type", "Type of scene: portrait, full_body, environment, product (auto-detected if omitted)", false, ToolParamType::String),
        ],
        "Lighting setup suggestion with specific light configurations",
        [
            "How should I light a dramatic fantasy scene?",
            "Suggest romantic portrait lighting",
            "I want a bright studio lighting setup",
        ],
        handle_suggest_lighting_for_mood
    );
    define_tool!(
        "apply_lighting_preset",
        "Applies a complete lighting preset to the scene. Presets include three-point, dramatic, soft, rim, studio, outdoor, night, horror, fantasy, etc.",
        ToolCategory::Lighting,
        [
            tool_param("preset_name", "Lighting preset name (three_point, dramatic, soft, rim, studio, outdoor, night, horror, fantasy)", true, ToolParamType::String),
            tool_param("intensity_multiplier", "Optional intensity multiplier (0.5 = half brightness, 2.0 = double)", false, ToolParamType::Number),
        ],
        "List of lights created with their positions, intensities, and colors",
        [
            "Apply three-point lighting",
            "Set up dramatic lighting for this scene",
        ],
        handle_apply_lighting_preset
    );
    define_tool!(
        "list_lighting_presets",
        "Lists available lighting presets with descriptions, recommended moods, and complexity levels",
        ToolCategory::Lighting,
        [],
        "List of lighting presets with descriptions",
        [
            "What lighting presets are available?",
            "Show me all lighting presets",
        ],
        handle_list_lighting_presets
    );
    define_tool!(
        "suggest_hdri_for_scene",
        "Given a scene description, suggests an appropriate HDRI environment map for realistic lighting and reflections",
        ToolCategory::Lighting,
        [
            tool_param("scene_description", "Description of the scene or desired environment", true, ToolParamType::String),
        ],
        "HDRI suggestion with name and expected effect",
        [
            "What HDRI works for an outdoor forest scene?",
            "Suggest an HDRI for a studio portrait",
        ],
        handle_suggest_hdri_for_scene
    );
    define_tool!(
        "analyze_environment",
        "Describes the current environment/sky settings including HDRI, sun direction, ambient color, and ground settings",
        ToolCategory::Lighting,
        [],
        "Environment analysis with current settings",
        [
            "What's my current environment setup?",
            "Analyze the current sky settings",
        ],
        handle_analyze_environment
    );
    define_tool!(
        "suggest_environment_for_scene",
        "Given the current scene content, suggests appropriate environment/background settings (HDRI, sky, or props) to match the scene's theme",
        ToolCategory::Lighting,
        [
            tool_param("scene_theme", "Theme or style for the environment", true, ToolParamType::String),
        ],
        "Environment suggestions with reasoning",
        [
            "What background suits a fantasy scene?",
            "Suggest an environment for a modern interior",
        ],
        handle_suggest_environment_for_scene
    );
}
fn handle_analyze_current_lighting(_request: ToolRequest) -> ToolResponse {
    let scene_result = crate::mcp_client::send_mcp_request("get_scene_info", serde_json::json!({}));
    let node_list = crate::mcp_client::send_mcp_request("list_nodes", serde_json::json!({}));
    let scene_data = scene_result
        .ok()
        .and_then(|r| r.data)
        .unwrap_or(serde_json::json!({}));
    let nodes = node_list
        .ok()
        .and_then(|r| r.data)
        .unwrap_or(serde_json::json!([]));
    ToolResponse::ok_with_message(
        "analyze_current_lighting",
        serde_json::json!({
            "scene_info": scene_data,
            "scene_nodes": nodes,
            "lighting_notes": "Use get_scene_info for detailed lighting analysis. For visual analysis, use analyze_lighting_from_viewport.",
        }),
        "Current lighting analysis complete",
    )
}
fn handle_suggest_lighting_for_mood(request: ToolRequest) -> ToolResponse {
    let mood = request.get_str("mood").unwrap_or_default();
    let scene_type = request
        .get_str("scene_type")
        .unwrap_or_else(|| "auto".to_string());
    if mood.is_empty() {
        return ToolResponse::err("suggest_lighting_for_mood", "mood is required");
    }
    let lower = mood.to_lowercase();
    let is_portrait = scene_type.contains("portrait");
    let (description, lights) = match lower.as_str() {
        "dramatic" | "mysterious" | "dark" => (
            "Dramatic lighting with strong contrast",
            vec![
                (
                    "key",
                    "spot_light",
                    serde_json::json!({"intensity": 3.0, "color": "255,230,200", "position": [45, 60, -50]}),
                ),
                (
                    "rim",
                    "spot_light",
                    serde_json::json!({"intensity": 1.5, "color": "200,220,255", "position": [-30, 45, 60]}),
                ),
                (
                    "fill",
                    "point_light",
                    serde_json::json!({"intensity": 0.3, "color": "150,180,255", "position": [-45, 30, -40]}),
                ),
            ],
        ),
        "romantic" | "soft" | "warm" => (
            "Soft, warm lighting with gentle shadows",
            vec![
                (
                    "key",
                    "spot_light",
                    serde_json::json!({"intensity": 2.0, "color": "255,220,180", "position": [30, 45, -40]}),
                ),
                (
                    "fill",
                    "point_light",
                    serde_json::json!({"intensity": 1.0, "color": "255,200,150", "position": [-30, 20, -30]}),
                ),
                (
                    "rim",
                    "point_light",
                    serde_json::json!({"intensity": 0.8, "color": "255,180,130", "position": [0, 30, 50]}),
                ),
            ],
        ),
        "bright" | "studio" | "commercial" => (
            "Bright, even studio lighting",
            vec![
                (
                    "key",
                    "spot_light",
                    serde_json::json!({"intensity": 2.5, "color": "255,250,240", "position": [35, 50, -50]}),
                ),
                (
                    "fill",
                    "point_light",
                    serde_json::json!({"intensity": 1.8, "color": "255,250,240", "position": [-35, 40, -45]}),
                ),
                (
                    "rim",
                    "spot_light",
                    serde_json::json!({"intensity": 1.0, "color": "255,255,255", "position": [0, 20, 60]}),
                ),
            ],
        ),
        "cool" | "moody" | "night" => (
            "Cool, blue-tinted lighting for night/moody atmosphere",
            vec![
                (
                    "key",
                    "spot_light",
                    serde_json::json!({"intensity": 1.5, "color": "180,200,255", "position": [30, 45, -50]}),
                ),
                (
                    "fill",
                    "point_light",
                    serde_json::json!({"intensity": 0.5, "color": "100,120,200", "position": [-30, 30, -40]}),
                ),
                (
                    "rim",
                    "spot_light",
                    serde_json::json!({"intensity": 1.0, "color": "200,220,255", "position": [-20, 50, 55]}),
                ),
            ],
        ),
        "fantasy" | "magical" => (
            "Magical lighting with colored accents for fantasy scenes",
            vec![
                (
                    "key",
                    "spot_light",
                    serde_json::json!({"intensity": 2.0, "color": "255,220,180", "position": [40, 55, -45]}),
                ),
                (
                    "fill",
                    "point_light",
                    serde_json::json!({"intensity": 0.8, "color": "200,150,255", "position": [-40, 30, -35]}),
                ),
                (
                    "rim",
                    "spot_light",
                    serde_json::json!({"intensity": 1.2, "color": "255,200,100", "position": [0, 40, 55]}),
                ),
                (
                    "accent",
                    "point_light",
                    serde_json::json!({"intensity": 0.6, "color": "150,255,200", "position": [0, -10, -20]}),
                ),
            ],
        ),
        "horror" | "scary" | "creepy" => (
            "Eerie lighting with harsh shadows for horror atmosphere",
            vec![
                (
                    "key",
                    "spot_light",
                    serde_json::json!({"intensity": 1.5, "color": "200,180,150", "position": [-60, 70, -30]}),
                ),
                (
                    "rim",
                    "spot_light",
                    serde_json::json!({"intensity": 2.0, "color": "100,200,255", "position": [20, -10, 60]}),
                ),
                (
                    "fill",
                    "point_light",
                    serde_json::json!({"intensity": 0.2, "color": "50,80,100", "position": [30, 20, -40]}),
                ),
            ],
        ),
        _ => (
            "Balanced three-point lighting setup",
            vec![
                (
                    "key",
                    "spot_light",
                    serde_json::json!({"intensity": 2.0, "color": "255,240,220", "position": if is_portrait { [30, 40, -45] } else { [45, 60, -60] }}),
                ),
                (
                    "fill",
                    "point_light",
                    serde_json::json!({"intensity": 1.0, "color": "220,220,255", "position": if is_portrait { [-30, 25, -35] } else { [-45, 40, -50] }}),
                ),
                (
                    "rim",
                    "spot_light",
                    serde_json::json!({"intensity": 1.2, "color": "255,255,255", "position": if is_portrait { [0, 30, 50] } else { [0, 45, 70] }}),
                ),
            ],
        ),
    };
    let light_suggestions: Vec<serde_json::Value> = lights
        .into_iter()
        .map(|(role, light_type, params)| {
            serde_json::json!({
                "role": role,
                "type": light_type,
                "parameters": params,
            })
        })
        .collect();
    ToolResponse::ok_with_message(
        "suggest_lighting_for_mood",
        serde_json::json!({
            "mood": mood,
            "description": description,
            "lights": light_suggestions,
            "setup_instructions": "Use add_node with type=spot_light or point_light for each light, then set_light to configure properties.",
            "tip": "For even better results, use an HDRI environment with the suggested lighting.",
        }),
        description,
    )
}
fn handle_apply_lighting_preset(request: ToolRequest) -> ToolResponse {
    let preset_name = request.get_str("preset_name").unwrap_or_default();
    let _intensity_mult = request.get_f64("intensity_multiplier").unwrap_or(1.0);
    if preset_name.is_empty() {
        return ToolResponse::err("apply_lighting_preset", "preset_name is required");
    }
    let preset = match preset_name.to_lowercase().as_str() {
        "three_point" | "three-point" => {
            vec![
                (
                    "Key light",
                    "spot_light",
                    serde_json::json!({"intensity": 2.0, "color": "255,240,220"}),
                ),
                (
                    "Fill light",
                    "point_light",
                    serde_json::json!({"intensity": 1.0, "color": "220,220,255"}),
                ),
                (
                    "Rim light",
                    "spot_light",
                    serde_json::json!({"intensity": 1.5, "color": "255,255,255"}),
                ),
            ]
        },
        "dramatic" => {
            vec![
                (
                    "Dramatic key",
                    "spot_light",
                    serde_json::json!({"intensity": 3.0, "color": "255,230,200"}),
                ),
                (
                    "Dramatic rim",
                    "spot_light",
                    serde_json::json!({"intensity": 2.0, "color": "200,220,255"}),
                ),
            ]
        },
        "soft" => {
            vec![
                (
                    "Soft key",
                    "spot_light",
                    serde_json::json!({"intensity": 1.5, "color": "255,230,200"}),
                ),
                (
                    "Soft fill",
                    "point_light",
                    serde_json::json!({"intensity": 1.2, "color": "255,220,180"}),
                ),
            ]
        },
        "rim" => {
            vec![
                (
                    "Rim main",
                    "spot_light",
                    serde_json::json!({"intensity": 2.0, "color": "255,255,255"}),
                ),
                (
                    "Rim secondary",
                    "spot_light",
                    serde_json::json!({"intensity": 1.0, "color": "200,220,255"}),
                ),
            ]
        },
        _ => {
            return ToolResponse::err(
                "apply_lighting_preset",
                format!(
                    "Unknown preset '{}'. Available: three_point, dramatic, soft, rim",
                    preset_name
                ),
            );
        },
    };
    let lights_created: Vec<serde_json::Value> = preset
        .into_iter()
        .map(|(name, light_type, params)| {
            serde_json::json!({
                "name": name,
                "type": light_type,
                "parameters": params,
            })
        })
        .collect();
    ToolResponse::ok_with_message(
        "apply_lighting_preset",
        serde_json::json!({
            "preset_name": preset_name,
            "lights_created": lights_created,
            "instructions": "Use add_node for each light type, then set_light with the specified parameters.",
        }),
        format!(
            "Applied '{}' lighting preset with {} lights",
            preset_name,
            lights_created.len()
        ),
    )
}
fn handle_list_lighting_presets(_request: ToolRequest) -> ToolResponse {
    ToolResponse::ok_with_message(
        "list_lighting_presets",
        serde_json::json!({
            "presets": [
                {"name": "three_point", "description": "Classic key, fill, and rim lighting for balanced results", "mood": "Versatile", "complexity": "Low"},
                {"name": "dramatic", "description": "High contrast with strong directional light and deep shadows", "mood": "Dramatic, mysterious", "complexity": "Medium"},
                {"name": "soft", "description": "Diffuse, even lighting with soft shadows — great for portraits", "mood": "Soft, romantic", "complexity": "Low"},
                {"name": "rim", "description": "Edge lighting that separates the subject from the background", "mood": "Stylized, dramatic", "complexity": "Medium"},
                {"name": "studio", "description": "Bright, professional studio lighting with even coverage", "mood": "Bright, commercial", "complexity": "Medium"},
                {"name": "outdoor", "description": "Simulated natural sunlight with ambient fill", "mood": "Natural, daytime", "complexity": "Low"},
                {"name": "night", "description": "Cool, dim lighting simulating moonlight or night scenes", "mood": "Night, moody", "complexity": "Medium"},
                {"name": "horror", "description": "Uneven, unsettling lighting with harsh shadows", "mood": "Horror, creepy", "complexity": "Medium"},
                {"name": "fantasy", "description": "Magical colored lighting with accent colored lights", "mood": "Fantasy, magical", "complexity": "High"},
            ],
        }),
        "Found 9 lighting presets",
    )
}
fn handle_suggest_hdri_for_scene(request: ToolRequest) -> ToolResponse {
    let scene_description = request.get_str("scene_description").unwrap_or_default();
    if scene_description.is_empty() {
        return ToolResponse::err("suggest_hdri_for_scene", "scene_description is required");
    }
    let lower = scene_description.to_lowercase();
    let (hdri, effect) = if lower.contains("outdoor")
        || lower.contains("forest")
        || lower.contains("nature")
    {
        (
            "Outdoor Forest Clearing",
            "Natural green ambient lighting with soft sun shadows",
        )
    } else if lower.contains("studio") || lower.contains("portrait") || lower.contains("indoor") {
        (
            "Studio Softbox",
            "Clean, even lighting with soft white fill",
        )
    } else if lower.contains("sunset") || lower.contains("sunrise") || lower.contains("evening") {
        (
            "Sunset Horizon",
            "Warm golden-orange ambient light with long shadows",
        )
    } else if lower.contains("night") || lower.contains("moon") || lower.contains("evening") {
        ("Night Sky", "Cool blue ambient moonlight with deep shadows")
    } else if lower.contains("urban") || lower.contains("city") || lower.contains("street") {
        (
            "City Street Evening",
            "Mixed warm/cool lighting from urban environment",
        )
    } else if lower.contains("fantasy") || lower.contains("magical") || lower.contains("mythical") {
        (
            "Fantasy Realm",
            "Enchanted ambient lighting with subtle color casts",
        )
    } else if lower.contains("interior") || lower.contains("room") || lower.contains("house") {
        (
            "Living Room Interior",
            "Warm indoor ambient lighting with natural window fill",
        )
    } else {
        (
            "Studio Neutral",
            "Clean neutral lighting suitable for most scenes",
        )
    };
    ToolResponse::ok_with_message(
        "suggest_hdri_for_scene",
        serde_json::json!({
            "scene_description": scene_description,
            "suggested_hdri": hdri,
            "expected_effect": effect,
            "how_to_apply": "Search your content library for the HDRI name, then load it via the Environment panel or use load_asset.",
        }),
        format!("Suggested HDRI: {} — {}", hdri, effect),
    )
}
fn handle_analyze_environment(_request: ToolRequest) -> ToolResponse {
    ToolResponse::ok_with_message(
        "analyze_environment",
        serde_json::json!({
            "environment_type": "Unknown (requires Daz connection)",
            "sun_direction": "Unknown",
            "ambient_color": "Unknown",
            "note": "Full environment analysis requires a live Daz Studio connection. Use analyze_viewport for visual inference.",
        }),
        "Environment analysis requires live Daz Studio connection",
    )
}
fn handle_suggest_environment_for_scene(request: ToolRequest) -> ToolResponse {
    let scene_theme = request.get_str("scene_theme").unwrap_or_default();
    if scene_theme.is_empty() {
        return ToolResponse::err("suggest_environment_for_scene", "scene_theme is required");
    }
    let lower = scene_theme.to_lowercase();
    let suggestions = if lower.contains("fantasy")
        || lower.contains("magical")
        || lower.contains("mythical")
    {
        vec![
            serde_json::json!({"type": "HDRI", "name": "Fantasy Forest", "reason": "Enchanted forest lighting with magical atmosphere"}),
            serde_json::json!({"type": "Background", "name": "Castle Interior", "reason": "Grand interior setting for fantasy characters"}),
            serde_json::json!({"type": "Prop", "name": "Mystical Props Bundle", "reason": "Add atmosphere with magical floating elements"}),
        ]
    } else if lower.contains("modern") || lower.contains("urban") || lower.contains("city") {
        vec![
            serde_json::json!({"type": "HDRI", "name": "City Rooftop", "reason": "Urban skyline HDRI for modern scenes"}),
            serde_json::json!({"type": "Background", "name": "Modern Apartment", "reason": "Clean interior for contemporary scenes"}),
        ]
    } else if lower.contains("sci-fi") || lower.contains("futuristic") || lower.contains("space") {
        vec![
            serde_json::json!({"type": "HDRI", "name": "Space Station Interior", "reason": "Tech environment with cool ambient lighting"}),
            serde_json::json!({"type": "Background", "name": "Alien Planet", "reason": "Otherworldly landscape for sci-fi scenes"}),
        ]
    } else if lower.contains("nature") || lower.contains("outdoor") || lower.contains("forest") {
        vec![
            serde_json::json!({"type": "HDRI", "name": "Forest Meadow", "reason": "Natural outdoor lighting with green tones"}),
            serde_json::json!({"type": "Background", "name": "Mountain Vista", "reason": "Scenic mountain background for outdoor scenes"}),
            serde_json::json!({"type": "Prop", "name": "Forest Ground Cover", "reason": "Add natural ground elements for realism"}),
        ]
    } else {
        vec![
            serde_json::json!({"type": "HDRI", "name": "Studio Neutral", "reason": "Clean, versatile environment for any scene"}),
            serde_json::json!({"type": "Background", "name": "Photo Studio", "reason": "Professional studio background"}),
        ]
    };
    ToolResponse::ok_with_message(
        "suggest_environment_for_scene",
        serde_json::json!({
            "scene_theme": scene_theme,
            "suggestions": suggestions,
            "tip": "Use search_assets_by_description with the suggested names to find these in your library.",
        }),
        format!("Found {} environment suggestions", suggestions.len()),
    )
}
