use crate::local_ai as local;

#[tauri::command]
pub fn get_models_dir() -> String {
    local::get_models_dir().to_string_lossy().to_string()
}

#[tauri::command]
pub fn list_local_models() -> Vec<local::LocalModelInfo> {
    local::list_local_models()
}

#[tauri::command]
pub fn start_local_server(model_path: String, port: u16) -> Result<(), String> {
    local::start_local_server(&model_path, port)
}

#[tauri::command]
pub fn stop_local_server() -> Result<(), String> {
    local::stop_local_server();
    Ok(())
}

#[tauri::command]
pub fn chat_with_local(prompt: String, model: String) -> Result<String, String> {
    local::chat_with_local(prompt, model)
}

#[tauri::command]
pub fn is_local_server_running() -> bool {
    local::is_local_server_running()
}