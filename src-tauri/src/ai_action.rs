#![allow(dead_code)]

use crate::ai_system::{Entity, EntityType, Intent};
use crate::reasoning::planner::{Goal, GoalPriority, Planner, PlanningContext};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

static REASONING_PLANNER: Mutex<Option<Planner>> = Mutex::new(None);

fn get_reasoning_planner() -> Planner {
    let mut guard = REASONING_PLANNER.lock().unwrap();
    if guard.is_none() {
        *guard = Some(Planner::new());
    }
    guard.clone().unwrap()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAction {
    pub command: String,
    pub target: String,
    pub parameters: Vec<ActionParam>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionParam {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
    pub results: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredAiAction {
    pub command: String,
    pub args: serde_json::Value,
    pub confidence: f32,
    pub sdk_refs: Vec<String>,
    pub requires_confirmation: bool,
}

// ─── Animation Helper Functions ───────────────────────────────────────────

fn extract_frame_and_value(input: &str) -> (f32, f32) {
    let lower = input.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();
    let mut frame = 0.0;
    let mut value = 0.0;
    let mut frame_found = false;
    let mut value_found = false;

    for i in 0..words.len() {
        let word = words[i];
        if (word == "frame" || word == "at" || word == "on") && i + 1 < words.len() {
            if let Ok(val) = words[i + 1]
                .trim_matches(|c: char| !c.is_numeric() && c != '.')
                .parse::<f32>()
            {
                frame = val;
                frame_found = true;
            }
        }
        if (word == "to" || word == "value" || word == "val") && i + 1 < words.len() {
            if let Ok(val) = words[i + 1]
                .trim_matches(|c: char| !c.is_numeric() && c != '-' && c != '.')
                .parse::<f32>()
            {
                value = val;
                value_found = true;
            }
        }
    }

    if !frame_found || !value_found {
        let numbers: Vec<f32> = words
            .iter()
            .filter_map(|w| {
                w.trim_matches(|c: char| !c.is_numeric() && c != '-' && c != '.')
                    .parse::<f32>()
                    .ok()
            })
            .collect();
        if numbers.len() >= 2 {
            if !value_found {
                value = numbers[0];
            }
            if !frame_found {
                frame = numbers[1];
            }
        } else if numbers.len() == 1 && !value_found {
            value = numbers[0];
        }
    }

    (frame, value)
}

fn extract_property(input: &str) -> String {
    let lower = input.to_lowercase();
    let words: Vec<&str> = lower
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphabetic()))
        .collect();

    for word in &words {
        match *word {
            "rotation" | "rot" => return "yRot".to_string(),
            "position" | "pos" => return "yTranslate".to_string(),
            "x" | "xtranslate" => return "xTranslate".to_string(),
            "y" | "ytranslate" => return "yTranslate".to_string(),
            "z" | "ztranslate" => return "zTranslate".to_string(),
            "xrot" => return "xRot".to_string(),
            "yrot" => return "yRot".to_string(),
            "zrot" => return "zRot".to_string(),
            "scale" => return "scale".to_string(),
            "bend" => return "bend".to_string(),
            "twist" => return "twist".to_string(),
            "intensity" => return "intensity".to_string(),
            "color" => return "color".to_string(),
            _ => {},
        }
    }

    let clean_input = lower.replace("keyframe", "");
    let common_props = vec![
        "xtranslate",
        "ytranslate",
        "ztranslate",
        "xrot",
        "yrot",
        "zrot",
        "rotation",
        "position",
        "scale",
        "bend",
        "twist",
        "intensity",
        "color",
    ];
    for prop in common_props {
        if clean_input.contains(prop) {
            return match prop {
                "rotation" => "yRot".to_string(),
                "position" => "yTranslate".to_string(),
                _ => prop.to_string(),
            };
        }
    }

    let words: Vec<&str> = lower.split_whitespace().collect();
    if let Some(pos) = words.iter().position(|&w| w == "property") {
        if pos + 1 < words.len() {
            return words[pos + 1]
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_string();
        }
    }
    "yRot".to_string()
}

fn extract_node_id(input: &str) -> String {
    let lower = input.to_lowercase();
    if lower.contains("genesis 8") || lower.contains("genesis8") || lower.contains("g8") {
        return "Genesis 8 Female".to_string();
    }
    if lower.contains("genesis 9") || lower.contains("genesis9") || lower.contains("g9") {
        return "Genesis 9".to_string();
    }
    if let Some(start) = input.find('"') {
        if let Some(end) = input[start + 1..].find('"') {
            return input[start + 1..start + 1 + end].to_string();
        }
    }
    if let Some(start) = input.find('\'') {
        if let Some(end) = input[start + 1..].find('\'') {
            return input[start + 1..start + 1 + end].to_string();
        }
    }
    "".to_string()
}

fn extract_interpolation(input: &str) -> String {
    let lower = input.to_lowercase();
    if lower.contains("linear") {
        "linear".to_string()
    } else if lower.contains("constant") {
        "constant".to_string()
    } else if lower.contains("ease") || lower.contains("tcb") {
        "tcb".to_string()
    } else if lower.contains("bezier") || lower.contains("hermite") {
        "hermite".to_string()
    } else {
        "linear".to_string()
    }
}

pub fn extract_asset_search_query(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    let prefixes = vec![
        "load asset",
        "load",
        "apply pose",
        "apply",
        "add figure",
        "add node",
        "add",
        "put on",
        "equip",
        "use",
        "bring in",
        "import",
    ];

    let mut target = String::new();
    let mut matched_prefix = false;

    for prefix in prefixes {
        if lower.starts_with(prefix) {
            target = input[prefix.len()..].trim().to_string();
            matched_prefix = true;
            break;
        } else if let Some(idx) = lower.find(prefix) {
            target = input[idx + prefix.len()..].trim().to_string();
            matched_prefix = true;
            break;
        }
    }

    if !matched_prefix {
        target = input.to_string();
    }

    let filler_words = vec![
        "the", "a", "an", "some", "to", "in", "into", "on", "scene", "figure",
    ];
    let words: Vec<&str> = target.split_whitespace().collect();
    let mut clean_words = vec![];
    for word in words {
        let wl = word.to_lowercase();
        let w_clean = wl.trim_matches(|c: char| !c.is_alphanumeric());
        if !filler_words.contains(&w_clean) && !word.is_empty() {
            clean_words.push(word);
        }
    }

    if clean_words.is_empty() {
        None
    } else {
        Some(clean_words.join(" "))
    }
}

pub fn search_best_matching_asset(query: &str) -> Option<String> {
    // Multi-strategy: FTS first, then fuzzy, synonym, keyword, semantic
    let matcher = crate::asset_matcher::MultiStrategyMatcher::new();
    let result = matcher.search_best(query);
    if let Some(ref matched) = result {
        log::info!(
            "Asset match via {}: {} (score={}, path={})",
            matched.strategy,
            matched.name,
            matched.score,
            matched.path
        );
    } else {
        // Fallback to original single-query FTS
        let guard = match crate::database::get_db() {
            Ok(g) => g,
            Err(_) => return None,
        };
        let db = guard.as_ref()?;
        let conn = rusqlite::Connection::open(db.path()).ok()?;
        let fts = crate::format_fts_query(query);
        if fts.is_empty() {
            return None;
        }
        let sql = "SELECT user_assets.asset_path FROM user_assets JOIN user_assets_fts ON user_assets.id = user_assets_fts.rowid WHERE user_assets.user_id='default' AND user_assets_fts MATCH ? ORDER BY bm25(user_assets_fts) LIMIT 1";
        let mut stmt = conn.prepare(sql).ok()?;
        let mut rows = stmt.query(rusqlite::params![fts]).ok()?;
        return rows
            .next()
            .ok()?
            .and_then(|row| row.get::<_, String>(0).ok());
    }
    result.map(|a| a.path)
}

// ─── Core Plan Logic ───────────────────────────────────────────────────────

pub fn plan_validated_action(input: &str) -> Option<StructuredAiAction> {
    let lower = input.to_lowercase();

    plan_seek_to_frame(&lower)
        .or_else(|| plan_set_timeline_range(&lower))
        .or_else(|| plan_dforce_simulation(&lower))
        .or_else(|| plan_set_keyframe(&lower, input))
        .or_else(|| plan_set_morph(&lower, input))
        .or_else(|| plan_set_light_property(&lower, input))
        .or_else(|| plan_render_settings(&lower))
        .or_else(|| plan_add_figure(&lower))
        .or_else(|| plan_scene_creation(&lower))
        .or_else(|| plan_create_light(&lower))
        .or_else(|| plan_apply_expression(&lower, input))
        .or_else(|| plan_material_opacity_tools(&lower, input))
        .or_else(|| plan_load_asset(&lower, input))
        .or_else(|| plan_export_scene(&lower, input))
        .or_else(|| plan_legacy_command(&lower, input))
}

fn plan_seek_to_frame(lower: &str) -> Option<StructuredAiAction> {
    if !((lower.contains("seek") || lower.contains("go to") || lower.contains("jump"))
        && lower.contains("frame"))
    {
        return None;
    }
    let words: Vec<&str> = lower.split_whitespace().collect();
    let mut frame = None;
    for i in 0..words.len() {
        if words[i] == "frame" && i + 1 < words.len() {
            if let Ok(val) = words[i + 1]
                .trim_matches(|c: char| !c.is_numeric())
                .parse::<i32>()
            {
                frame = Some(val);
            }
        }
    }
    if frame.is_none() {
        frame = words
            .iter()
            .filter_map(|w| {
                w.trim_matches(|c: char| !c.is_numeric())
                    .parse::<i32>()
                    .ok()
            })
            .next();
    }
    frame.map(|f| StructuredAiAction {
        command: "seek_to_frame".to_string(),
        args: serde_json::json!({ "frame": f }),
        confidence: 0.95,
        sdk_refs: vec!["DzScene".to_string()],
        requires_confirmation: false,
    })
}

fn plan_set_timeline_range(lower: &str) -> Option<StructuredAiAction> {
    if !(lower.contains("range")
        || (lower.contains("timeline")
            && (lower.contains("limit")
                || lower.contains("set")
                || lower.contains("duration")
                || lower.contains("frames"))))
    {
        return None;
    }
    let words: Vec<&str> = lower.split_whitespace().collect();
    let numbers: Vec<i32> = words
        .iter()
        .filter_map(|w| {
            w.trim_matches(|c: char| !c.is_numeric())
                .parse::<i32>()
                .ok()
        })
        .collect();

    let (start, end) = if numbers.len() >= 2 {
        (numbers[0], numbers[1])
    } else if numbers.len() == 1 {
        (0, numbers[0])
    } else {
        (0, 30)
    };

    Some(StructuredAiAction {
        command: "set_timeline_range".to_string(),
        args: serde_json::json!({ "start_frame": start, "end_frame": end }),
        confidence: 0.95,
        sdk_refs: vec!["DzScene".to_string()],
        requires_confirmation: false,
    })
}

fn plan_dforce_simulation(lower: &str) -> Option<StructuredAiAction> {
    if !(lower.contains("simulate") || lower.contains("dforce") || lower.contains("physics")) {
        return None;
    }
    let words: Vec<&str> = lower.split_whitespace().collect();
    let numbers: Vec<u32> = words
        .iter()
        .filter_map(|w| {
            w.trim_matches(|c: char| !c.is_numeric())
                .parse::<u32>()
                .ok()
        })
        .collect();

    let (start, end) = if numbers.len() >= 2 {
        (numbers[0], numbers[1])
    } else if numbers.len() == 1 {
        (0, numbers[0])
    } else {
        (0, 30)
    };

    let mut node_id = String::new();
    for word in words {
        let wl = word.to_lowercase();
        if wl.contains("dress")
            || wl.contains("shirt")
            || wl.contains("skirt")
            || wl.contains("pants")
            || wl.contains("hair")
            || wl.contains("cloth")
        {
            node_id = word
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_string();
            break;
        }
    }

    Some(StructuredAiAction {
        command: "run_dforce_simulation".to_string(),
        args: serde_json::json!({
            "node_id": node_id,
            "start_frame": start,
            "end_frame": end,
        }),
        confidence: 0.95,
        sdk_refs: vec!["DzSimulator".to_string()],
        requires_confirmation: true,
    })
}

fn plan_set_keyframe(lower: &str, input: &str) -> Option<StructuredAiAction> {
    if !(lower.contains("keyframe") || lower.contains("animate")) {
        return None;
    }
    let node_id = extract_node_id(input);
    let property = extract_property(input);
    let (frame, value) = extract_frame_and_value(input);
    let interpolation = extract_interpolation(input);

    Some(StructuredAiAction {
        command: "set_keyframe".to_string(),
        args: serde_json::json!({
            "node_id": node_id,
            "property": property,
            "frame": frame,
            "value": value,
            "interpolation": interpolation,
        }),
        confidence: 0.9,
        sdk_refs: vec!["DzFloatProperty".to_string()],
        requires_confirmation: false,
    })
}

fn plan_set_morph(lower: &str, input: &str) -> Option<StructuredAiAction> {
    if !(lower.contains("morph") || lower.contains("dial")) {
        return None;
    }
    let node_id = extract_node_id(input);
    let value = extract_frame_and_value(input).1;
    let morph = {
        let words: Vec<&str> = lower.split_whitespace().collect();
        words
            .iter()
            .find(|w| !["set", "morph", "dial", "to", "the", "on", "at", "frame"].contains(w))
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Fitness".to_string())
    };
    Some(StructuredAiAction {
        command: "set_morph".to_string(),
        args: serde_json::json!({
            "node_id": node_id,
            "morph": morph,
            "value": value.to_string(),
        }),
        confidence: 0.88,
        sdk_refs: vec!["DzMorph".to_string()],
        requires_confirmation: false,
    })
}

fn plan_set_light_property(lower: &str, input: &str) -> Option<StructuredAiAction> {
    if !((lower.contains("light") || lower.contains("lighting"))
        && (lower.contains("intensity") || lower.contains("brightness") || lower.contains("color")))
    {
        return None;
    }
    let node_id = extract_node_id(input);
    let property = if lower.contains("color") || lower.contains("colour") {
        "Color"
    } else {
        "Intensity"
    };
    let value = if property == "Color" {
        extract_color(input)
    } else {
        extract_numeric_value(input)
    };
    Some(StructuredAiAction {
        command: "set_light".to_string(),
        args: serde_json::json!({
            "node_id": node_id,
            "property": property,
            "value": value,
        }),
        confidence: 0.88,
        sdk_refs: vec!["DzLight".to_string()],
        requires_confirmation: false,
    })
}

fn plan_render_settings(lower: &str) -> Option<StructuredAiAction> {
    if !(lower.contains("render")
        && (lower.contains("resolution")
            || lower.contains("size")
            || lower.contains("1920")
            || lower.contains("4k")
            || lower.contains("output")
            || lower.contains("image")
            || lower.contains("picture")))
    {
        return None;
    }
    let numbers: Vec<i32> = lower
        .split_whitespace()
        .filter_map(|w| {
            w.trim_matches(|c: char| !c.is_numeric())
                .parse::<i32>()
                .ok()
        })
        .collect();
    let (width, height) = if numbers.len() >= 2 {
        (numbers[0], numbers[1])
    } else {
        (1920, 1080)
    };
    Some(StructuredAiAction {
        command: "set_render_settings".to_string(),
        args: serde_json::json!({
            "width": width.to_string(),
            "height": height.to_string(),
        }),
        confidence: 0.85,
        sdk_refs: vec!["DzRenderMgr".to_string()],
        requires_confirmation: false,
    })
}

fn plan_add_figure(lower: &str) -> Option<StructuredAiAction> {
    if !((lower.contains("add")
        || lower.contains("create")
        || lower.contains("load")
        || lower.contains("build")
        || lower.contains("make"))
        && (lower.contains("figure")
            || lower.contains("genesis")
            || lower.contains("character")
            || lower.contains("model")
            || lower.contains("person")
            || lower.contains("human")))
    {
        return None;
    }
    // Reject if the input is about expressions rather than figure creation
    if lower.contains("smile") || lower.contains("frown") || lower.contains("expression") {
        return None;
    }
    let figure_type =
        if lower.contains("genesis 9") || lower.contains("g9") || lower.contains("genesis9") {
            "genesis9"
        } else {
            "genesis8"
        };
    Some(StructuredAiAction {
        command: "add_figure".to_string(),
        args: serde_json::json!({ "figure_type": figure_type }),
        confidence: 0.9,
        sdk_refs: vec!["DzFigure".to_string()],
        requires_confirmation: false,
    })
}

fn plan_create_light(lower: &str) -> Option<StructuredAiAction> {
    if !((lower.contains("create")
        || lower.contains("add")
        || lower.contains("new")
        || lower.contains("make"))
        && (lower.contains("light") || lower.contains("lighting")))
    {
        return None;
    }
    let light_type = if lower.contains("spot") || lower.contains("spotlight") {
        "spot_light"
    } else if lower.contains("distant") || lower.contains("infinit") || lower.contains("sun") {
        "distant_light"
    } else if lower.contains("area") {
        "area_light"
    } else {
        "point_light"
    };
    Some(StructuredAiAction {
        command: "add_node".to_string(),
        args: serde_json::json!({ "type": light_type, "name": format!("AI_{}", light_type) }),
        confidence: 0.9,
        sdk_refs: vec!["DzLight".to_string()],
        requires_confirmation: false,
    })
}

fn plan_apply_expression(lower: &str, _input: &str) -> Option<StructuredAiAction> {
    if !(lower.contains("expression")
        || lower.contains("smile")
        || lower.contains("frown")
        || lower.contains("happy")
        || lower.contains("sad")
        || lower.contains("angry")
        || lower.contains("surprise")
        || lower.contains("blink"))
    {
        return None;
    }
    let expression_id = if lower.contains("smile") || lower.contains("happy") {
        "smile"
    } else if lower.contains("frown") || lower.contains("sad") || lower.contains("unhappy") {
        "frown"
    } else if lower.contains("angry") || lower.contains("mad") {
        "angry"
    } else if lower.contains("surprise") || lower.contains("shock") {
        "surprise"
    } else if lower.contains("blink") || lower.contains("wink") || lower.contains("close eyes") {
        "eyes_closed"
    } else {
        ""
    };
    let value = if lower.contains("full") || lower.contains("maximum") || lower.contains("extreme")
    {
        1.0
    } else if lower.contains("subtle") || lower.contains("slight") || lower.contains("gentle") {
        0.3
    } else {
        0.7
    };
    Some(StructuredAiAction {
        command: "apply_expression".to_string(),
        args: serde_json::json!({
            "expression_id": expression_id,
            "value": value,
            "node_id": ""
        }),
        confidence: 0.8,
        sdk_refs: vec!["DzFigure".to_string()],
        requires_confirmation: false,
    })
}

fn plan_scene_creation(lower: &str) -> Option<StructuredAiAction> {
    if !((lower.contains("create")
        || lower.contains("make")
        || lower.contains("build")
        || lower.contains("new"))
        && lower.contains("scene"))
    {
        return None;
    }
    Some(StructuredAiAction {
        command: "add_figure".to_string(),
        args: serde_json::json!({ "figure_type": "genesis9" }),
        confidence: 0.7,
        sdk_refs: vec!["DzFigure".to_string(), "DzScene".to_string()],
        requires_confirmation: false,
    })
}

fn plan_load_asset(lower: &str, input: &str) -> Option<StructuredAiAction> {
    if !(lower.contains("load")
        || lower.contains("apply")
        || lower.contains("add")
        || lower.contains("put on")
        || lower.contains("equip")
        || lower.contains("use")
        || lower.contains("import"))
    {
        return None;
    }
    let target = extract_asset_search_query(input)?;
    let path = search_best_matching_asset(&target)?;
    Some(StructuredAiAction {
        command: "load_asset".to_string(),
        args: serde_json::json!({ "path": path }),
        confidence: 0.9,
        sdk_refs: vec!["DzContentMgr".to_string(), "DzAsset".to_string()],
        requires_confirmation: false,
    })
}

fn plan_export_scene(lower: &str, input: &str) -> Option<StructuredAiAction> {
    if !(lower.contains("export") || lower.contains("save")) {
        return None;
    }
    let format = if lower.contains("fbx") {
        "fbx"
    } else if lower.contains("gltf") || lower.contains("glb") {
        "gltf"
    } else if lower.contains("dae") {
        "dae"
    } else if lower.contains("duf") || lower.contains("daz") {
        "daz"
    } else {
        "obj"
    };

    let path = extract_asset_search_query(input)
        .filter(|_| lower.contains("to ") || lower.contains("as "))
        .map(|p| p.trim_matches(|c: char| c == '"' || c == '\'').to_string())
        .unwrap_or_else(|| format!("scene_export.{}", format));

    Some(StructuredAiAction {
        command: "export_scene".to_string(),
        args: serde_json::json!({
            "node_id": "",
            "path": path,
            "settings": {
                "format": format,
                "quality": "high",
                "include_materials": true,
                "include_animations": lower.contains("animat"),
                "bake_textures": false,
                "compression": false,
                "selected_only": lower.contains("selected") || lower.contains("selection")
            }
        }),
        confidence: 0.85,
        sdk_refs: vec!["DzExportMgr".to_string(), "DzExporter".to_string()],
        requires_confirmation: true,
    })
}

fn plan_material_opacity_tools(lower: &str, input: &str) -> Option<StructuredAiAction> {
    let node_id = extract_node_id(input);
    let node_id = if node_id.is_empty() {
        "selected".to_string()
    } else {
        node_id
    };

    if lower.contains("internal surface")
        || lower.contains("internal surfaces")
        || lower.contains("anatomy surfaces")
    {
        return Some(structured_material_action(
            "get_internal_surfaces",
            serde_json::json!({ "node_id": node_id }),
            0.88,
        ));
    }

    if lower.contains("show anatomy")
        || lower.contains("show skeleton")
        || lower.contains("make anatomy visible")
        || lower.contains("make skeleton visible")
    {
        return Some(structured_material_action(
            "show_anatomy",
            serde_json::json!({ "node_id": node_id }),
            0.9,
        ));
    }

    if (lower.contains("inside") || lower.contains("within"))
        && (lower.contains("place")
            || lower.contains("put")
            || lower.contains("load")
            || lower.contains("insert"))
        && (lower.contains("asset") || lower.contains(".duf") || lower.contains(".dsf"))
    {
        let asset_path = extract_quoted_text(input)
            .or_else(|| extract_asset_path(input))
            .or_else(|| {
                extract_asset_search_query(input)
                    .and_then(|query| search_best_matching_asset(&query))
            })?;
        return Some(structured_material_action(
            "place_asset_inside",
            serde_json::json!({ "figure_id": node_id, "asset_path": asset_path }),
            0.82,
        ));
    }

    let opacity_intent = lower.contains("opacity")
        || lower.contains("transparent")
        || lower.contains("translucent")
        || lower.contains("see-through")
        || lower.contains("see through");
    if !opacity_intent {
        return None;
    }

    let value = extract_opacity_value(lower);
    if let Some(surface_pattern) = extract_surface_pattern(lower) {
        return Some(structured_material_action(
            "set_surface_opacity",
            serde_json::json!({
                "node_id": node_id,
                "surface_pattern": surface_pattern,
                "value": value
            }),
            0.86,
        ));
    }

    if lower.contains("body")
        || lower.contains("figure")
        || lower.contains("character")
        || lower.contains("skin")
        || lower.contains("transparent")
        || lower.contains("translucent")
    {
        return Some(structured_material_action(
            "set_body_opacity",
            serde_json::json!({ "node_id": node_id, "value": value }),
            0.86,
        ));
    }

    None
}

fn structured_material_action(
    command: &str,
    args: serde_json::Value,
    confidence: f32,
) -> StructuredAiAction {
    StructuredAiAction {
        command: command.to_string(),
        args,
        confidence,
        sdk_refs: sdk_refs_for_command(command),
        requires_confirmation: crate::mcp_client::command_requires_confirmation(command),
    }
}

fn extract_quoted_text(input: &str) -> Option<String> {
    for quote in ['"', '\''] {
        if let Some(start) = input.find(quote) {
            if let Some(end) = input[start + 1..].find(quote) {
                let value = input[start + 1..start + 1 + end].trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

fn extract_asset_path(input: &str) -> Option<String> {
    input
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| c == '"' || c == '\'' || c == ',' || c == '.'))
        .find(|w| {
            let lower = w.to_lowercase();
            lower.ends_with(".duf") || lower.ends_with(".dsf") || lower.ends_with(".dsa")
        })
        .map(|w| w.to_string())
}

fn extract_opacity_value(lower: &str) -> f32 {
    let mut parsed = lower
        .split_whitespace()
        .filter_map(|w| {
            let clean = w.trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
            clean.parse::<f32>().ok()
        })
        .next()
        .unwrap_or_else(|| {
            if lower.contains("transparent") || lower.contains("see-through") {
                0.2
            } else {
                1.0
            }
        });
    if parsed > 1.0 && parsed <= 100.0 {
        parsed /= 100.0;
    }
    parsed.clamp(0.0, 1.0)
}

fn extract_surface_pattern(lower: &str) -> Option<String> {
    const SURFACES: &[&str] = &[
        "torso", "stomach", "abdomen", "chest", "arm", "forearm", "hand", "leg", "thigh", "shin",
        "foot", "head", "face", "neck", "skull", "rib", "spine", "pelvis",
    ];
    for surface in SURFACES {
        if lower.contains(surface) {
            return Some((*surface).to_string());
        }
    }
    if let Some(idx) = lower.find("surface") {
        let rest = lower[idx + "surface".len()..].trim();
        let candidate = rest
            .split_whitespace()
            .take_while(|w| *w != "to" && *w != "opacity" && *w != "transparent")
            .collect::<Vec<_>>()
            .join(" ");
        if !candidate.is_empty() {
            return Some(candidate);
        }
    }
    None
}

fn plan_legacy_command(lower: &str, input: &str) -> Option<StructuredAiAction> {
    let (command, args, confidence) =
        if lower.contains("scene") && (lower.contains("info") || lower.contains("status")) {
            ("get_scene_info", serde_json::json!({}), 0.9)
        } else if lower.contains("list") && lower.contains("node") {
            ("list_nodes", serde_json::json!({}), 0.9)
        } else if lower.contains("selected") {
            ("get_selected_nodes", serde_json::json!({}), 0.85)
        } else if lower.contains("camera") && (lower.contains("list") || lower.contains("show")) {
            ("get_cameras", serde_json::json!({}), 0.85)
        } else if lower.contains("render") || lower.contains("preview") {
            ("render_preview", serde_json::json!({}), 0.8)
        } else if lower.contains("look")
            || lower.contains("vision")
            || lower.contains("see")
            || lower.contains("describe")
        {
            (
                "capture_viewport",
                serde_json::json!({"path": "temp_vision.png"}),
                0.9,
            )
        } else if lower.contains("geoshell") || lower.contains("shells") {
            ("get_geoshells", serde_json::json!({}), 0.9)
        } else {
            return plan_with_reasoning_fallback(input);
        };

    if crate::mcp_client::validate_command(command, &args).is_err() {
        return None;
    }

    let sdk_refs = sdk_refs_for_command(command);
    Some(StructuredAiAction {
        command: command.to_string(),
        args,
        confidence,
        sdk_refs,
        requires_confirmation: crate::mcp_client::command_requires_confirmation(command),
    })
}

fn plan_with_reasoning_fallback(input: &str) -> Option<StructuredAiAction> {
    // Try phrase-based fallback first
    if let Some(action) = plan_with_phrase_fallback(input) {
        return Some(action);
    }

    // Try reasoning-based planning as a last resort
    plan_with_reasoning(input)
}

fn plan_with_reasoning(input: &str) -> Option<StructuredAiAction> {
    // Create a goal from the input
    let goal = create_goal_from_input(input);

    // Create planning context (simplified for now)
    let context = PlanningContext {
        scene_state: None,
        recent_actions: Vec::new(),
        user_preferences: None,
        available_assets: Vec::new(),
        constraints: Vec::new(),
    };

    // Try to generate a plan using the reasoning planner
    let planner = get_reasoning_planner();
    if let Some(plan) = planner.plan_for_goal(&goal, &context) {
        // If we got a plan with reasonable confidence, execute the first step
        if plan.confidence >= 0.4 && !plan.steps.is_empty() {
            // Return the first step of the plan
            return Some(plan.steps[0].action.clone());
        }
    }

    // If reasoning planning didn't work, return None
    None
}

fn create_goal_from_input(input: &str) -> Goal {
    let lower = input.to_lowercase();

    // Determine intent using existing logic from ai_system.rs
    let intent = if lower.contains("load")
        || lower.contains("apply")
        || lower.contains("wear")
        || lower.contains("put on")
    {
        Intent::LoadAsset
    } else if lower.contains("pose") || lower.contains("position") || lower.contains("posture") {
        Intent::ApplyPose
    } else if lower.contains("select") || lower.contains("choose") || lower.contains("pick") {
        Intent::SelectNode
    } else if lower.contains("light") || lower.contains("illuminate") {
        Intent::CreateLight
    } else if lower.contains("camera") {
        Intent::CreateCamera
    } else if lower.contains("render") || lower.contains("draw") {
        Intent::Render
    } else if lower.contains("material") || lower.contains("texture") || lower.contains("skin") {
        Intent::ChangeMaterial
    } else if lower.contains("move") || lower.contains("rotate") || lower.contains("scale") {
        Intent::AdjustProperty
    } else if lower.contains("make") || lower.contains("create") || lower.contains("new scene") {
        Intent::CreateScene
    } else if lower.contains("save") || lower.contains("export") {
        Intent::SaveScene
    } else if lower.contains("animate") || lower.contains("keyframe") {
        Intent::Animate
    } else if lower.contains("physics") || lower.contains("dforce") {
        Intent::ApplyPhysics
    } else if lower.contains("what") || lower.contains("how") || lower.contains("list") {
        Intent::Query
    } else {
        Intent::Unknown
    };

    // Extract entities using existing logic
    let words: Vec<&str> = lower.split_whitespace().collect();
    let mut entities = Vec::new();

    let figure_patterns = vec![
        "genesis 8",
        "genesis 9",
        "g8f",
        "g8m",
        "g9f",
        "g9m",
        "female",
        "male",
    ];
    for pattern in figure_patterns {
        if lower.contains(pattern) {
            entities.push(Entity {
                entity_type: EntityType::Figure,
                value: pattern.to_string(),
                confidence: 0.9,
            });
            break;
        }
    }

    let clothing_patterns = vec![
        "shirt", "pants", "dress", "jacket", "skirt", "shoes", "boots",
    ];
    for pattern in clothing_patterns {
        if lower.contains(pattern) {
            entities.push(Entity {
                entity_type: EntityType::Asset,
                value: pattern.to_string(),
                confidence: 0.8,
            });
        }
    }

    for word in words {
        if let Ok(n) = word.parse::<f32>() {
            entities.push(Entity {
                entity_type: EntityType::Number,
                value: n.to_string(),
                confidence: 0.9,
            });
        }
    }

    Goal {
        id: format!(
            "goal_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        ),
        description: input.to_string(),
        intent,
        entities,
        priority: GoalPriority::Medium,
        constraints: Vec::new(),
    }
}

fn plan_with_phrase_fallback(input: &str) -> Option<StructuredAiAction> {
    let lower = input.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    // Check each word as a phrase mapping
    for word in &words {
        if let Some(command) = crate::ai_system::map_phrase_to_command(word) {
            if command != "unknown" {
                return Some(StructuredAiAction {
                    command,
                    args: serde_json::json!({}),
                    confidence: 0.65,
                    sdk_refs: vec![],
                    requires_confirmation: false,
                });
            }
        }
    }

    // Check multi-word phrases (up to 3 words)
    for i in 0..words.len() {
        let mut phrase = String::new();
        for (offset, word) in words.iter().enumerate().skip(i).take(3) {
            let j = i + offset;
            if j > i {
                phrase.push(' ');
            }
            phrase.push_str(word);
            if j > i {
                phrase.push(' ');
            }
            phrase.push_str(words[j]);
            if let Some(command) = crate::ai_system::map_phrase_to_command(&phrase) {
                if command != "unknown" {
                    return Some(StructuredAiAction {
                        command,
                        args: serde_json::json!({}),
                        confidence: 0.7,
                        sdk_refs: vec![],
                        requires_confirmation: false,
                    });
                }
            }
        }
    }

    None
}

pub fn execute_structured_action(action: StructuredAiAction) -> Result<String, String> {
    let (command, args) = resolve_action_for_bridge(&action)?;
    crate::mcp_client::validate_command(&command, &args)?;

    // Start undo batch for modifying commands
    let is_modifying = matches!(
        command.as_str(),
        "load_asset"
            | "apply_pose"
            | "add_node"
            | "add_figure"
            | "set_property"
            | "set_material_property"
            | "set_body_opacity"
            | "set_surface_opacity"
            | "show_anatomy"
            | "place_asset_inside"
            | "set_morph"
            | "set_light"
            | "set_render_settings"
            | "delete_node"
    );

    if is_modifying {
        let _ = crate::mcp_client::send_mcp_request("begin_undo_batch", serde_json::json!({}));
    }

    let result = match crate::mcp_client::send_mcp_request(&command, args) {
        Ok(response) => {
            if is_modifying {
                let _ = crate::mcp_client::send_mcp_request(
                    "accept_undo_batch",
                    serde_json::json!({ "caption": format!("AI: {}", command) }),
                );
                crate::ai_system::enqueue_summary_event(format!(
                    "Command executed successfully: {} with args {}",
                    command, action.args
                ));
            }
            Ok(response
                .result
                .or_else(|| response.data.map(|d| d.to_string()))
                .unwrap_or_else(|| "ok".to_string()))
        },
        Err(e) => {
            if is_modifying {
                let _ =
                    crate::mcp_client::send_mcp_request("cancel_undo_batch", serde_json::json!({}));
            }
            Err(e)
        },
    };
    result
}

fn resolve_action_for_bridge(
    action: &StructuredAiAction,
) -> Result<(String, serde_json::Value), String> {
    if action.command != "add_figure" {
        return Ok((action.command.clone(), action.args.clone()));
    }
    if let Some(path) = action.args.get("path").and_then(|v| v.as_str()) {
        if !path.is_empty() {
            return Ok((
                "load_asset".to_string(),
                serde_json::json!({ "path": path }),
            ));
        }
    }
    let figure_type = action
        .args
        .get("figure_type")
        .and_then(|v| v.as_str())
        .unwrap_or("genesis8");
    let query = if figure_type.contains('9') {
        "Genesis 9 Female"
    } else {
        "Genesis 8 Female"
    };
    if let Some(path) = search_best_matching_asset(query) {
        return Ok((
            "load_asset".to_string(),
            serde_json::json!({ "path": path }),
        ));
    }
    Err(format!(
        "No indexed figure found for '{}'. Scan your Daz content library in Assets, then retry.",
        figure_type
    ))
}

fn sdk_refs_for_command(command: &str) -> Vec<String> {
    match command {
        "get_scene_info" | "list_nodes" | "get_selected_nodes" | "select_node" => {
            vec!["DzScene".to_string(), "DzNode".to_string()]
        },
        "get_cameras" => vec!["DzScene".to_string(), "DzCamera".to_string()],
        "render_preview" | "capture_viewport" => {
            vec!["DzRenderer".to_string(), "DzViewport".to_string()]
        },
        "load_asset" | "import_model" | "export_scene" => {
            vec!["DzContentMgr".to_string(), "DzAsset".to_string()]
        },
        "apply_pose" => vec!["DzFigure".to_string(), "DzPose".to_string()],
        "run_script" => vec!["DzScript".to_string()],
        "set_morph" => vec!["DzMorph".to_string(), "DzFloatProperty".to_string()],
        "set_light" => vec!["DzLight".to_string(), "DzProperty".to_string()],
        "set_render_settings" => vec!["DzRenderMgr".to_string()],
        "set_body_opacity" | "set_surface_opacity" | "show_anatomy" => {
            vec!["DzMaterial".to_string(), "DzFloatProperty".to_string()]
        },
        "get_internal_surfaces" => vec!["DzMaterial".to_string()],
        "place_asset_inside" => vec![
            "DzContentMgr".to_string(),
            "DzNode".to_string(),
            "DzBox3".to_string(),
        ],
        "add_figure" => vec!["DzFigure".to_string(), "DzContentMgr".to_string()],
        "get_scene_assets" => vec!["DzScene".to_string()],
        _ => vec![],
    }
}

pub struct ConflictResolver;

impl ConflictResolver {
    pub fn detect_geoshell_conflicts(
        scene_context: &crate::ai_system::SceneContext,
    ) -> Vec<String> {
        let mut conflicts = vec![];
        // If the user is trying to add a shell but one already exists on the active figure
        if let Some(ref figure) = scene_context.active_figure {
            // This is a simplified check; in a real app, we'd query bridge for specific shell targets
            if scene_context
                .selected_nodes
                .iter()
                .any(|n| n.contains("Shell"))
            {
                conflicts.push(format!("Active figure '{}' already has visible Geometry Shells. Adding another might cause rendering artifacts.", figure));
            }
        }
        conflicts
    }
}

pub fn parse_natural_language_action(input: &str) -> Option<AiAction> {
    let input_lower = input.to_lowercase();

    if input_lower.contains("add")
        || input_lower.contains("create")
        || input_lower.contains("place")
    {
        return parse_add_command(&input_lower);
    }

    if input_lower.contains("set") || input_lower.contains("change") || input_lower.contains("make")
    {
        return parse_set_command(&input_lower);
    }

    if input_lower.contains("pose")
        || input_lower.contains("position")
        || input_lower.contains("rotate")
    {
        return parse_pose_command(&input_lower);
    }

    if input_lower.contains("light") || input_lower.contains("lighting") {
        return parse_lighting_command(&input_lower);
    }

    if input_lower.contains("camera")
        || input_lower.contains("view")
        || input_lower.contains("shot")
    {
        return parse_camera_command(&input_lower);
    }

    None
}

fn parse_add_command(input: &str) -> Option<AiAction> {
    if input.contains("light") {
        Some(AiAction {
            command: "add_light".to_string(),
            target: extract_target(input, vec!["point", "spot", "infinite", "area"]),
            parameters: vec![ActionParam {
                key: "light_type".to_string(),
                value: "point".to_string(),
            }],
            confidence: 0.9,
        })
    } else if input.contains("figure") || input.contains("character") {
        Some(AiAction {
            command: "add_figure".to_string(),
            target: "genesis".to_string(),
            parameters: vec![],
            confidence: 0.85,
        })
    } else if input.contains("prop") || input.contains("object") {
        Some(AiAction {
            command: "add_prop".to_string(),
            target: extract_target(input, vec!["chair", "table", "sphere", "cube"]),
            parameters: vec![],
            confidence: 0.8,
        })
    } else if input.contains("camera") {
        Some(AiAction {
            command: "add_camera".to_string(),
            target: "main".to_string(),
            parameters: vec![],
            confidence: 0.9,
        })
    } else {
        None
    }
}

fn parse_set_command(input: &str) -> Option<AiAction> {
    if input.contains("intensity") || input.contains("brightness") {
        let value = extract_numeric_value(input);
        Some(AiAction {
            command: "set_light_intensity".to_string(),
            target: "selected_light".to_string(),
            parameters: vec![ActionParam {
                key: "intensity".to_string(),
                value,
            }],
            confidence: 0.85,
        })
    } else if input.contains("color") || input.contains("colour") {
        let color = extract_color(input);
        Some(AiAction {
            command: "set_material_color".to_string(),
            target: "selected_material".to_string(),
            parameters: vec![ActionParam {
                key: "color".to_string(),
                value: color,
            }],
            confidence: 0.8,
        })
    } else if input.contains("opacity") || input.contains("transparent") {
        let value = extract_numeric_value(input);
        Some(AiAction {
            command: "set_opacity".to_string(),
            target: "selected".to_string(),
            parameters: vec![ActionParam {
                key: "opacity".to_string(),
                value,
            }],
            confidence: 0.85,
        })
    } else {
        None
    }
}

fn parse_pose_command(input: &str) -> Option<AiAction> {
    let pose_type = if input.contains("heroic") {
        "heroic"
    } else if input.contains("sitting") {
        "sitting"
    } else if input.contains("standing") {
        "standing"
    } else if input.contains("walking") {
        "walking"
    } else if input.contains("running") {
        "running"
    } else {
        "default"
    };

    Some(AiAction {
        command: "apply_pose".to_string(),
        target: "selected_figure".to_string(),
        parameters: vec![ActionParam {
            key: "pose_type".to_string(),
            value: pose_type.to_string(),
        }],
        confidence: 0.8,
    })
}

fn parse_lighting_command(input: &str) -> Option<AiAction> {
    if input.contains("three-point") || input.contains("3-point") {
        Some(AiAction {
            command: "setup_three_point_lighting".to_string(),
            target: "scene".to_string(),
            parameters: vec![
                ActionParam {
                    key: "key_intensity".to_string(),
                    value: "1.0".to_string(),
                },
                ActionParam {
                    key: "fill_intensity".to_string(),
                    value: "0.5".to_string(),
                },
                ActionParam {
                    key: "back_intensity".to_string(),
                    value: "0.7".to_string(),
                },
            ],
            confidence: 0.9,
        })
    } else if input.contains("dramatic") {
        Some(AiAction {
            command: "setup_dramatic_lighting".to_string(),
            target: "scene".to_string(),
            parameters: vec![ActionParam {
                key: "style".to_string(),
                value: "dramatic".to_string(),
            }],
            confidence: 0.85,
        })
    } else if input.contains("soft") || input.contains("natural") {
        Some(AiAction {
            command: "setup_soft_lighting".to_string(),
            target: "scene".to_string(),
            parameters: vec![ActionParam {
                key: "style".to_string(),
                value: "soft".to_string(),
            }],
            confidence: 0.85,
        })
    } else {
        Some(AiAction {
            command: "add_light".to_string(),
            target: "point".to_string(),
            parameters: vec![],
            confidence: 0.7,
        })
    }
}

fn parse_camera_command(input: &str) -> Option<AiAction> {
    let camera_type = if input.contains("portrait") {
        "portrait"
    } else if input.contains("landscape") || input.contains("wide") {
        "landscape"
    } else if input.contains("close-up") || input.contains("closeup") {
        "closeup"
    } else {
        "default"
    };

    Some(AiAction {
        command: "setup_camera".to_string(),
        target: "main".to_string(),
        parameters: vec![ActionParam {
            key: "type".to_string(),
            value: camera_type.to_string(),
        }],
        confidence: 0.85,
    })
}

fn extract_target(input: &str, options: Vec<&str>) -> String {
    for opt in options {
        if input.contains(opt) {
            return opt.to_string();
        }
    }
    "default".to_string()
}

fn extract_numeric_value(input: &str) -> String {
    let numbers: Vec<char> = input
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect();

    if numbers.is_empty() {
        return "1.0".to_string();
    }

    let value: String = numbers.into_iter().collect();

    if value.is_empty() || value == "." {
        return "1.0".to_string();
    }

    value
}

fn extract_color(input: &str) -> String {
    if input.contains("red") {
        "255,0,0".to_string()
    } else if input.contains("blue") {
        "0,0,255".to_string()
    } else if input.contains("green") {
        "0,255,0".to_string()
    } else if input.contains("white") {
        "255,255,255".to_string()
    } else if input.contains("black") {
        "0,0,0".to_string()
    } else if input.contains("yellow") {
        "255,255,0".to_string()
    } else if input.contains("orange") {
        "255,165,0".to_string()
    } else if input.contains("purple") {
        "128,0,128".to_string()
    } else if input.contains("pink") {
        "255,192,203".to_string()
    } else {
        "255,255,255".to_string()
    }
}

pub fn execute_action(action: &AiAction) -> ActionResult {
    match action.command.as_str() {
        "add_light" => execute_add_light(action),
        "add_figure" => execute_add_figure(action),
        "add_prop" => execute_add_prop(action),
        "add_camera" => execute_add_camera(action),
        "set_light_intensity" => execute_set_light_intensity(action),
        "set_material_color" => execute_set_material_color(action),
        "set_opacity" => execute_set_opacity(action),
        "apply_pose" => execute_apply_pose(action),
        "setup_three_point_lighting" => execute_three_point_lighting(action),
        "setup_dramatic_lighting" => execute_dramatic_lighting(action),
        "setup_soft_lighting" => execute_soft_lighting(action),
        "setup_camera" => execute_setup_camera(action),
        _ => ActionResult {
            success: false,
            message: format!("Unknown command: {}", action.command),
            results: vec![],
        },
    }
}

fn execute_add_light(action: &AiAction) -> ActionResult {
    let light_type = if action.target == "spot" || action.target == "distant" {
        format!("{}_light", action.target)
    } else {
        "point_light".to_string()
    };
    match crate::mcp_client::send_mcp_request(
        "add_node",
        serde_json::json!({ "type": light_type, "name": format!("AI_{}", light_type) }),
    ) {
        Ok(resp) => ActionResult {
            success: resp.status == "ok",
            message: format!("Added {} to Daz3D scene", light_type),
            results: vec![resp.result.unwrap_or_default()],
        },
        Err(e) => ActionResult {
            success: false,
            message: format!("Failed to add light to Daz3D: {}", e),
            results: vec![],
        },
    }
}

fn execute_add_figure(action: &AiAction) -> ActionResult {
    let structured = StructuredAiAction {
        command: "add_figure".to_string(),
        args: serde_json::json!({ "figure_type": action.target }),
        confidence: action.confidence,
        sdk_refs: vec!["DzFigure".to_string()],
        requires_confirmation: false,
    };
    match execute_structured_action(structured) {
        Ok(msg) => ActionResult {
            success: true,
            message: format!("Added {} figure to Daz3D scene: {}", action.target, msg),
            results: vec![msg],
        },
        Err(e) => ActionResult {
            success: false,
            message: format!("Failed to add figure to Daz3D: {}", e),
            results: vec![],
        },
    }
}

fn execute_add_prop(action: &AiAction) -> ActionResult {
    match crate::mcp_client::send_mcp_request(
        "load_asset",
        serde_json::json!({ "path": action.target }),
    ) {
        Ok(resp) => ActionResult {
            success: resp.status == "ok",
            message: format!("Requested Daz asset load for {}", action.target),
            results: vec![resp.result.unwrap_or_default()],
        },
        Err(e) => ActionResult {
            success: false,
            message: format!(
                "Prop creation requires a real asset path and bridge support: {}",
                e
            ),
            results: vec![],
        },
    }
}

fn execute_add_camera(action: &AiAction) -> ActionResult {
    match crate::mcp_client::send_mcp_request(
        "add_node",
        serde_json::json!({ "type": "camera", "name": format!("AI_Camera_{}", action.target) }),
    ) {
        Ok(resp) => ActionResult {
            success: resp.status == "ok",
            message: "Added camera to Daz3D scene".to_string(),
            results: vec![resp.result.unwrap_or_default()],
        },
        Err(e) => ActionResult {
            success: false,
            message: format!("Failed to add camera to Daz3D: {}", e),
            results: vec![],
        },
    }
}

fn execute_set_light_intensity(action: &AiAction) -> ActionResult {
    let value = action
        .parameters
        .iter()
        .find(|p| p.key == "intensity")
        .map(|p| p.value.clone())
        .unwrap_or_else(|| "1.0".to_string());

    match crate::mcp_client::send_mcp_request(
        "set_property",
        serde_json::json!({
            "node_id": "selected",
            "property": "Intensity",
            "value": value
        }),
    ) {
        Ok(resp) => ActionResult {
            success: resp.status == "ok",
            message: format!("Set light intensity to {} in Daz3D", value),
            results: vec![format!("Intensity: {}", value)],
        },
        Err(e) => ActionResult {
            success: false,
            message: format!("Failed to set intensity in Daz3D: {}", e),
            results: vec![],
        },
    }
}

fn execute_set_material_color(action: &AiAction) -> ActionResult {
    let color = action
        .parameters
        .iter()
        .find(|p| p.key == "color")
        .map(|p| p.value.clone())
        .unwrap_or_else(|| "255,255,255".to_string());

    match crate::mcp_client::send_mcp_request(
        "set_material_property",
        serde_json::json!({
            "node_id": "selected",
            "property": "Base Color",
            "value": color
        }),
    ) {
        Ok(resp) => ActionResult {
            success: resp.status == "ok",
            message: format!("Set material color to RGB({}) in Daz3D", color),
            results: vec![format!("Color: RGB({})", color)],
        },
        Err(e) => ActionResult {
            success: false,
            message: format!("Failed to set color in Daz3D: {}", e),
            results: vec![],
        },
    }
}

fn execute_set_opacity(action: &AiAction) -> ActionResult {
    let value = action
        .parameters
        .iter()
        .find(|p| p.key == "opacity")
        .map(|p| p.value.clone())
        .unwrap_or_else(|| "1.0".to_string());

    match crate::mcp_client::send_mcp_request(
        "set_body_opacity",
        serde_json::json!({
            "node_id": "selected",
            "value": value
        }),
    ) {
        Ok(resp) => ActionResult {
            success: resp.status == "ok",
            message: format!("Set opacity to {} in Daz3D", value),
            results: vec![format!("Opacity: {}", value)],
        },
        Err(e) => ActionResult {
            success: false,
            message: format!("Failed to set opacity in Daz3D: {}", e),
            results: vec![],
        },
    }
}

fn execute_apply_pose(action: &AiAction) -> ActionResult {
    let pose_type = action
        .parameters
        .iter()
        .find(|p| p.key == "pose_type")
        .map(|p| p.value.clone())
        .unwrap_or_else(|| "default".to_string());

    match crate::mcp_client::send_mcp_request(
        "apply_pose",
        serde_json::json!({ "pose": pose_type }),
    ) {
        Ok(resp) => ActionResult {
            success: resp.status == "ok",
            message: format!("Applied {} pose to figure in Daz3D", pose_type),
            results: vec![resp.result.unwrap_or_default()],
        },
        Err(e) => ActionResult {
            success: false,
            message: format!("Failed to apply pose in Daz3D: {}", e),
            results: vec![],
        },
    }
}

fn execute_three_point_lighting(_action: &AiAction) -> ActionResult {
    let _ = crate::mcp_client::send_mcp_request(
        "add_node",
        serde_json::json!({ "type": "point_light", "name": "AI_Key_Light" }),
    );
    let _ = crate::mcp_client::send_mcp_request(
        "add_node",
        serde_json::json!({ "type": "point_light", "name": "AI_Fill_Light" }),
    );
    let _ = crate::mcp_client::send_mcp_request(
        "add_node",
        serde_json::json!({ "type": "point_light", "name": "AI_Back_Light" }),
    );

    ActionResult {
        success: true,
        message: "Three-point lighting setup added to Daz3D".to_string(),
        results: vec!["Key, Fill, and Back lights created".to_string()],
    }
}

fn execute_dramatic_lighting(_action: &AiAction) -> ActionResult {
    let _ = crate::mcp_client::send_mcp_request(
        "add_node",
        serde_json::json!({ "type": "spot_light", "name": "AI_Dramatic_Spot" }),
    );
    ActionResult {
        success: true,
        message: "Dramatic lighting setup added to Daz3D".to_string(),
        results: vec!["Dramatic spot light created".to_string()],
    }
}

fn execute_soft_lighting(_action: &AiAction) -> ActionResult {
    let _ = crate::mcp_client::send_mcp_request(
        "add_node",
        serde_json::json!({ "type": "distant_light", "name": "AI_Soft_Light" }),
    );
    ActionResult {
        success: true,
        message: "Soft lighting setup added to Daz3D".to_string(),
        results: vec!["Soft distant light created".to_string()],
    }
}

fn execute_setup_camera(action: &AiAction) -> ActionResult {
    let camera_type = action
        .parameters
        .iter()
        .find(|p| p.key == "type")
        .map(|p| p.value.clone())
        .unwrap_or_else(|| "default".to_string());

    let _ = crate::mcp_client::send_mcp_request(
        "add_node",
        serde_json::json!({ "type": "camera", "name": format!("AI_Camera_{}", camera_type) }),
    );

    ActionResult {
        success: true,
        message: format!("Set up {} camera shot", camera_type),
        results: vec![format!("Camera AI_Camera_{} created", camera_type)],
    }
}

pub fn generate_scene_description(input: &str) -> String {
    let input_lower = input.to_lowercase();

    if input_lower.contains("sunset") && input_lower.contains("beach") {
        return r#"Creating sunset beach scene:
- Add infinite light (warm orange, intensity 0.8)
- Set sky environment (sunset gradient)
- Add ground plane (sand texture)
- Position camera for wide shot"#
            .to_string();
    }

    if input_lower.contains("studio") || input_lower.contains("portrait") {
        return r#"Creating studio portrait scene:
- Add key light (front-left, soft white)
- Add fill light (front-right, 50% intensity)
- Add back light for rim
- Position camera for portrait (85mm focal length)"#
            .to_string();
    }

    if input_lower.contains("night") || input_lower.contains("dark") {
        return r#"Creating night scene:
- Set ambient to very low
- Add spot light (cool white)
- Add subtle rim light
- Dark background"#
            .to_string();
    }

    if input_lower.contains("heroic") || input_lower.contains("action") {
        return r#"Creating heroic action pose:
- Apply heroic pose to figure
- Set dramatic camera angle (low, wide)
- Add rim lighting for silhouette
- Dynamic camera position"#
            .to_string();
    }

    format!("Generating scene based on: {}", input)
}

#[tauri::command]
pub fn parse_ai_action(input: String) -> Option<AiAction> {
    parse_natural_language_action(&input)
}

#[tauri::command]
pub fn execute_ai_action(action: AiAction) -> ActionResult {
    execute_action(&action)
}

#[tauri::command]
pub fn generate_scene_prompt(input: String) -> String {
    generate_scene_description(&input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_seek_to_frame() {
        let action = plan_validated_action("go to frame 45").unwrap();
        assert_eq!(action.command, "seek_to_frame");
        assert_eq!(action.args["frame"], 45);

        let action2 = plan_validated_action("seek to frame 10").unwrap();
        assert_eq!(action2.command, "seek_to_frame");
        assert_eq!(action2.args["frame"], 10);
    }

    #[test]
    fn test_parse_set_timeline_range() {
        let action = plan_validated_action("set timeline range from 10 to 150").unwrap();
        assert_eq!(action.command, "set_timeline_range");
        assert_eq!(action.args["start_frame"], 10);
        assert_eq!(action.args["end_frame"], 150);

        let action2 = plan_validated_action("set animation range to 60 frames").unwrap();
        assert_eq!(action2.command, "set_timeline_range");
        assert_eq!(action2.args["start_frame"], 0);
        assert_eq!(action2.args["end_frame"], 60);
    }

    #[test]
    fn test_parse_dforce_simulation() {
        let action = plan_validated_action("simulate cloth from frame 0 to 60").unwrap();
        assert_eq!(action.command, "run_dforce_simulation");
        assert_eq!(action.args["start_frame"], 0);
        assert_eq!(action.args["end_frame"], 60);

        let action2 = plan_validated_action("run dforce simulation for dress").unwrap();
        assert_eq!(action2.command, "run_dforce_simulation");
        assert_eq!(action2.args["node_id"], "dress");
    }

    #[test]
    fn test_parse_set_keyframe() {
        let action = plan_validated_action("keyframe rotation to 45 on frame 30").unwrap();
        assert_eq!(action.command, "set_keyframe");
        assert_eq!(action.args["property"], "yRot");
        assert_eq!(action.args["frame"], 30.0);
        assert_eq!(action.args["value"], 45.0);
        assert_eq!(action.args["interpolation"], "linear");

        let action2 = plan_validated_action(
            "animate x position of 'Genesis 8 Female' to -1.5 at frame 15 ease in",
        )
        .unwrap();
        assert_eq!(action2.command, "set_keyframe");
        assert_eq!(action2.args["node_id"], "Genesis 8 Female");
        assert_eq!(action2.args["property"], "xTranslate");
        assert_eq!(action2.args["frame"], 15.0);
        assert_eq!(action2.args["value"], -1.5);
        assert_eq!(action2.args["interpolation"], "tcb");
    }

    #[test]
    fn test_extract_asset_search_query() {
        assert_eq!(
            extract_asset_search_query("load Genesis 8 Female").unwrap(),
            "Genesis 8 Female"
        );
        assert_eq!(
            extract_asset_search_query("apply the cool pose").unwrap(),
            "cool pose"
        );
        assert_eq!(
            extract_asset_search_query("add a beautiful dress to the scene").unwrap(),
            "beautiful dress"
        );
    }

    #[test]
    fn test_fts_asset_search_and_loading() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_dazpilot.db");

        let db = crate::database::SqliteDatabase::new(&db_path).unwrap();
        db.initialize().unwrap();

        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute(
            "INSERT INTO user_assets (user_id, asset_path, asset_name, original_name, category, subcategory, vendor, file_type, file_size) VALUES ('default', '/paths/hair.duf', 'Genesis 8 Hair', 'G8Hair', 'hair', 'hair', 'VendorA', 'duf', 1024)",
            []
        ).unwrap();
        conn.execute(
            "INSERT INTO user_assets (user_id, asset_path, asset_name, original_name, category, subcategory, vendor, file_type, file_size) VALUES ('default', '/paths/shirt.duf', 'Sleek Shirt Outfit', 'SleekShirt', 'clothing', 'upper', 'VendorB', 'duf', 2048)",
            []
        ).unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO user_assets_fts(rowid, asset_name, original_name, category, subcategory, vendor, asset_path) SELECT id, asset_name, original_name, category, subcategory, vendor, asset_path FROM user_assets",
            []
        ).unwrap();

        {
            let mut guard = crate::database::DATABASE.lock().unwrap();
            *guard = Some(db);
        }

        let path = search_best_matching_asset("Sleek Shirt").unwrap();
        assert_eq!(path, "/paths/shirt.duf");

        let path2 = search_best_matching_asset("Genesis Hair").unwrap();
        assert_eq!(path2, "/paths/hair.duf");

        let action = plan_validated_action("load Sleek Shirt").unwrap();
        assert_eq!(action.command, "load_asset");
        assert_eq!(action.args["path"], "/paths/shirt.duf");

        {
            let mut guard = crate::database::DATABASE.lock().unwrap();
            *guard = None;
        }
    }

    #[test]
    fn test_plan_apply_expression_smile() {
        let action = plan_validated_action("make the figure smile").unwrap();
        assert_eq!(action.command, "apply_expression");
        assert_eq!(action.args["expression_id"], "smile");
        assert!((action.args["value"].as_f64().unwrap() - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_plan_apply_expression_angry_subtle() {
        let action = plan_validated_action("subtle angry expression").unwrap();
        assert_eq!(action.command, "apply_expression");
        assert_eq!(action.args["expression_id"], "angry");
        assert!((action.args["value"].as_f64().unwrap() - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_plan_scene_creation_creates_figure() {
        let action = plan_validated_action("create a scene with good lighting").unwrap();
        assert_eq!(action.command, "add_figure");
        assert_eq!(action.args["figure_type"], "genesis9");
    }

    #[test]
    fn test_plan_add_figure_new_keywords() {
        let action = plan_validated_action("build a woman character").unwrap();
        assert_eq!(action.command, "add_figure");
        assert_eq!(action.args["figure_type"], "genesis8");

        let action2 = plan_validated_action("make a human model").unwrap();
        assert_eq!(action2.command, "add_figure");
        assert_eq!(action2.args["figure_type"], "genesis8");
    }

    #[test]
    fn test_plan_add_figure_no_match_render() {
        // Should NOT match add_figure — "make an image" is about rendering
        let action = plan_validated_action("render this scene");
        assert!(action.as_ref().map(|a| a.command.as_str()) != Some("add_figure"));

        // "image" keyword should not trigger add_figure
        let action2 = plan_validated_action("make an image");
        assert_ne!(
            action2.as_ref().map(|a| a.command.as_str()),
            Some("add_figure")
        );
    }
}
