pub mod core;
pub mod mcp_client;
pub mod ai_service;
pub mod database;
pub mod agents;
pub mod library_scanner;
pub mod animation;
pub mod physics;
pub mod ai_system;
pub mod advanced;
pub mod ollama_service;
pub mod sdk_indexer;
pub mod ai_action;
pub mod local_ai;
pub mod local_ai_cmd;
pub mod model_download;
pub mod vision_service;
pub mod viewport_sync;
pub mod ai_providers;
pub mod asset_fixer;

use ollama_service::{check_ollama_status, get_ollama_models, pull_ollama_model, ollama_chat};
use asset_fixer::{
    ConflictScanResult, AssetFixResult, ShellInfo
};

use tauri::Manager;
use tauri::Emitter;
use log::info;
use std::sync::Mutex;

static DB_INITIALIZED: Mutex<bool> = Mutex::new(false);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(std::sync::Arc::new(viewport_sync::ViewportSyncState {
            enabled: std::sync::Mutex::new(false),
            fps: std::sync::Mutex::new(2),
        }))
        .setup(|app| {
            info!("DazPilot App starting...");
            
            // Initialize database
            if let Ok(app_data) = app.path().app_data_dir() {
                std::fs::create_dir_all(&app_data).ok();
                match database::init_database(&app_data) {
                    Ok(_) => {
                        info!("Database initialized successfully");
                        *DB_INITIALIZED.lock().unwrap() = true;
                    }
                    Err(e) => {
                        log::error!("Failed to initialize database: {}", e);
                    }
                }
            } else {
                log::warn!("Could not get app data directory");
            }
            
            if let Some(window) = app.get_webview_window("main") {
                info!("Main window created successfully");
                window.set_title("DazPilot").ok();
            }

            match ai_service::init_ai_service(ai_service::AiBackend::LocalLlamaCpp) {
                Ok(_) => info!("AI service initialized with local GGUF backend"),
                Err(e) => log::warn!("AI service init failed (non-fatal, first launch?): {}", e),
            }
            
            animation::init_animation_system();
            info!("Animation system initialized");
            
            viewport_sync::init_viewport_sync(app.handle());
            info!("Viewport sync loop started");
            
            info!("App setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_version,
            check_connection_status,
            mcp_client::connect_to_daz3d,
            mcp_client::disconnect_from_daz3d,
            get_mcp_commands,
            mcp_client::check_daz3d_connection_status,
            mcp_client::send_daz3d_command,
            execute_command,
            send_chat_message,
            chat_with_ai,
            get_ai_status,
            install_daz3d_plugin,
            get_plugin_status,
            get_app_setting,
            save_app_setting,
            select_directory,
            select_plugins_directory,
            download_and_install_plugin,
            execute_agent,
            get_content_paths,
            add_custom_content_path,
            remove_custom_content_path,
            scan_library,
            get_assets_by_category,
            search_assets,
            load_asset_in_daz,
            toggle_favourite,
            get_favourites,
            get_timeline_state,
            get_playback_state,
            play_animation,
            pause_animation,
            stop_animation,
            seek_to_frame,
            set_playback_speed,
            toggle_loop,
            apply_pose,
            get_pose_library,
            set_active_figure,
            get_physics_settings,
            enable_dforce,
            set_simulation_quality,
            run_simulation,
            stop_simulation,
            get_collision_zones,
            add_collision_zone,
            create_particle_system,
            start_particle_emission,
            stop_particle_emission,
            bake_to_keyframes,
            parse_natural_language,
            map_phrase_to_command,
            learn_phrase,
            get_phrase_mappings,
            build_user_profile,
            build_scene_context,
            execute_ai_command,
            get_ai_capabilities,
            get_default_import_settings,
            import_model,
            get_default_export_settings,
            ai_system::get_session_summary,
            export_scene,
            batch_export,
            create_scene_sequence,
            create_scene_composition,
            add_camera_action,
            add_transition,
            get_supported_import_formats,
            get_supported_export_formats,
            get_export_presets,
            get_scene_info,
            list_nodes,
            check_ollama_status,
            get_ollama_models,
            pull_ollama_model,
            ollama_chat,
            process_chat_message,
            get_provider_models,
            test_ai_connection,
            sdk_indexer::get_deep_sdk_index,
            sdk_indexer::get_sdk_class_deep,
            sdk_indexer::search_sdk_deep,
            sdk_indexer::get_sdk_method_help,
            sdk_indexer::get_sdk_related_classes,
            sdk_indexer::get_all_sdk_classes,
            sdk_indexer::get_sdk_indexer_status,
            sdk_indexer::set_sdk_indexer_path,
            ai_action::parse_ai_action,
            ai_action::execute_ai_action,
            ai_action::generate_scene_prompt,
            local_ai_cmd::start_local_server,
            local_ai_cmd::stop_local_server,
            local_ai_cmd::chat_with_local,
            local_ai_cmd::is_local_server_running,
            local_ai_cmd::list_local_models,
            local_ai_cmd::get_models_dir,
            model_download::download_gguf_model,
            set_viewport_sync,
            set_viewport_fps,
            scan_conflicts,
            fix_shell_zones,
            auto_fix_all_conflicts,
            analyze_shell_file,
            ai_ask_question,
            get_asset_conflicts,
            execute_approved_script
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_app_version() -> String {
    "0.1.0".to_string()
}

#[tauri::command]
fn check_connection_status() -> String {
    mcp_client::check_connection_status()
}

#[tauri::command]
fn get_mcp_commands() -> Result<Vec<mcp_client::McpCommand>, String> {
    Ok(mcp_client::get_mcp_command_list())
}

#[tauri::command]
async fn execute_command(command: String, args: serde_json::Value) -> Result<mcp_client::McpResponse, String> {
    mcp_client::send_mcp_request(&command, args)
}

#[tauri::command]
async fn send_chat_message(message: String) -> Result<String, String> {
    let response = ai_service::chat(message)?;
    Ok(response.content)
}

#[tauri::command]
async fn chat_with_ai(prompt: String) -> Result<ai_service::ChatResponse, String> {
    ai_service::chat(prompt)
}

#[tauri::command]
fn get_ai_status() -> ai_service::ModelInfo {
    ai_service::get_model_info().unwrap_or(ai_service::ModelInfo {
        name: "phi-2-q4".to_string(),
        size: 0,
        loaded: true,
    })
}

fn bridge_plugin_filename() -> &'static str {
    if cfg!(target_os = "windows") {
        "DazPilotBridge.dll"
    } else if cfg!(target_os = "macos") {
        "libDazPilotBridge.dylib"
    } else {
        "libDazPilotBridge.so"
    }
}

fn default_daz_path() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    let daz_candidates = vec![
        std::path::PathBuf::from(r"C:\Program Files\DAZ 3D\DAZStudio4\plugins"),
        std::path::PathBuf::from(r"C:\Program Files (x86)\DAZ 3D\DAZStudio4\plugins"),
    ];

    #[cfg(target_os = "macos")]
    let daz_candidates = vec![
        dirs::home_dir()
            .unwrap_or_default()
            .join("Library/Application Support/DAZ 3D/Studio4/plugins"),
        std::path::PathBuf::from("/Applications/DAZ 3D/DAZStudio4/plugins"),
    ];

    #[cfg(all(unix, not(target_os = "macos")))]
    let daz_candidates = vec![
        dirs::home_dir()
            .unwrap_or_default()
            .join(".local/share/DAZ 3D/Studio4/plugins"),
        std::path::PathBuf::from("/opt/DAZ 3D/DAZStudio4/plugins"),
    ];

    daz_candidates
        .iter()
        .find(|p| p.parent().map(|pp| pp.exists()).unwrap_or(false))
        .cloned()
        .unwrap_or_else(|| daz_candidates[0].clone())
}

#[tauri::command]
fn install_daz3d_plugin(custom_path: Option<String>) -> Result<String, String> {
    let plugin_name = bridge_plugin_filename();
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {}", e))?
        .parent()
        .ok_or("Failed to get parent directory")?
        .to_path_buf();

    let mut candidate_paths = vec![exe_dir.join(plugin_name)];
    #[cfg(target_os = "macos")]
    if exe_dir.ends_with("MacOS") {
        if let Some(contents_dir) = exe_dir.parent() {
            candidate_paths.push(contents_dir.join("Resources").join(plugin_name));
        }
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let repo_root = std::path::PathBuf::from(manifest_dir)
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_default();
        candidate_paths.push(repo_root.join("src-tauri").join("resources").join(plugin_name));
        candidate_paths.push(repo_root.join("plugins").join("daz3d-bridge").join("dist").join(plugin_name));
        candidate_paths.push(repo_root.join("plugins").join("daz3d-bridge").join("dist").join("Release").join(plugin_name));
    }

    let resource_path = candidate_paths
        .iter()
        .find(|p| p.exists())
        .ok_or_else(|| format!("{} not found. Build or download the bridge plugin first.", plugin_name))?;

    let daz_plugins_path = if let Some(ref path) = custom_path {
        if !path.is_empty() {
            std::path::PathBuf::from(path)
        } else {
            default_daz_path()
        }
    } else {
        default_daz_path()
    };

    let dest_path = daz_plugins_path.join(plugin_name);

    std::fs::create_dir_all(&daz_plugins_path)
        .map_err(|e| format!("Failed to create plugins directory: {}", e))?;

    std::fs::copy(resource_path, &dest_path)
        .map_err(|e| format!("Failed to copy plugin: {}", e))?;

    info!("Plugin installed to Daz3D plugins folder: {}", dest_path.display());
    Ok(format!("Plugin installed to: {}", dest_path.display()))
}

#[tauri::command]
fn get_plugin_status(custom_path: Option<String>) -> Result<String, String> {
    let plugin_name = bridge_plugin_filename();
    let mut daz_candidates = vec![default_daz_path().join(plugin_name)];

    #[cfg(target_os = "windows")]
    daz_candidates.push(std::path::PathBuf::from(r"C:\Program Files (x86)\DAZ 3D\DAZStudio4\plugins").join(plugin_name));

    if let Some(ref path) = custom_path {
        if !path.is_empty() {
            let path_buf = std::path::PathBuf::from(path);
            let plugin_path = if path_buf.is_file() {
                path_buf
            } else {
                path_buf.join(plugin_name)
            };
            daz_candidates.insert(0, plugin_path);
        }
    }

    if daz_candidates.iter().any(|p| p.exists()) {
        Ok("installed".to_string())
    } else {
        Ok("not_installed".to_string())
    }
}

#[tauri::command]
fn get_app_setting(key: String) -> Result<Option<String>, String> {
    database::get_setting(&key)
}

#[tauri::command]
fn save_app_setting(app: tauri::AppHandle, key: String, value: String) -> Result<(), String> {
    database::save_setting(&key, &value)?;
    
    if key == "daz_bridge_port" || key == "daz_bridge_host" {
        let port_str = database::get_setting("daz_bridge_port")?
            .unwrap_or_else(|| "8765".to_string());
        let host_str = database::get_setting("daz_bridge_host")?
            .unwrap_or_else(|| "127.0.0.1".to_string());
            
        if let Ok(app_data) = app.path().app_data_dir() {
            std::fs::create_dir_all(&app_data).ok();
            let config_path = app_data.join("bridge_config.json");
            let config_json = serde_json::json!({
                "host": host_str,
                "port": port_str.parse::<u16>().unwrap_or(8765)
            });
            if let Ok(config_str) = serde_json::to_string_pretty(&config_json) {
                if let Err(e) = std::fs::write(&config_path, config_str) {
                    log::error!("Failed to write bridge_config.json: {}", e);
                } else {
                    log::info!("Successfully synchronized connection config with C++ plugin at {:?}", config_path);
                }
            }
        }
    }
    
    Ok(())
}

#[tauri::command]
fn select_directory(title: String) -> Result<Option<String>, String> {
    let folder = rfd::FileDialog::new()
        .set_title(&title)
        .pick_folder();
    
    Ok(folder.map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
fn select_plugins_directory() -> Result<Option<String>, String> {
    let folder = rfd::FileDialog::new()
        .set_title("Select Daz Studio Plugins Directory")
        .pick_folder();
    
    Ok(folder.map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
fn download_and_install_plugin(app: tauri::AppHandle, custom_path: Option<String>) -> Result<String, String> {
    let plugin_name = bridge_plugin_filename();
    let daz_plugins_path = if let Some(ref path) = custom_path {
        if !path.is_empty() {
            std::path::PathBuf::from(path)
        } else {
            default_daz_path()
        }
    } else {
        default_daz_path()
    };

    let dest_path = daz_plugins_path.join(plugin_name);
    
    std::fs::create_dir_all(&daz_plugins_path)
        .map_err(|e| format!("Failed to create plugins directory: {}", e))?;

    let dest_str = dest_path.to_string_lossy().to_string();

    let url = format!(
        "https://github.com/millsydotdev/DazPilot/releases/latest/download/{}",
        plugin_name
    );
    
    crate::model_download::download_model(&app, &url, &dest_str)?;

    info!("Plugin downloaded and installed to Daz3D plugins folder: {}", dest_path.display());
    Ok(format!("Plugin successfully installed to: {}", dest_path.display()))
}

#[tauri::command]
fn execute_agent(agent_type: String, input: String) -> Result<agents::AgentResponse, String> {
    let request = agents::AgentRequest {
        agent_type,
        input,
        context: None,
    };
    Ok(agents::execute_agent(request))
}

#[tauri::command]
fn get_scene_info() -> Result<serde_json::Value, String> {
    mcp_client::send_mcp_request("get_scene_info", serde_json::json!({}))
        .map(|r| r.data.unwrap_or_else(|| serde_json::json!({})))
}

#[tauri::command]
fn list_nodes() -> Result<Vec<serde_json::Value>, String> {
    let data = mcp_client::send_mcp_request("list_nodes", serde_json::json!({}))?
        .data
        .unwrap_or_else(|| serde_json::json!({ "nodes": [] }));
    Ok(data
        .get("nodes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default())
}

#[tauri::command]
fn get_content_paths() -> Result<Vec<library_scanner::ContentPath>, String> {
    let mut paths = library_scanner::get_default_content_paths();
    
    let db_guard = database::get_db()?;
    if let Some(ref db) = *db_guard {
        let conn = rusqlite::Connection::open(db.path()).map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, source_path, source_name FROM content_sources WHERE source_type = 'custom'")
            .map_err(|e| e.to_string())?;
        
        let custom_rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let path: String = row.get(1)?;
            let name: String = row.get(2).unwrap_or_else(|_| "".to_string());
            
            Ok(library_scanner::ContentPath {
                id: Some(id),
                path,
                name: if name.is_empty() { "Custom Library".to_string() } else { name },
                is_default: false,
            })
        }).map_err(|e| e.to_string())?;
        
        for row in custom_rows.flatten() {
            paths.push(row);
        }
    }
    
    Ok(paths)
}

#[tauri::command]
fn add_custom_content_path(path: String, name: String) -> Result<i64, String> {
    let db_guard = database::get_db()?;
    if let Some(ref db) = *db_guard {
        let conn = rusqlite::Connection::open(db.path()).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO content_sources (user_id, source_path, source_type, source_name) VALUES ('default', ?1, 'custom', ?2)",
            rusqlite::params![path, name],
        ).map_err(|e| e.to_string())?;
        let row_id = conn.last_insert_rowid();
        Ok(row_id)
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
fn remove_custom_content_path(id: i64) -> Result<(), String> {
    let db_guard = database::get_db()?;
    if let Some(ref db) = *db_guard {
        let conn = rusqlite::Connection::open(db.path()).map_err(|e| e.to_string())?;
        let _ = conn.execute("DELETE FROM user_assets WHERE source_id = ?1", rusqlite::params![id]);
        conn.execute("DELETE FROM content_sources WHERE id = ?1", rusqlite::params![id]).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
fn scan_library(paths: Vec<String>) -> library_scanner::ScanResult {
    // Clear default user assets from database before scanning so we don't keep stale files
    if let Ok(db_guard) = crate::database::get_db() {
        if let Some(db) = db_guard.as_ref() {
            if let Ok(conn) = rusqlite::Connection::open(db.path()) {
                let _ = conn.execute("DELETE FROM user_assets WHERE user_id='default'", []);
            }
        }
    }

    if paths.is_empty() {
        let default_paths = library_scanner::get_default_content_paths();
        let path_strings: Vec<String> = default_paths.into_iter().map(|p| p.path).collect();
        library_scanner::scan_multiple_paths(&path_strings)
    } else {
        library_scanner::scan_multiple_paths(&paths)
    }
}

#[tauri::command]
fn get_assets_by_category(category: String) -> Vec<library_scanner::AssetInfo> {
    // Try DB first; fall back to a live scan if DB is empty
    if let Ok(assets) = search_assets_in_db(Some(category.clone()), None) {
        if !assets.is_empty() {
            return assets;
        }
    }
    let default_paths = library_scanner::get_default_content_paths();
    let path_strings: Vec<String> = default_paths.into_iter().map(|p| p.path).collect();
    let result = library_scanner::scan_multiple_paths(&path_strings);
    match category.as_str() {
        "figures" => result.categorized.figures,
        "clothing" => result.categorized.clothing,
        "hair" => result.categorized.hair,
        "poses" => result.categorized.poses,
        "materials" => result.categorized.materials,
        "morphs" => result.categorized.morphs,
        "environments" => result.categorized.environments,
        "lights" => result.categorized.lights,
        "cameras" => result.categorized.cameras,
        "animations" => result.categorized.animations,
        _ => result.categorized.other,
    }
}

pub fn format_fts_query(query: &str) -> String {
    let mut fts_query = String::new();
    for word in query.split_whitespace() {
        let clean: String = word.chars().filter(|&c| c.is_alphanumeric() || c == '_' || c == '-').collect();
        if !clean.is_empty() {
            if !fts_query.is_empty() {
                fts_query.push_str(" AND ");
            }
            fts_query.push_str(&format!("\"{}*\"", clean));
        }
    }
    fts_query
}

fn search_assets_in_db(
    category: Option<String>,
    query: Option<String>,
) -> Result<Vec<library_scanner::AssetInfo>, String> {
    let guard = database::get_db()?;
    let db = guard.as_ref().ok_or("Database not initialised")?;
    let conn = rusqlite::Connection::open(db.path()).map_err(|e| e.to_string())?;

    let (sql, params_vec): (String, Vec<String>) = match (category.as_deref(), query.as_deref()) {
        (Some(cat), Some(q)) if cat != "all" => {
            let fts = format_fts_query(q);
            if fts.is_empty() {
                (
                    "SELECT asset_path, asset_name, file_type, file_size, category, subcategory FROM user_assets WHERE user_id='default' AND category=? ORDER BY asset_name LIMIT 500".into(),
                    vec![cat.to_string()],
                )
            } else {
                (
                    "SELECT asset_path, asset_name, file_type, file_size, category, subcategory FROM user_assets JOIN user_assets_fts ON user_assets.id = user_assets_fts.rowid WHERE user_assets.user_id='default' AND user_assets.category=? AND user_assets_fts MATCH ? ORDER BY bm25(user_assets_fts) LIMIT 500".into(),
                    vec![cat.to_string(), fts],
                )
            }
        }
        (Some(cat), None) if cat != "all" => (
            "SELECT asset_path, asset_name, file_type, file_size, category, subcategory FROM user_assets WHERE user_id='default' AND category=? ORDER BY asset_name LIMIT 500".into(),
            vec![cat.to_string()],
        ),
        (_, Some(q)) => {
            let fts = format_fts_query(q);
            if fts.is_empty() {
                (
                    "SELECT asset_path, asset_name, file_type, file_size, category, subcategory FROM user_assets WHERE user_id='default' ORDER BY asset_name LIMIT 500".into(),
                    vec![],
                )
            } else {
                (
                    "SELECT asset_path, asset_name, file_type, file_size, category, subcategory FROM user_assets JOIN user_assets_fts ON user_assets.id = user_assets_fts.rowid WHERE user_assets.user_id='default' AND user_assets_fts MATCH ? ORDER BY bm25(user_assets_fts) LIMIT 500".into(),
                    vec![fts],
                )
            }
        }
        _ => (
            "SELECT asset_path, asset_name, file_type, file_size, category, subcategory FROM user_assets WHERE user_id='default' ORDER BY asset_name LIMIT 500".into(),
            vec![],
        ),
    };

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let assets = stmt
        .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok(library_scanner::AssetInfo {
                path: row.get(0)?,
                name: row.get(1)?,
                file_type: row.get::<_, String>(2).unwrap_or_default(),
                size: row.get::<_, i64>(3).unwrap_or(0) as u64,
                category: row.get::<_, String>(4).unwrap_or_default(),
                subcategory: row.get(5)?,
                metadata: std::collections::HashMap::new(),
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(assets)
}

#[tauri::command]
fn search_assets(query: String, category: String) -> Result<Vec<library_scanner::AssetInfo>, String> {
    let cat = if category == "all" { None } else { Some(category) };
    let q = if query.is_empty() { None } else { Some(query) };
    search_assets_in_db(cat, q)
}

#[tauri::command]
fn load_asset_in_daz(path: String) -> Result<String, String> {
    mcp_client::send_mcp_request("load_asset", serde_json::json!({ "path": path }))
        .map(|r| r.result.unwrap_or_else(|| "Asset load requested".to_string()))
}

#[tauri::command]
fn toggle_favourite(asset_path: String) -> Result<bool, String> {
    let guard = database::get_db()?;
    let db = guard.as_ref().ok_or("Database not initialised")?;
    let conn = rusqlite::Connection::open(db.path()).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS favourites (
            user_id TEXT NOT NULL, asset_path TEXT NOT NULL,
            added_at TEXT DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (user_id, asset_path));",
    ).map_err(|e| e.to_string())?;
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM favourites WHERE user_id='default' AND asset_path=?1",
            rusqlite::params![asset_path],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) > 0;
    if exists {
        conn.execute("DELETE FROM favourites WHERE user_id='default' AND asset_path=?1", rusqlite::params![asset_path])
            .map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        conn.execute("INSERT OR IGNORE INTO favourites (user_id, asset_path) VALUES ('default', ?1)", rusqlite::params![asset_path])
            .map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
fn get_favourites() -> Result<Vec<String>, String> {
    let guard = database::get_db()?;
    let db = guard.as_ref().ok_or("Database not initialised")?;
    let conn = rusqlite::Connection::open(db.path()).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS favourites (
            user_id TEXT NOT NULL, asset_path TEXT NOT NULL,
            added_at TEXT DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (user_id, asset_path));",
    ).ok();
    let mut stmt = conn
        .prepare("SELECT asset_path FROM favourites WHERE user_id='default' ORDER BY added_at DESC")
        .map_err(|e| e.to_string())?;
    let paths = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(paths)
}


#[tauri::command]
fn get_timeline_state() -> animation::TimelineState {
    animation::get_timeline_state()
}

#[tauri::command]
fn get_playback_state() -> animation::PlaybackState {
    animation::get_playback_state()
}

#[tauri::command]
fn play_animation() -> animation::AnimationResult {
    animation::play();
    animation::AnimationResult {
        success: true,
        message: "Playback started".to_string(),
        data: None,
    }
}

#[tauri::command]
fn pause_animation() -> animation::AnimationResult {
    animation::pause();
    animation::AnimationResult {
        success: true,
        message: "Playback paused".to_string(),
        data: None,
    }
}

#[tauri::command]
fn stop_animation() -> animation::AnimationResult {
    animation::stop();
    animation::AnimationResult {
        success: true,
        message: "Playback stopped".to_string(),
        data: None,
    }
}

#[tauri::command]
fn seek_to_frame(frame: f32) -> animation::AnimationResult {
    animation::set_current_frame(frame);
    animation::AnimationResult {
        success: true,
        message: format!("Seeked to frame {}", frame),
        data: None,
    }
}

#[tauri::command]
fn set_playback_speed(speed: f32) -> animation::AnimationResult {
    animation::set_playback_speed(speed);
    animation::AnimationResult {
        success: true,
        message: format!("Speed set to {}", speed),
        data: None,
    }
}

#[tauri::command]
fn toggle_loop() -> animation::AnimationResult {
    animation::toggle_loop();
    animation::AnimationResult {
        success: true,
        message: "Loop toggled".to_string(),
        data: Some(serde_json::json!(animation::get_playback_state().loop_enabled)),
    }
}

#[tauri::command]
fn apply_pose(pose_file: String, figure_id: String) -> animation::AnimationResult {
    animation::apply_pose_to_figure(&pose_file, &figure_id)
}

#[tauri::command]
fn get_pose_library() -> Vec<animation::Pose> {
    animation::get_pose_library()
}

#[tauri::command]
fn set_active_figure(figure_id: String) -> animation::AnimationResult {
    animation::set_active_figure(&figure_id)
}

#[tauri::command]
fn get_physics_settings() -> physics::PhysicsSettings {
    physics::get_default_physics_settings()
}

#[tauri::command]
fn enable_dforce(node_id: String, enabled: bool) -> physics::SimulationResult {
    physics::enable_dforce(&node_id, enabled)
}

#[tauri::command]
fn set_simulation_quality(quality: String) -> physics::SimulationResult {
    let q = match quality.as_str() {
        "preview" => physics::SimulationQuality::Preview,
        "medium" => physics::SimulationQuality::Medium,
        "high" => physics::SimulationQuality::High,
        "ultra" => physics::SimulationQuality::Ultra,
        _ => physics::SimulationQuality::Medium,
    };
    physics::set_simulation_quality(q)
}

#[tauri::command]
fn run_simulation(start_frame: u32, end_frame: u32) -> physics::SimulationResult {
    physics::run_simulation(start_frame, end_frame)
}

#[tauri::command]
fn stop_simulation() -> physics::SimulationResult {
    physics::stop_simulation()
}

#[tauri::command]
fn get_collision_zones() -> Vec<physics::CollisionZone> {
    physics::get_default_collision_zones()
}

#[tauri::command]
fn add_collision_zone(zone: physics::CollisionZone) -> physics::SimulationResult {
    physics::add_collision_zone(zone)
}

#[tauri::command]
fn create_particle_system(name: String, emitter: String) -> physics::ParticleSystem {
    physics::create_particle_system(&name, &emitter)
}

#[tauri::command]
fn start_particle_emission(system: physics::ParticleSystem) -> physics::SimulationResult {
    physics::start_particle_emission(&system)
}

#[tauri::command]
fn stop_particle_emission(system_name: String) -> physics::SimulationResult {
    physics::stop_particle_emission(&system_name)
}

#[tauri::command]
fn bake_to_keyframes(start_frame: u32, end_frame: u32) -> physics::BakeResult {
    physics::bake_to_keyframes(start_frame, end_frame)
}

#[tauri::command]
fn parse_natural_language(input: String) -> ai_system::ParsedCommand {
    ai_system::parse_natural_language(&input)
}

#[tauri::command]
fn map_phrase_to_command(phrase: String) -> Option<String> {
    ai_system::map_phrase_to_command(&phrase)
}

#[tauri::command]
fn learn_phrase(phrase: String, command: String, category: String) {
    ai_system::learn_phrase(&phrase, &command, &category)
}

#[tauri::command]
fn get_phrase_mappings() -> Vec<ai_system::PhraseMapping> {
    ai_system::get_phrase_mappings()
}

#[tauri::command]
fn build_user_profile(user_id: String) -> ai_system::UserProfile {
    ai_system::build_user_profile(&user_id)
}

#[tauri::command]
fn build_scene_context() -> ai_system::SceneContext {
    ai_system::build_scene_context()
}

#[tauri::command]
fn execute_ai_command(parsed: ai_system::ParsedCommand) -> ai_system::AiResponse {
    ai_system::execute_ai_command(parsed)
}

#[tauri::command]
fn get_ai_capabilities() -> Vec<String> {
    ai_system::get_ai_capabilities()
}

#[tauri::command]
fn get_default_import_settings() -> advanced::ImportSettings {
    advanced::get_default_import_settings()
}

#[tauri::command]
fn import_model(path: String, settings: advanced::ImportSettings) -> advanced::ImportResult {
    advanced::import_model(&path, settings)
}

#[tauri::command]
fn get_default_export_settings() -> advanced::ExportSettings {
    advanced::get_default_export_settings()
}

#[tauri::command]
fn export_scene(node_id: String, path: String, settings: advanced::ExportSettings) -> advanced::ExportResult {
    advanced::export_scene(&node_id, &path, settings)
}

#[tauri::command]
fn batch_export(batch: advanced::BatchExport) -> advanced::BatchResult {
    advanced::batch_export(batch)
}

#[tauri::command]
fn create_scene_sequence(name: String) -> advanced::SceneSequence {
    advanced::create_scene_sequence(&name)
}

#[tauri::command]
fn create_scene_composition(name: String, description: String) -> advanced::SceneComposition {
    advanced::create_scene_composition(&name, &description)
}

#[tauri::command]
fn add_camera_action(mut composition: advanced::SceneComposition, action: advanced::CameraAction) -> advanced::SceneComposition {
    advanced::add_camera_action(&mut composition, action);
    composition
}

#[tauri::command]
fn add_transition(mut composition: advanced::SceneComposition, from_seq: String, to_seq: String, trans_type: advanced::TransitionType, duration: f32) -> advanced::SceneComposition {
    advanced::add_transition(&mut composition, &from_seq, &to_seq, trans_type, duration);
    composition
}

#[tauri::command]
fn get_supported_import_formats() -> Vec<String> {
    advanced::get_supported_import_formats()
}

#[tauri::command]
fn get_supported_export_formats() -> Vec<String> {
    advanced::get_supported_export_formats()
}

#[tauri::command]
fn get_export_presets() -> std::collections::HashMap<String, advanced::ExportSettings> {
    advanced::get_export_presets()
}

fn build_sdk_context_for_message(message: &str) -> String {
    let keywords: Vec<&str> = message.split_whitespace()
        .filter(|w| w.len() > 3)
        .collect();
    
    if keywords.is_empty() {
        return String::new();
    }
    
    let search_terms: Vec<String> = keywords.iter()
        .take(5)
        .map(|s| s.to_lowercase())
        .collect();
    
    let all_classes = sdk_indexer::get_all_class_names();
    let relevant_classes: Vec<_> = all_classes.into_iter()
        .filter(|name| {
            let name_lower = name.to_lowercase();
            search_terms.iter().any(|term| name_lower.contains(term))
        })
        .take(10)
        .collect();
    
    if relevant_classes.is_empty() {
        return String::new();
    }
    
    let mut context = String::from("RELEVANT SDK CLASSES FOR THIS QUERY:\n");
    
    for class_name in relevant_classes {
        if let Some(class) = sdk_indexer::get_sdk_class_deep(class_name.clone()) {
            context.push_str(&format!("\n## {} (from {})\n", class.name, class.file));
            if !class.description.is_empty() {
                context.push_str(&format!("Description: {}\n", class.description));
            }
            
            context.push_str("Key methods:\n");
            for method in class.methods.iter().take(15) {
                let params = if method.parameters.is_empty() {
                    String::new()
                } else {
                    format!("({})", method.parameters.join(", "))
                };
                context.push_str(&format!("  - {}: {}{}\n", method.name, method.return_type, params));
            }
            
            if !class.related_classes.is_empty() {
                context.push_str(&format!("Related classes: {}\n", class.related_classes.join(", ")));
            }
        }
    }
    
    context
}

#[tauri::command]
async fn get_provider_models(provider: String) -> Result<Vec<String>, String> {
    let api_key = match provider.as_str() {
        "openai" | "custom-openai" => database::get_setting("openai_api_key")?,
        "gemini" => database::get_setting("gemini_api_key")?,
        "anthropic" => database::get_setting("anthropic_api_key")?,
        _ => None,
    };
    let base_url = match provider.as_str() {
        "ollama" => database::get_setting("ollama_host")?,
        "openai" | "custom-openai" => database::get_setting("openai_base_url")?,
        _ => None,
    };
    ai_providers::fetch_models(&provider, api_key, base_url).await
}

#[tauri::command]
async fn test_ai_connection(provider: String, api_key: String, base_url: Option<String>) -> Result<bool, String> {
    let key = if api_key.is_empty() { None } else { Some(api_key) };
    let models = ai_providers::fetch_models(&provider, key, base_url).await?;
    Ok(!models.is_empty())
}

#[tauri::command]
async fn process_chat_message(
    app_handle: tauri::AppHandle,
    message: String,
    images: Option<Vec<String>>,
    provider: Option<String>,
    model: Option<String>,
) -> Result<String, String> {
    let action = ai_action::plan_validated_action(&message);
    let sdk_context = build_sdk_context_for_message(&message);
    
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

    let mut vision_context = String::new();
    if message.to_lowercase().contains("look") || message.to_lowercase().contains("see") || message.to_lowercase().contains("describe") {
        if let Ok(analysis) = vision_service::analyze_current_viewport().await {
            vision_context = format!(
                "\nVIEWPORT VISION ANALYSIS:\n- Description: {}\n- Detected Nodes: {:?}\n- Lighting: {}\n",
                analysis.description, analysis.detected_nodes, analysis.lighting_style
            );
        }
    }

    let mut spatial_context = String::new();
    let lower_msg = message.to_lowercase();
    if lower_msg.contains("left") || lower_msg.contains("right") || lower_msg.contains("behind") || lower_msg.contains("front") || lower_msg.contains("above") || lower_msg.contains("below") || lower_msg.contains("where") || lower_msg.contains("spatial") || lower_msg.contains("position") || lower_msg.contains("near") {
        spatial_context = vision_service::fetch_spatial_context();
    }

    let scene_context = ai_system::build_scene_context();
    let conflicts = ai_action::ConflictResolver::detect_geoshell_conflicts(&scene_context);
    let conflict_info = if !conflicts.is_empty() {
        format!("\nSCENE CONFLICTS DETECTED:\n{}\n", conflicts.join("\n"))
    } else {
        String::new()
    };

    let execution_summary = if let Some(ref planned) = action {
        if planned.requires_confirmation {
            format!(
                "Planned action '{}' needs confirmation before execution.",
                planned.command
            )
        } else {
            match ai_action::execute_structured_action(planned.clone()) {
                Ok(result) => format!("Executed '{}': {}", planned.command, result),
                Err(e) => format!("Planned '{}', but execution failed: {}", planned.command, e),
            }
        }
    } else {
        "No executable bridge action was inferred; answer with guidance only.".to_string()
    };

    if std::env::var("DAZPILOT_DEV_MOCK_AI").ok().as_deref() == Some("1") {
        return Ok(format!("Plan: {}\n{}{}{}{}", message, execution_summary, vision_context, spatial_context, conflict_info));
    }

    let prompt = format!(
        "You are DazPilot, an expert AI co-pilot for Daz Studio.\n\n\
         User request: {}\n\n\
         Scene Context: {:?}\n\
         {}\n\
         {}\n\
         SDK context:\n\
         {}\n\
         {}\n\
         Execution state:\n\
         {}\n\n\
         If the user's request requires a complex scene change, custom behavior, or is not covered by the Execution state, you MUST write a DazScript (Javascript) macro inside a ```javascript code block to accomplish it. Use the SDK Context provided. Otherwise, provide a concise summary of what was done and what the scene looks like.",
        message,
        scene_context,
        vision_context,
        spatial_context,
        sdk_context,
        conflict_info,
        execution_summary
    );

    let active_provider = match provider {
        Some(p) => p,
        None => database::get_setting("ai_provider")?
            .unwrap_or_else(|| {
                std::env::var("DAZPILOT_AI_BACKEND")
                    .unwrap_or_else(|_| "local-gguf".to_string())
            }),
    };

    let active_model = match model {
        Some(m) => m,
        None => database::get_setting("ai_model")?
            .unwrap_or_else(|| "phi-2-q4".to_string()),
    };

    let api_key = match active_provider.as_str() {
        "openai" | "custom-openai" => database::get_setting("openai_api_key")?,
        "gemini" => database::get_setting("gemini_api_key")?,
        "anthropic" => database::get_setting("anthropic_api_key")?,
        _ => None,
    };

    let base_url = match active_provider.as_str() {
        "ollama" => database::get_setting("ollama_host")?,
        "openai" | "custom-openai" => database::get_setting("openai_base_url")?,
        _ => None,
    };

    let temp_str = database::get_setting("ai_temperature")?.unwrap_or_else(|| "0.7".to_string());
    let temperature = temp_str.parse::<f32>().unwrap_or(0.7);

    let max_tok_str = database::get_setting("ai_max_tokens")?.unwrap_or_else(|| "2048".to_string());
    let max_tokens = max_tok_str.parse::<u32>().unwrap_or(2048);

    let response_text = ai_providers::run_chat(
        &active_provider,
        &active_model,
        prompt,
        api_key,
        base_url,
        temperature,
        max_tokens,
        cleaned_images,
    ).await?;
    
    // Extract Javascript macro if present - emit for user approval instead of auto-executing
    if let Some(script) = extract_javascript_macro(&response_text) {
        let script_id = format!("script-{}", chrono::Utc::now().timestamp_millis());
        let _ = app_handle.emit("script-suggestion", serde_json::json!({
            "id": script_id,
            "script": script,
            "context": message,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }));
        return Ok(format!(
            "AI suggested a DazScript macro. Please review and approve it in the Script Approval panel.\n\nScript ID: {}\n\n{}",
            script_id, response_text
        ));
    }
    
    Ok(response_text)
}

fn extract_javascript_macro(text: &str) -> Option<String> {
    if let Some(start) = text.find("```javascript") {
        if let Some(end) = text[start + 13..].find("```") {
            return Some(text[start + 13..start + 13 + end].trim().to_string());
        }
    }
    if let Some(start) = text.find("```js") {
        if let Some(end) = text[start + 5..].find("```") {
            return Some(text[start + 5..start + 5 + end].trim().to_string());
        }
    }
    None
}

#[tauri::command]
fn execute_approved_script(script: String) -> Result<String, String> {
    info!("User approved script execution");
    mcp_client::send_mcp_request("run_script", serde_json::json!({
        "script": script,
        "args": {}
    }))
    .map(|r| r.result.unwrap_or_else(|| "Script executed successfully".to_string()))
}

#[tauri::command]
fn set_viewport_sync(enabled: bool, state: tauri::State<'_, std::sync::Arc<viewport_sync::ViewportSyncState>>) {
    *state.enabled.lock().unwrap() = enabled;
}

#[tauri::command]
fn set_viewport_fps(fps: u32, state: tauri::State<'_, std::sync::Arc<viewport_sync::ViewportSyncState>>) {
    *state.fps.lock().unwrap() = fps.clamp(1, 10);
}

#[tauri::command]
fn scan_conflicts(root_path: String) -> ConflictScanResult {
    asset_fixer::scan_asset_conflicts(&root_path)
}

#[tauri::command]
fn fix_shell_zones(shell_path: String, prefix: String) -> AssetFixResult {
    asset_fixer::fix_shell_material_zones(&shell_path, &prefix)
}

#[tauri::command]
fn auto_fix_all_conflicts(root_path: String, output_dir: String) -> AssetFixResult {
    asset_fixer::auto_fix_conflicts(&root_path, &output_dir)
}

#[tauri::command]
fn analyze_shell_file(path: String) -> Option<ShellInfo> {
    asset_fixer::analyze_shell(&path)
}

#[tauri::command]
fn ai_ask_question(question: String, options: Vec<String>, allow_custom: bool) -> String {
    let question_id = format!("q-{}", chrono::Utc::now().timestamp_millis());
    log::info!("AI question asked: {} - Options: {:?}, Custom: {}", question, options, allow_custom);
    question_id
}

#[tauri::command]
fn get_asset_conflicts() -> vision_service::AssetConflictReport {
    vision_service::detect_asset_conflicts_from_scene()
}
