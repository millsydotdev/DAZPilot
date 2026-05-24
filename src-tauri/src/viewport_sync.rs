use crate::mcp_client;
use serde_json::json;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

pub static BRIDGE_CONNECTED: AtomicBool = AtomicBool::new(false);

pub struct ViewportSyncState {
    pub enabled: Mutex<bool>,
    pub fps: Mutex<u32>,
}

pub fn init_viewport_sync(app: &AppHandle) {
    let app_handle = app.clone();
    let state = app_handle.state::<Arc<ViewportSyncState>>().inner().clone();

    std::thread::spawn(move || {
        let mut consecutive_failures: u32 = 0;

        loop {
            let is_enabled = *state.enabled.lock().unwrap();
            let fps = *state.fps.lock().unwrap();

            if is_enabled {
                match mcp_client::send_mcp_request("capture_viewport", json!({ "path": "stream" }))
                {
                    Ok(resp) => {
                        consecutive_failures = 0;
                        BRIDGE_CONNECTED.store(true, Ordering::SeqCst);

                        if let Some(data) = resp
                            .data
                            .as_ref()
                            .and_then(|d| d.get("data"))
                            .and_then(|v| v.as_str())
                        {
                            app_handle
                                .emit("viewport-update", json!({ "image": data }))
                                .ok();
                        }
                    },
                    Err(e) => {
                        consecutive_failures += 1;
                        BRIDGE_CONNECTED.store(false, Ordering::SeqCst);

                        if consecutive_failures == 1 || consecutive_failures % 10 == 0 {
                            app_handle
                                .emit(
                                    "viewport-error",
                                    json!({
                                        "error": e,
                                        "failures": consecutive_failures
                                    }),
                                )
                                .ok();
                        }
                    },
                }
            } else {
                consecutive_failures = 0;
            }

            let sleep_ms = 1000 / fps.max(1);
            std::thread::sleep(Duration::from_millis(sleep_ms as u64));
        }
    });
}
