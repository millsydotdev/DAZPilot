use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::time::Duration;
use crate::{local_ai, ollama_service};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModels {
    pub provider: String,
    pub models: Vec<String>,
}

// Deserialization helper for OpenAI Models API
#[derive(Debug, Deserialize)]
struct OpenAiModel {
    id: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModel>,
}

// Deserialization helper for Gemini Models API
#[derive(Debug, Deserialize)]
struct GeminiModel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GeminiModelsResponse {
    models: Vec<GeminiModel>,
}

// Client helper for reqwest
fn get_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(90))
        .build()
        .unwrap_or_default()
}

/// Fetch available models from an API provider dynamically
pub async fn fetch_models(
    provider: &str,
    api_key: Option<String>,
    base_url: Option<String>,
) -> Result<Vec<String>, String> {
    let client = get_client();

    match provider {
        "ollama" => {
            let host = base_url.unwrap_or_else(|| {
                crate::database::get_setting("ollama_host")
                    .unwrap_or_default()
                    .unwrap_or_else(|| "http://localhost:11434".to_string())
            });
            let url = format!("{}/api/tags", host.trim_end_matches('/'));
            match client.get(&url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        if let Ok(res) = resp.json::<ollama_service::OllamaModelsResponse>().await {
                            let mut models: Vec<String> = res.models.into_iter().map(|m| m.name).collect();
                            models.sort();
                            return Ok(models);
                        }
                    }
                    Err(format!("Ollama returned status: {}", status))
                }
                Err(e) => Err(format!("Failed to connect to Ollama: {}", e)),
            }
        }
        "openai" | "custom-openai" => {
            let key = api_key.ok_or_else(|| "API Key is required to fetch OpenAI models.".to_string())?;
            let default_endpoint = if provider == "openai" {
                "https://api.openai.com/v1"
            } else {
                "http://localhost:1234/v1" // Default for LM Studio/local OpenAI-compatible
            };
            let endpoint = base_url.unwrap_or_else(|| default_endpoint.to_string());
            let url = format!("{}/models", endpoint.trim_end_matches('/'));

            let mut headers = HeaderMap::new();
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", key)).map_err(|e| e.to_string())?);

            match client.get(&url).headers(headers).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        if let Ok(res) = resp.json::<OpenAiModelsResponse>().await {
                            let mut models: Vec<String> = res.data.into_iter().map(|m| m.id).collect();
                            models.sort();
                            return Ok(models);
                        }
                    }
                    Err(format!("OpenAI/Compatible endpoint returned status: {}", status))
                }
                Err(e) => Err(format!("Failed to fetch OpenAI models: {}", e)),
            }
        }
        "gemini" => {
            let key = api_key.ok_or_else(|| "API Key is required to fetch Gemini models.".to_string())?;
            let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={}", key);

            match client.get(&url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        if let Ok(res) = resp.json::<GeminiModelsResponse>().await {
                            let mut models: Vec<String> = res.models.into_iter()
                                .map(|m| m.name.replace("models/", ""))
                                .collect();
                            models.sort();
                            return Ok(models);
                        }
                    }
                    Err(format!("Gemini API returned status: {}", status))
                }
                Err(e) => Err(format!("Failed to fetch Gemini models: {}", e)),
            }
        }
        "anthropic" => {
            // Anthropic doesn't have a public models list API endpoint, return popular options
            Ok(vec![
                "claude-3-5-sonnet-latest".to_string(),
                "claude-3-5-haiku-latest".to_string(),
                "claude-3-opus-latest".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
                "claude-3-5-haiku-20241022".to_string(),
                "claude-3-opus-20240229".to_string(),
                "claude-3-sonnet-20240229".to_string(),
                "claude-3-haiku-20240307".to_string(),
            ])
        }
        "local-gguf" => {
            let models = local_ai::list_local_models();
            let mut names: Vec<String> = models.into_iter().map(|m| m.name).collect();
            names.sort();
            if names.is_empty() {
                names.push("phi-2-q4.gguf".to_string()); // Default fallback
            }
            Ok(names)
        }
        _ => Err(format!("Unknown provider: {}", provider)),
    }
}

/// Send a chat completion request to the chosen AI provider
pub async fn run_chat(
    provider: &str,
    model: &str,
    prompt: String,
    api_key: Option<String>,
    base_url: Option<String>,
    temperature: f32,
    max_tokens: u32,
    images: Option<Vec<String>>,
) -> Result<String, String> {
    let client = get_client();

    match provider {
        "ollama" => {
            let host = base_url.unwrap_or_else(|| {
                crate::database::get_setting("ollama_host")
                    .unwrap_or_default()
                    .unwrap_or_else(|| "http://localhost:11434".to_string())
            });
            
            // Re-map images to OllamaChat request format (needs raw base64)
            let cleaned_images: Option<Vec<String>> = images.map(|imgs| {
                imgs.into_iter()
                    .map(|img| {
                        if let Some(pos) = img.find("base64,") {
                            img[pos + 7..].to_string()
                        } else {
                            img
                        }
                    })
                    .collect()
            });

            let service = ollama_service::OllamaService::with_host(&host);
            if !service.is_running().await {
                return Err("Ollama service is not running or unreachable at the configured host.".to_string());
            }

            let chat_response = service.chat(
                model,
                vec![ollama_service::ChatMessage {
                    role: "user".to_string(),
                    content: prompt,
                    images: cleaned_images,
                }],
                temperature,
            ).await?;

            Ok(chat_response.message.content)
        }
        "openai" | "custom-openai" => {
            let key = api_key.ok_or_else(|| "API Key is required for OpenAI/Compatible chat.".to_string())?;
            let default_endpoint = if provider == "openai" {
                "https://api.openai.com/v1"
            } else {
                "http://localhost:1234/v1"
            };
            let endpoint = base_url.unwrap_or_else(|| default_endpoint.to_string());
            let url = format!("{}/chat/completions", endpoint.trim_end_matches('/'));

            let mut headers = HeaderMap::new();
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", key)).map_err(|e| e.to_string())?);
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            // Build payload containing text and optional base64 images
            let payload = if let Some(ref imgs) = images {
                if !imgs.is_empty() {
                    let mut content_array = serde_json::json!([
                        { "type": "text", "text": prompt }
                    ]);

                    for img in imgs {
                        // Prepend standard data URI prefix if it's missing
                        let formatted_img = if img.starts_with("data:image/") {
                            img.clone()
                        } else {
                            format!("data:image/jpeg;base64,{}", img)
                        };
                        content_array.as_array_mut().unwrap().push(serde_json::json!({
                            "type": "image_url",
                            "image_url": { "url": formatted_img }
                        }));
                    }

                    serde_json::json!({
                        "model": model,
                        "messages": [
                            { "role": "user", "content": content_array }
                        ],
                        "temperature": temperature,
                        "max_tokens": max_tokens
                    })
                } else {
                    serde_json::json!({
                        "model": model,
                        "messages": [
                            { "role": "user", "content": prompt }
                        ],
                        "temperature": temperature,
                        "max_tokens": max_tokens
                    })
                }
            } else {
                serde_json::json!({
                    "model": model,
                    "messages": [
                        { "role": "user", "content": prompt }
                    ],
                    "temperature": temperature,
                    "max_tokens": max_tokens
                })
            };

            match client.post(&url).headers(headers).json(&payload).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let res_json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
                        if let Some(choices) = res_json.get("choices").and_then(|c| c.as_array()) {
                            if let Some(first) = choices.first() {
                                if let Some(content) = first.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_str()) {
                                    return Ok(content.to_string());
                                }
                            }
                        }
                        Err("Failed to parse response text from OpenAI".to_string())
                    } else {
                        let err_text = resp.text().await.unwrap_or_default();
                        Err(format!("OpenAI error ({}): {}", url, err_text))
                    }
                }
                Err(e) => Err(format!("OpenAI request failed: {}", e)),
            }
        }
        "gemini" => {
            let key = api_key.ok_or_else(|| "Gemini API Key is required.".to_string())?;
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model, key
            );

            // Construct Gemini contents array
            let mut parts = vec![serde_json::json!({ "text": prompt })];

            if let Some(ref imgs) = images {
                for img in imgs {
                    // Extract clean base64 and figure out mime-type if present
                    let mut mime_type = "image/jpeg".to_string();
                    let clean_b64 = if img.starts_with("data:image/") {
                        if let Some(comma_pos) = img.find(',') {
                            if let Some(colon_pos) = img[..comma_pos].find(':') {
                                if let Some(semi_pos) = img[colon_pos..comma_pos].find(';') {
                                    mime_type = img[colon_pos + 1..colon_pos + semi_pos].to_string();
                                }
                            }
                            img[comma_pos + 1..].to_string()
                        } else {
                            img.clone()
                        }
                    } else {
                        img.clone()
                    };

                    parts.push(serde_json::json!({
                        "inlineData": {
                            "mimeType": mime_type,
                            "data": clean_b64
                        }
                    }));
                }
            }

            let payload = serde_json::json!({
                "contents": [
                    { "parts": parts }
                ],
                "generationConfig": {
                    "temperature": temperature,
                    "maxOutputTokens": max_tokens
                }
            });

            match client.post(&url).json(&payload).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let res_json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
                        if let Some(candidates) = res_json.get("candidates").and_then(|c| c.as_array()) {
                            if let Some(first) = candidates.first() {
                                if let Some(parts) = first.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array()) {
                                    if let Some(p_first) = parts.first() {
                                        if let Some(text) = p_first.get("text").and_then(|t| t.as_str()) {
                                            return Ok(text.to_string());
                                        }
                                    }
                                }
                            }
                        }
                        Err("Failed to parse text from Gemini response".to_string())
                    } else {
                        let err_text = resp.text().await.unwrap_or_default();
                        Err(format!("Gemini API error: {}", err_text))
                    }
                }
                Err(e) => Err(format!("Gemini request failed: {}", e)),
            }
        }
        "anthropic" => {
            let key = api_key.ok_or_else(|| "Anthropic API Key is required.".to_string())?;
            let url = "https://api.anthropic.com/v1/messages";

            let mut headers = HeaderMap::new();
            headers.insert("x-api-key", HeaderValue::from_str(&key).map_err(|e| e.to_string())?);
            headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            let payload = if let Some(ref imgs) = images {
                if !imgs.is_empty() {
                    let mut content_array = vec![serde_json::json!({
                        "type": "text",
                        "text": prompt
                    })];

                    for img in imgs {
                        let mut mime_type = "image/jpeg".to_string();
                        let clean_b64 = if img.starts_with("data:image/") {
                            if let Some(comma_pos) = img.find(',') {
                                if let Some(colon_pos) = img[..comma_pos].find(':') {
                                    if let Some(semi_pos) = img[colon_pos..comma_pos].find(';') {
                                        mime_type = img[colon_pos + 1..colon_pos + semi_pos].to_string();
                                    }
                                }
                                img[comma_pos + 1..].to_string()
                            } else {
                                img.clone()
                            }
                        } else {
                            img.clone()
                        };

                        content_array.push(serde_json::json!({
                            "type": "image",
                            "source": {
                                "type": "base64",
                                "media_type": mime_type,
                                "data": clean_b64
                            }
                        }));
                    }

                    serde_json::json!({
                        "model": model,
                        "messages": [
                            { "role": "user", "content": content_array }
                        ],
                        "max_tokens": max_tokens,
                        "temperature": temperature
                    })
                } else {
                    serde_json::json!({
                        "model": model,
                        "messages": [
                            { "role": "user", "content": prompt }
                        ],
                        "max_tokens": max_tokens,
                        "temperature": temperature
                    })
                }
            } else {
                serde_json::json!({
                    "model": model,
                    "messages": [
                        { "role": "user", "content": prompt }
                    ],
                    "max_tokens": max_tokens,
                    "temperature": temperature
                })
            };

            match client.post(url).headers(headers).json(&payload).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let res_json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
                        if let Some(content) = res_json.get("content").and_then(|c| c.as_array()) {
                            if let Some(first) = content.first() {
                                if let Some(text) = first.get("text").and_then(|t| t.as_str()) {
                                    return Ok(text.to_string());
                                }
                            }
                        }
                        Err("Failed to parse text from Anthropic response".to_string())
                    } else {
                        let err_text = resp.text().await.unwrap_or_default();
                        Err(format!("Anthropic API error: {}", err_text))
                    }
                }
                Err(e) => Err(format!("Anthropic request failed: {}", e)),
            }
        }
        "local-gguf" => {
            if !local_ai::is_local_server_running() {
                let model_path = local_ai::first_local_model_path()
                    .ok_or_else(|| "No local GGUF model found. Choose one in settings or download GGUF weights first.".to_string())?;
                local_ai::start_local_server(&model_path.to_string_lossy(), local_ai::get_local_ai_port())?;
            }

            let mut gguf_prompt = prompt.clone();
            if let Some(ref imgs) = images {
                if !imgs.is_empty() {
                    gguf_prompt = format!(
                        "[System Note: User attached {} image(s). Local GGUF mode currently runs offline without visual eyes.]\n\n{}",
                        imgs.len(),
                        prompt
                    );
                }
            }

            let response_text = local_ai::chat_with_local(gguf_prompt, model.to_string()).await?;
            Ok(response_text)
        }
        _ => Err(format!("Unsupported provider: {}", provider)),
    }
}
