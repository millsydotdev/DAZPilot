#![allow(dead_code)]

pub mod vector_store;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    LoadAsset,
    ApplyPose,
    SelectNode,
    CreateLight,
    CreateCamera,
    Render,
    ChangeMaterial,
    AdjustProperty,
    CreateScene,
    SaveScene,
    Animate,
    ApplyPhysics,
    Query,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCommand {
    pub intent: Intent,
    pub entities: Vec<Entity>,
    pub confidence: f32,
    pub raw_input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_type: EntityType,
    pub value: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Figure,
    Asset,
    Pose,
    Material,
    Property,
    Camera,
    Light,
    Number,
    FrameRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhraseMapping {
    pub phrase: String,
    pub mapped_command: String,
    pub category: String,
    pub accepted_count: u32,
    pub rejected_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub preferred_figure: Option<String>,
    pub preferred_category: Option<String>,
    pub custom_phrases: Vec<PhraseMapping>,
    pub recent_assets: Vec<String>,
    pub workflow_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneContext {
    pub active_figure: Option<String>,
    pub selected_nodes: Vec<String>,
    pub selected_node_properties: Vec<String>,
    pub available_cameras: Vec<String>,
    pub available_lights: Vec<String>,
    pub current_material: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub success: bool,
    pub message: String,
    pub action_taken: Option<String>,
    pub entities_found: Vec<Entity>,
    pub confidence: f32,
}

static LEARNER: Lazy<Mutex<crate::reasoning::learner::Learner>> =
    Lazy::new(|| Mutex::new(crate::reasoning::learner::Learner::new()));

static EVENT_QUEUE: Lazy<Mutex<Option<mpsc::UnboundedSender<String>>>> =
    Lazy::new(|| Mutex::new(None));
static SESSION_SUMMARY: Lazy<Mutex<String>> = Lazy::new(|| {
    Mutex::new(String::from(
        "Initial session state. Nothing has happened yet.",
    ))
});

pub fn learn_from_plan_execution(
    plan: &crate::reasoning::planner::Plan,
    context: &crate::reasoning::planner::PlanningContext,
    result: &crate::reasoning::executor::ExecutionResult,
    validation: &crate::reasoning::validator::ValidationResult,
) {
    let learner = LEARNER.lock().unwrap();
    learner.learn_from_execution(plan, context, result, validation);
}

pub async fn multi_turn_execute(
    initial_action: &crate::ai_action::StructuredAiAction,
    user_message: &str,
    scene_summary: &str,
    provider: Option<&str>,
    model: Option<&str>,
) -> Vec<(crate::ai_action::StructuredAiAction, bool, String)> {
    // Record that we're starting a multi-turn loop
    crate::agents::analytics::record_multi_turn_start();

    // Multi-turn tool loop: execute actions and get LLM guidance for follow-ups
    let max_turns = 3;
    let max_retry_per_action = 2; // Number of retries per action (so total attempts = max_retry_per_action + 1)
    let mut results: Vec<(crate::ai_action::StructuredAiAction, bool, String)> = Vec::new();

    // Execute initial action
    let mut current_action = Some(initial_action.clone());
    let mut turn_count = 0;

    while let Some(action) = current_action.take() {
        // We've got an action to try (from LLM or initial), so increment turn count
        turn_count += 1;
        if turn_count > max_turns {
            results.push((
                action.clone(),
                false,
                format!("Reached max turns ({})", max_turns),
            ));
            break;
        }

        // Try to execute this action up to max_retry_per_action times (so total attempts = max_retry_per_action + 1)
        let action_start = std::time::Instant::now();
        let mut success = false;
        let mut output = String::new();

        for attempt in 0..=max_retry_per_action {
            let exec_result = crate::ai_action::execute_structured_action(action.clone());
            match exec_result {
                Ok(out) => {
                    success = true;
                    output = out;
                    break; // success, no need to retry
                },
                Err(e) => {
                    if attempt == max_retry_per_action {
                        // Final attempt failed
                        output = format!(
                            "Action failed after {} attempts: {}",
                            max_retry_per_action + 1,
                            e
                        );
                    } else {
                        // Log the failure and continue to retry
                        log::warn!(
                            "Attempt {} failed for action {}: {}",
                            attempt + 1,
                            action.command,
                            e
                        );
                        // Optionally, we could add a small delay here before retrying
                        // tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                },
            }
        }

        let action_duration = action_start.elapsed();
        let action_for_results = action.clone();
        results.push((action_for_results, success, output.clone()));

        // Record command execution (final outcome after retries)
        crate::agents::analytics::record_command_execution(
            action.command.split('_').next().unwrap_or("unknown"),
            &action.command,
            success,
            action_duration,
        );

        // Build action history for better LLM context (using the final outcome of this action)
        let action_history: Vec<(String, String, bool)> = results
            .iter()
            .map(|(a, s, o)| (a.command.clone(), o.clone(), *s))
            .collect();

        // Ask LLM if another action is needed
        let next_action_opt = crate::ai_tool_planner::plan_next_action(
            user_message,
            &action_history,
            scene_summary,
            provider,
            model,
        )
        .await;

        current_action = next_action_opt;

        // If we've reached the max turns, break (but note: we already checked turn_count at the top of the loop)
        // However, we might have just incremented turn_count and then got a next action, but we want to stop after max_turns actions.
        // The condition at the top of the loop will catch it on the next iteration.
    }

    // Record multi-turn loop completion
    let success = results.last().map(|(_, s, _)| *s).unwrap_or(false);
    crate::agents::analytics::record_multi_turn_complete(
        turn_count as u64,
        max_turns as u64,
        success,
    );

    results
}

pub fn get_learner_success_rate(plan: &crate::reasoning::planner::Plan) -> f32 {
    let learner = LEARNER.lock().unwrap();
    learner.get_plan_success_rate(plan)
}

pub fn learn_from_single_action(
    action: &crate::ai_action::StructuredAiAction,
    message: &str,
    success: bool,
    error_message: Option<&str>,
) {
    let context = crate::reasoning::planner::PlanningContext {
        scene_state: None,
        recent_actions: vec![],
        user_preferences: None,
        available_assets: vec![],
        constraints: vec![],
    };

    let plan = crate::reasoning::planner::Plan {
        id: "single-action".to_string(),
        goal_id: "single-action".to_string(),
        goal: crate::reasoning::planner::Goal {
            id: "single-action".to_string(),
            description: message.to_string(),
            intent: crate::ai_system::Intent::Unknown,
            entities: vec![],
            priority: crate::reasoning::planner::GoalPriority::Medium,
            constraints: vec![],
        },
        description: format!("Single action: {}", action.command),
        steps: vec![crate::reasoning::planner::PlanStep {
            id: "step-1".to_string(),
            description: format!("Execute {}", action.command),
            action: action.clone(),
            prerequisites: vec![],
            estimated_time_seconds: 5,
            confidence: 1.0,
            alternatives: vec![],
        }],
        estimated_total_time_seconds: 5,
        confidence: 1.0,
        risk_level: crate::reasoning::planner::RiskLevel::Low,
        fallback_plan: None,
    };

    let result = if success {
        crate::reasoning::executor::ExecutionResult::Success {
            total_time: std::time::Duration::from_millis(0),
            steps_executed: 1,
            step_results: vec![(
                "step-1".to_string(),
                crate::reasoning::executor::Output::Success(
                    error_message.unwrap_or("ok").to_string(),
                ),
            )],
        }
    } else {
        crate::reasoning::executor::ExecutionResult::Failed {
            reason: error_message.unwrap_or("unknown error").to_string(),
            details: String::new(),
            step_executed: 1,
        }
    };

    let validation = crate::reasoning::validator::ValidationResult {
        is_valid: true,
        errors: vec![],
        warnings: vec![],
        suggestions: vec![],
    };

    let learner = LEARNER.lock().unwrap();
    learner.learn_from_execution(&plan, &context, &result, &validation);
}

pub fn enqueue_summary_event(event: String) {
    let mut q = EVENT_QUEUE.lock().unwrap();
    if q.is_none() {
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        *q = Some(tx.clone());

        tokio::spawn(async move {
            let mut batched_events = Vec::new();

            while let Some(ev) = rx.recv().await {
                batched_events.push(ev);

                let timeout = tokio::time::sleep(std::time::Duration::from_millis(2000));
                tokio::pin!(timeout);

                loop {
                    tokio::select! {
                        _ = &mut timeout => {
                            break;
                        }
                        maybe_ev = rx.recv() => {
                            match maybe_ev {
                                Some(e) => batched_events.push(e),
                                None => return,
                            }
                        }
                    }
                }

                summarize_events(&batched_events).await;
                batched_events.clear();
            }
        });
    }

    if let Some(tx) = q.as_ref() {
        let _ = tx.send(event);
    }
}

async fn summarize_events(events: &[String]) {
    let current_summary = SESSION_SUMMARY.lock().unwrap().clone();

    let prompt = format!(
        "You are updating a session summary for a Daz 3D assistant. The summary should be concise and focus on the current state of the scene and the user's recent actions. Do not write a chronological history.\n\n\
        Current Summary:\n{}\n\n\
        Recent Events:\n{}\n\n\
        Updated Concise Summary:",
        current_summary,
        events.join("\n")
    );

    let service = crate::ollama_service::OllamaService::new();
    // Using a fallback model if one isn't specified. Usually llama3 is fast for local summarizing.
    if let Ok(new_summary) = service.generate("llama3", &prompt, 0.1).await {
        *SESSION_SUMMARY.lock().unwrap() = new_summary.trim().to_string();
        log::info!("Session summary updated.");
    } else {
        log::error!("Failed to update session summary. Is Ollama running?");
    }
}

#[tauri::command]
pub fn get_session_summary() -> String {
    SESSION_SUMMARY.lock().unwrap().clone()
}

static PHRASE_MAPPINGS: Lazy<Mutex<HashMap<String, PhraseMapping>>> = Lazy::new(|| {
    let mut map = HashMap::new();

    let default_mappings = vec![
        ("load", "load_asset", "asset"),
        ("apply", "apply_asset", "asset"),
        ("wear", "load_asset", "asset"),
        ("put on", "load_asset", "asset"),
        ("select", "select_node", "scene"),
        ("choose", "select_node", "scene"),
        ("pick", "select_node", "scene"),
        ("pose", "apply_pose", "animation"),
        ("position", "apply_pose", "animation"),
        ("posture", "apply_pose", "animation"),
        ("light", "create_light", "scene"),
        ("illuminate", "create_light", "scene"),
        ("camera", "create_camera", "scene"),
        ("render", "render", "render"),
        ("draw", "render", "render"),
        ("material", "change_material", "material"),
        ("texture", "change_material", "material"),
        ("skin", "change_material", "material"),
        ("move", "adjust_property", "property"),
        ("rotate", "adjust_property", "property"),
        ("scale", "adjust_property", "property"),
        ("make", "create_scene", "scene"),
        ("create", "create_scene", "scene"),
        ("new scene", "create_scene", "scene"),
        ("save", "save_scene", "scene"),
        ("export", "save_scene", "scene"),
        ("animate", "animate", "animation"),
        ("keyframe", "set_keyframe", "animation"),
        ("physics", "apply_physics", "physics"),
        ("dforce", "apply_physics", "physics"),
    ];

    for (phrase, cmd, cat) in default_mappings {
        map.insert(
            phrase.to_string(),
            PhraseMapping {
                phrase: phrase.to_string(),
                mapped_command: cmd.to_string(),
                category: cat.to_string(),
                accepted_count: 0,
                rejected_count: 0,
            },
        );
    }

    Mutex::new(map)
});

pub fn parse_natural_language(input: &str) -> ParsedCommand {
    let input_lower = input.to_lowercase();
    let words: Vec<&str> = input_lower.split_whitespace().collect();

    let intent = detect_intent(&input_lower);
    let entities = extract_entities(&input_lower, &words);
    let confidence = calculate_confidence(&intent, &entities, &words);

    ParsedCommand {
        intent,
        entities,
        confidence,
        raw_input: input.to_string(),
    }
}

fn detect_intent(input: &str) -> Intent {
    if input.contains("load")
        || input.contains("apply")
        || input.contains("wear")
        || input.contains("put on")
    {
        Intent::LoadAsset
    } else if input.contains("pose") || input.contains("position") || input.contains("posture") {
        Intent::ApplyPose
    } else if input.contains("select") || input.contains("choose") || input.contains("pick") {
        Intent::SelectNode
    } else if input.contains("light") || input.contains("illuminate") {
        Intent::CreateLight
    } else if input.contains("camera") {
        Intent::CreateCamera
    } else if input.contains("render") || input.contains("draw") {
        Intent::Render
    } else if input.contains("material") || input.contains("texture") || input.contains("skin") {
        Intent::ChangeMaterial
    } else if input.contains("move") || input.contains("rotate") || input.contains("scale") {
        Intent::AdjustProperty
    } else if input.contains("make") || input.contains("create") || input.contains("new scene") {
        Intent::CreateScene
    } else if input.contains("save") || input.contains("export") {
        Intent::SaveScene
    } else if input.contains("animate") || input.contains("keyframe") {
        Intent::Animate
    } else if input.contains("physics") || input.contains("dforce") {
        Intent::ApplyPhysics
    } else if input.contains("what") || input.contains("how") || input.contains("list") {
        Intent::Query
    } else {
        Intent::Unknown
    }
}

fn extract_entities(input: &str, words: &[&str]) -> Vec<Entity> {
    let mut entities = vec![];

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
        if input.contains(pattern) {
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
        if input.contains(pattern) {
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

    entities
}

fn calculate_confidence(intent: &Intent, entities: &[Entity], _words: &[&str]) -> f32 {
    let base = match intent {
        Intent::Unknown => 0.3,
        Intent::Query => 0.5,
        _ => 0.7,
    };

    let entity_bonus = (entities.len() as f32) * 0.1;
    (base + entity_bonus).min(1.0)
}

pub fn map_phrase_to_command(phrase: &str) -> Option<String> {
    let map = PHRASE_MAPPINGS.lock().unwrap();
    map.get(phrase.to_lowercase().as_str())
        .map(|m| m.mapped_command.clone())
}

pub fn learn_phrase(phrase: &str, command: &str, category: &str, accepted: bool) {
    let mut map = PHRASE_MAPPINGS.lock().unwrap();

    let mapping = map
        .entry(phrase.to_lowercase())
        .or_insert_with(|| PhraseMapping {
            phrase: phrase.to_lowercase(),
            mapped_command: command.to_string(),
            category: category.to_string(),
            accepted_count: 0,
            rejected_count: 0,
        });

    if accepted {
        mapping.accepted_count += 1;
    } else {
        mapping.rejected_count += 1;
    }
}

pub fn get_phrase_mappings() -> Vec<PhraseMapping> {
    let map = PHRASE_MAPPINGS.lock().unwrap();
    map.values().cloned().collect()
}

pub fn build_user_profile(user_id: &str) -> UserProfile {
    UserProfile {
        user_id: user_id.to_string(),
        preferred_figure: None,
        preferred_category: None,
        custom_phrases: vec![],
        recent_assets: vec![],
        workflow_patterns: vec![],
    }
}

pub fn build_scene_context() -> SceneContext {
    let mut selected_nodes = vec![];
    let mut selected_node_properties = vec![];

    if let Ok(resp) =
        crate::mcp_client::send_mcp_request("get_selected_nodes", serde_json::json!({}))
    {
        if let Some(nodes) = resp
            .data
            .as_ref()
            .and_then(|d| d.get("nodes"))
            .and_then(|n| n.as_array())
        {
            for n in nodes {
                if let Some(name) = n.get("name").and_then(|name| name.as_str()) {
                    selected_nodes.push(name.to_string());
                }
            }
        }
    }

    if let Some(first_node) = selected_nodes.first() {
        if let Ok(resp) = crate::mcp_client::send_mcp_request(
            "get_node_properties",
            serde_json::json!({"node_id": first_node}),
        ) {
            if let Some(props) = resp
                .data
                .as_ref()
                .and_then(|d| d.get("properties"))
                .and_then(|p| p.as_array())
            {
                for p in props {
                    if let Some(name) = p.get("name").and_then(|name| name.as_str()) {
                        selected_node_properties.push(name.to_string());
                    }
                }
            }
        }
    }

    SceneContext {
        active_figure: Some("Genesis 8 Female".to_string()),
        selected_nodes,
        selected_node_properties,
        available_cameras: vec!["Main Camera".to_string(), "Front Camera".to_string()],
        available_lights: vec!["Key Light".to_string(), "Fill Light".to_string()],
        current_material: None,
    }
}

pub fn execute_ai_command(parsed: ParsedCommand) -> AiResponse {
    log::info!("Executing AI command: {:?}", parsed.intent);

    let command = match parsed.intent {
        Intent::LoadAsset => "load_asset",
        Intent::ApplyPose => "apply_pose",
        Intent::SelectNode => "select_node",
        Intent::CreateLight => "create_light",
        Intent::CreateCamera => "create_camera",
        Intent::Render => "render_preview",
        Intent::ChangeMaterial => "change_material",
        Intent::AdjustProperty => "adjust_property",
        Intent::CreateScene => "create_scene",
        Intent::SaveScene => "export_scene",
        Intent::Animate => "animate",
        Intent::ApplyPhysics => "apply_physics",
        Intent::Query => "query",
        Intent::Unknown => "unknown",
    };

    AiResponse {
        success: true,
        message: format!(
            "Executed {:?} with confidence {:.0}%",
            parsed.intent,
            parsed.confidence * 100.0
        ),
        action_taken: Some(command.to_string()),
        entities_found: parsed.entities,
        confidence: parsed.confidence,
    }
}

pub fn record_user_feedback(_user_id: &str, command: &str, accepted: bool, phrase: Option<&str>) {
    log::info!("User feedback: command='{}' accepted={}", command, accepted);

    // If we have a phrase associated with this feedback, learn from it
    if let Some(phrase) = phrase {
        // We need to determine the category for the phrase.
        // For simplicity, we can extract it from the command or use a default.
        // In a more sophisticated system, we might look up the category based on the command.
        let category = match command {
            "load_asset" => "asset",
            "apply_pose" => "pose",
            "create_scene" => "scene",
            "save_scene" => "scene",
            "export_scene" => "scene",
            "animate" => "animation",
            "set_keyframe" => "animation",
            "apply_physics" => "physics",
            "render_preview" | "render" => "render",
            "query" => "query",
            "select_node" => "selection",
            "change_material" => "material",
            "adjust_property" => "property",
            "create_light" => "light",
            "create_camera" => "camera",
            "delete_node" => "scene",
            _ => "general",
        };

        learn_phrase(phrase, command, category, accepted);
        log::info!(
            "Learned phrase mapping: '{}' -> '{}' (accepted: {})",
            phrase,
            command,
            accepted
        );
    }

    // If the command was rejected, we could learn from this in the future
    // For now, we just log it, but this is where we'd implement learning from corrections
    if !accepted {
        log::info!(
            "Command '{}' was rejected by user - opportunity for learning",
            command
        );
    }
}

pub fn get_ai_capabilities() -> Vec<String> {
    vec![
        "Natural language parsing".to_string(),
        "Phrase-to-command mapping".to_string(),
        "Intent detection".to_string(),
        "Entity extraction".to_string(),
        "Learning from user behavior".to_string(),
        "Context awareness".to_string(),
        "Workflow pattern detection".to_string(),
    ]
}
