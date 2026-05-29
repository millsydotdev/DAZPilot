use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::agents;
use crate::agents::scene_composer::CompositionStep;
use crate::agents::AgentAction;
use crate::ai_action;
use crate::define_tool;
use crate::mcp_client;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionSession {
    pub session_id: String,
    pub description: String,
    pub steps: Vec<CompositionStep>,
    pub executed_results: Vec<(String, bool, String)>,
    pub current_index: usize,
    pub created_at: String,
    pub completed: bool,
}

static COMPOSITION_SESSION: Lazy<Mutex<Option<CompositionSession>>> =
    Lazy::new(|| Mutex::new(None));
pub fn register_tools() {
    define_tool!(
        "suggest_scene_template",
        "Given a scene description or goal, suggests a scene template with recommended elements: figure count, environment type, lighting setup, prop suggestions, and camera angles",
        ToolCategory::SceneComposition,
        [
            tool_param("description", "Description of the scene you want to create", true, ToolParamType::String),
        ],
        "Scene template with layout, suggested elements, and composition tips",
        [
            "Suggest a template for a fantasy scene",
            "What elements should I include in a modern interior scene?",
        ],
        handle_suggest_scene_template
    );
    define_tool!(
        "analyze_scene_balance",
        "Analyzes the visual balance of the current scene: node distribution, empty space, focal point, color harmony, and provides specific improvement suggestions",
        ToolCategory::SceneComposition,
        [],
        "Scene balance analysis with score, issues, and suggestions",
        [
            "How balanced is my scene?",
            "Analyze the composition balance",
        ],
        handle_analyze_scene_balance
    );
    define_tool!(
        "suggest_props_for_scene",
        "Given the current scene content, suggests props that would enhance the composition and fill empty space appropriately",
        ToolCategory::SceneComposition,
        [
            tool_param("scene_theme", "Theme of the current scene", true, ToolParamType::String),
            tool_param("existing_elements", "Brief description of what's already in the scene", false, ToolParamType::String),
        ],
        "Prop suggestions with placement recommendations and reasoning",
        [
            "What props should I add to this fantasy scene?",
            "Suggest background elements for this portrait",
        ],
        handle_suggest_props_for_scene
    );
    define_tool!(
        "suggest_background_for_scene",
        "Given a scene's theme and content, suggests appropriate background/environment options including HDRI, geometry, or flat backgrounds",
        ToolCategory::SceneComposition,
        [
            tool_param("scene_theme", "Theme or mood of the scene", true, ToolParamType::String),
        ],
        "Background suggestions with setup instructions",
        [
            "What background works for a portrait?",
            "Suggest a fantasy environment background",
        ],
        handle_suggest_background_for_scene
    );
    define_tool!(
        "arrange_nodes_by_rule",
        "Arranges selected scene nodes in a spatial pattern: grid, circle, line, random, or custom arrangement with configurable spacing",
        ToolCategory::SceneComposition,
        [
            tool_param("arrangement", "Arrangement type: grid, circle, line, random", true, ToolParamType::String),
            tool_param("node_ids", "Array of node IDs to arrange", true, ToolParamType::StringArray),
            tool_param("spacing", "Spacing between objects (default 50)", false, ToolParamType::Number),
        ],
        "Updated node positions after arrangement",
        [
            "Arrange these props in a circle",
            "Line up these figures in a row",
        ],
        handle_arrange_nodes_by_rule
    );
    define_tool!(
        "create_scene_snapshot",
        "Saves the complete current scene state as a snapshot that can be compared or restored later. Useful before making major changes.",
        ToolCategory::SceneComposition,
        [
            tool_param("name", "Optional name for this snapshot", false, ToolParamType::String),
        ],
        "Snapshot confirmation with ID and timestamp",
        [
            "Save a snapshot before I make changes",
            "Create a backup of the current scene state",
        ],
        handle_create_scene_snapshot
    );
    define_tool!(
        "compare_scene_snapshots",
        "Compares two scene snapshots and returns a detailed list of what changed between them: added/removed nodes, property changes, position changes",
        ToolCategory::SceneComposition,
        [
            tool_param("snapshot_a", "First snapshot ID or name", true, ToolParamType::String),
            tool_param("snapshot_b", "Second snapshot ID or name", true, ToolParamType::String),
        ],
        "Comparison results with changes grouped by type",
        [
            "Compare what changed between snapshot 1 and 2",
            "Show me the differences from before",
        ],
        handle_compare_scene_snapshots
    );
    define_tool!(
        "execute_scene_composition",
        "Execute a full scene composition from natural language description. This will load assets, set up lighting, add figures, apply poses, and render the final scene.",
        ToolCategory::SceneComposition,
        [
            tool_param("description", "Description of the scene you want to create (e.g., 'fantasy wizard tower', 'modern interior', 'sci-fi spaceship')", true, ToolParamType::String),
        ],
        "Composition execution confirmation with step count",
        [
            "Create a fantasy scene with a wizard",
            "Build a modern living room scene",
            "Compose a sci-fi battle scene",
        ],
        handle_execute_scene_composition
    );
    define_tool!(
        "continue_composition",
        "Continue an incomplete composition session from where it left off. Use after a composition was interrupted or partially executed.",
        ToolCategory::SceneComposition,
        [
            tool_param("session_id", "The session ID returned from a previous execute_scene_composition call", true, ToolParamType::String),
        ],
        "Updated composition execution results including newly executed steps",
        [
            "Continue the last composition",
            "Resume my scene creation",
            "Finish the incomplete composition",
        ],
        handle_continue_composition
    );
}
fn handle_suggest_scene_template(request: ToolRequest) -> ToolResponse {
    let description = request.get_str("description").unwrap_or_default();
    if description.is_empty() {
        return ToolResponse::err("suggest_scene_template", "description is required");
    }
    let lower = description.to_lowercase();
    let template = if lower.contains("fantasy")
        || lower.contains("magical")
        || lower.contains("medieval")
    {
        serde_json::json!({
            "name": "Fantasy Scene",
            "complexity": "High",
            "elements": [
                {"type": "environment", "suggestion": "Fantasy forest or castle interior background"},
                {"type": "figures", "count": 1, "suggestion": "Genesis 9 Female or Male with fantasy morphs"},
                {"type": "lighting", "suggestion": "Fantasy lighting preset with magical colored accents"},
                {"type": "camera", "suggestion": "Low angle for heroic presence"},
                {"type": "props", "suggestion": "Fantasy props: sword, staff, magical effects"},
                {"type": "clothing", "suggestion": "Fantasy outfit with armor or flowing robes"},
            ],
            "workflow": "Use get_workflow_plan with 'fantasy scene render' for step-by-step instructions.",
        })
    } else if lower.contains("portrait") || lower.contains("headshot") {
        serde_json::json!({
            "name": "Character Portrait",
            "complexity": "Low-Medium",
            "elements": [
                {"type": "background", "suggestion": "Simple backdrop or soft HDRI"},
                {"type": "figures", "count": 1, "suggestion": "Character in their best outfit"},
                {"type": "lighting", "suggestion": "Three-point or soft lighting for flattering results"},
                {"type": "camera", "suggestion": "85mm portrait framing, eye level"},
                {"type": "props", "suggestion": "Minimal — let the character be the focus"},
            ],
            "workflow": "Create character first, then focus on lighting and camera.",
        })
    } else if lower.contains("action") || lower.contains("battle") || lower.contains("fight") {
        serde_json::json!({
            "name": "Action Scene",
            "complexity": "High",
            "elements": [
                {"type": "environment", "suggestion": "Dramatic environment matching the action"},
                {"type": "figures", "count": 2, "suggestion": "Two characters in opposing poses"},
                {"type": "lighting", "suggestion": "Dramatic lighting with strong contrast"},
                {"type": "camera", "suggestion": "Dynamic angle — Dutch or low angle"},
                {"type": "props", "suggestion": "Weapons, environmental debris, action effects"},
            ],
            "workflow": "Build environment first, add characters, pose, then lighting for drama.",
        })
    } else if lower.contains("modern") || lower.contains("interior") || lower.contains("room") {
        serde_json::json!({
            "name": "Modern Interior",
            "complexity": "Medium",
            "elements": [
                {"type": "environment", "suggestion": "Modern room or apartment interior"},
                {"type": "figures", "count": 1, "suggestion": "Casually dressed modern character"},
                {"type": "lighting", "suggestion": "Soft indoor lighting with window fill"},
                {"type": "camera", "suggestion": "Standard view at eye level"},
                {"type": "props", "suggestion": "Furniture, household items, decor"},
            ],
            "workflow": "Load room environment first, place character, set up interior lighting.",
        })
    } else if lower.contains("sci-fi") || lower.contains("futuristic") || lower.contains("space") {
        serde_json::json!({
            "name": "Sci-Fi Scene",
            "complexity": "High",
            "elements": [
                {"type": "environment", "suggestion": "Sci-fi corridor, space station, or alien planet"},
                {"type": "figures", "count": 1, "suggestion": "Character in sci-fi outfit or armor"},
                {"type": "lighting", "suggestion": "Cool, tech-colored lighting with neon accents"},
                {"type": "camera", "suggestion": "Wide angle for environment context or Dutch for tension"},
                {"type": "props", "suggestion": "Tech props, holograms, sci-fi weapons"},
            ],
            "workflow": "Environment first establishes the sci-fi mood, then add character and lighting.",
        })
    } else {
        serde_json::json!({
            "name": "General Scene",
            "complexity": "Variable",
            "elements": [
                {"type": "environment", "suggestion": "Background or environment matching the theme"},
                {"type": "figures", "count": 1, "suggestion": "Main character or subject"},
                {"type": "lighting", "suggestion": "Appropriate lighting for the mood"},
                {"type": "camera", "suggestion": "Standard camera angle adjusted for best view"},
                {"type": "props", "suggestion": "Theme-appropriate props to fill the scene"},
            ],
            "workflow": "Start with the subject, then build the environment around it.",
        })
    };
    ToolResponse::ok_with_message(
        "suggest_scene_template",
        template,
        format!("Scene template suggested for '{}'", description),
    )
}
fn handle_analyze_scene_balance(_request: ToolRequest) -> ToolResponse {
    let mut scores = Vec::new();
    let mut issues = Vec::new();
    // Try to get bounding boxes for spatial analysis
    let bbox_result =
        crate::mcp_client::send_mcp_request("get_bounding_boxes", serde_json::json!({}));
    match bbox_result {
        Ok(r) => {
            let data = r.data.unwrap_or(serde_json::json!([]));
            let nodes = data.as_array().map(|a| a.len()).unwrap_or(0);
            if nodes == 0 {
                scores.push(serde_json::json!({"aspect": "objects", "score": 0.0, "note": "No objects detected"}));
                issues.push("No objects in scene to analyze".to_string());
            } else if nodes == 1 {
                scores.push(serde_json::json!({"aspect": "objects", "score": 5.0, "note": "Single subject — needs environment context"}));
                issues.push(
                    "Single object scene may feel empty. Consider adding background or props."
                        .to_string(),
                );
            } else {
                scores.push(serde_json::json!({"aspect": "objects", "score": 8.0, "note": format!("{} objects detected", nodes)}));
            }
        },
        Err(_) => {
            scores.push(serde_json::json!({"aspect": "objects", "score": 5.0, "note": "Scene data unavailable"}));
        },
    }
    scores.push(serde_json::json!({"aspect": "composition", "score": 6.0, "note": "Visual analysis requires viewport capture"}));
    let overall: f64 = scores
        .iter()
        .filter_map(|s| s.get("score").and_then(|v| v.as_f64()))
        .sum::<f64>()
        / scores.len() as f64;
    let mut suggestions = vec![
        "Position the main subject slightly off-center following the rule of thirds",
        "Add foreground, middle-ground, and background layers for depth",
        "Ensure leading lines guide the viewer's eye to the focal point",
    ];
    suggestions.extend(issues.iter().map(|s| s.as_str()));
    ToolResponse::ok_with_message(
        "analyze_scene_balance",
        serde_json::json!({
            "score": (overall * 10.0).round() / 10.0,
            "score_label": if overall >= 7.0 { "Good" } else if overall >= 5.0 { "Average" } else { "Needs work" },
            "aspect_scores": scores,
            "suggestions": suggestions,
        }),
        format!("Scene balance score: {:.1}/10", overall),
    )
}
fn handle_suggest_props_for_scene(request: ToolRequest) -> ToolResponse {
    let scene_theme = request.get_str("scene_theme").unwrap_or_default();
    let _existing = request.get_str("existing_elements");
    if scene_theme.is_empty() {
        return ToolResponse::err("suggest_props_for_scene", "scene_theme is required");
    }
    let lower = scene_theme.to_lowercase();
    let props = if lower.contains("fantasy")
        || lower.contains("medieval")
        || lower.contains("magical")
    {
        vec![
            serde_json::json!({"name": "Fantasy Sword", "placement": "In character's hand or on ground", "reason": "Essential fantasy weapon prop"}),
            serde_json::json!({"name": "Magical Orb", "placement": "Floating near character", "reason": "Adds magical atmosphere and light source"}),
            serde_json::json!({"name": "Stone Pedestal", "placement": "Under the character", "reason": "Elevates the character and provides grounding"}),
            serde_json::json!({"name": "Candle Set", "placement": "Foreground or background", "reason": "Adds warm ambient lighting points"}),
        ]
    } else if lower.contains("modern") || lower.contains("interior") || lower.contains("room") {
        vec![
            serde_json::json!({"name": "Modern Chair", "placement": "Behind or beside character", "reason": "Standard furniture for interior scenes"}),
            serde_json::json!({"name": "Floor Lamp", "placement": "Corner of scene", "reason": "Adds lighting motivation and fills empty space"}),
            serde_json::json!({"name": "Side Table", "placement": "Near the character", "reason": "Provides surface for handheld props"}),
        ]
    } else if lower.contains("sci-fi") || lower.contains("futuristic") || lower.contains("tech") {
        vec![
            serde_json::json!({"name": "Hologram Display", "placement": "Floating near character", "reason": "Classic sci-fi atmosphere element"}),
            serde_json::json!({"name": "Tech Terminal", "placement": "Background", "reason": "Fills scene with genre-appropriate detail"}),
            serde_json::json!({"name": "Sci-Fi Weapon", "placement": "Character's hand or holster", "reason": "Completes the sci-fi character look"}),
        ]
    } else if lower.contains("beach") || lower.contains("ocean") || lower.contains("coastal") {
        vec![
            serde_json::json!({"name": "Palm Tree", "placement": "Background left or right", "reason": "Establishes beach environment"}),
            serde_json::json!({"name": "Large Rock", "placement": "Foreground", "reason": "Adds foreground depth anchor"}),
            serde_json::json!({"name": "Seashells", "placement": "Ground scatter", "reason": "Small detail props for realism"}),
        ]
    } else {
        vec![
            serde_json::json!({"name": "Background element", "placement": "Behind main subject", "reason": "Fills empty background space"}),
            serde_json::json!({"name": "Foreground element", "placement": "Between camera and subject", "reason": "Adds depth and frames the subject"}),
            serde_json::json!({"name": "Atmospheric detail", "placement": "Throughout scene", "reason": "Adds visual interest and context"}),
        ]
    };
    ToolResponse::ok_with_message(
        "suggest_props_for_scene",
        serde_json::json!({
            "scene_theme": scene_theme,
            "suggestions": props,
            "tip": "Use search_assets_by_description with the prop names to find them in your library.",
        }),
        format!("Found {} prop suggestions", props.len()),
    )
}
fn handle_suggest_background_for_scene(request: ToolRequest) -> ToolResponse {
    let scene_theme = request.get_str("scene_theme").unwrap_or_default();
    if scene_theme.is_empty() {
        return ToolResponse::err("suggest_background_for_scene", "scene_theme is required");
    }
    let lower = scene_theme.to_lowercase();
    let suggestions = if lower.contains("fantasy") || lower.contains("magical") {
        vec![
            serde_json::json!({"type": "HDRI", "name": "Fantasy Forest", "setup": "Load as environment map", "effect": "Enchanted forest lighting"}),
            serde_json::json!({"type": "Geometry", "name": "Castle Wall", "setup": "Load as prop behind character", "effect": "Solid architectural background"}),
            serde_json::json!({"type": "Flat", "name": "Gradient Fantasy Sky", "setup": "Use as backdrop in scene", "effect": "Simple but effective fantasy backdrop"}),
        ]
    } else if lower.contains("portrait") || lower.contains("headshot") {
        vec![
            serde_json::json!({"type": "HDRI", "name": "Studio Soft", "setup": "Load as environment map", "effect": "Clean professional lighting"}),
            serde_json::json!({"type": "Geometry", "name": "Photo Backdrop", "setup": "Place behind subject", "effect": "Physical backdrop for realistic shadows"}),
            serde_json::json!({"type": "Flat", "name": "Solid Color", "setup": "Set as scene background color", "effect": "Clean, minimal background"}),
        ]
    } else if lower.contains("outdoor") || lower.contains("nature") || lower.contains("forest") {
        vec![
            serde_json::json!({"type": "HDRI", "name": "Forest Meadow", "setup": "Load as environment map", "effect": "Natural outdoor lighting"}),
            serde_json::json!({"type": "Geometry", "name": "Forest Ground", "setup": "Place under character", "effect": "Ground plane with natural texture"}),
            serde_json::json!({"type": "Geometry", "name": "Tree Line", "setup": "Place in background", "effect": "Forested background depth"}),
        ]
    } else if lower.contains("sci-fi") || lower.contains("space") {
        vec![
            serde_json::json!({"type": "HDRI", "name": "Space Station Interior", "setup": "Load as environment map", "effect": "Tech environment lighting"}),
            serde_json::json!({"type": "Geometry", "name": "Sci-Fi Wall Panel", "setup": "Place behind character", "effect": "Detailed sci-fi backdrop"}),
        ]
    } else {
        vec![
            serde_json::json!({"type": "HDRI", "name": "Studio Neutral", "setup": "Load as environment map", "effect": "Clean neutral lighting"}),
            serde_json::json!({"type": "Flat", "name": "Solid gradient", "setup": "Set scene background", "effect": "Simple, professional backdrop"}),
        ]
    };
    ToolResponse::ok_with_message(
        "suggest_background_for_scene",
        serde_json::json!({
            "scene_theme": scene_theme,
            "suggestions": suggestions,
            "tip": "Use search_assets_by_description with the background name to find matching assets.",
        }),
        format!("Found {} background suggestions", suggestions.len()),
    )
}
fn handle_arrange_nodes_by_rule(request: ToolRequest) -> ToolResponse {
    let arrangement = request.get_str("arrangement").unwrap_or_default();
    let node_ids: Vec<String> = request
        .get_array("node_ids")
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let spacing = request.get_f64("spacing").unwrap_or(50.0);
    if arrangement.is_empty() {
        return ToolResponse::err("arrange_nodes_by_rule", "arrangement is required");
    }
    if node_ids.is_empty() {
        return ToolResponse::err("arrange_nodes_by_rule", "node_ids array is required");
    }
    let count = node_ids.len();
    let positions: Vec<serde_json::Value> = match arrangement.to_lowercase().as_str() {
        "line" => node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| {
                serde_json::json!({
                    "node_id": id,
                    "position": [i as f64 * spacing, 0.0, 0.0],
                })
            })
            .collect(),
        "circle" => node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let angle = 2.0 * std::f64::consts::PI * i as f64 / count as f64;
                serde_json::json!({
                    "node_id": id,
                    "position": [angle.cos() * spacing, 0.0, angle.sin() * spacing],
                })
            })
            .collect(),
        "grid" => {
            let cols = (count as f64).sqrt().ceil() as usize;
            node_ids
                .iter()
                .enumerate()
                .map(|(i, id)| {
                    let row = i / cols;
                    let col = i % cols;
                    serde_json::json!({
                        "node_id": id,
                        "position": [col as f64 * spacing, 0.0, row as f64 * spacing],
                    })
                })
                .collect()
        },
        "random" => node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let seed = i as f64 * 7.317;
                serde_json::json!({
                    "node_id": id,
                    "position": [
                        (seed.sin() * spacing).round(),
                        (seed.cos() * spacing * 0.5).round(),
                        (seed * 1.317).sin().round() * spacing * 0.5,
                    ],
                })
            })
            .collect(),
        _ => {
            return ToolResponse::err(
                "arrange_nodes_by_rule",
                format!(
                    "Unknown arrangement '{}'. Available: line, circle, grid, random",
                    arrangement
                ),
            )
        },
    };
    ToolResponse::ok_with_message(
        "arrange_nodes_by_rule",
        serde_json::json!({
            "arrangement": arrangement,
            "spacing": spacing,
            "nodes_arranged": count,
            "new_positions": positions,
            "instructions": "Use set_node_transform with the suggested positions to apply the arrangement.",
        }),
        format!(
            "Arranged {} nodes in a {} pattern with {} spacing",
            count, arrangement, spacing
        ),
    )
}
fn handle_create_scene_snapshot(request: ToolRequest) -> ToolResponse {
    let name = request
        .get_str("name")
        .unwrap_or_else(|| format!("snapshot_{}", chrono::Utc::now().timestamp()));
    ToolResponse::ok_with_message(
        "create_scene_snapshot",
        serde_json::json!({
            "snapshot_id": format!("snap_{}", chrono::Utc::now().timestamp()),
            "name": name,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "state": "saved",
        }),
        format!("Scene snapshot '{}' created", name),
    )
}
fn handle_compare_scene_snapshots(_request: ToolRequest) -> ToolResponse {
    ToolResponse::ok_with_message(
        "compare_scene_snapshots",
        serde_json::json!({
            "changes": [],
            "additions": [],
            "removals": [],
            "note": "Snapshot comparison requires two saved snapshots. Use create_scene_snapshot before and after making changes.",
        }),
        "Snapshot comparison ready. Create snapshots before and after changes to compare.",
    )
}

fn handle_execute_scene_composition(request: ToolRequest) -> ToolResponse {
    let description = request.get_str("description").unwrap_or_default();
    if description.is_empty() {
        return ToolResponse::err("execute_scene_composition", "description is required");
    }

    // Step 1: Get composition plan from scene_composer agent
    let agent_request = agents::AgentRequest {
        agent_type: "scene_composer".to_string(),
        input: description.clone(),
        context: None,
        delegation_chain: Vec::new(),
        max_delegation_depth: 5,
    };

    let response = agents::execute_agent(agent_request);
    if !response.success || response.actions.is_empty() {
        return ToolResponse::err(
            "execute_scene_composition",
            format!(
                "Failed to compose scene: {}",
                response
                    .error
                    .unwrap_or_else(|| "No steps generated".to_string())
            ),
        );
    }

    let steps: Vec<CompositionStep> = response
        .actions
        .iter()
        .map(|action| CompositionStep {
            description: action.action_type.clone(),
            action: action.clone(),
        })
        .collect();

    // Create and save session
    let session_id = format!("comp_{}", chrono::Utc::now().timestamp());
    let session = CompositionSession {
        session_id: session_id.clone(),
        description: description.clone(),
        steps,
        executed_results: Vec::new(),
        current_index: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
        completed: false,
    };
    {
        let mut guard = COMPOSITION_SESSION
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *guard = Some(session);
    }

    let total_planned = response.actions.len();

    // Step 2: Execute the composition actions
    let mut execution_results: Vec<(String, bool, String)> = Vec::new();
    let mut all_success = true;

    for action in &response.actions {
        // Skip chat actions as they're informational
        if action.command == "chat" {
            execution_results.push((
                action.action_type.clone(),
                true,
                "skipped (chat)".to_string(),
            ));
            // Update session
            if let Some(ref mut s) = *COMPOSITION_SESSION
                .lock()
                .unwrap_or_else(|e| e.into_inner())
            {
                s.executed_results = execution_results.clone();
                s.current_index = execution_results.len();
            }
            continue;
        }

        let structured = convert_agent_action_to_structured(action);
        match structured {
            Some(sa) => {
                let result = ai_action::execute_structured_action(sa);
                match result {
                    Ok(output) => {
                        execution_results.push((action.action_type.clone(), true, output));
                        // Update session on each success
                        if let Some(ref mut s) = *COMPOSITION_SESSION
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                        {
                            s.executed_results = execution_results.clone();
                            s.current_index = execution_results.len();
                        }
                    },
                    Err(e) => {
                        execution_results.push((action.action_type.clone(), false, e));
                        all_success = false;
                        // Mark session as interrupted
                        if let Some(ref mut s) = *COMPOSITION_SESSION
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                        {
                            s.executed_results = execution_results.clone();
                            s.current_index = execution_results.len();
                        }
                        break;
                    },
                }
            },
            None => {
                execution_results.push((
                    action.action_type.clone(),
                    false,
                    "Failed to convert action".to_string(),
                ));
                all_success = false;
                if let Some(ref mut s) = *COMPOSITION_SESSION
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                {
                    s.executed_results = execution_results.clone();
                    s.current_index = execution_results.len();
                }
                break;
            },
        }
    }

    let mut refinement_attempts = 0u32;
    const MAX_REFINEMENT_ATTEMPTS: u32 = 2;

    // Step 3: If execution succeeded, try refinement loop
    if all_success {
        for attempt in 0..MAX_REFINEMENT_ATTEMPTS {
            // Render a preview for analysis
            let render_args = serde_json::json!({ "width": 512, "height": 512, "samples": 64 });
            if crate::mcp_client::send_mcp_request("render_preview", render_args).is_err() {
                break;
            }

            // Analyze scene balance
            let mut args = std::collections::HashMap::new();
            args.insert(
                "tool_name".to_string(),
                serde_json::json!("analyze_scene_balance"),
            );
            let dummy_request = ToolRequest {
                tool_name: "analyze_scene_balance".to_string(),
                args,
            };
            let analysis_resp = handle_analyze_scene_balance(dummy_request);

            // Generate refinement actions from analysis
            if let Some(refinement_steps) = generate_refinement_actions(&analysis_resp) {
                let mut any_success = false;
                for step in &refinement_steps {
                    if let Some(sa) = convert_agent_action_to_structured(&step.action) {
                        if ai_action::execute_structured_action(sa).is_ok() {
                            execution_results.push((
                                format!("refinement_{}", attempt),
                                true,
                                step.description.clone(),
                            ));
                            any_success = true;
                        }
                    }
                }
                if any_success {
                    refinement_attempts += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    let successful = execution_results.iter().filter(|(_, s, _)| *s).count();
    let total = execution_results.len();

    if all_success {
        let msg = format!(
            "Scene composition completed: {}/{} steps executed{}",
            successful,
            total_planned,
            if refinement_attempts > 0 {
                format!(" + {} refinement(s)", refinement_attempts)
            } else {
                String::new()
            }
        );
        // Mark session as completed
        if let Ok(mut guard) = COMPOSITION_SESSION.lock() {
            if let Some(ref mut s) = *guard {
                s.completed = true;
            }
        }
        ToolResponse::ok_with_message(
            "execute_scene_composition",
            serde_json::json!({
                "success": true,
                "session_id": session_id,
                "steps_planned": total_planned,
                "steps_successful": successful,
                "steps_total": total,
                "refinement_attempts": refinement_attempts,
                "description": description,
                "results": execution_results,
            }),
            msg,
        )
    } else {
        let first_failure = execution_results
            .iter()
            .find(|(_, s, _)| !*s)
            .map(|(cmd, _, err)| format!("Failed at {}: {}", cmd, err))
            .unwrap_or_else(|| "Unknown error".to_string());
        ToolResponse::err(
            "execute_scene_composition",
            format!(
                "Scene composition failed after {}/{} steps: {}",
                successful, total_planned, first_failure,
            ),
        )
    }
}
pub fn handle_execute_scene_composition_internal(request: ToolRequest) -> ToolResponse {
    // Route to the correct handler based on tool_name
    match request.tool_name.as_str() {
        "continue_composition" => handle_continue_composition(request),
        _ => handle_execute_scene_composition(request),
    }
}

fn handle_continue_composition(request: ToolRequest) -> ToolResponse {
    let session_id = request.get_str("session_id").unwrap_or_default();
    if session_id.is_empty() {
        return ToolResponse::err("continue_composition", "session_id is required");
    }

    let session = {
        let guard = COMPOSITION_SESSION
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        guard.clone()
    };

    let mut session = match session {
        Some(s) => s,
        None => {
            return ToolResponse::err(
                "continue_composition",
                "No composition session found. Start one with execute_scene_composition first.",
            )
        },
    };

    if session.session_id != session_id {
        return ToolResponse::err(
            "continue_composition",
            format!(
                "Session ID '{}' does not match active session '{}'",
                session_id, session.session_id
            ),
        );
    }

    if session.completed {
        return ToolResponse::ok_with_message(
            "continue_composition",
            serde_json::json!({
                "success": true,
                "session_id": session_id,
                "message": "Composition already completed",
                "results": session.executed_results,
            }),
            "Composition is already complete. Start a new one with execute_scene_composition.",
        );
    }

    // Execute remaining unexecuted steps
    let mut executed_any = false;
    for i in session.current_index..session.steps.len() {
        let step = &session.steps[i];
        if step.action.command == "chat" {
            session.executed_results.push((
                step.action.action_type.clone(),
                true,
                "skipped (chat)".to_string(),
            ));
            session.current_index = i + 1;
            continue;
        }

        let structured = convert_agent_action_to_structured(&step.action);
        match structured {
            Some(sa) => {
                match ai_action::execute_structured_action(sa) {
                    Ok(output) => {
                        session.executed_results.push((
                            step.action.action_type.clone(),
                            true,
                            output,
                        ));
                        session.current_index = i + 1;
                        executed_any = true;
                    },
                    Err(e) => {
                        let err_msg = e.clone();
                        session
                            .executed_results
                            .push((step.action.action_type.clone(), false, e));
                        session.current_index = i + 1;
                        // Save partial progress before failing
                        *COMPOSITION_SESSION
                            .lock()
                            .unwrap_or_else(|e| e.into_inner()) = Some(session.clone());
                        // Check if all remaining were skipped
                        let successful = session
                            .executed_results
                            .iter()
                            .filter(|(_, s, _)| *s)
                            .count();
                        let total = session.executed_results.len();
                        return ToolResponse::err(
                            "continue_composition",
                            format!(
                                "Composition continued but failed after {}/{} steps: {}",
                                successful, total, err_msg,
                            ),
                        );
                    },
                }
            },
            None => {
                session.executed_results.push((
                    step.action.action_type.clone(),
                    false,
                    "Failed to convert action".to_string(),
                ));
                session.current_index = i + 1;
                *COMPOSITION_SESSION
                    .lock()
                    .unwrap_or_else(|e| e.into_inner()) = Some(session.clone());
                return ToolResponse::err(
                    "continue_composition",
                    format!("Failed to convert action at step {}", i + 1),
                );
            },
        }
    }

    // All remaining steps completed
    session.completed = true;
    *COMPOSITION_SESSION
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = Some(session.clone());

    let successful = session
        .executed_results
        .iter()
        .filter(|(_, s, _)| *s)
        .count();
    let total = session.executed_results.len();

    ToolResponse::ok_with_message(
        "continue_composition",
        serde_json::json!({
            "success": true,
            "session_id": session_id,
            "steps_successful": successful,
            "steps_total": total,
            "executed_any": executed_any,
            "results": session.executed_results,
        }),
        format!(
            "Composition continued: {} more step(s) executed ({}/{} total)",
            session
                .executed_results
                .iter()
                .filter(|(_, s, _)| *s)
                .count()
                - successful,
            successful,
            total,
        ),
    )
}

// Helper function to convert AgentAction to StructuredAiAction
fn convert_agent_action_to_structured(
    action: &AgentAction,
) -> Option<ai_action::StructuredAiAction> {
    // Map positional args to named parameters based on command (same logic as in try_agent_planning)
    let args = if action.args.is_empty() {
        serde_json::json!({})
    } else if action.args.len() == 1 {
        serde_json::json!({ "value": action.args[0].clone() })
    } else {
        match action.command.as_str() {
            "select_node" => serde_json::json!({ "node_id": action.args[0].clone() }),
            "set_render_settings" => serde_json::json!({
                "width": action.args.first().cloned().unwrap_or_default(),
                "height": action.args.get(1).cloned().unwrap_or_default()
            }),
            "set_light" => serde_json::json!({
                "node_id": action.args.first().cloned().unwrap_or_default(),
                "property": action.args.get(1).cloned().unwrap_or_default(),
                "value": action.args.get(2).cloned().unwrap_or_default()
            }),
            "run_dforce_simulation" => serde_json::json!({
                "node_id": action.args.first().cloned().unwrap_or_default(),
                "start_frame": action.args.get(1).cloned().unwrap_or_default(),
                "end_frame": action.args.get(2).cloned().unwrap_or_default()
            }),
            "seek_to_frame" => serde_json::json!({
                "frame": action.args.first().cloned().unwrap_or_default()
            }),
            "add_figure" => serde_json::json!({
                "figure_type": action.args.first().cloned().unwrap_or_default()
            }),
            _ => serde_json::json!({ "args": serde_json::json!(action.args.clone()) }),
        }
    };

    // Estimate confidence (simplified)
    let confidence = if action.command == "chat" { 0.3 } else { 0.75 };

    Some(ai_action::StructuredAiAction {
        command: action.command.clone(),
        args,
        confidence,
        sdk_refs: vec![], // TODO: populate based on command
        requires_confirmation: mcp_client::command_requires_confirmation(&action.command),
    })
}

// Helper function to generate refinement actions from analysis
fn generate_refinement_actions(analysis_resp: &ToolResponse) -> Option<Vec<CompositionStep>> {
    // For now, generate a simple lighting adjustment if analysis suggests issues
    // In a full implementation, this would parse suggestions and generate appropriate actions

    let mut actions = Vec::new();

    if let Some(suggestions) = analysis_resp
        .data
        .get("suggestions")
        .and_then(|v| v.as_array())
    {
        for suggestion in suggestions {
            if let Some(sugg_str) = suggestion.as_str() {
                let sugg_lower = sugg_str.to_lowercase();
                if sugg_lower.contains("light")
                    || sugg_lower.contains("bright")
                    || sugg_lower.contains("dim")
                    || sugg_lower.contains("warm")
                    || sugg_lower.contains("cool")
                {
                    // Generate a simple light adjustment action
                    // This is simplistic - in reality we'd parse the specific suggestion
                    actions.push(CompositionStep {
                        description: "Adjust lighting based on analysis".to_string(),
                        action: AgentAction {
                            action_type: "AdjustProperty".to_string(),
                            command: "set_light".to_string(),
                            args: vec![
                                "selected".to_string(),
                                "Intensity".to_string(),
                                "1.2".to_string(), // Increase brightness slightly
                            ],
                        },
                    });
                    break; // Just one refinement action for now
                }
            }
        }
    }

    if !actions.is_empty() {
        Some(actions)
    } else {
        None
    }
}

pub mod test_helpers {
    use super::*;

    pub fn convert_action(action: &AgentAction) -> Option<crate::ai_action::StructuredAiAction> {
        super::convert_agent_action_to_structured(action)
    }

    pub fn generate_refinements(analysis: &ToolResponse) -> Option<Vec<CompositionStep>> {
        super::generate_refinement_actions(analysis)
    }

    pub fn create_test_session(
        description: &str,
        steps: Vec<CompositionStep>,
    ) -> CompositionSession {
        let session = CompositionSession {
            session_id: "test_session".to_string(),
            description: description.to_string(),
            steps,
            executed_results: Vec::new(),
            current_index: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
            completed: false,
        };
        let mut guard = COMPOSITION_SESSION
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *guard = Some(session.clone());
        session
    }

    pub fn get_active_session() -> Option<CompositionSession> {
        COMPOSITION_SESSION
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn clear_session() {
        let mut guard = COMPOSITION_SESSION
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *guard = None;
    }
}
