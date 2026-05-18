#![allow(dead_code)]
#![allow(unused_imports)]

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::{local_ai, ollama_service};

static AI_SERVICE: Lazy<Mutex<Option<AiService>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size: u64,
    pub loaded: bool,
}

#[derive(Debug, Clone)]
pub enum AiBackend {
    LocalLlamaCpp,
    ExternalApi(String),
    Mock,
}

pub struct AiService {
    backend: AiBackend,
    loaded: bool,
    model_name: String,
}

impl AiService {
    pub fn new(backend: AiBackend) -> Self {
        Self {
            backend,
            loaded: false,
            model_name: "phi-2-q4".to_string(),
        }
    }

    pub fn load_model(&mut self, model_path: &str) -> Result<(), String> {
        match self.backend {
            AiBackend::LocalLlamaCpp => {
                log::info!("Loading model from: {}", model_path);
                self.loaded = true;
                Ok(())
            }
            AiBackend::ExternalApi(_) => {
                log::info!("Using external API - no local model needed");
                self.loaded = true;
                Ok(())
            }
            AiBackend::Mock => {
                log::info!("Mock AI service - no model loading needed");
                self.loaded = true;
                Ok(())
            }
        }
    }

    pub fn chat(&self, request: ChatRequest) -> Result<ChatResponse, String> {
        if !self.loaded {
            return Err("Model not loaded".to_string());
        }

        match self.backend {
            AiBackend::LocalLlamaCpp => self.local_chat(request),
            AiBackend::Mock => self.mock_chat(request),
            AiBackend::ExternalApi(ref url) => {
                self.external_chat(request, url)
            }
        }
    }

    fn local_chat(&self, request: ChatRequest) -> Result<ChatResponse, String> {
        if !local_ai::is_local_server_running() {
            let model_path = local_ai::first_local_model_path()
                .ok_or_else(|| "No local GGUF model found. Use first launch setup to download a GGUF model.".to_string())?;
            local_ai::start_local_server(&model_path.to_string_lossy(), local_ai::get_local_ai_port())?;
        }

        let prompt = request
            .messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");
        let content = local_ai::chat_with_local(prompt, request.model.clone())?;
        Ok(ChatResponse {
            content,
            model: request.model,
            finish_reason: "stop".to_string(),
        })
    }

    fn mock_chat(&self, request: ChatRequest) -> Result<ChatResponse, String> {
        let last_message = request.messages.last()
            .map(|m| m.content.as_str())
            .unwrap_or("");

        let response = generate_mock_response(last_message, request.temperature);
        
        Ok(ChatResponse {
            content: response,
            model: self.model_name.clone(),
            finish_reason: "stop".to_string(),
        })
    }

    fn external_chat(&self, request: ChatRequest, _url: &str) -> Result<ChatResponse, String> {
        let model_name = request.model.clone();
        let msgs: Vec<ollama_service::ChatMessage> = request.messages.iter()
            .map(|m| ollama_service::ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
                images: None,
            })
            .collect();

        // Use current runtime handle to avoid creating nested runtimes
        let response = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(ollama_service::ollama_chat(
                request.model,
                msgs,
                request.temperature
            ))
        }).map_err(|e| format!("Ollama error: {}", e))?;

        Ok(ChatResponse {
            content: response.message.content,
            model: model_name,
            finish_reason: "stop".to_string(),
        })
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            name: self.model_name.clone(),
            size: 0,
            loaded: self.loaded,
        }
    }
}

fn generate_mock_response(input: &str, _temperature: f32) -> String {
    let input_lower = input.to_lowercase();

    if input_lower.contains("select") || input_lower.contains("node") || input_lower.contains("figure") {
        return "I'll select that figure in the scene. Executing: select_node(node_id=\"genesis_8_female\")".to_string();
    }
    if input_lower.contains("load") || input_lower.contains("asset") || input_lower.contains("morph") {
        return "Loading the asset. Executing: load_asset(path=\"My Library/Assets/clothing.dsf\", options={})".to_string();
    }
    if input_lower.contains("pose") || input_lower.contains("apply") {
        return "Applying the pose to your figure. Executing: apply_pose(figure_id=\"genesis_8_female\", pose_path=\"poses/walking.dsf\")".to_string();
    }
    if input_lower.contains("render") || input_lower.contains("rendering") {
        return "Starting a preview render. Executing: render_preview(camera=\"main_camera\", size=[1920,1080], settings={quality=\"high\"})".to_string();
    }
    if input_lower.contains("create") || input_lower.contains("light") {
        return "Creating a new light. Executing: create_light(type=\"directional\", position=[0,5,5], settings={intensity=1.0})".to_string();
    }
    if input_lower.contains("help") || input_lower.contains("what can") {
        return "I can help you with:\n- Selecting and manipulating figures (Genesis 8/9)\n- Loading assets and morphs\n- Applying poses\n- Creating and adjusting lights\n- Setting up cameras\n- Managing materials\n- Rendering previews\n\nJust describe what you want to do in natural language!".to_string();
    }

    format!("I understand you want to: \"{}\". I can help with that! What specific action would you like me to perform?", input)
}

pub fn init_ai_service(backend: AiBackend) -> Result<(), String> {
    let selected_backend = if std::env::var("DAZPILOT_DEV_MOCK_AI").ok().as_deref() == Some("1") {
        AiBackend::Mock
    } else {
        backend
    };
    let mut service = AiService::new(selected_backend);
    service.load_model("models/phi-2-q4.gguf")?;
    
    let mut global = AI_SERVICE.lock().unwrap();
    *global = Some(service);
    
    Ok(())
}

pub fn chat(prompt: String) -> Result<ChatResponse, String> {
    let global = AI_SERVICE.lock().unwrap();
    if let Some(ref service) = *global {
        let request = ChatRequest {
            messages: vec![
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt,
                }
            ],
            model: "phi-2-q4".to_string(),
            temperature: 0.7,
            max_tokens: 512,
        };
        service.chat(request)
    } else {
        Err("AI service not initialized".to_string())
    }
}

pub fn get_model_info() -> Option<ModelInfo> {
    let global = AI_SERVICE.lock().unwrap();
    global.as_ref().map(|s| s.get_model_info())
}

pub fn is_ai_ready() -> bool {
    let global = AI_SERVICE.lock().unwrap();
    global.as_ref().map(|s| s.is_loaded()).unwrap_or(false)
}
