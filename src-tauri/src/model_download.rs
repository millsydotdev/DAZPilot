use std::fs::File;
use std::io::{Read, Write};
use tauri::{AppHandle, Emitter};

#[derive(Clone, serde::Serialize)]
struct DownloadProgress {
    progress: u32,
    total: Option<u64>,
    downloaded: u64,
}

pub fn download_model(app: &AppHandle, url: &str, dest_path: &str) -> Result<(), String> {
    println!("Downloading model from: {}", url);
    println!("Destination: {}", dest_path);

    let response = reqwest::blocking::get(url)
        .map_err(|e| format!("Failed to download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed: {}", response.status()));
    }

    let total_size = response.headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    let mut file = File::create(dest_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    let mut downloaded: u64 = 0;
    let mut chunk = vec![0u8; 8192];
    let mut last_progress_emit = 0;

    let mut reader = response;
    loop {
        let bytes_read = reader.read(&mut chunk).map_err(|e| e.to_string())?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&chunk[..bytes_read]).map_err(|e| e.to_string())?;
        downloaded += bytes_read as u64;
        
        if let Some(total) = total_size {
            let progress = (downloaded as f64 / total as f64 * 100.0) as u32;
            
            // Only emit every 1% to avoid spamming the frontend
            if progress > last_progress_emit || progress == 100 {
                last_progress_emit = progress;
                let _ = app.emit("download-progress", DownloadProgress {
                    progress,
                    total: Some(total),
                    downloaded,
                });
                println!("Download progress: {}%", progress);
            }
        } else {
            // Emit raw bytes if total size is unknown (e.g. chunked transfer)
            let _ = app.emit("download-progress", DownloadProgress {
                progress: 0,
                total: None,
                downloaded,
            });
        }
    }

    println!("Download complete!");
    Ok(())
}

#[tauri::command]
pub fn download_gguf_model(app: AppHandle, url: String, filename: String) -> Result<String, String> {
    let models_dir = crate::local_ai::get_models_dir();
    std::fs::create_dir_all(&models_dir).map_err(|e| e.to_string())?;
    
    let dest_path = models_dir.join(&filename);
    let dest_str = dest_path.to_string_lossy().to_string();
    
    download_model(&app, &url, &dest_str)?;
    
    Ok(dest_str)
}
