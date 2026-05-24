use std::fs::File;
use std::io::Write;
use tauri::{AppHandle, Emitter};

#[derive(Clone, serde::Serialize)]
struct DownloadProgress {
    progress: u32,
    total: Option<u64>,
    downloaded: u64,
}

pub async fn download_model(app: &AppHandle, url: &str, dest_path: &str) -> Result<(), String> {
    println!("Downloading model from: {}", url);
    println!("Destination: {}", dest_path);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed: {}", response.status()));
    }

    let total_size = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download: {}", e))?;

    let mut file = File::create(dest_path).map_err(|e| format!("Failed to create file: {}", e))?;

    file.write_all(&bytes).map_err(|e| e.to_string())?;

    let downloaded = bytes.len() as u64;

    if let Some(total) = total_size {
        let progress = (downloaded as f64 / total as f64 * 100.0) as u32;
        let _ = app.emit(
            "download-progress",
            DownloadProgress {
                progress,
                total: Some(total),
                downloaded,
            },
        );
    } else {
        let _ = app.emit(
            "download-progress",
            DownloadProgress {
                progress: 100,
                total: None,
                downloaded,
            },
        );
    }

    println!("Download complete! {} bytes", downloaded);
    Ok(())
}

#[tauri::command]
pub async fn download_gguf_model(
    app: AppHandle,
    url: String,
    filename: String,
) -> Result<String, String> {
    let models_dir = crate::local_ai::get_models_dir();
    std::fs::create_dir_all(&models_dir).map_err(|e| e.to_string())?;

    let dest_path = models_dir.join(&filename);
    let dest_str = dest_path.to_string_lossy().to_string();

    download_model(&app, &url, &dest_str).await?;

    Ok(dest_str)
}
