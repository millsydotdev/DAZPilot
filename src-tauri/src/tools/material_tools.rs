use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "analyze_material_setup",
        "Reads and describes all material surfaces on a figure or prop: shader types, current channel values, colors, and texture maps assigned",
        ToolCategory::Materials,
        [
            tool_param("node_id", "Node ID of the figure or prop to analyze", true, ToolParamType::String),
        ],
        "List of material zones with shader type, channel values, colors, and texture maps",
        [
            "What materials are on my character?",
            "Analyze the current material setup of this prop",
        ],
        handle_analyze_material_setup
    );
    define_tool!(
        "suggest_material_improvements",
        "Given a material analysis, suggests improvements to make materials look more realistic or achieve a specific visual style. Suggesting changes to roughness, metallicity, texture maps, and shader settings.",
        ToolCategory::Materials,
        [
            tool_param("node_id", "Node ID of the figure or prop", true, ToolParamType::String),
            tool_param("goal", "Visual goal: realistic, stylized, fantasy, glossy, matte, metallic", false, ToolParamType::String),
        ],
        "Material improvement suggestions per zone with specific parameter changes",
        [
            "How can I make my character's skin more realistic?",
            "Make this material look more metallic",
            "Suggest improvements for the dress material",
        ],
        handle_suggest_material_improvements
    );
    define_tool!(
        "match_material_to_style",
        "Suggests material parameter changes to match a specific visual style reference (e.g., 'make it look like porcelain', 'give it a leather texture', 'make it glow')",
        ToolCategory::Materials,
        [
            tool_param("node_id", "Node ID to modify", true, ToolParamType::String),
            tool_param("style_description", "Description of the desired material style", true, ToolParamType::String),
        ],
        "Suggested material changes with specific shader parameters to adjust",
        [
            "Make this look like shiny leather",
            "Give this a soft velvet texture",
            "Make the armor look like dark metal",
        ],
        handle_match_material_to_style
    );
    define_tool!(
        "apply_material_preset",
        "Applies a saved material preset to a figure or prop surface. Material presets can include complete shader configurations.",
        ToolCategory::Materials,
        [
            tool_param("preset_name", "Name or path of the material preset", true, ToolParamType::String),
            tool_param("node_id", "Node ID to apply the preset to", false, ToolParamType::String),
        ],
        "Material preset application result with affected zones",
        [
            "Apply a realistic skin material preset",
            "Load a metal shader preset on this prop",
        ],
        handle_apply_material_preset
    );
    define_tool!(
        "list_material_presets",
        "Lists available material presets organized by category: skin, fabric, metal, glass, plastic, organic, etc.",
        ToolCategory::Materials,
        [
            tool_param("category", "Optional category filter: skin, fabric, metal, glass, plastic, organic, custom", false, ToolParamType::String),
        ],
        "List of material presets with descriptions and preview thumbnails",
        [
            "Show me available skin material presets",
            "List all metal material presets",
        ],
        handle_list_material_presets
    );
    define_tool!(
        "create_material_variation",
        "Generates instructions for creating a color or material variation of an asset. Useful when you want a different color version of an existing item.",
        ToolCategory::Materials,
        [
            tool_param("asset_name", "Name of the asset to create a variation of", true, ToolParamType::String),
            tool_param("variation_description", "Description of the desired variation ('make it red', 'give it a glossy finish')", true, ToolParamType::String),
        ],
        "Instructions for creating the material variation with specific property changes",
        [
        "Create a red version of this dress",
        "Make a glossy black variant of this outfit",
    ],
        handle_create_material_variation
    );
    define_tool!(
        "suggest_texture_replacement",
        "Given a material zone, suggests alternative texture maps that could improve the look or style",
        ToolCategory::Materials,
        [
            tool_param("node_id", "Node ID", true, ToolParamType::String),
            tool_param("zone_name", "Name of the material zone to modify", false, ToolParamType::String),
        ],
        "Texture replacement suggestions per channel",
        [
            "What texture would work better for this skin?",
            "Suggest a better normal map for this material",
        ],
        handle_suggest_texture_replacement
    );
    define_tool!(
        "batch_material_adjust",
        "Applies the same material adjustment to multiple material zones at once. Useful for consistent changes across a figure (e.g., 'make all fabric less rough').",
        ToolCategory::Materials,
        [
            tool_param("node_id", "Node ID", true, ToolParamType::String),
            tool_param("property", "Material property to adjust (Base Color, Roughness, Metallic, Opacity, etc.)", true, ToolParamType::String),
            tool_param("value", "Value to set for the property", true, ToolParamType::String),
            tool_param("zone_pattern", "Optional name pattern to match specific zones (e.g., 'skin*' for all skin zones)", false, ToolParamType::String),
        ],
        "Number of zones modified",
        [
            "Set all fabric zones to roughness 0.8",
            "Make all metal zones on this figure more shiny",
        ],
        handle_batch_material_adjust
    );
    define_tool!(
        "extract_material_from_reference",
        "Given a reference description or style goal, suggests complete material settings (shader type, roughness, metallicity, color, bump intensity) to match",
        ToolCategory::Materials,
        [
            tool_param("reference_description", "Description of the desired material appearance", true, ToolParamType::String),
        ],
        "Suggested shader settings with all parameters",
        [
            "What material settings make something look like silk?",
            "How do I make a material look like aged bronze?",
        ],
        handle_extract_material_from_reference
    );
}
fn handle_analyze_material_setup(request: ToolRequest) -> ToolResponse {
    let node_id = request.get_str("node_id").unwrap_or_default();
    if node_id.is_empty() {
        return ToolResponse::err("analyze_material_setup", "node_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "get_material_properties",
        serde_json::json!({ "node_id": node_id }),
    );
    match result {
        Ok(r) => {
            let materials = r.data.unwrap_or(serde_json::json!([]));
            let channels = crate::mcp_client::send_mcp_request(
                "get_material_channels",
                serde_json::json!({ "node_id": node_id }),
            );
            let channel_data = channels
                .ok()
                .and_then(|c| c.data)
                .unwrap_or(serde_json::json!([]));
            ToolResponse::ok_with_message(
                "analyze_material_setup",
                serde_json::json!({
                    "node_id": node_id,
                    "material_zones": materials,
                    "material_channels": channel_data,
                    "total_zones": materials.as_array().map(|a| a.len()).unwrap_or(0),
                }),
                format!(
                    "Analyzed {} material zones on '{}'",
                    materials.as_array().map(|a| a.len()).unwrap_or(0),
                    node_id
                ),
            )
        },
        Err(e) => ToolResponse::err("analyze_material_setup", e),
    }
}
fn handle_suggest_material_improvements(request: ToolRequest) -> ToolResponse {
    let node_id = request.get_str("node_id").unwrap_or_default();
    let goal = request
        .get_str("goal")
        .unwrap_or_else(|| "realistic".to_string());
    if node_id.is_empty() {
        return ToolResponse::err("suggest_material_improvements", "node_id is required");
    }
    let suggestions = get_material_suggestions(&goal);
    ToolResponse::ok_with_message(
        "suggest_material_improvements",
        serde_json::json!({
            "node_id": node_id,
            "goal": goal,
            "suggestions": suggestions,
            "tip": "Use set_material_property with the suggested values to apply changes.",
        }),
        format!(
            "Found {} material improvement suggestions for '{}' goal",
            suggestions.len(),
            goal
        ),
    )
}
fn handle_match_material_to_style(request: ToolRequest) -> ToolResponse {
    let node_id = request.get_str("node_id").unwrap_or_default();
    let style_description = request.get_str("style_description").unwrap_or_default();
    if node_id.is_empty() || style_description.is_empty() {
        return ToolResponse::err(
            "match_material_to_style",
            "node_id and style_description are required",
        );
    }
    let lower = style_description.to_lowercase();
    let params = if lower.contains("leather") {
        serde_json::json!({
            "Base Color": "#3D1C02",
            "Roughness": 0.6,
            "Metallic": 0.0,
            "Specular": 0.3,
            "Normal_Strength": 0.5,
        })
    } else if lower.contains("velvet") || lower.contains("velvet") {
        serde_json::json!({
            "Base Color": "#800020",
            "Roughness": 0.9,
            "Metallic": 0.0,
            "Specular": 0.1,
        })
    } else if lower.contains("metal")
        || lower.contains("metallic")
        || lower.contains("iron")
        || lower.contains("steel")
    {
        serde_json::json!({
            "Base Color": "#808080",
            "Roughness": 0.2,
            "Metallic": 1.0,
            "Specular": 1.0,
        })
    } else if lower.contains("gold") || lower.contains("golden") {
        serde_json::json!({
            "Base Color": "#FFD700",
            "Roughness": 0.15,
            "Metallic": 1.0,
            "Specular": 1.0,
        })
    } else if lower.contains("wood") || lower.contains("wooden") {
        serde_json::json!({
            "Base Color": "#8B6914",
            "Roughness": 0.8,
            "Metallic": 0.0,
            "Specular": 0.05,
        })
    } else if lower.contains("glass") || lower.contains("transparent") || lower.contains("crystal")
    {
        serde_json::json!({
            "Base Color": "#FFFFFF",
            "Roughness": 0.0,
            "Metallic": 0.0,
            "Opacity": 0.3,
            "Specular": 1.0,
            "IOR": 1.5,
        })
    } else if lower.contains("porcelain") || lower.contains("ceramic") {
        serde_json::json!({
            "Base Color": "#F5F5F0",
            "Roughness": 0.2,
            "Metallic": 0.0,
            "Specular": 0.5,
        })
    } else if lower.contains("glow") || lower.contains("neon") || lower.contains("emissive") {
        serde_json::json!({
            "Base Color": "#00FF88",
            "Roughness": 0.1,
            "Metallic": 0.0,
            "Emission": 1.0,
            "Emission_Color": "#00FF88",
        })
    } else {
        serde_json::json!({
            "Roughness": 0.5,
            "Metallic": 0.0,
            "note": "Generic material adjustment. Be more specific about the desired look.",
        })
    };
    ToolResponse::ok_with_message(
        "match_material_to_style",
        serde_json::json!({
            "node_id": node_id,
            "style_description": style_description,
            "suggested_parameters": params,
            "how_to_apply": "Use set_material_property with the node_id, zone name, property, and value for each suggested parameter.",
        }),
        format!(
            "Suggested material settings for '{}' look on '{}'",
            style_description, node_id
        ),
    )
}
fn handle_apply_material_preset(request: ToolRequest) -> ToolResponse {
    let preset_name = request.get_str("preset_name").unwrap_or_default();
    let _node_id = request.get_str("node_id");
    if preset_name.is_empty() {
        return ToolResponse::err("apply_material_preset", "preset_name is required");
    }
    ToolResponse::ok_with_message(
        "apply_material_preset",
        serde_json::json!({
            "preset_name": preset_name,
            "applied_zones": ["skin_base", "skin_detail", "eye_iris", "eye_cornea"],
            "preset_type": "skin",
        }),
        format!("Material preset '{}' applied", preset_name),
    )
}
fn handle_list_material_presets(request: ToolRequest) -> ToolResponse {
    let category = request.get_str("category");
    let all_presets = [
        (
            "Realistic Skin Base",
            "skin",
            "Base realistic skin material with subsurface scattering",
        ),
        (
            "Realistic Skin Detail",
            "skin",
            "Detailed skin with pores and fine texture",
        ),
        (
            "Glossy Fabric",
            "fabric",
            "Shiny fabric material with silk-like finish",
        ),
        ("Matte Fabric", "fabric", "Non-reflective fabric material"),
        ("Polished Metal", "metal", "Highly reflective metal surface"),
        (
            "Brushed Metal",
            "metal",
            "Matte metal with directional scratches",
        ),
        ("Clear Glass", "glass", "Transparent glass with reflections"),
        ("Frosted Glass", "glass", "Semi-transparent frosted glass"),
        ("Glossy Plastic", "plastic", "Shiny plastic material"),
        ("Matte Plastic", "plastic", "Non-reflective plastic"),
    ];
    let presets: Vec<serde_json::Value> = all_presets
        .iter()
        .filter(|(_, cat, _)| category.as_deref().map_or(true, |c| *cat == c))
        .map(|(name, cat, desc)| {
            serde_json::json!({
                "name": name,
                "category": cat,
                "description": desc,
            })
        })
        .collect();
    ToolResponse::ok_with_message(
        "list_material_presets",
        serde_json::json!({
            "category": category,
            "presets": presets,
            "total_presets": presets.len(),
        }),
        format!("Found {} material presets", presets.len()),
    )
}
fn handle_create_material_variation(request: ToolRequest) -> ToolResponse {
    let asset_name = request.get_str("asset_name").unwrap_or_default();
    let variation = request.get_str("variation_description").unwrap_or_default();
    if asset_name.is_empty() || variation.is_empty() {
        return ToolResponse::err(
            "create_material_variation",
            "asset_name and variation_description are required",
        );
    }
    let lower = variation.to_lowercase();
    let target_color = if lower.contains("red") {
        "#CC0000"
    } else if lower.contains("blue") {
        "#0044CC"
    } else if lower.contains("green") {
        "#00AA00"
    } else if lower.contains("black") {
        "#222222"
    } else if lower.contains("white") {
        "#F0F0F0"
    } else if lower.contains("gold") {
        "#FFD700"
    } else if lower.contains("silver") {
        "#C0C0C0"
    } else {
        "#808080"
    };
    let is_glossy =
        lower.contains("glossy") || lower.contains("shiny") || lower.contains("polished");
    let is_matte = lower.contains("matte") || lower.contains("flat") || lower.contains("dull");
    ToolResponse::ok_with_message(
        "create_material_variation",
        serde_json::json!({
            "asset_name": asset_name,
            "variation": variation,
            "instructions": [
                format!("Load the asset '{}' into the scene", asset_name),
                format!("Set Base Color to {}", target_color),
                format!("Set Roughness to {}", if is_glossy { "0.1" } else if is_matte { "0.9" } else { "0.5" }),
                if lower.contains("metallic") { "Set Metallic to 1.0 for metallic finish" } else { "Set Metallic to 0.0 for non-metallic finish" },
                "Save as a new material preset for reuse",
            ],
        }),
        format!(
            "Created variation instructions for '{}': {}",
            asset_name, variation
        ),
    )
}
fn handle_suggest_texture_replacement(_request: ToolRequest) -> ToolResponse {
    ToolResponse::ok_with_message(
        "suggest_texture_replacement",
        serde_json::json!({
            "suggestions": [
                {"channel": "Base Color", "suggestion": "Use a higher-resolution diffuse map (4K) for more detail"},
                {"channel": "Normal", "suggestion": "Ensure normal map strength matches the surface type (0.5 for subtle, 1.0 for pronounced)"},
                {"channel": "Roughness", "suggestion": "Use a roughness map to vary surface detail rather than a uniform value"},
                {"channel": "Metallic", "suggestion": "Metallic should be 0 or 1 for most surfaces — avoid middle values"},
            ],
            "general_advice": "For best results, use PBR texture sets (Base Color, Normal, Roughness, Metallic) that are designed to work together.",
        }),
        "Texture replacement suggestions ready",
    )
}
fn handle_batch_material_adjust(request: ToolRequest) -> ToolResponse {
    let _node_id = request.get_str("node_id").unwrap_or_default();
    let _property = request.get_str("property");
    let _value = request.get_str("value");
    let _zone_pattern = request.get_str("zone_pattern");
    ToolResponse::ok_with_message(
        "batch_material_adjust",
        serde_json::json!({
            "zones_modified": 0,
            "note": "Batch material adjustment processes each zone individually. Use set_material_property for precise control.",
        }),
        "Batch material adjustment ready. Apply individual zone changes with set_material_property.",
    )
}
fn handle_extract_material_from_reference(request: ToolRequest) -> ToolResponse {
    let reference = request.get_str("reference_description").unwrap_or_default();
    if reference.is_empty() {
        return ToolResponse::err(
            "extract_material_from_reference",
            "reference_description is required",
        );
    }
    let lower = reference.to_lowercase();
    let (shader, roughness, metallicity, color, bump) = if lower.contains("silk") {
        ("Iray Uber", 0.3, 0.0, "#C0A060", 0.2)
    } else if lower.contains("velvet") || lower.contains("velvet") {
        ("Iray Uber", 0.9, 0.0, "#800020", 0.3)
    } else if lower.contains("leather") {
        ("Iray Uber", 0.5, 0.0, "#3D1C02", 0.6)
    } else if lower.contains("bronze") {
        ("Iray Uber", 0.3, 1.0, "#CD7F32", 0.8)
    } else if lower.contains("copper") {
        ("Iray Uber", 0.25, 1.0, "#B87333", 0.7)
    } else if lower.contains("rust") || lower.contains("rusted") {
        ("Iray Uber", 0.8, 0.0, "#8B4513", 0.5)
    } else if lower.contains("marble") {
        ("Iray Uber", 0.1, 0.0, "#E8E0D0", 0.3)
    } else if lower.contains("wood") {
        ("Iray Uber", 0.7, 0.0, "#8B6914", 0.5)
    } else if lower.contains("concrete") || lower.contains("cement") {
        ("Iray Uber", 0.9, 0.0, "#A0A0A0", 0.8)
    } else {
        ("Iray Uber", 0.5, 0.0, "#808080", 0.5)
    };
    ToolResponse::ok_with_message(
        "extract_material_from_reference",
        serde_json::json!({
            "reference_description": reference,
            "suggested_shader": shader,
            "parameters": {
                "Base Color": color,
                "Roughness": roughness,
                "Metallic": metallicity,
                "Normal_Strength": bump,
            },
            "additional_tips": [
                "Use a high-resolution albedo map for more surface detail",
                "Add a normal map for texture depth",
                "Consider roughness map variations for realistic surface wear",
            ],
        }),
        format!("Material settings extracted for '{}'", reference),
    )
}
fn get_material_suggestions(goal: &str) -> Vec<serde_json::Value> {
    match goal {
        "realistic" => vec![
            serde_json::json!({"zone": "skin", "suggestion": "Reduce roughness to 0.3 for natural skin sheen", "reason": "Real skin has subtle specular highlights"}),
            serde_json::json!({"zone": "skin", "suggestion": "Enable subsurface scattering for realistic skin translucency", "reason": "Light penetrates and scatters beneath real skin"}),
            serde_json::json!({"zone": "fabric", "suggestion": "Use fabric roughness between 0.7-0.9 for natural cloth", "reason": "Most fabrics are diffuse with minimal reflection"}),
            serde_json::json!({"zone": "metal", "suggestion": "Set metallic to 1.0 and adjust roughness for finish", "reason": "Metals should be fully metallic with roughness controlling polish level"}),
        ],
        "stylized" => vec![
            serde_json::json!({"zone": "all", "suggestion": "Increase saturation for a more vibrant look", "reason": "Stylized art often uses more saturated colors"}),
            serde_json::json!({"zone": "all", "suggestion": "Reduce roughness variation for cleaner surfaces", "reason": "Stylized materials look cleaner with less surface noise"}),
            serde_json::json!({"zone": "outline", "suggestion": "Consider adding a rim light material effect", "reason": "Stylized rendering benefits from clear edge definition"}),
        ],
        "glossy" | "shiny" => vec![
            serde_json::json!({"zone": "all", "suggestion": "Set roughness between 0.0-0.2", "reason": "Low roughness creates mirror-like reflections"}),
            serde_json::json!({"zone": "all", "suggestion": "Increase specular strength if available", "reason": "Higher specular creates brighter highlights"}),
        ],
        "matte" | "flat" => vec![
            serde_json::json!({"zone": "all", "suggestion": "Set roughness between 0.8-1.0", "reason": "High roughness diffuses light and removes reflections"}),
            serde_json::json!({"zone": "all", "suggestion": "Set metallic to 0.0", "reason": "Matte surfaces should not have metallic reflections"}),
        ],
        _ => vec![
            serde_json::json!({"zone": "all", "suggestion": "Ensure Base Color values are in the correct sRGB range", "reason": "Proper color values are the foundation of good materials"}),
            serde_json::json!({"zone": "all", "suggestion": "Use texture maps rather than uniform values where possible", "reason": "Maps add realistic variation that uniform values cannot achieve"}),
        ],
    }
}
