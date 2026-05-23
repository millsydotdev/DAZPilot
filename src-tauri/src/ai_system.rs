#![allow(dead_code)]

pub mod vector_store;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
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
    pub usage_count: u32,
    pub learned: bool,
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

static EVENT_QUEUE: Lazy<Mutex<Option<mpsc::UnboundedSender<String>>>> = Lazy::new(|| Mutex::new(None));
static SESSION_SUMMARY: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::from("Initial session state. Nothing has happened yet.")));

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
        "You are updating a technical architecture document. You MUST NOT write chronological history (e.g. 'I did X then Y'). You MUST only output the static current state of the architecture.\n\n\
        Current State:\n{}\n\n\
        Recent Events to integrate:\n{}\n\n\
        Output ONLY the updated state document.",
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
        map.insert(phrase.to_string(), PhraseMapping {
            phrase: phrase.to_string(),
            mapped_command: cmd.to_string(),
            category: cat.to_string(),
            usage_count: 0,
            learned: false,
        });
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
    if input.contains("load") || input.contains("apply") || input.contains("wear") || input.contains("put on") {
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
    
    let figure_patterns = vec!["genesis 8", "genesis 9", "g8f", "g8m", "g9f", "g9m", "female", "male"];
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
    
    let clothing_patterns = vec!["shirt", "pants", "dress", "jacket", "skirt", "shoes", "boots"];
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
    map.get(phrase.to_lowercase().as_str()).map(|m| m.mapped_command.clone())
}

pub fn learn_phrase(phrase: &str, command: &str, category: &str) {
    let mut map = PHRASE_MAPPINGS.lock().unwrap();
    
    let mapping = map.entry(phrase.to_lowercase()).or_insert_with(|| PhraseMapping {
        phrase: phrase.to_lowercase(),
        mapped_command: command.to_string(),
        category: category.to_string(),
        usage_count: 0,
        learned: true,
    });
    
    mapping.usage_count += 1;
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

    if let Ok(resp) = crate::mcp_client::send_mcp_request("get_selected_nodes", serde_json::json!({})) {
        if let Some(nodes) = resp.data.as_ref().and_then(|d| d.get("nodes")).and_then(|n| n.as_array()) {
            for n in nodes {
                if let Some(name) = n.get("name").and_then(|name| name.as_str()) {
                    selected_nodes.push(name.to_string());
                }
            }
        }
    }

    if let Some(first_node) = selected_nodes.first() {
        if let Ok(resp) = crate::mcp_client::send_mcp_request("get_node_properties", serde_json::json!({"node_id": first_node})) {
            if let Some(props) = resp.data.as_ref().and_then(|d| d.get("properties")).and_then(|p| p.as_array()) {
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
        message: format!("Executed {:?} with confidence {:.0}%", parsed.intent, parsed.confidence * 100.0),
        action_taken: Some(command.to_string()),
        entities_found: parsed.entities,
        confidence: parsed.confidence,
    }
}

pub fn record_user_feedback(_user_id: &str, command: &str, accepted: bool) {
    log::info!("User feedback: command='{}' accepted={}", command, accepted);
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
