use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::Emitter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub const DEFAULT_INBOX_HOST: &str = "127.0.0.1";
pub const DEFAULT_INBOX_PORT: u16 = 8766;

static INBOX_ITEMS: Lazy<Mutex<Vec<AppInboxItem>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInboxItem {
    pub id: String,
    pub request_type: String,
    pub context_scope: String,
    pub context_label: String,
    pub payload_id: Option<String>,
    pub summary: String,
    pub payload: serde_json::Value,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
struct InboxWireRequest {
    #[serde(default = "default_request_type")]
    request_type: String,
    #[serde(default = "default_context_scope")]
    context_scope: String,
    #[serde(default = "default_context_label")]
    context_label: String,
    payload_id: Option<String>,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    payload: serde_json::Value,
}

fn default_request_type() -> String {
    "analyze_context".to_string()
}

fn default_context_scope() -> String {
    "viewport".to_string()
}

fn default_context_label() -> String {
    "Viewport".to_string()
}

pub fn start_listener(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let addr = format!("{}:{}", DEFAULT_INBOX_HOST, DEFAULT_INBOX_PORT);
        let listener = match TcpListener::bind(&addr).await {
            Ok(listener) => listener,
            Err(e) => {
                log::warn!("DazPilot app inbox could not bind {}: {}", addr, e);
                return;
            },
        };
        log::info!("DazPilot app inbox listening on {}", addr);

        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                continue;
            };
            let app = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let mut buffer = Vec::new();
                if socket.read_to_end(&mut buffer).await.is_err() {
                    let _ = socket
                        .write_all(b"{\"status\":\"error\",\"error\":\"read failed\"}\n")
                        .await;
                    return;
                }
                let raw = String::from_utf8_lossy(&buffer);
                match accept_wire_request(raw.trim()) {
                    Ok(item) => {
                        {
                            let mut items = INBOX_ITEMS.lock().unwrap_or_else(|e| e.into_inner());
                            items.push(item.clone());
                            if items.len() > 100 {
                                let overflow = items.len() - 100;
                                items.drain(0..overflow);
                            }
                        }
                        let _ = app.emit("dazpilot-inbox-item", &item);
                        let response = serde_json::json!({
                            "status": "ok",
                            "id": item.id,
                        });
                        let _ = socket.write_all(format!("{}\n", response).as_bytes()).await;
                    },
                    Err(e) => {
                        let response = serde_json::json!({
                            "status": "error",
                            "error": e,
                        });
                        let _ = socket.write_all(format!("{}\n", response).as_bytes()).await;
                    },
                }
            });
        }
    });
}

fn accept_wire_request(raw: &str) -> Result<AppInboxItem, String> {
    let wire: InboxWireRequest =
        serde_json::from_str(raw).map_err(|e| format!("invalid inbox payload: {}", e))?;
    Ok(AppInboxItem {
        id: format!("inbox-{}", chrono::Utc::now().timestamp_millis()),
        request_type: wire.request_type,
        context_scope: wire.context_scope,
        context_label: wire.context_label,
        payload_id: wire.payload_id,
        summary: wire.summary,
        payload: wire.payload,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
pub fn get_dazpilot_inbox_items() -> Vec<AppInboxItem> {
    INBOX_ITEMS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

#[tauri::command]
pub fn clear_dazpilot_inbox_items() {
    INBOX_ITEMS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_minimal_payload_with_defaults() {
        let item = accept_wire_request(r#"{"summary":"hello"}"#).unwrap();
        assert_eq!(item.request_type, "analyze_context");
        assert_eq!(item.context_scope, "viewport");
        assert_eq!(item.context_label, "Viewport");
        assert_eq!(item.summary, "hello");
    }

    #[test]
    fn rejects_invalid_payload() {
        assert!(accept_wire_request("nope").is_err());
    }
}
