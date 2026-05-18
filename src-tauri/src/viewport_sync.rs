use tauri::{AppHandle, Manager, Emitter};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::mcp_client;
use serde_json::json;

pub struct ViewportSyncState {
    pub enabled: Mutex<bool>,
    pub fps: Mutex<u32>,
}

pub fn init_viewport_sync(app: &AppHandle) {
    let app_handle = app.clone();
    let state = app_handle.state::<Arc<ViewportSyncState>>().inner().clone();

    std::thread::spawn(move || {
        loop {
            let is_enabled = *state.enabled.lock().unwrap();
            let fps = *state.fps.lock().unwrap();

            if is_enabled {
                if let Ok(resp) = mcp_client::send_mcp_request("capture_viewport", json!({ "path": "stream" })) {
                    if let Some(data) = resp.data.as_ref()
                        .and_then(|d| d.get("data"))
                        .and_then(|v| v.as_str()) {
                        app_handle.emit("viewport-update", json!({ "image": data })).ok();
                    }
                }
            }

            let sleep_ms = 1000 / fps.max(1);
            std::thread::sleep(Duration::from_millis(sleep_ms as u64));
        }
    });
}
