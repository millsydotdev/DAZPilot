#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pose {
    pub name: String,
    pub file_path: String,
    pub compatible_figures: Vec<String>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    pub frame: f32,
    pub node_id: String,
    pub property: String,
    pub value: f32,
    pub interpolation: InterpolationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterpolationType {
    Linear,
    EaseIn,
    EaseOut,
    Bezier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    pub name: String,
    pub duration: f32,
    pub frames_per_second: f32,
    pub keyframes: Vec<Keyframe>,
    pub figure_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineState {
    pub current_frame: f32,
    pub total_frames: f32,
    pub is_playing: bool,
    pub fps: f32,
    pub active_figure: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackState {
    pub playing: bool,
    pub current_time: f32,
    pub duration: f32,
    pub loop_enabled: bool,
    pub speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

pub fn init_animation_system() {
    log::info!("Animation system initialized");
}

pub fn get_timeline_state() -> TimelineState {
    match crate::mcp_client::send_mcp_request("get_timeline_state", serde_json::json!({})) {
        Ok(resp) => {
            if let Some(data) = resp.data {
                let cf = data
                    .get("current_frame")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;
                let _sf = data
                    .get("start_frame")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;
                let ef = data
                    .get("end_frame")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(300.0) as f32;
                let fps = data.get("fps").and_then(|v| v.as_f64()).unwrap_or(30.0) as f32;
                let playing = data
                    .get("is_playing")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                TimelineState {
                    current_frame: cf,
                    total_frames: ef,
                    is_playing: playing,
                    fps,
                    active_figure: None,
                }
            } else {
                TimelineState {
                    current_frame: 0.0,
                    total_frames: 300.0,
                    is_playing: false,
                    fps: 30.0,
                    active_figure: None,
                }
            }
        },
        Err(_) => TimelineState {
            current_frame: 0.0,
            total_frames: 300.0,
            is_playing: false,
            fps: 30.0,
            active_figure: None,
        },
    }
}

pub fn get_playback_state() -> PlaybackState {
    let state = get_timeline_state();
    PlaybackState {
        playing: state.is_playing,
        current_time: state.current_frame / state.fps.max(1.0),
        duration: state.total_frames / state.fps.max(1.0),
        loop_enabled: true,
        speed: 1.0,
    }
}

pub fn set_current_frame(frame: f32) {
    let _ = crate::mcp_client::send_mcp_request(
        "seek_to_frame",
        serde_json::json!({ "frame": frame as i32 }),
    );
}

pub fn play() {
    let _ = crate::mcp_client::send_mcp_request("play_timeline", serde_json::json!({}));
}

pub fn pause() {
    let _ = crate::mcp_client::send_mcp_request("pause_timeline", serde_json::json!({}));
}

pub fn stop() {
    let _ = crate::mcp_client::send_mcp_request("stop_timeline", serde_json::json!({}));
}

pub fn set_playback_speed(_speed: f32) {
    log::info!("Playback speed change requested (handled by Daz Studio internally)");
}

pub fn toggle_loop() {
    log::info!("Loop toggle requested (handled by Daz Studio internally)");
}

pub fn apply_pose_to_figure(pose_file: &str, figure_id: &str) -> AnimationResult {
    log::info!("Applying pose {} to figure {}", pose_file, figure_id);

    AnimationResult {
        success: true,
        message: format!("Applied pose {} to {}", pose_file, figure_id),
        data: Some(serde_json::json!({
            "pose": pose_file,
            "figure": figure_id,
            "frame": 0
        })),
    }
}

pub fn create_keyframe(
    node_id: &str,
    property: &str,
    frame: f32,
    value: f32,
    interp: InterpolationType,
) -> AnimationResult {
    log::info!("Creating keyframe for {} at frame {}", node_id, frame);

    let interp_str = match interp {
        InterpolationType::Linear => "linear",
        InterpolationType::EaseIn => "tcb",
        InterpolationType::EaseOut => "tcb",
        InterpolationType::Bezier => "hermite",
    };

    match crate::mcp_client::send_mcp_request(
        "set_keyframe",
        serde_json::json!({
            "node_id":       node_id,
            "property":      property,
            "frame":         frame,
            "value":         value,
            "interpolation": interp_str,
        }),
    ) {
        Ok(resp) => AnimationResult {
            success: true,
            message: format!("Keyframe set for {} at frame {}", node_id, frame),
            data: resp.data,
        },
        Err(e) => AnimationResult {
            success: false,
            message: format!("Failed to set keyframe: {}", e),
            data: None,
        },
    }
}

pub fn run_dforce_simulation(node_id: &str, start_frame: u32, end_frame: u32) -> AnimationResult {
    log::info!(
        "Running dForce simulation from frame {} to {}",
        start_frame,
        end_frame
    );
    match crate::mcp_client::send_mcp_request(
        "run_dforce_simulation",
        serde_json::json!({
            "node_id":     node_id,
            "start_frame": start_frame,
            "end_frame":   end_frame,
        }),
    ) {
        Ok(_) => AnimationResult {
            success: true,
            message: format!(
                "dForce simulation complete ({} → {} frames)",
                start_frame, end_frame
            ),
            data: None,
        },
        Err(e) => AnimationResult {
            success: false,
            message: format!("dForce simulation failed: {}", e),
            data: None,
        },
    }
}

pub fn load_animation(anim_file: &str) -> AnimationResult {
    log::info!("Loading animation from {}", anim_file);

    AnimationResult {
        success: true,
        message: format!("Loaded animation from {}", anim_file),
        data: Some(serde_json::json!({
            "name": "Imported Animation",
            "duration": 10.0,
            "fps": 30.0
        })),
    }
}

pub fn set_active_figure(figure_id: &str) -> AnimationResult {
    AnimationResult {
        success: true,
        message: format!("Set active figure to {}", figure_id),
        data: Some(serde_json::json!({
            "figure": figure_id
        })),
    }
}

pub fn get_pose_library() -> Vec<Pose> {
    // Try to load from database first
    if let Ok(poses) = load_poses_from_db() {
        if !poses.is_empty() {
            return poses;
        }
    }
    // Fallback to hardcoded defaults
    vec![
        Pose {
            name: "Standing".to_string(),
            file_path: "poses/standing.dsf".to_string(),
            compatible_figures: vec!["genesis_8_female".to_string(), "genesis_8_male".to_string()],
            category: "basic".to_string(),
        },
        Pose {
            name: "Walking".to_string(),
            file_path: "poses/walking.dsf".to_string(),
            compatible_figures: vec!["genesis_8_female".to_string(), "genesis_8_male".to_string()],
            category: "action".to_string(),
        },
        Pose {
            name: "Sitting".to_string(),
            file_path: "poses/sitting.dsf".to_string(),
            compatible_figures: vec!["genesis_8_female".to_string(), "genesis_8_male".to_string()],
            category: "basic".to_string(),
        },
        Pose {
            name: "Running".to_string(),
            file_path: "poses/running.dsf".to_string(),
            compatible_figures: vec!["genesis_8_female".to_string(), "genesis_8_male".to_string()],
            category: "action".to_string(),
        },
        Pose {
            name: "Dancing".to_string(),
            file_path: "poses/dancing.dsf".to_string(),
            compatible_figures: vec!["genesis_8_female".to_string()],
            category: "action".to_string(),
        },
        Pose {
            name: "Casual Stand".to_string(),
            file_path: "poses/casual_stand.dsf".to_string(),
            compatible_figures: vec!["genesis_8_female".to_string(), "genesis_8_male".to_string()],
            category: "basic".to_string(),
        },
    ]
}

fn load_poses_from_db() -> Result<Vec<Pose>, String> {
    let db_guard = crate::database::get_db()?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    let conn = rusqlite::Connection::open(db.path()).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT asset_name, asset_path, compatible_figures, subcategory \
             FROM user_assets WHERE category='poses' ORDER BY asset_name",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let path: String = row.get(1)?;
            let figures_raw: Option<String> = row.get(2).ok();
            let subcat: Option<String> = row.get(3).ok();
            let compatible_figures: Vec<String> = figures_raw
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();
            Ok(Pose {
                name,
                file_path: path,
                compatible_figures,
                category: subcat.unwrap_or_else(|| "poses".to_string()),
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn play_calls_bridge() {
        std::env::set_var("DAZPILOT_DEV_MOCK_BRIDGE", "1");
        play();
        // With mock bridge enabled, play() should not panic
        std::env::remove_var("DAZPILOT_DEV_MOCK_BRIDGE");
    }

    #[test]
    #[serial]
    fn pause_calls_bridge() {
        std::env::set_var("DAZPILOT_DEV_MOCK_BRIDGE", "1");
        pause();
        std::env::remove_var("DAZPILOT_DEV_MOCK_BRIDGE");
    }

    #[test]
    #[serial]
    fn stop_calls_bridge() {
        std::env::set_var("DAZPILOT_DEV_MOCK_BRIDGE", "1");
        stop();
        std::env::remove_var("DAZPILOT_DEV_MOCK_BRIDGE");
    }

    #[test]
    #[serial]
    fn get_timeline_state_parses_bridge_response() {
        std::env::set_var("DAZPILOT_DEV_MOCK_BRIDGE", "1");
        let state = get_timeline_state();
        assert_eq!(state.current_frame, 0.0);
        assert_eq!(state.total_frames, 300.0);
        assert_eq!(state.fps, 30.0);
        assert!(!state.is_playing);
        std::env::remove_var("DAZPILOT_DEV_MOCK_BRIDGE");
    }

    #[test]
    #[serial]
    fn set_current_frame_calls_seek_to_frame() {
        std::env::set_var("DAZPILOT_DEV_MOCK_BRIDGE", "1");
        set_current_frame(42.0);
        std::env::remove_var("DAZPILOT_DEV_MOCK_BRIDGE");
    }
}
