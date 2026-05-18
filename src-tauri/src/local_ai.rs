use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use once_cell::sync::Lazy;

static LLAMA_SERVER: Lazy<Arc<Mutex<Option<LlamaServer>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModelInfo {
    pub name: String,
    pub size_mb: u64,
    pub loaded: bool,
}

pub struct LlamaServer {
    child: std::process::Child,
    port: u16,
}

impl LlamaServer {
    pub fn start(model_path: &str, port: u16) -> Result<Self, String> {
        let exe_path = bundled_resource_dir()?
            .join("llama")
            .join(llama_server_binary_name());

        if !exe_path.exists() {
            return Err(format!("{} not found at: {}", llama_server_binary_name(), exe_path.display()));
        }

        let child = Command::new(&exe_path)
            .args(&[
                "-m", model_path,
                "-c", "4096",
                "--port", &port.to_string(),
                "--host", "127.0.0.1",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start llama-server: {}", e))?;

        thread::sleep(std::time::Duration::from_secs(2));

        Ok(LlamaServer {
            child,
            port,
        })
    }

    pub fn chat(&self, prompt: &str, model: &str) -> Result<String, String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let request_body = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "stream": false
        });

        let response = client
            .post(format!("http://127.0.0.1:{}/v1/chat/completions", self.port))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server error: {}", response.status()));
        }

        let json: serde_json::Value = response
            .json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("No content in response")?;

        Ok(content.to_string())
    }

    pub fn stop(&mut self) {
        let _ = self.child.kill();
    }
}

fn bundled_resource_dir() -> Result<PathBuf, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {}", e))?
        .parent()
        .ok_or("Failed to get parent directory")?
        .to_path_buf();

    #[cfg(target_os = "macos")]
    {
        if exe_dir.ends_with("MacOS") {
            if let Some(contents_dir) = exe_dir.parent() {
                return Ok(contents_dir.join("Resources"));
            }
        }
    }

    Ok(exe_dir)
}

fn llama_server_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "llama-server.exe"
    } else {
        "llama-server"
    }
}

pub fn get_models_dir() -> std::path::PathBuf {
    bundled_resource_dir()
        .unwrap_or_default()
        .join("models")
}

pub fn list_local_models() -> Vec<LocalModelInfo> {
    let models_dir = get_models_dir();
    let mut models = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&models_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "gguf" || e == "bin") {
                let size = std::fs::metadata(&path)
                    .map(|m| m.len() / (1024 * 1024))
                    .unwrap_or(0);
                
                models.push(LocalModelInfo {
                    name: path.file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    size_mb: size,
                    loaded: false,
                });
            }
        }
    }

    models
}

pub fn first_local_model_path() -> Option<std::path::PathBuf> {
    let models_dir = get_models_dir();
    let mut paths: Vec<_> = std::fs::read_dir(&models_dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().map_or(false, |e| e == "gguf" || e == "bin"))
        .collect();
    paths.sort();
    paths.into_iter().next()
}

pub fn start_local_server(model_path: &str, port: u16) -> Result<(), String> {
    let mut server = LLAMA_SERVER.lock().unwrap();
    
    if server.is_some() {
        return Ok(()); 
    }
    
    let srv = LlamaServer::start(model_path, port)?;
    *server = Some(srv);
    
    println!("Local AI server started on port {}", port);
    Ok(())
}

pub fn stop_local_server() {
    let mut server = LLAMA_SERVER.lock().unwrap();
    if let Some(mut srv) = server.take() {
        srv.stop();
        println!("Local AI server stopped");
    }
}

pub fn chat_with_local(prompt: String, model: String) -> Result<String, String> {
    let server = LLAMA_SERVER.lock().unwrap();
    
    if let Some(ref srv) = *server {
        srv.chat(&prompt, &model)
    } else {
        Err("Local server not running".to_string())
    }
}

pub fn is_local_server_running() -> bool {
    let server = LLAMA_SERVER.lock().unwrap();
    server.is_some()
}
