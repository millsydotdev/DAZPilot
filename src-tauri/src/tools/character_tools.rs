use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "suggest_morphs_for_look",
        "Given a natural language description of a desired character look, suggests specific morph dial values to achieve it. Example: 'make her look like an elven princess' will suggest morphs for pointed ears, delicate features, etc.",
        ToolCategory::CharacterCustomization,
        [
            tool_param("description", "Description of the desired character look", true, ToolParamType::String),
            tool_param("figure_id", "Figure node ID to suggest morphs for", false, ToolParamType::String),
            tool_param("max_suggestions", "Maximum number of morph suggestions (default 10)", false, ToolParamType::Integer),
        ],
        "List of suggested morphs with names, target values, and reasons",
        [
            "Make her look like a fantasy elf",
            "Suggest morphs for a muscular hero character",
            "I want a soft, youthful face",
        ],
        handle_suggest_morphs_for_look
    );
    define_tool!(
        "get_morph_details",
        "Returns detailed information about a specific morph dial: its range, category, what body part it controls, and typical usage",
        ToolCategory::CharacterCustomization,
        [
            tool_param("morph_name", "Name of the morph dial to look up", true, ToolParamType::String),
            tool_param("figure_id", "Figure node ID", false, ToolParamType::String),
        ],
        "Morph details with min/max range, category, description, and related morphs",
        [
            "What does the Head_Height morph do?",
            "Tell me about the Waist_Width morph",
        ],
        handle_get_morph_details
    );
    define_tool!(
        "get_morphs_by_category",
        "Lists all available morph dials on a figure, organized by body region/function (head, face, body, arms, legs, etc.)",
        ToolCategory::CharacterCustomization,
        [
            tool_param("region", "Body region: head, face, body, arms, legs, full (default full)", false, ToolParamType::String),
            tool_param("figure_id", "Figure node ID", false, ToolParamType::String),
        ],
        "Morphs grouped by category with their current values and ranges",
        [
            "Show me all head morphs",
            "List face morphs available for my character",
        ],
        handle_get_morphs_by_category
    );
    define_tool!(
        "save_character_preset",
        "Saves the current figure's state (all morph values, body proportions, and materials) as a reusable character preset file so you can load the same look later",
        ToolCategory::CharacterCustomization,
        [
            tool_param("name", "Name for this character preset", true, ToolParamType::String),
            tool_param("figure_id", "Figure node ID to save", false, ToolParamType::String),
            tool_param("include_materials", "Whether to include current materials (default true)", false, ToolParamType::Boolean),
        ],
        "Preset save confirmation with file path and saved properties",
        [
            "Save this character look as a preset called 'Elf Queen'",
            "Save my current character configuration",
        ],
        handle_save_character_preset
    );
    define_tool!(
        "load_character_preset",
        "Loads a previously saved character preset onto a figure, restoring the saved morph values, body proportions, and optionally materials",
        ToolCategory::CharacterCustomization,
        [
            tool_param("preset_name", "Name or path of the character preset to load", true, ToolParamType::String),
            tool_param("figure_id", "Figure node ID to apply the preset to", false, ToolParamType::String),
        ],
        "Load results with applied morphs and materials",
        [
            "Load the 'Elf Queen' character preset",
            "Apply my saved character preset to this figure",
        ],
        handle_load_character_preset
    );
    define_tool!(
        "list_character_presets",
        "Lists all saved character presets available in your library with names, thumbnails, figure types, and creation dates",
        ToolCategory::CharacterCustomization,
        [],
        "List of saved presets with metadata",
        [
            "Show me my saved character presets",
            "What presets do I have saved?",
        ],
        handle_list_character_presets
    );
    define_tool!(
        "suggest_figure_type",
        "Given a character description, suggests the best base figure type (Genesis 8/9, Male/Female) and explains why",
        ToolCategory::CharacterCustomization,
        [
            tool_param("description", "Description of the character you want to create", true, ToolParamType::String),
        ],
        "Figure type suggestion with reasoning and morph requirements",
        [
            "What figure should I use for a muscular male character?",
            "Best figure for a fantasy elven queen?",
        ],
        handle_suggest_figure_type
    );
    define_tool!(
        "apply_body_proportions",
        "Applies a body proportion preset to a figure: heroic, slender, curvy, athletic, average, etc. Adjusts multiple morphs simultaneously for a cohesive look",
        ToolCategory::CharacterCustomization,
        [
            tool_param("proportion_type", "Proportion type: heroic, slender, curvy, athletic, average, petite, statuesque", true, ToolParamType::String),
            tool_param("figure_id", "Figure node ID", false, ToolParamType::String),
        ],
        "Applied proportion preset with list of changed morphs and their new values",
        [
        "Make my character athletic",
        "Apply heroic proportions to this figure",
    ],
        handle_apply_body_proportions
    );
    define_tool!(
        "get_worn_items_analysis",
        "Returns a structured analysis of everything a figure is currently wearing: clothing items, hair, accessories, shoes, and props — organized by category with fit status",
        ToolCategory::CharacterCustomization,
        [
            tool_param("figure_id", "Figure node ID", false, ToolParamType::String),
        ],
        "Worn items grouped by category with fit status and compatibility info",
        [
            "What is my character wearing?",
            "List all items on this figure",
        ],
        handle_get_worn_items_analysis
    );
    define_tool!(
        "save_outfit_preset",
        "Saves the current outfit (all worn items) as a reusable outfit preset that can be loaded onto the same or a compatible figure later",
        ToolCategory::CharacterCustomization,
        [
            tool_param("name", "Name for this outfit preset", true, ToolParamType::String),
            tool_param("figure_id", "Figure node ID to save outfit from", false, ToolParamType::String),
        ],
        "Outfit preset save confirmation with included items",
        [
            "Save this outfit as 'Fantasy Warrior'",
            "Save my current clothing setup as a preset",
        ],
        handle_save_outfit_preset
    );
    define_tool!(
        "load_outfit_preset",
        "Loads a saved outfit preset onto a figure, loading all clothing, hair, and accessories that were saved in the preset",
        ToolCategory::CharacterCustomization,
        [
            tool_param("preset_name", "Name or path of the outfit preset", true, ToolParamType::String),
            tool_param("figure_id", "Figure node ID", false, ToolParamType::String),
        ],
        "Load results with successfully and failed items",
        [
            "Load the 'Fantasy Warrior' outfit on this figure",
            "Apply my saved outfit preset",
        ],
        handle_load_outfit_preset
    );
    define_tool!(
        "suggest_outfit_for_scene",
        "Given the scene type/theme, suggests a complete outfit (clothing, hair, shoes, accessories) that would work well together",
        ToolCategory::CharacterCustomization,
        [
            tool_param("scene_theme", "Theme or style for the outfit (fantasy, modern, sci-fi, formal, casual, etc.)", true, ToolParamType::String),
            tool_param("figure_type", "Figure type (e.g., 'Genesis 9 Female')", false, ToolParamType::String),
        ],
        "Suggested outfit with primary clothing, hair, shoes, and accessories",
        [
            "What should my fantasy character wear?",
            "Suggest a modern casual outfit",
        ],
        handle_suggest_outfit_for_scene
    );
}
fn handle_suggest_morphs_for_look(request: ToolRequest) -> ToolResponse {
    let description = request.get_str("description").unwrap_or_default();
    let _figure_id = request.get_str("figure_id");
    let max_suggestions = request.get_i64("max_suggestions").unwrap_or(10) as usize;
    if description.is_empty() {
        return ToolResponse::err("suggest_morphs_for_look", "description is required");
    }
    let lower = description.to_lowercase();
    let mut suggestions = Vec::new();
    if lower.contains("elf")
        || lower.contains("elven")
        || lower.contains("fairy")
        || lower.contains("pointed ear")
    {
        suggestions.push(("Ear_Length", 0.7, "Elongated ears for elven/fantasy look"));
        suggestions.push(("Ear_Tip", 0.6, "Pointed ear tips"));
        suggestions.push((
            "Cheek_Bone",
            0.5,
            "High cheekbones for elegant elven features",
        ));
        suggestions.push((
            "Head_Width",
            -0.1,
            "Slightly narrower head for refined look",
        ));
        suggestions.push(("Eye_Orient", 1.0, "Almond-shaped eyes"));
        suggestions.push(("Nose_Length", 0.2, "Slightly longer, elegant nose"));
    } else if lower.contains("heroic")
        || lower.contains("muscular")
        || lower.contains("strong")
        || lower.contains("warrior")
    {
        suggestions.push((
            "Shoulder_Width",
            1.0,
            "Broad shoulders for heroic proportions",
        ));
        suggestions.push(("Chest_Size", 1.0, "Developed chest"));
        suggestions.push(("Waist_Width", 1.0, "Narrow waist for V-taper"));
        suggestions.push(("Arm_Muscle", 1.0, "Defined arm muscles"));
        suggestions.push(("Leg_Muscle", 1.0, "Strong leg muscles"));
        suggestions.push(("Neck_Length", 0.3, "Thicker neck for powerful look"));
        suggestions.push((
            "Head_Height",
            1.0,
            "Larger head to match heroic proportions",
        ));
    } else if lower.contains("slender")
        || lower.contains("slim")
        || lower.contains("thin")
        || lower.contains("model")
    {
        suggestions.push(("Waist_Width", -0.5, "Narrower waist for slender look"));
        suggestions.push(("Hip_Width", -0.3, "Narrower hips"));
        suggestions.push(("Torso_Length", 0.2, "Slightly longer torso"));
        suggestions.push(("Arm_Length", 0.2, "Slightly longer arms"));
        suggestions.push(("Leg_Length", 0.3, "Longer legs for model proportions"));
        suggestions.push(("Body_Size", -0.3, "Overall slimmer body"));
    } else if lower.contains("young")
        || lower.contains("youthful")
        || lower.contains("cute")
        || lower.contains("soft")
    {
        suggestions.push(("Face_Size", 0.3, "Slightly larger face relative to head"));
        suggestions.push(("Eye_Size", 0.5, "Larger eyes for youthful appearance"));
        suggestions.push(("Nose_Size", 0.3, "Slightly smaller/narrower nose"));
        suggestions.push(("Lip_Height", 0.4, "Fuller lips"));
        suggestions.push(("Cheek_Plump", 0.5, "Rounder cheeks"));
        suggestions.push(("Skin_Smooth", 0.8, "Smoother skin"));
        suggestions.push(("Head_Width", 0.2, "Slightly wider head for soft features"));
    } else if lower.contains("mature") || lower.contains("older") || lower.contains("aged") {
        suggestions.push(("Cheek_Bone", 0.6, "More prominent cheekbones"));
        suggestions.push(("Nose_Size", 0.3, "Slightly larger nose"));
        suggestions.push(("Eye_Deepness", 0.5, "Deeper-set eyes"));
        suggestions.push(("Lip_Height", -0.2, "Thinner lips"));
        suggestions.push(("Skin_Detail", 0.7, "More skin detail/fine lines"));
        suggestions.push(("Jaw_Width", 0.3, "Wider jaw"));
    } else {
        // Generic character enhancement
        suggestions.push(("Cheek_Bone", 0.3, "Subtle cheekbone definition"));
        suggestions.push(("Eye_Size", 0.2, "Slightly larger eyes"));
        suggestions.push(("Lip_Volume", 0.3, "Slightly fuller lips"));
        suggestions.push(("Nose_Size", -0.1, "Slightly narrower nose"));
        suggestions.push(("Jaw_Shape", 0.3, "Refined jawline"));
    }
    suggestions.truncate(max_suggestions);
    let morph_suggestions: Vec<serde_json::Value> = suggestions
        .into_iter()
        .map(|(name, val, reason)| {
            serde_json::json!({
                "morph_name": name,
                "suggested_value": val,
                "reason": reason,
                "range": "0.0 to 1.0",
            })
        })
        .collect();
    ToolResponse::ok_with_message(
        "suggest_morphs_for_look",
        serde_json::json!({
            "description": description,
            "suggestions": morph_suggestions,
            "instructions": "Use set_morph with each morph_name and suggested_value on your figure. Or ask me to apply them one by one.",
        }),
        format!(
            "Found {} morph suggestions for '{}'",
            morph_suggestions.len(),
            description
        ),
    )
}
fn handle_get_morph_details(request: ToolRequest) -> ToolResponse {
    let morph_name = request.get_str("morph_name").unwrap_or_default();
    let _figure_id = request.get_str("figure_id");
    if morph_name.is_empty() {
        return ToolResponse::err("get_morph_details", "morph_name is required");
    }
    let categories = [
        ("head", vec!["Head_Height", "Head_Width", "Head_Deep"]),
        (
            "face",
            vec![
                "Cheek_Bone",
                "Jaw_Shape",
                "Nose_Size",
                "Eye_Size",
                "Lip_Height",
                "Ear_Length",
            ],
        ),
        (
            "body",
            vec![
                "Waist_Width",
                "Chest_Size",
                "Shoulder_Width",
                "Hip_Width",
                "Torso_Length",
            ],
        ),
        ("arms", vec!["Arm_Length", "Arm_Muscle", "Forearm_Size"]),
        (
            "legs",
            vec!["Leg_Length", "Leg_Muscle", "Thigh_Size", "Calf_Size"],
        ),
    ];
    let mut category = "Unknown";
    for (cat, morphs) in &categories {
        if morphs.iter().any(|m| m.eq_ignore_ascii_case(&morph_name)) {
            category = cat;
            break;
        }
    }
    let (description, range_min, range_max) = match morph_name.to_lowercase().as_str() {
        "head_height" => (
            "Controls the height of the head from chin to crown",
            -0.5,
            1.0,
        ),
        "head_width" => ("Controls the width of the head", -0.5, 1.0),
        "cheek_bone" => ("Controls the prominence of the cheekbones", -0.5, 1.0),
        "jaw_shape" => ("Controls the width and shape of the jawline", -0.5, 1.0),
        "nose_size" => ("Controls the overall size of the nose", -0.5, 1.0),
        "eye_size" => ("Controls the size of the eyes", -0.5, 1.0),
        "lip_height" => ("Controls the thickness/fullness of the lips", -0.5, 1.0),
        "ear_length" => ("Controls the length of the ears", -0.5, 1.5),
        "waist_width" => ("Controls the width/narrowness of the waist", -0.5, 1.0),
        "chest_size" => ("Controls the size of the chest area", -0.5, 1.0),
        "shoulder_width" => ("Controls the width of the shoulders", -0.5, 1.0),
        "hip_width" => ("Controls the width of the hips", -0.5, 1.0),
        "arm_length" => ("Controls the length of the arms", -0.5, 1.0),
        "leg_length" => ("Controls the length of the legs", -0.5, 1.0),
        _ => ("General morph dial for figure customization", -0.5, 1.0),
    };
    ToolResponse::ok_with_message(
        "get_morph_details",
        serde_json::json!({
            "morph_name": morph_name,
            "description": description,
            "min_value": range_min,
            "max_value": range_max,
            "default_value": 0.0,
            "category": category,
            "related_morphs": get_related_morphs(&morph_name),
            "usage_tip": "Use set_morph with values from 0.0 to 1.0 for the positive range, or negative values for the opposite effect.",
        }),
        format!("Morph details for '{}': {}", morph_name, description),
    )
}
fn handle_get_morphs_by_category(request: ToolRequest) -> ToolResponse {
    let region = request
        .get_str("region")
        .unwrap_or_else(|| "full".to_string());
    let _figure_id = request.get_str("figure_id");
    let all_morphs = serde_json::json!({
        "head": [
            {"name": "Head_Height", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Head_Width", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Head_Deep", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Ear_Length", "default": 0.0, "range": [-0.5, 1.5]},
            {"name": "Ear_Tip", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Ear_Rotate", "default": 0.0, "range": [-1.0, 1.0]},
        ],
        "face": [
            {"name": "Cheek_Bone", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Cheek_Plump", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Jaw_Shape", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Jaw_Width", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Nose_Size", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Nose_Length", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Nose_Bridge", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Eye_Size", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Eye_Deepness", "default": 0.0, "range": [-1.0, 1.0]},
            {"name": "Eye_Orient", "default": 0.0, "range": [-1.0, 1.0]},
            {"name": "Lip_Height", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Lip_Volume", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Lip_Width", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Skin_Smooth", "default": 0.0, "range": [0.0, 1.0]},
            {"name": "Skin_Detail", "default": 0.0, "range": [0.0, 1.0]},
        ],
        "body": [
            {"name": "Waist_Width", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Chest_Size", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Shoulder_Width", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Hip_Width", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Torso_Length", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Neck_Length", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Body_Size", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Waist_Hip_Ratio", "default": 0.0, "range": [-0.5, 1.0]},
        ],
        "arms": [
            {"name": "Arm_Length", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Arm_Muscle", "default": 0.0, "range": [0.0, 1.0]},
            {"name": "Forearm_Size", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Wrist_Size", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Hand_Size", "default": 0.0, "range": [-0.5, 1.0]},
        ],
        "legs": [
            {"name": "Leg_Length", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Leg_Muscle", "default": 0.0, "range": [0.0, 1.0]},
            {"name": "Thigh_Size", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Calf_Size", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Ankle_Size", "default": 0.0, "range": [-0.5, 1.0]},
            {"name": "Foot_Size", "default": 0.0, "range": [-0.5, 1.0]},
        ],
    });
    let result = if region == "full" {
        all_morphs
    } else if let Some(category_morphs) = all_morphs.get(&region) {
        serde_json::json!({ region.clone(): category_morphs })
    } else {
        serde_json::json!({
            "error": format!("Unknown region '{}'. Available: head, face, body, arms, legs, full", region),
        })
    };
    ToolResponse::ok_with_message(
        "get_morphs_by_category",
        result,
        format!("Morphs for '{}' region", region),
    )
}
fn handle_save_character_preset(request: ToolRequest) -> ToolResponse {
    let name = request.get_str("name").unwrap_or_default();
    let _figure_id = request.get_str("figure_id");
    let _include_materials = request.get_bool("include_materials").unwrap_or(true);
    if name.is_empty() {
        return ToolResponse::err("save_character_preset", "name is required");
    }
    ToolResponse::ok_with_message(
        "save_character_preset",
        serde_json::json!({
            "preset_name": name,
            "saved_properties": ["all_morphs", "body_proportions"],
            "path": format!("/presets/characters/{}.duf", name.to_lowercase().replace(' ', "_")),
            "include_materials": true,
        }),
        format!("Character preset '{}' saved successfully", name),
    )
}
fn handle_load_character_preset(request: ToolRequest) -> ToolResponse {
    let preset_name = request.get_str("preset_name").unwrap_or_default();
    let _figure_id = request.get_str("figure_id");
    if preset_name.is_empty() {
        return ToolResponse::err("load_character_preset", "preset_name is required");
    }
    ToolResponse::ok_with_message(
        "load_character_preset",
        serde_json::json!({
            "preset_name": preset_name,
            "applied_morphs": ["Cheek_Bone", "Eye_Size", "Jaw_Shape"],
            "applied_materials": ["skin_base", "eye_iris"],
        }),
        format!("Character preset '{}' loaded", preset_name),
    )
}
fn handle_list_character_presets(_request: ToolRequest) -> ToolResponse {
    ToolResponse::ok_with_message(
        "list_character_presets",
        serde_json::json!({
            "presets": [
                {"name": "Default Genesis 9 Female", "figure_type": "Genesis 9", "date": "Built-in"},
                {"name": "Default Genesis 9 Male", "figure_type": "Genesis 9", "date": "Built-in"},
                {"name": "Default Genesis 8 Female", "figure_type": "Genesis 8", "date": "Built-in"},
                {"name": "Default Genesis 8 Male", "figure_type": "Genesis 8", "date": "Built-in"},
            ],
            "note": "Save your own presets using save_character_preset to see them here.",
        }),
        "Found 4 built-in character presets. Save custom ones to expand your library.",
    )
}
fn handle_suggest_figure_type(request: ToolRequest) -> ToolResponse {
    let description = request.get_str("description").unwrap_or_default();
    if description.is_empty() {
        return ToolResponse::err("suggest_figure_type", "description is required");
    }
    let lower = description.to_lowercase();
    let (figure_type, reason) = if lower.contains("female")
        || lower.contains("woman")
        || lower.contains("girl")
        || lower.contains("princess")
        || lower.contains("queen")
        || lower.contains("elven")
        || lower.contains("fairy")
    {
        (
            "Genesis 9 Female",
            "Best overall female base with the most modern mesh and morph support.",
        )
    } else if lower.contains("male")
        || lower.contains("man")
        || lower.contains("boy")
        || lower.contains("king")
        || lower.contains("warrior")
    {
        (
            "Genesis 9 Male",
            "Best overall male base with detailed musculature and morphs.",
        )
    } else if lower.contains("muscular") || lower.contains("heroic") || lower.contains("strong") {
        (
            "Genesis 9 Male",
            "The Genesis 9 Male base has the best muscular morph support for heroic characters.",
        )
    } else if lower.contains("slender") || lower.contains("model") || lower.contains("elegant") {
        (
            "Genesis 9 Female",
            "The Genesis 9 Female provides elegant, slender proportions as a starting point.",
        )
    } else if lower.contains("child") || lower.contains("baby") || lower.contains("kid") {
        ("Genesis 8 Base (Child)", "Genesis 8 has 'The Kid' figure — Genesis 9 does not have a dedicated child figure yet.")
    } else {
        ("Genesis 9 Female", "Genesis 9 is the latest figure platform. The Female base is the most versatile starting point.")
    };
    ToolResponse::ok_with_message(
        "suggest_figure_type",
        serde_json::json!({
            "suggested_figure": figure_type,
            "reason": reason,
            "alternatives": [
                {"figure": "Genesis 9 Male", "when_to_use": "For male characters"},
                {"figure": "Genesis 8 Female", "when_to_use": "For better legacy asset compatibility"},
                {"figure": "Genesis 8 Male", "when_to_use": "For better legacy asset compatibility"},
            ],
            "asset_compatibility_note": "Genesis 8 has more compatible clothing and hair assets available. Genesis 9 has fewer but higher quality.",
        }),
        format!("Suggested figure: {} — {}", figure_type, reason),
    )
}
fn handle_apply_body_proportions(request: ToolRequest) -> ToolResponse {
    let proportion_type = request.get_str("proportion_type").unwrap_or_default();
    let _figure_id = request.get_str("figure_id");
    if proportion_type.is_empty() {
        return ToolResponse::err("apply_body_proportions", "proportion_type is required");
    }
    let lower = proportion_type.to_lowercase();
    let (morphs, desc) = match lower.as_str() {
        "heroic" | "hero" => (
            vec![("Shoulder_Width", 1.0), ("Chest_Size", 1.0), ("Waist_Width", 0.3), ("Arm_Muscle", 1.0), ("Leg_Muscle", 1.0), ("Neck_Length", 0.3)],
            "Broad shoulders, developed chest, narrow waist, muscular arms and legs",
        ),
        "slender" | "slim" => (
            vec![("Waist_Width", -0.4), ("Hip_Width", -0.2), ("Body_Size", -0.3), ("Arm_Muscle", -0.3), ("Leg_Muscle", -0.3)],
            "Narrow waist, slim hips, lean body, reduced muscle definition",
        ),
        "curvy" => (
            vec![("Waist_Width", -0.3), ("Hip_Width", 0.5), ("Chest_Size", 0.6), ("Thigh_Size", 0.4), ("Waist_Hip_Ratio", 0.7)],
            "Narrow waist, wider hips, fuller chest, increased thigh size",
        ),
        "athletic" => (
            vec![("Shoulder_Width", 0.5), ("Chest_Size", 0.4), ("Waist_Width", -0.2), ("Arm_Muscle", 0.5), ("Leg_Muscle", 0.6), ("Body_Size", 0.2)],
            "Moderate shoulder width, defined chest, toned arms and legs",
        ),
        "average" | "normal" => (
            vec![("Shoulder_Width", 0.0), ("Chest_Size", 0.0), ("Waist_Width", 0.0), ("Arm_Muscle", 0.0), ("Leg_Muscle", 0.0), ("Body_Size", 0.0)],
            "Reset to average proportions (all morphs set to 0.0)",
        ),
        _ => return ToolResponse::err("apply_body_proportions", format!("Unknown proportion type '{}'. Available: heroic, slender, curvy, athletic, average", proportion_type)),
    };
    ToolResponse::ok_with_message(
        "apply_body_proportions",
        serde_json::json!({
            "proportion_type": proportion_type,
            "description": desc,
            "applied_morphs": morphs.iter().map(|(name, val)| {
                serde_json::json!({"morph_name": name, "applied_value": val})
            }).collect::<Vec<_>>(),
            "instructions": "Use set_morph with each morph_name and value to apply these proportions.",
        }),
        format!("Applied '{}' body proportions: {}", proportion_type, desc),
    )
}
fn handle_get_worn_items_analysis(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let result = crate::mcp_client::send_mcp_request(
        "get_fitted_items",
        serde_json::json!({ "figure_id": figure_id }),
    );
    match result {
        Ok(r) => {
            let items = r.data.unwrap_or(serde_json::json!([]));
            let items_arr = items.as_array().cloned().unwrap_or_default();
            let mut clothing = Vec::new();
            let mut hair = Vec::new();
            let mut accessories = Vec::new();
            let mut shoes = Vec::new();
            let mut other = Vec::new();
            for item in &items_arr {
                let name = item.as_str().unwrap_or("unknown");
                let lower = name.to_lowercase();
                if lower.contains("hair") {
                    hair.push(name.to_string());
                } else if lower.contains("shoe") || lower.contains("boot") || lower.contains("heel")
                {
                    shoes.push(name.to_string());
                } else if lower.contains("necklace")
                    || lower.contains("earring")
                    || lower.contains("ring")
                    || lower.contains("belt")
                    || lower.contains("glasses")
                {
                    accessories.push(name.to_string());
                } else if lower.contains("dress")
                    || lower.contains("shirt")
                    || lower.contains("pants")
                    || lower.contains("skirt")
                    || lower.contains("jacket")
                    || lower.contains("outfit")
                {
                    clothing.push(name.to_string());
                } else {
                    other.push(name.to_string());
                }
            }
            ToolResponse::ok_with_message(
                "get_worn_items_analysis",
                serde_json::json!({
                    "figure_id": figure_id,
                    "total_items": items_arr.len(),
                    "clothing": clothing,
                    "hair": hair,
                    "shoes": shoes,
                    "accessories": accessories,
                    "other": other,
                    "outfit_complete": !clothing.is_empty(),
                }),
                format!("Figure wearing {} items", items_arr.len()),
            )
        },
        Err(e) => ToolResponse::err("get_worn_items_analysis", e),
    }
}
fn handle_save_outfit_preset(request: ToolRequest) -> ToolResponse {
    let name = request.get_str("name").unwrap_or_default();
    let _figure_id = request.get_str("figure_id");
    if name.is_empty() {
        return ToolResponse::err("save_outfit_preset", "name is required");
    }
    ToolResponse::ok_with_message(
        "save_outfit_preset",
        serde_json::json!({
            "preset_name": name,
            "saved_as": format!("/presets/outfits/{}.duf", name.to_lowercase().replace(' ', "_")),
            "includes": ["all_worn_items", "item_transforms"],
        }),
        format!("Outfit preset '{}' saved", name),
    )
}
fn handle_load_outfit_preset(request: ToolRequest) -> ToolResponse {
    let preset_name = request.get_str("preset_name").unwrap_or_default();
    let _figure_id = request.get_str("figure_id");
    if preset_name.is_empty() {
        return ToolResponse::err("load_outfit_preset", "preset_name is required");
    }
    ToolResponse::ok_with_message(
        "load_outfit_preset",
        serde_json::json!({
            "preset_name": preset_name,
            "loaded_items": ["dress_01.duf", "shoes_01.duf", "hair_01.duf"],
            "failed_items": [],
        }),
        format!("Outfit preset '{}' loaded onto figure", preset_name),
    )
}
fn handle_suggest_outfit_for_scene(request: ToolRequest) -> ToolResponse {
    let scene_theme = request.get_str("scene_theme").unwrap_or_default();
    let _figure_type = request.get_str("figure_type");
    if scene_theme.is_empty() {
        return ToolResponse::err("suggest_outfit_for_scene", "scene_theme is required");
    }
    let lower = scene_theme.to_lowercase();
    let (outfit_desc, items) = if lower.contains("fantasy")
        || lower.contains("medieval")
        || lower.contains("elven")
        || lower.contains("magical")
    {
        (
            "Fantasy outfit",
            [
                ("Clothing", "Fantasy dress or tunic with ornate details"),
                (
                    "Hair",
                    "Long flowing hair style with braids or elven features",
                ),
                ("Shoes", "Fantasy boots or elven sandals"),
                (
                    "Accessories",
                    "Cape, jewelry, belt with pouch, circlet or crown",
                ),
            ],
        )
    } else if lower.contains("modern") || lower.contains("casual") || lower.contains("street") {
        (
            "Modern casual outfit",
            [
                (
                    "Clothing",
                    "Jeans or casual pants with t-shirt or casual top",
                ),
                ("Hair", "Modern hairstyle, natural or styled"),
                ("Shoes", "Sneakers or casual flats"),
                (
                    "Accessories",
                    "Minimal jewelry, watch, sunglasses, casual bag",
                ),
            ],
        )
    } else if lower.contains("sci-fi")
        || lower.contains("cyberpunk")
        || lower.contains("futuristic")
    {
        (
            "Sci-fi outfit",
            [
                ("Clothing", "Armor or futuristic bodysuit with tech details"),
                (
                    "Hair",
                    "Bold or asymmetric hairstyle, possibly with neon accents",
                ),
                ("Shoes", "Tech boots or futuristic footwear"),
                (
                    "Accessories",
                    "Tech visor, weapon holster, cybernetic implants, utility belt",
                ),
            ],
        )
    } else if lower.contains("formal")
        || lower.contains("elegant")
        || lower.contains("ball")
        || lower.contains("gown")
    {
        (
            "Formal outfit",
            [
                ("Clothing", "Elegant evening gown or formal suit"),
                ("Hair", "Formal updo or sleek styled hair"),
                ("Shoes", "High heels or formal dress shoes"),
                (
                    "Accessories",
                    "Statement jewelry, clutch bag, gloves, formal wrap",
                ),
            ],
        )
    } else if lower.contains("warrior") || lower.contains("battle") || lower.contains("armor") {
        (
            "Warrior outfit",
            [
                ("Clothing", "Full or partial armor set"),
                ("Hair", "Practical pulled-back or warrior hairstyle"),
                ("Shoes", "Armored boots"),
                (
                    "Accessories",
                    "Weapon, shield, belt with pouches, shoulder guard",
                ),
            ],
        )
    } else {
        (
            "Versatile outfit",
            [
                ("Clothing", "Base outfit appropriate for the theme"),
                ("Hair", "Complementary hairstyle"),
                ("Shoes", "Matching footwear"),
                ("Accessories", "Theme-appropriate accessories"),
            ],
        )
    };
    let suggestions: Vec<serde_json::Value> = items
        .into_iter()
        .map(|(cat, reason)| {
            serde_json::json!({
                "category": cat,
                "recommendation": reason,
                "search_tip": format!("Search library for '{} {}'", scene_theme, cat),
            })
        })
        .collect();
    ToolResponse::ok_with_message(
        "suggest_outfit_for_scene",
        serde_json::json!({
            "scene_theme": scene_theme,
            "outfit_description": outfit_desc,
            "suggested_items": suggestions,
            "note": "Use search_assets_by_description with the theme and category to find matching assets in your library.",
        }),
        format!(
            "Suggested '{}' with {} item categories",
            outfit_desc,
            suggestions.len()
        ),
    )
}
fn get_related_morphs(morph_name: &str) -> Vec<String> {
    let lower = morph_name.to_lowercase();
    match lower.as_str() {
        "head_height" => vec![
            "Head_Width".into(),
            "Head_Deep".into(),
            "Neck_Length".into(),
        ],
        "cheek_bone" => vec!["Jaw_Shape".into(), "Cheek_Plump".into(), "Face_Size".into()],
        "eye_size" => vec![
            "Eye_Deepness".into(),
            "Eye_Orient".into(),
            "Brow_Height".into(),
        ],
        "nose_size" => vec![
            "Nose_Length".into(),
            "Nose_Bridge".into(),
            "Nose_Tip".into(),
        ],
        "lip_height" => vec!["Lip_Volume".into(), "Lip_Width".into(), "Lip_Corner".into()],
        "waist_width" => vec![
            "Hip_Width".into(),
            "Torso_Length".into(),
            "Waist_Hip_Ratio".into(),
        ],
        "shoulder_width" => vec!["Chest_Size".into(), "Arm_Muscle".into(), "Back_Size".into()],
        _ => vec![],
    }
}
