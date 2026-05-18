#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use reqwest;
use std::time::Duration;

const OLLAMA_HOST: &str = "http://localhost:11434";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelsResponse {
    pub models: Vec<OllamaModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    pub options: ChatOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatOptions {
    pub temperature: f32,
    pub num_predict: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: ChatMessage,
    pub done: bool,
}

pub struct OllamaService {
    client: reqwest::Client,
    base_url: String,
}

impl OllamaService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
            base_url: OLLAMA_HOST.to_string(),
        }
    }

    pub fn with_host(host: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
            base_url: host.to_string(),
        }
    }

    pub async fn is_running(&self) -> bool {
        match self.client.get(&format!("{}/api/tags", self.base_url)).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    pub async fn list_models(&self) -> Result<Vec<OllamaModel>, String> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to list models: {}", response.status()));
        }
        
        let models: OllamaModelsResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(models.models)
    }

    pub async fn pull_model(&self, model_name: &str) -> Result<(), String> {
        let url = format!("{}/api/pull", self.base_url);
        let body = serde_json::json!({ "name": model_name });
        
        let response = self.client.post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to pull model: {}", response.status()));
        }
        
        Ok(())
    }

    pub async fn chat(&self, model: &str, messages: Vec<ChatMessage>, temperature: f32) -> Result<ChatResponse, String> {
        let url = format!("{}/api/chat", self.base_url);
        
        let request = ChatRequest {
            model: model.to_string(),
            messages,
            stream: false,
            options: ChatOptions {
                temperature,
                num_predict: 2048,
            },
        };
        
        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        
        if !response.status().is_success() {
            return Err(format!("Chat request failed: {}", response.status()));
        }
        
        let chat_response: ChatResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(chat_response)
    }

    pub async fn generate(&self, model: &str, prompt: &str, temperature: f32) -> Result<String, String> {
        let url = format!("{}/api/generate", self.base_url);
        
        let request = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": temperature,
                "num_predict": 2048
            }
        });
        
        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        
        if !response.status().is_success() {
            return Err(format!("Generate request failed: {}", response.status()));
        }
        
        #[derive(Deserialize)]
        struct GenerateResponse {
            response: String,
        }
        
        let gen_response: GenerateResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(gen_response.response)
    }
}

impl Default for OllamaService {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command]
pub async fn check_ollama_status() -> Result<bool, String> {
    let service = OllamaService::new();
    Ok(service.is_running().await)
}

#[tauri::command]
pub async fn get_ollama_models() -> Result<Vec<OllamaModel>, String> {
    let service = OllamaService::new();
    service.list_models().await
}

#[tauri::command]
pub async fn pull_ollama_model(model_name: String) -> Result<(), String> {
    let service = OllamaService::new();
    service.pull_model(&model_name).await
}

#[tauri::command]
pub async fn ollama_chat(model: String, messages: Vec<ChatMessage>, temperature: f32) -> Result<ChatResponse, String> {
    let service = OllamaService::new();
    service.chat(&model, messages, temperature).await
}