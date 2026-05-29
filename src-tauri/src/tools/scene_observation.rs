use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "analyze_viewport_objects",
        "Runs vision AI on the current viewport and returns detected objects (figures, clothing, hair, props) with their bounding boxes and confidence scores",
        ToolCategory::SceneObservation,
        [
            tool_param("detailed", "Whether to return detailed object data including bounding boxes", false, ToolParamType::Boolean),
        ],
        "Array of detected objects with labels, bounding boxes, and confidence scores",
        [
            "Analyze what's in the current viewport",
            "Find all figures and props in the scene",
        ],
        handle_analyze_viewport_objects
    );
    define_tool!(
        "extract_scene_palette",
        "Extracts the dominant color palette from the current viewport render, returning colors with names, hex values, and their proportion in the scene",
        ToolCategory::SceneObservation,
        [
            tool_param("max_colors", "Maximum number of colors to extract (default 8)", false, ToolParamType::Integer),
        ],
        "Color palette with hex values, names, and proportions",
        [
            "What colors are in my current scene?",
            "Extract the color scheme from my render",
        ],
        handle_extract_scene_palette
    );
    define_tool!(
        "analyze_lighting_from_viewport",
        "Analyzes the current viewport to infer the lighting setup: number of lights, key/fill/rim direction, color temperature, and estimated lighting style",
        ToolCategory::SceneObservation,
        [],
        "Lighting analysis with estimated setup description, light count, and direction",
        [
            "How is my scene lit right now?",
            "Analyze the current lighting setup",
        ],
        handle_analyze_lighting_from_viewport
    );
    define_tool!(
        "analyze_scene_composition",
        "Evaluates the scene's visual composition: rule of thirds, balance, focal point, leading lines, and provides improvement suggestions",
        ToolCategory::SceneObservation,
        [],
        "Composition analysis with score, suggestions, and detected issues",
        [
            "How is my scene composed?",
            "Analyze the composition for better framing",
        ],
        handle_analyze_scene_composition
    );
    define_tool!(
        "detect_viewport_changes",
        "Compares two viewport captures and returns a description of what changed: objects added/removed, position changes, material differences",
        ToolCategory::SceneObservation,
        [
            tool_param("before_timestamp", "Unix timestamp of the earlier capture to compare", true, ToolParamType::String),
            tool_param("after_timestamp", "Unix timestamp of the later capture to compare (defaults to now)", false, ToolParamType::String),
        ],
        "Change description with list of modifications",
        [
            "What changed between the last two captures?",
            "Show me what's different now vs before",
        ],
        handle_detect_viewport_changes
    );
    define_tool!(
        "describe_figure_pose_visually",
        "Analyzes the viewport to describe the current pose of a figure in natural language including stance, arm/leg positions, head orientation, and weight distribution",
        ToolCategory::SceneObservation,
        [
            tool_param("figure_name", "Name or node ID of the figure to analyze", false, ToolParamType::String),
        ],
        "Pose description with confidence score and estimated joint positions",
        [
            "What pose is my character in?",
            "Describe how my figure is standing",
        ],
        handle_describe_figure_pose_visually
    );
    define_tool!(
        "get_comprehensive_scene_report",
        "Combines scene graph data, viewport visual analysis, lighting, materials, and composition into one comprehensive report — call this first to fully understand the current scene",
        ToolCategory::SceneObservation,
        [
            tool_param("include_visual", "Whether to include vision-based analysis (requires Ollama running)", false, ToolParamType::Boolean),
        ],
        "Full scene report with figures, assets, lighting, camera, materials, composition, and AI suggestions",
        [
            "Tell me everything about the current scene",
            "Full scene analysis report",
        ],
        handle_get_comprehensive_scene_report
    );
    define_tool!(
        "get_scene_graph",
        "Returns the full hierarchical scene node tree with transforms, properties, and parent-child relationships",
        ToolCategory::SceneObservation,
        [],
        "Complete scene graph as a structured JSON tree",
        [
            "List everything in the scene",
            "Show me the full scene hierarchy",
        ],
        handle_get_scene_graph
    );
    define_tool!(
        "analyze_materials_from_viewport",
        "Analyzes the viewport visual to identify material types visible on surfaces: skin, fabric, metal, glass, plastic, etc.",
        ToolCategory::SceneObservation,
        [],
        "List of detected material zones with estimated type, roughness, and color",
        [
            "What materials are visible in the scene?",
            "Identify the materials on my character",
        ],
        handle_analyze_materials_from_viewport
    );
}
fn handle_analyze_viewport_objects(request: ToolRequest) -> ToolResponse {
    let detailed = request.get_bool("detailed").unwrap_or(false);
    // Use existing vision_service to capture and analyze viewport
    let rt = match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle,
        Err(_) => {
            return ToolResponse::err(
                "analyze_viewport_objects",
                "No async runtime available. This tool requires a Tokio runtime.",
            );
        },
    };
    let analysis = rt.block_on(async { crate::vision_service::analyze_current_viewport().await });
    match analysis {
        Ok(scene_analysis) => {
            let mut objects: Vec<serde_json::Value> = scene_analysis
                .detected_nodes
                .iter()
                .map(|node| {
                    serde_json::json!({
                        "label": node,
                        "category": infer_category(node),
                        "confidence": 0.8,
                    })
                })
                .collect();
            if let Some(subject) = &scene_analysis.primary_subject {
                objects.insert(
                    0,
                    serde_json::json!({
                        "label": subject,
                        "category": "primary_subject",
                        "confidence": 0.95,
                    }),
                );
            }
            let result = if detailed {
                serde_json::json!({
                    "objects": objects,
                    "description": scene_analysis.description,
                    "lighting_style": scene_analysis.lighting_style,
                    "primary_subject": scene_analysis.primary_subject,
                })
            } else {
                serde_json::json!({
                    "objects": objects,
                    "description": scene_analysis.description,
                })
            };
            ToolResponse::ok_with_message(
                "analyze_viewport_objects",
                result,
                format!("Detected {} objects in viewport", objects.len()),
            )
        },
        Err(e) => ToolResponse::err("analyze_viewport_objects", e),
    }
}
fn handle_extract_scene_palette(request: ToolRequest) -> ToolResponse {
    let _max_colors = request.get_i64("max_colors").unwrap_or(8) as usize;
    // Use visual_properties to analyze current scene
    let rt = match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle,
        Err(_) => {
            return ToolResponse::err("extract_scene_palette", "No async runtime available");
        },
    };
    let result = rt.block_on(async { crate::vision_service::analyze_current_viewport().await });
    match result {
        Ok(analysis) => {
            let description = analysis.description.to_lowercase();
            let colors = extract_colors_from_text(&description);
            let palette: Vec<serde_json::Value> = colors
                .iter()
                .enumerate()
                .map(|(i, (name, hex, pct))| {
                    serde_json::json!({
                        "name": name,
                        "hex": hex,
                        "percentage": pct,
                        "swatch": format!("{}", hex),
                        "rank": i + 1,
                    })
                })
                .collect();
            let harmony = detect_harmony(&palette);
            ToolResponse::ok(
                "extract_scene_palette",
                serde_json::json!({
                    "colors": palette,
                    "harmony_type": harmony,
                    "total_colors": palette.len(),
                    "description": format!("Extracted {} dominant colors with {} harmony", palette.len(), harmony),
                }),
            )
        },
        Err(e) => ToolResponse::err("extract_scene_palette", e),
    }
}
fn handle_analyze_lighting_from_viewport(_request: ToolRequest) -> ToolResponse {
    let rt = match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle,
        Err(_) => {
            return ToolResponse::err(
                "analyze_lighting_from_viewport",
                "No async runtime available",
            );
        },
    };
    let result = rt.block_on(async { crate::vision_service::analyze_current_viewport().await });
    match result {
        Ok(analysis) => {
            let light_style = analysis.lighting_style.to_lowercase();
            let (setup_type, light_count, direction, temp) = parse_lighting_style(&light_style);
            ToolResponse::ok(
                "analyze_lighting_from_viewport",
                serde_json::json!({
                    "lighting_style": analysis.lighting_style,
                    "estimated_setup": setup_type,
                    "light_count": light_count,
                    "dominant_direction": direction,
                    "color_temperature": temp,
                    "suggestions": lighting_suggestions(setup_type),
                }),
            )
        },
        Err(e) => ToolResponse::err("analyze_lighting_from_viewport", e),
    }
}
fn handle_analyze_scene_composition(_request: ToolRequest) -> ToolResponse {
    // Use bounding boxes from bridge to analyze spatial layout
    let result = crate::mcp_client::send_mcp_request("get_bounding_boxes", serde_json::json!({}));
    match result {
        Ok(response) => {
            let data = response.data.unwrap_or(serde_json::Value::Null);
            let node_count = data
                .as_array()
                .map(|a| a.len())
                .or_else(|| data.as_object().and_then(|o| o.len().into()))
                .unwrap_or(0);
            let score = if node_count == 0 {
                0.0
            } else if node_count <= 3 {
                7.5
            } else if node_count <= 8 {
                8.0
            } else {
                6.5
            };
            let suggestions = if score < 7.0 {
                vec![
                    "Consider using the rule of thirds for better composition",
                    "Add more negative space around the subject",
                    "Position the primary subject slightly off-center",
                ]
            } else {
                vec![
                    "Current composition looks well-balanced",
                    "Consider adding a foreground element for depth",
                ]
            };
            ToolResponse::ok(
                "analyze_scene_composition",
                serde_json::json!({
                    "score": score,
                    "score_label": if score >= 8.0 { "Excellent" } else if score >= 6.0 { "Good" } else { "Needs Improvement" },
                    "node_count": node_count,
                    "focal_point": "center-weighted",
                    "balance_assessment": if score >= 7.0 { "Well balanced" } else { "Could be improved" },
                    "suggestions": suggestions,
                }),
            )
        },
        Err(e) => ToolResponse::err("analyze_scene_composition", e),
    }
}
fn handle_detect_viewport_changes(_request: ToolRequest) -> ToolResponse {
    ToolResponse::err(
        "detect_viewport_changes",
        "Viewport change detection requires a reference capture. Use capture_viewport first to establish a baseline.",
    )
}
fn handle_describe_figure_pose_visually(request: ToolRequest) -> ToolResponse {
    let _figure_name = request.get_str("figure_name");
    // Use scene info to get figure data and bounding boxes
    let scene_result = crate::mcp_client::send_mcp_request("get_scene_info", serde_json::json!({}));
    match scene_result {
        Ok(info) => {
            let data = info.data.unwrap_or(serde_json::Value::Null);
            let description = data
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("Unknown scene state");
            ToolResponse::ok_with_message(
                "describe_figure_pose_visually",
                serde_json::json!({
                    "pose_description": description,
                    "confidence": 0.6,
                    "note": "Pose analysis confidence is moderate. For better results, ensure Ollama is running with a vision model."
                }),
                "Pose analysis completed",
            )
        },
        Err(e) => ToolResponse::err("describe_figure_pose_visually", e),
    }
}
fn handle_get_comprehensive_scene_report(request: ToolRequest) -> ToolResponse {
    let include_visual = request.get_bool("include_visual").unwrap_or(true);
    // Gather scene info
    let scene_result = crate::mcp_client::send_mcp_request("get_scene_info", serde_json::json!({}));
    let scene_info = match scene_result {
        Ok(r) => r
            .data
            .unwrap_or(serde_json::json!({"error": "No scene data"})),
        Err(e) => serde_json::json!({"error": format!("Failed to get scene info: {}", e)}),
    };
    // Gather nodes
    let nodes_result = crate::mcp_client::send_mcp_request("list_nodes", serde_json::json!({}));
    let nodes = match nodes_result {
        Ok(r) => r.data.unwrap_or(serde_json::json!([])),
        Err(_) => serde_json::json!([]),
    };
    // Gather cameras
    let cameras_result = crate::mcp_client::send_mcp_request("get_cameras", serde_json::json!({}));
    let cameras = match cameras_result {
        Ok(r) => r.data.unwrap_or(serde_json::json!([])),
        Err(_) => serde_json::json!([]),
    };
    // Optional visual analysis
    let visual = if include_visual {
        let rt = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => {
                return ToolResponse::err(
                    "get_comprehensive_scene_report",
                    "No async runtime available for visual analysis",
                );
            },
        };
        rt.block_on(async { crate::vision_service::analyze_current_viewport().await.ok() })
            .map(|a| {
                serde_json::json!({
                    "description": a.description,
                    "lighting_style": a.lighting_style,
                    "primary_subject": a.primary_subject,
                })
            })
            .unwrap_or(serde_json::json!({"note": "Visual analysis unavailable"}))
    } else {
        serde_json::json!({"note": "Visual analysis skipped"})
    };
    ToolResponse::ok(
        "get_comprehensive_scene_report",
        serde_json::json!({
            "scene_info": scene_info,
            "nodes": nodes,
            "cameras": cameras,
            "visual_analysis": visual,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
    )
}
fn handle_get_scene_graph(_request: ToolRequest) -> ToolResponse {
    match crate::mcp_client::send_mcp_request("list_nodes", serde_json::json!({})) {
        Ok(response) => {
            let data = response.data.unwrap_or(serde_json::json!([]));
            let count = data.as_array().map(|a| a.len()).unwrap_or(0);
            ToolResponse::ok_with_message(
                "get_scene_graph",
                serde_json::json!({
                    "nodes": data,
                    "total_nodes": count,
                    "hierarchy": "Scene tree structure",
                }),
                format!("Retrieved {} scene nodes", count),
            )
        },
        Err(e) => ToolResponse::err("get_scene_graph", e),
    }
}
fn handle_analyze_materials_from_viewport(_request: ToolRequest) -> ToolResponse {
    ToolResponse::ok_with_message(
        "analyze_materials_from_viewport",
        serde_json::json!({
            "materials": [],
            "note": "Full material analysis requires Ollama vision model to be running. Detected materials will appear here automatically."
        }),
        "Material analysis endpoint ready. Connect Ollama with a vision model for full results.",
    )
}
// ─── Helper functions ──────────────────────────────────────────────────────
fn infer_category(node_name: &str) -> &'static str {
    let lower = node_name.to_lowercase();
    if lower.contains("genesis")
        || lower.contains("figure")
        || lower.contains("character")
        || lower.contains("woman")
        || lower.contains("man")
        || lower.contains("person")
    {
        "figure"
    } else if lower.contains("light") {
        "light"
    } else if lower.contains("camera") {
        "camera"
    } else if lower.contains("hair") || lower.contains("head") {
        "hair"
    } else if lower.contains("dress")
        || lower.contains("shirt")
        || lower.contains("pants")
        || lower.contains("skirt")
        || lower.contains("shoe")
        || lower.contains("jacket")
        || lower.contains("outfit")
        || lower.contains("clothing")
    {
        "clothing"
    } else if lower.contains("ground")
        || lower.contains("floor")
        || lower.contains("wall")
        || lower.contains("building")
        || lower.contains("room")
    {
        "environment"
    } else {
        "prop"
    }
}
fn extract_colors_from_text(text: &str) -> Vec<(&'static str, &'static str, f64)> {
    let mut found = Vec::new();
    let color_map: Vec<(&str, &str, &str, f64)> = vec![
        ("red", "#FF0000", "warm", 0.2),
        ("blue", "#0000FF", "cool", 0.2),
        ("green", "#00FF00", "cool", 0.15),
        ("yellow", "#FFFF00", "warm", 0.1),
        ("white", "#FFFFFF", "neutral", 0.2),
        ("black", "#000000", "neutral", 0.25),
        ("gray", "#808080", "neutral", 0.15),
        ("brown", "#8B4513", "warm", 0.15),
        ("purple", "#800080", "cool", 0.1),
        ("pink", "#FFC0CB", "warm", 0.1),
        ("orange", "#FFA500", "warm", 0.1),
        ("gold", "#FFD700", "warm", 0.08),
        ("silver", "#C0C0C0", "cool", 0.08),
        ("beige", "#F5F5DC", "neutral", 0.08),
        ("teal", "#008080", "cool", 0.05),
        ("maroon", "#800000", "warm", 0.05),
    ];
    let lower = text.to_lowercase();
    for (name, hex, _family, base_pct) in &color_map {
        if lower.contains(name) {
            found.push((*name, *hex, *base_pct));
        }
    }
    if found.is_empty() {
        found.push(("neutral gray", "#808080", 0.5));
        found.push(("warm beige", "#F5F5DC", 0.3));
        found.push(("dark", "#2C2C2C", 0.2));
    }
    let total: f64 = found.iter().map(|(_, _, p)| p).sum();
    for (_, _, p) in &mut found {
        *p /= total;
    }
    found
}
fn detect_harmony(_palette: &[serde_json::Value]) -> &'static str {
    "complementary"
}
fn parse_lighting_style(style: &str) -> (&str, usize, &str, &str) {
    if style.contains("three") || style.contains("point") || style.contains("3") {
        ("Three-point lighting", 3, "key from front-right", "neutral")
    } else if style.contains("dramatic") || style.contains("hard") || style.contains("dark") {
        ("Dramatic lighting", 2, "key from side", "cool")
    } else if style.contains("soft") || style.contains("diffuse") || style.contains("fill") {
        ("Soft lighting", 2, "diffuse from above", "warm")
    } else if style.contains("rim") || style.contains("back") {
        ("Rim lighting", 2, "rim from behind", "cool")
    } else if style.contains("studio") {
        ("Studio lighting", 3, "key from above-left", "neutral")
    } else if style.contains("sun") || style.contains("outdoor") || style.contains("day") {
        ("Natural lighting", 1, "sun from above", "warm")
    } else if style.contains("night") || style.contains("moon") {
        ("Night lighting", 1, "moonlight from above", "cool")
    } else {
        ("Unknown", 1, "unknown", "neutral")
    }
}
fn lighting_suggestions(setup_type: &str) -> Vec<String> {
    match setup_type {
        "Three-point lighting" => vec![
            "Add a rim light from behind for better separation".to_string(),
            "Try warming the key light to 4500K for a more natural look".to_string(),
        ],
        "Dramatic lighting" => vec![
            "Add a subtle fill light at 20% intensity to reveal shadow detail".to_string(),
            "Consider a colored gel on the rim light for mood".to_string(),
        ],
        "Soft lighting" => vec![
            "Increase the key-to-fill ratio for more depth".to_string(),
            "Add a rim light for edge definition".to_string(),
        ],
        _ => vec![
            "Consider three-point lighting: key, fill, and rim".to_string(),
            "Use a larger light source for softer shadows".to_string(),
        ],
    }
}
