#![allow(dead_code)]

use serde::{Deserialize, Serialize};

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
        if word == "frame" || word == "at" || word == "on" {
            if i + 1 < words.len() {
                if let Ok(val) = words[i+1].trim_matches(|c: char| !c.is_numeric() && c != '.').parse::<f32>() {
                    frame = val;
                    frame_found = true;
                }
            }
        }
        if word == "to" || word == "value" || word == "val" {
            if i + 1 < words.len() {
                if let Ok(val) = words[i+1].trim_matches(|c: char| !c.is_numeric() && c != '-' && c != '.').parse::<f32>() {
                    value = val;
                    value_found = true;
                }
            }
        }
    }

    if !frame_found || !value_found {
        let numbers: Vec<f32> = words.iter()
            .filter_map(|w| w.trim_matches(|c: char| !c.is_numeric() && c != '-' && c != '.').parse::<f32>().ok())
            .collect();
        if numbers.len() >= 2 {
            if !value_found { value = numbers[0]; }
            if !frame_found { frame = numbers[1]; }
        } else if numbers.len() == 1 {
            if !value_found { value = numbers[0]; }
        }
    }

    (frame, value)
}

fn extract_property(input: &str) -> String {
    let lower = input.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace()
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
            _ => {}
        }
    }

    let clean_input = lower.replace("keyframe", "");
    let common_props = vec![
        "xtranslate", "ytranslate", "ztranslate",
        "xrot", "yrot", "zrot",
        "rotation", "position", "scale", "bend", "twist",
        "intensity", "color"
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
            return words[pos + 1].trim_matches(|c: char| !c.is_alphanumeric()).to_string();
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
        if let Some(end) = input[start+1..].find('"') {
            return input[start+1..start+1+end].to_string();
        }
    }
    if let Some(start) = input.find('\'') {
        if let Some(end) = input[start+1..].find('\'') {
            return input[start+1..start+1+end].to_string();
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
        "load asset", "load", "apply pose", "apply", "add figure", "add node", "add", "put on", "equip", "use", "bring in", "import"
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
    
    let filler_words = vec!["the", "a", "an", "some", "to", "in", "into", "on", "scene", "figure"];
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
    
    if let Some(row) = rows.next().ok()? {
        row.get::<_, String>(0).ok()
    } else {
        None
    }
}

// ─── Core Plan Logic ───────────────────────────────────────────────────────

pub fn plan_validated_action(input: &str) -> Option<StructuredAiAction> {
    let lower = input.to_lowercase();

    // 1. Seek to frame
    if (lower.contains("seek") || lower.contains("go to") || lower.contains("jump")) && lower.contains("frame") {
        let words: Vec<&str> = lower.split_whitespace().collect();
        let mut frame = None;
        for i in 0..words.len() {
            if words[i] == "frame" && i + 1 < words.len() {
                if let Ok(val) = words[i+1].trim_matches(|c: char| !c.is_numeric()).parse::<i32>() {
                    frame = Some(val);
                }
            }
        }
        if frame.is_none() {
            frame = words.iter()
                .filter_map(|w| w.trim_matches(|c: char| !c.is_numeric()).parse::<i32>().ok())
                .next();
        }
        if let Some(f) = frame {
            return Some(StructuredAiAction {
                command: "seek_to_frame".to_string(),
                args: serde_json::json!({ "frame": f }),
                confidence: 0.95,
                sdk_refs: vec!["DzScene".to_string()],
                requires_confirmation: false,
            });
        }
    }

    // 2. Set timeline range
    if lower.contains("range") || (lower.contains("timeline") && (lower.contains("limit") || lower.contains("set") || lower.contains("duration") || lower.contains("frames"))) {
        let words: Vec<&str> = lower.split_whitespace().collect();
        let numbers: Vec<i32> = words.iter()
            .filter_map(|w| w.trim_matches(|c: char| !c.is_numeric()).parse::<i32>().ok())
            .collect();
        
        let (start, end) = if numbers.len() >= 2 {
            (numbers[0], numbers[1])
        } else if numbers.len() == 1 {
            (0, numbers[0])
        } else {
            (0, 30)
        };
        
        return Some(StructuredAiAction {
            command: "set_timeline_range".to_string(),
            args: serde_json::json!({ "start_frame": start, "end_frame": end }),
            confidence: 0.95,
            sdk_refs: vec!["DzScene".to_string()],
            requires_confirmation: false,
        });
    }

    // 3. run_dforce_simulation
    if lower.contains("simulate") || lower.contains("dforce") || lower.contains("physics") {
        let words: Vec<&str> = lower.split_whitespace().collect();
        let numbers: Vec<u32> = words.iter()
            .filter_map(|w| w.trim_matches(|c: char| !c.is_numeric()).parse::<u32>().ok())
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
            if wl.contains("dress") || wl.contains("shirt") || wl.contains("skirt") || wl.contains("pants") || wl.contains("hair") || wl.contains("cloth") {
                node_id = word.trim_matches(|c: char| !c.is_alphanumeric()).to_string();
                break;
            }
        }

        return Some(StructuredAiAction {
            command: "run_dforce_simulation".to_string(),
            args: serde_json::json!({
                "node_id": node_id,
                "start_frame": start,
                "end_frame": end,
            }),
            confidence: 0.95,
            sdk_refs: vec!["DzSimulator".to_string()],
            requires_confirmation: true,
        });
    }

    // 4. set_keyframe
    if lower.contains("keyframe") || lower.contains("animate") {
        let node_id = extract_node_id(input);
        let property = extract_property(input);
        let (frame, value) = extract_frame_and_value(input);
        let interpolation = extract_interpolation(input);

        return Some(StructuredAiAction {
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
        });
    }

    // 5. load_asset using Full-Text Search
    if lower.contains("load") || lower.contains("apply") || lower.contains("add") || lower.contains("put on") || lower.contains("equip") || lower.contains("use") || lower.contains("import") {
        if let Some(target) = extract_asset_search_query(input) {
            if let Some(path) = search_best_matching_asset(&target) {
                return Some(StructuredAiAction {
                    command: "load_asset".to_string(),
                    args: serde_json::json!({ "path": path }),
                    confidence: 0.9,
                    sdk_refs: vec!["DzContentMgr".to_string(), "DzAsset".to_string()],
                    requires_confirmation: false,
                });
            }
        }
    }

    // ─── Legacy/Generic Commands ──────────────────────────────────────────────

    let (command, args, confidence) = if lower.contains("scene") && (lower.contains("info") || lower.contains("status")) {
        ("get_scene_info", serde_json::json!({}), 0.9)
    } else if lower.contains("list") && lower.contains("node") {
        ("list_nodes", serde_json::json!({}), 0.9)
    } else if lower.contains("selected") {
        ("get_selected_nodes", serde_json::json!({}), 0.85)
    } else if lower.contains("camera") && (lower.contains("list") || lower.contains("show")) {
        ("get_cameras", serde_json::json!({}), 0.85)
    } else if lower.contains("render") || lower.contains("preview") {
        ("render_preview", serde_json::json!({}), 0.8)
    } else if lower.contains("look") || lower.contains("vision") || lower.contains("see") || lower.contains("describe") {
        ("capture_viewport", serde_json::json!({"path": "temp_vision.png"}), 0.9)
    } else if lower.contains("geoshell") || lower.contains("shells") {
        ("get_geoshells", serde_json::json!({}), 0.9)
    } else {
        return None;
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

pub fn execute_structured_action(action: StructuredAiAction) -> Result<String, String> {
    crate::mcp_client::validate_command(&action.command, &action.args)?;
    
    // Start undo batch for modifying commands
    let is_modifying = match action.command.as_str() {
        "load_asset" | "apply_pose" | "add_node" | "set_property" | "set_material_property" | "delete_node" => true,
        _ => false,
    };

    if is_modifying {
        let _ = crate::mcp_client::send_mcp_request("begin_undo_batch", serde_json::json!({}));
    }

    let result = match crate::mcp_client::send_mcp_request(&action.command, action.args.clone()) {
        Ok(response) => {
            if is_modifying {
                let _ = crate::mcp_client::send_mcp_request("accept_undo_batch", serde_json::json!({ "caption": format!("AI: {}", action.command) }));
                crate::ai_system::enqueue_summary_event(format!("Command executed successfully: {} with args {}", action.command, action.args));
            }
            Ok(response.result.or_else(|| response.data.map(|d| d.to_string())).unwrap_or_else(|| "ok".to_string()))
        }
        Err(e) => {
            if is_modifying {
                let _ = crate::mcp_client::send_mcp_request("cancel_undo_batch", serde_json::json!({}));
            }
            Err(e)
        }
    };
    result
}

fn sdk_refs_for_command(command: &str) -> Vec<String> {
    match command {
        "get_scene_info" | "list_nodes" | "get_selected_nodes" | "select_node" => {
            vec!["DzScene".to_string(), "DzNode".to_string()]
        }
        "get_cameras" => vec!["DzScene".to_string(), "DzCamera".to_string()],
        "render_preview" | "capture_viewport" => vec!["DzRenderer".to_string(), "DzViewport".to_string()],
        "load_asset" | "import_model" | "export_scene" => vec!["DzContentMgr".to_string(), "DzAsset".to_string()],
        "apply_pose" => vec!["DzFigure".to_string(), "DzPose".to_string()],
        "run_script" => vec!["DzScript".to_string()],
        _ => vec![],
    }
}

pub struct ConflictResolver;

impl ConflictResolver {
    pub fn detect_geoshell_conflicts(scene_context: &crate::ai_system::SceneContext) -> Vec<String> {
        let mut conflicts = vec![];
        // If the user is trying to add a shell but one already exists on the active figure
        if let Some(ref figure) = scene_context.active_figure {
            // This is a simplified check; in a real app, we'd query bridge for specific shell targets
            if scene_context.selected_nodes.iter().any(|n| n.contains("Shell")) {
                conflicts.push(format!("Active figure '{}' already has visible Geometry Shells. Adding another might cause rendering artifacts.", figure));
            }
        }
        conflicts
    }
}

pub fn parse_natural_language_action(input: &str) -> Option<AiAction> {
    let input_lower = input.to_lowercase();
    
    if input_lower.contains("add") || input_lower.contains("create") || input_lower.contains("place") {
        return parse_add_command(&input_lower);
    }
    
    if input_lower.contains("set") || input_lower.contains("change") || input_lower.contains("make") {
        return parse_set_command(&input_lower);
    }
    
    if input_lower.contains("pose") || input_lower.contains("position") || input_lower.contains("rotate") {
        return parse_pose_command(&input_lower);
    }
    
    if input_lower.contains("light") || input_lower.contains("lighting") {
        return parse_lighting_command(&input_lower);
    }
    
    if input_lower.contains("camera") || input_lower.contains("view") || input_lower.contains("shot") {
        return parse_camera_command(&input_lower);
    }
    
    None
}

fn parse_add_command(input: &str) -> Option<AiAction> {
    if input.contains("light") {
        Some(AiAction {
            command: "add_light".to_string(),
            target: extract_target(input, vec!["point", "spot", "infinite", "area"]),
            parameters: vec![
                ActionParam { key: "light_type".to_string(), value: "point".to_string() },
            ],
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
            parameters: vec![
                ActionParam { key: "intensity".to_string(), value },
            ],
            confidence: 0.85,
        })
    } else if input.contains("color") || input.contains("colour") {
        let color = extract_color(input);
        Some(AiAction {
            command: "set_material_color".to_string(),
            target: "selected_material".to_string(),
            parameters: vec![
                ActionParam { key: "color".to_string(), value: color },
            ],
            confidence: 0.8,
        })
    } else if input.contains("opacity") || input.contains("transparent") {
        let value = extract_numeric_value(input);
        Some(AiAction {
            command: "set_opacity".to_string(),
            target: "selected".to_string(),
            parameters: vec![
                ActionParam { key: "opacity".to_string(), value },
            ],
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
        parameters: vec![
            ActionParam { key: "pose_type".to_string(), value: pose_type.to_string() },
        ],
        confidence: 0.8,
    })
}

fn parse_lighting_command(input: &str) -> Option<AiAction> {
    if input.contains("three-point") || input.contains("3-point") {
        Some(AiAction {
            command: "setup_three_point_lighting".to_string(),
            target: "scene".to_string(),
            parameters: vec![
                ActionParam { key: "key_intensity".to_string(), value: "1.0".to_string() },
                ActionParam { key: "fill_intensity".to_string(), value: "0.5".to_string() },
                ActionParam { key: "back_intensity".to_string(), value: "0.7".to_string() },
            ],
            confidence: 0.9,
        })
    } else if input.contains("dramatic") {
        Some(AiAction {
            command: "setup_dramatic_lighting".to_string(),
            target: "scene".to_string(),
            parameters: vec![
                ActionParam { key: "style".to_string(), value: "dramatic".to_string() },
            ],
            confidence: 0.85,
        })
    } else if input.contains("soft") || input.contains("natural") {
        Some(AiAction {
            command: "setup_soft_lighting".to_string(),
            target: "scene".to_string(),
            parameters: vec![
                ActionParam { key: "style".to_string(), value: "soft".to_string() },
            ],
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
        parameters: vec![
            ActionParam { key: "type".to_string(), value: camera_type.to_string() },
        ],
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
    let numbers: Vec<char> = input.chars()
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
    match crate::mcp_client::send_mcp_request("add_node", serde_json::json!({ "type": light_type, "name": format!("AI_{}", light_type) })) {
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
    match crate::mcp_client::send_mcp_request("add_figure", serde_json::json!({ "type": action.target })) {
        Ok(resp) => ActionResult {
            success: resp.status == "ok",
            message: format!("Added {} figure to Daz3D scene", action.target),
            results: vec![resp.result.unwrap_or_default()],
        },
        Err(e) => ActionResult {
            success: false,
            message: format!("Failed to add figure to Daz3D: {}", e),
            results: vec![],
        },
    }
}

fn execute_add_prop(action: &AiAction) -> ActionResult {
    match crate::mcp_client::send_mcp_request("load_asset", serde_json::json!({ "path": action.target })) {
        Ok(resp) => ActionResult {
            success: resp.status == "ok",
            message: format!("Requested Daz asset load for {}", action.target),
            results: vec![resp.result.unwrap_or_default()],
        },
        Err(e) => ActionResult {
            success: false,
            message: format!("Prop creation requires a real asset path and bridge support: {}", e),
            results: vec![],
        },
    }
}

fn execute_add_camera(action: &AiAction) -> ActionResult {
    match crate::mcp_client::send_mcp_request("add_node", serde_json::json!({ "type": "camera", "name": format!("AI_Camera_{}", action.target) })) {
        Ok(resp) => ActionResult {
            success: resp.status == "ok",
            message: format!("Added camera to Daz3D scene"),
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
    let value = action.parameters.iter()
        .find(|p| p.key == "intensity")
        .map(|p| p.value.clone())
        .unwrap_or_else(|| "1.0".to_string());
    
    match crate::mcp_client::send_mcp_request("set_property", serde_json::json!({
        "node_id": "selected",
        "property": "Intensity",
        "value": value
    })) {
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
    let color = action.parameters.iter()
        .find(|p| p.key == "color")
        .map(|p| p.value.clone())
        .unwrap_or_else(|| "255,255,255".to_string());
    
    match crate::mcp_client::send_mcp_request("set_material_property", serde_json::json!({
        "node_id": "selected",
        "property": "Base Color",
        "value": color
    })) {
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
    let value = action.parameters.iter()
        .find(|p| p.key == "opacity")
        .map(|p| p.value.clone())
        .unwrap_or_else(|| "1.0".to_string());
    
    match crate::mcp_client::send_mcp_request("set_material_property", serde_json::json!({
        "node_id": "selected",
        "property": "Opacity",
        "value": value
    })) {
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
    let pose_type = action.parameters.iter()
        .find(|p| p.key == "pose_type")
        .map(|p| p.value.clone())
        .unwrap_or_else(|| "default".to_string());
    
    match crate::mcp_client::send_mcp_request("apply_pose", serde_json::json!({ "pose": pose_type })) {
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
    let _ = crate::mcp_client::send_mcp_request("add_node", serde_json::json!({ "type": "point_light", "name": "AI_Key_Light" }));
    let _ = crate::mcp_client::send_mcp_request("add_node", serde_json::json!({ "type": "point_light", "name": "AI_Fill_Light" }));
    let _ = crate::mcp_client::send_mcp_request("add_node", serde_json::json!({ "type": "point_light", "name": "AI_Back_Light" }));

    ActionResult {
        success: true,
        message: "Three-point lighting setup added to Daz3D".to_string(),
        results: vec!["Key, Fill, and Back lights created".to_string()],
    }
}

fn execute_dramatic_lighting(_action: &AiAction) -> ActionResult {
    let _ = crate::mcp_client::send_mcp_request("add_node", serde_json::json!({ "type": "spot_light", "name": "AI_Dramatic_Spot" }));
    ActionResult {
        success: true,
        message: "Dramatic lighting setup added to Daz3D".to_string(),
        results: vec!["Dramatic spot light created".to_string()],
    }
}

fn execute_soft_lighting(_action: &AiAction) -> ActionResult {
    let _ = crate::mcp_client::send_mcp_request("add_node", serde_json::json!({ "type": "distant_light", "name": "AI_Soft_Light" }));
    ActionResult {
        success: true,
        message: "Soft lighting setup added to Daz3D".to_string(),
        results: vec!["Soft distant light created".to_string()],
    }
}

fn execute_setup_camera(action: &AiAction) -> ActionResult {
    let camera_type = action.parameters.iter()
        .find(|p| p.key == "type")
        .map(|p| p.value.clone())
        .unwrap_or_else(|| "default".to_string());
    
    let _ = crate::mcp_client::send_mcp_request("add_node", serde_json::json!({ "type": "camera", "name": format!("AI_Camera_{}", camera_type) }));
    
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
- Position camera for wide shot"#.to_string();
    }
    
    if input_lower.contains("studio") || input_lower.contains("portrait") {
        return r#"Creating studio portrait scene:
- Add key light (front-left, soft white)
- Add fill light (front-right, 50% intensity)
- Add back light for rim
- Position camera for portrait (85mm focal length)"#.to_string();
    }
    
    if input_lower.contains("night") || input_lower.contains("dark") {
        return r#"Creating night scene:
- Set ambient to very low
- Add spot light (cool white)
- Add subtle rim light
- Dark background"#.to_string();
    }
    
    if input_lower.contains("heroic") || input_lower.contains("action") {
        return r#"Creating heroic action pose:
- Apply heroic pose to figure
- Set dramatic camera angle (low, wide)
- Add rim lighting for silhouette
- Dynamic camera position"#.to_string();
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

        let action2 = plan_validated_action("animate x position of 'Genesis 8 Female' to -1.5 at frame 15 ease in").unwrap();
        assert_eq!(action2.command, "set_keyframe");
        assert_eq!(action2.args["node_id"], "Genesis 8 Female");
        assert_eq!(action2.args["property"], "xTranslate");
        assert_eq!(action2.args["frame"], 15.0);
        assert_eq!(action2.args["value"], -1.5);
        assert_eq!(action2.args["interpolation"], "tcb");
    }

    #[test]
    fn test_extract_asset_search_query() {
        assert_eq!(extract_asset_search_query("load Genesis 8 Female").unwrap(), "Genesis 8 Female");
        assert_eq!(extract_asset_search_query("apply the cool pose").unwrap(), "cool pose");
        assert_eq!(extract_asset_search_query("add a beautiful dress to the scene").unwrap(), "beautiful dress");
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
}
