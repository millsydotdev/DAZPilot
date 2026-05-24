use serde::{Deserialize, Serialize};

/// Visual metadata extracted from the asset. The primary source is AI vision
/// on thumbnail images (see `describe_all_assets`). The name-based extraction
/// is a lightweight fallback when no vision model is available.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VisualProperties {
    pub colors: Vec<String>,
    pub styles: Vec<String>,
}

/// Minimal keyword fallback — only catches the most obvious patterns from names.
/// The real visual understanding comes from Ollama vision on thumbnails.
pub fn extract_visual_properties(name: &str, tags: &[String], _category: &str) -> VisualProperties {
    let combined = format!("{} {}", name, tags.join(" ")).to_lowercase();

    let basic_colors = ["red", "blue", "green", "black", "white", "gold", "silver", "pink", "purple", "yellow", "orange", "brown", "gray", "grey", "dark", "neon", "metallic"];
    let colors: Vec<String> = basic_colors.iter()
        .filter(|c| combined.contains(*c))
        .map(|c| c.to_string())
        .collect();

    let basic_styles = ["casual", "formal", "fantasy", "sci-fi", "scifi", "gothic", "steampunk", "retro", "vintage", "sport", "beach", "winter", "asian", "wedding", "medieval", "cyberpunk", "magical", "cute", "sexy", "elegant", "military", "western", "underwear", "lingerie"];
    let styles: Vec<String> = basic_styles.iter()
        .filter(|s| combined.contains(*s))
        .map(|s| {
            if *s == "scifi" { "sci-fi".to_string() } else { s.to_string() }
        })
        .collect();

    VisualProperties { colors, styles }
}

/// Scores how well an asset matches the scene's described themes/styles.
/// Works with BOTH vision descriptions (via description text matching) and
/// fallback name-based properties.
pub fn score_scene_coherence(
    description: Option<&str>,
    fallback_props: &VisualProperties,
    scene_keywords: &[String],
) -> f32 {
    let mut score: f32 = 0.0;
    let total = scene_keywords.len();
    if total == 0 {
        return 0.0;
    }

    let desc_text = description.unwrap_or("").to_lowercase();

    for kw in scene_keywords {
        let kw_lower = kw.to_lowercase();
        // Check vision description first (richest signal)
        if desc_text.contains(&kw_lower) {
            score += 0.7;
            continue;
        }
        // Fall back to name-based properties
        if fallback_props.styles.iter().any(|s| s == &kw_lower) {
            score += 0.35;
        } else if fallback_props.colors.iter().any(|c| c == &kw_lower) {
            score += 0.25;
        }
    }

    f32::min(score / total as f32, 1.0)
}

/// Walk all assets that have a thumbnail but no description yet, and generate
/// one via Ollama vision. Returns count of new descriptions generated.
pub async fn describe_all_assets() -> usize {
    let rows: Vec<(String, String)> = {
        let guard = match crate::database::get_db() {
            Ok(g) => g,
            Err(_) => return 0,
        };
        let db = match guard.as_ref() {
            Some(d) => d,
            None => return 0,
        };
        let conn = match rusqlite::Connection::open(db.path()) {
            Ok(c) => c,
            Err(_) => return 0,
        };

        let mut stmt = match conn.prepare(
            "SELECT asset_path, thumbnail_path FROM user_assets WHERE user_id='default' AND thumbnail_path IS NOT NULL AND (visual_description IS NULL OR visual_description = '') LIMIT 100"
        ) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        stmt
            .query_map(rusqlite::params![], |row| {
                let path: String = row.get(0)?;
                let thumb: String = row.get(1)?;
                Ok((path, thumb))
            })
            .ok()
            .map(|r| r.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    };

    if rows.is_empty() {
        return 0;
    }

    let mut count = 0;
    for (asset_path, thumb_path) in &rows {
        let thumb_full = if std::path::Path::new(thumb_path).is_absolute() {
            thumb_path.clone()
        } else {
            // Thumbnails from library scanner are stored as relative paths
            // Try to resolve against common Daz content directories
            let default_paths = crate::library_scanner::get_default_content_paths();
            let mut found = None;
            for dp in &default_paths {
                let candidate = format!("{}/{}", dp.path.trim_end_matches('/'), thumb_path);
                if std::path::Path::new(&candidate).exists() {
                    found = Some(candidate);
                    break;
                }
            }
            match found {
                Some(p) => p,
                None => continue,
            }
        };

        let description = match describe_thumbnail(&thumb_full).await {
            Some(d) => d,
            None => continue,
        };

        let _ = {
            let guard = match crate::database::get_db() {
                Ok(g) => g,
                Err(_) => return count,
            };
            let db = match guard.as_ref() {
                Some(d) => d,
                None => return count,
            };
            let conn = match rusqlite::Connection::open(db.path()) {
                Ok(c) => c,
                Err(_) => return 0,
            };
            conn.execute(
                "UPDATE user_assets SET visual_description=?1 WHERE asset_path=?2",
                rusqlite::params![description, asset_path],
            )
        };
        count += 1;
    }
    count
}

async fn describe_thumbnail(path: &str) -> Option<String> {
    let image_bytes = std::fs::read(path).ok()?;
    if image_bytes.is_empty() {
        return None;
    }
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&image_bytes);

    let host = crate::database::get_setting("ollama_host")
        .ok().flatten().filter(|h| !h.is_empty())
        .unwrap_or_else(|| "http://localhost:11434".to_string());
    let model = crate::database::get_setting("ollama_vision_model")
        .ok().flatten().filter(|m| !m.is_empty())
        .unwrap_or_else(|| "llava".to_string());

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build().ok()?;

    let request = serde_json::json!({
        "model": model,
        "messages": [{
            "role": "user",
            "content": "Describe this 3D model asset in 1-2 sentences. Focus on: what type of item it is, colors, materials, style, length/fit, and overall vibe. Example: 'A red silk evening gown with gold embroidery, plunging neckline, long sleeves. Formal elegant fantasy style.' or 'Black leather jacket with silver studs, cropped fit, casual punk style.'",
            "images": [b64]
        }],
        "stream": false,
        "options": { "temperature": 0.15, "num_predict": 256 }
    });

    let resp = client
        .post(format!("{}/api/chat", host))
        .json(&request)
        .send().await.ok()?;

    if !resp.status().is_success() {
        return None;
    }

    #[derive(Deserialize)]
    struct ChatResp {
        message: ChatMsg,
    }
    #[derive(Deserialize)]
    struct ChatMsg {
        content: String,
    }

    let data: ChatResp = resp.json().await.ok()?;
    let desc = data.message.content.trim().to_string();
    if desc.is_empty() || desc.len() < 10 {
        return None;
    }
    Some(desc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_colors() {
        let props = extract_visual_properties("Red Silk Dress", &[], "clothing");
        assert!(props.colors.contains(&"red".to_string()));
    }

    #[test]
    fn test_extract_styles() {
        let props = extract_visual_properties("Cyberpunk Neon Dress", &[], "clothing");
        assert!(props.styles.contains(&"cyberpunk".to_string()));
    }

    #[test]
    fn test_scene_coherence_vision_desc() {
        let desc = Some("A red silk evening gown with gold embroidery, formal elegant style.");
        let fallback = VisualProperties::default();
        let score = score_scene_coherence(desc, &fallback, &["formal".into(), "red".into()]);
        assert!(score > 0.0);
    }

    #[test]
    fn test_scene_coherence_fallback_only() {
        let fallback = VisualProperties {
            styles: vec!["gothic".into()],
            colors: vec![],
        };
        let score = score_scene_coherence(None, &fallback, &["gothic".into()]);
        assert!(score > 0.0);
    }

    #[test]
    fn test_scene_coherence_no_match() {
        let fallback = VisualProperties::default();
        let score = score_scene_coherence(None, &fallback, &["fantasy".into()]);
        assert_eq!(score, 0.0);
    }
}
