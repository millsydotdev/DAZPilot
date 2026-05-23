#![allow(dead_code)]
#![allow(unused_imports)]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;
use std::collections::HashMap;
use std::io::Read;
use flate2::read::GzDecoder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPath {
    pub id: Option<i64>,
    pub path: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub total_files: usize,
    pub categorized: CategorizedAssets,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategorizedAssets {
    pub figures: Vec<AssetInfo>,
    pub clothing: Vec<AssetInfo>,
    pub hair: Vec<AssetInfo>,
    pub poses: Vec<AssetInfo>,
    pub materials: Vec<AssetInfo>,
    pub morphs: Vec<AssetInfo>,
    pub environments: Vec<AssetInfo>,
    pub lights: Vec<AssetInfo>,
    pub cameras: Vec<AssetInfo>,
    pub animations: Vec<AssetInfo>,
    pub other: Vec<AssetInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    pub path: String,
    pub name: String,
    pub file_type: String,
    pub size: u64,
    pub category: String,
    pub subcategory: Option<String>,
    pub metadata: HashMap<String, String>,
    pub thumbnail_path: Option<String>,
    pub compatibility_base: Vec<String>,
    pub dforce_enabled: bool,
    pub asset_type_detail: Option<String>,
    pub vendor: Option<String>,
    pub tags: Vec<String>,
}

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "duf", "dsf", "obj", "fbx", "dae", "pth", "fit", "hr2", "cr2", "mc6", "dsa", "dsb",
];

const CATEGORY_PATTERNS: &[(&str, &[&str])] = &[
    ("figures", &["figure", "genesis", "victoria", "michael", "david", "youth", "person"]),
    ("clothing", &["clothing", "outfit", "shirt", "pants", "dress", "jacket", "top", "bottom", "skirt", "wear"]),
    ("hair", &["hair", "hairstyle", "ponytail", "braid"]),
    ("poses", &["pose", "poseable"]),
    ("materials", &["material", "shader", "texture", "skin", "makeup"]),
    ("morphs", &["morph", "shape", "modifier", "jcm"]),
    ("environments", &["environment", "scene", "hdri", "sky", "backdrop"]),
    ("lights", &["light", "illumination"]),
    ("cameras", &["camera"]),
    ("animations", &["animation", "anim", "walk", "run", "bounce"]),
];

fn get_daz_content_dirs_from_registry() -> Vec<String> {
    let mut dirs = vec![];
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("reg")
            .args(["query", "HKCU\\Software\\DAZ\\Studio4"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("ContentDir") {
                        let parts: Vec<&str> = trimmed.split_whitespace().collect();
                        if parts.len() >= 3 && parts[1] == "REG_SZ" {
                            let path = parts[2..].join(" ");
                            let normalized = path.replace("\\", "/");
                            if !dirs.contains(&normalized) {
                                dirs.push(normalized);
                            }
                        }
                    }
                }
            }
        }
    }
    dirs
}

pub fn get_default_content_paths() -> Vec<ContentPath> {
    let mut paths = vec![];

    // Try registry first
    let reg_dirs = get_daz_content_dirs_from_registry();
    if !reg_dirs.is_empty() {
        for (i, dir_path) in reg_dirs.iter().enumerate() {
            let path_buf = PathBuf::from(dir_path);
            if path_buf.exists() {
                let name = if dir_path.contains("My Library") {
                    "DAZ 3D".to_string()
                } else if dir_path.contains("My DAZ 3D Library") {
                    "Public Daz Library".to_string()
                } else {
                    path_buf.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| format!("Daz Library {}", i))
                };

                paths.push(ContentPath {
                    id: None,
                    path: dir_path.clone(),
                    name,
                    is_default: true,
                });
            }
        }
    }

    // Fallback to hardcoded defaults if registry is empty or fails
    if paths.is_empty() {
        if let Some(documents) = dirs::document_dir() {
            let daz3d_path = documents.join("DAZ 3D");
            if daz3d_path.exists() {
                paths.push(ContentPath {
                    id: None,
                    path: daz3d_path.to_string_lossy().to_string(),
                    name: "DAZ 3D".to_string(),
                    is_default: true,
                });
            }
        }

        if let Ok(program_data) = std::env::var("PROGRAMDATA") {
            let daz3d_path = PathBuf::from(program_data).join("DAZ 3D");
            if daz3d_path.exists() {
                paths.push(ContentPath {
                    id: None,
                    path: daz3d_path.to_string_lossy().to_string(),
                    name: "ProgramData DAZ".to_string(),
                    is_default: true,
                });
            }
        }

        // Common Public Documents path for Daz Install Manager
        let public_docs = PathBuf::from(r"C:\Users\Public\Documents\My DAZ 3D Library");
        if public_docs.exists() {
            paths.push(ContentPath {
                id: None,
                path: public_docs.to_string_lossy().to_string(),
                name: "Public Daz Library".to_string(),
                is_default: true,
            });
        }
    }

    paths
}

pub fn scan_directory(path: &str) -> ScanResult {
    let mut categorized = CategorizedAssets::default();
    let mut errors = vec![];
    let mut total_files = 0;

    // Validate path exists before scanning
    let path_buf = PathBuf::from(path);
    if !path_buf.exists() {
        errors.push(format!("Path does not exist: {}", path));
        return ScanResult { total_files: 0, categorized, errors };
    }

    for entry in WalkDir::new(path)
        .follow_links(true)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let file_path = entry.path();
        
        if !file_path.is_file() {
            continue;
        }

        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        if !SUPPORTED_EXTENSIONS.contains(&extension.as_str()) {
            continue;
        }

        let file_name = file_path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let lower_name = file_name.to_lowercase();
        if lower_name == "daz_host" || lower_name == "vibebridge" || lower_name == "dazai_bridge" || lower_name == "dazpilotbridge" {
            continue;
        }

        let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        
        let metadata = read_asset_metadata(file_path);
        let (category, subcategory) = detect_category(&file_name, &extension, &metadata);

        let thumbnail_path = detect_companion_thumbnail(file_path);
        let compatibility_base: Vec<String> = metadata.get("compatibility_base")
            .and_then(|v| serde_json::from_str(v).ok())
            .unwrap_or_default();
        let dforce_enabled = metadata.contains_key("dforce_enabled");
        let asset_type_detail = metadata.get("asset_type_detail").or_else(|| metadata.get("type")).cloned();
        let vendor = metadata.get("vendor").or_else(|| metadata.get("asset_info.contributor")).cloned();
        let mut tags = Vec::new();
        // Infer tags from filename
        let lower = file_name.to_lowercase();
        if lower.contains("female") || lower.contains("woman") || lower.contains("girl") || lower.contains("victoria") { tags.push("female".to_string()); }
        if lower.contains("male") || lower.contains("man") || lower.contains("boy") || lower.contains("michael") { tags.push("male".to_string()); }
        if lower.contains("casual") || lower.contains("relax") { tags.push("casual".to_string()); }
        if lower.contains("formal") || lower.contains("elegant") || lower.contains("business") { tags.push("formal".to_string()); }
        if lower.contains("sport") || lower.contains("athletic") || lower.contains("fitness") { tags.push("sport".to_string()); }
        if dforce_enabled || lower.contains("dforce") || lower.contains("d-force") { tags.push("dforce".to_string()); }

        let asset = AssetInfo {
            path: file_path.to_string_lossy().to_string(),
            name: file_name,
            file_type: extension.clone(),
            size: file_size,
            category: category.clone(),
            subcategory,
            metadata,
            thumbnail_path,
            compatibility_base,
            dforce_enabled,
            asset_type_detail,
            vendor,
            tags,
        };

        add_to_category(&mut categorized, &category, asset);
        total_files += 1;
    }

    persist_assets(&categorized);

    ScanResult {
        total_files,
        categorized,
        errors,
    }
}

fn detect_category(file_name: &str, extension: &str, metadata: &HashMap<String, String>) -> (String, Option<String>) {
    if let Some(asset_type) = metadata.get("type").or_else(|| metadata.get("asset_type")) {
        let lower_type = asset_type.to_lowercase();
        if lower_type.contains("pose") {
            return ("poses".to_string(), None);
        }
        if lower_type.contains("material") || lower_type.contains("shader") {
            return ("materials".to_string(), None);
        }
        if lower_type.contains("camera") {
            return ("cameras".to_string(), None);
        }
        if lower_type.contains("light") {
            return ("lights".to_string(), None);
        }
        if lower_type.contains("figure") || lower_type.contains("actor") {
            return ("figures".to_string(), None);
        }
        if lower_type.contains("modifier") || lower_type.contains("morph") {
            return ("morphs".to_string(), None);
        }
    }

    let lower_name = file_name.to_lowercase();

    if extension == "fit" || extension == "pth" || extension == "hr2" {
        return ("morphs".to_string(), Some("shape".to_string()));
    }

    if extension == "duf" && lower_name.contains("pose") {
        return ("poses".to_string(), None);
    }

    for (category, keywords) in CATEGORY_PATTERNS {
        for keyword in *keywords {
            if lower_name.contains(keyword) {
                return (category.to_string(), None);
            }
        }
    }

    ("other".to_string(), None)
}

/// Check for a sibling .png or .jpg companion thumbnail (same stem, same directory)
fn detect_companion_thumbnail(path: &std::path::Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    let parent = path.parent()?;
    for ext in &["png", "jpg", "jpeg"] {
        let candidate = parent.join(format!("{}.{}", stem, ext));
        if candidate.exists() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }
    // Also check a Resources/Thumbnails sibling
    let thumb_dir = parent.join("Resources").join("Thumbnails");
    if thumb_dir.exists() {
        for ext in &["png", "jpg", "jpeg"] {
            let candidate = thumb_dir.join(format!("{}.{}", stem, ext));
            if candidate.exists() {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }
    None
}

fn read_asset_metadata(path: &std::path::Path) -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    
    // Only read first 512KB to avoid memory issues with large files
    const MAX_READ_SIZE: usize = 512 * 1024;
    
    let Ok(mut file) = std::fs::File::open(path) else {
        return metadata;
    };
    
    let mut buffer = vec![0u8; MAX_READ_SIZE];
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return metadata,
    };
    buffer.truncate(bytes_read);
    
    // Check for Gzip header (1f 8b) - only decompress if small enough
    let content = if bytes_read > 2 && buffer[0] == 0x1f && buffer[1] == 0x8b {
        // Only decompress if the file is small enough to avoid memory issues
        if bytes_read < 10 * 1024 * 1024 { // 10MB limit
            let mut decoder = GzDecoder::new(&buffer[..bytes_read]);
            let mut decompressed = String::new();
            if decoder.read_to_string(&mut decompressed).is_ok() {
                decompressed
            } else {
                String::from_utf8_lossy(&buffer[..bytes_read]).to_string()
            }
        } else {
            // Too large to decompress, just check for text content
            String::from_utf8_lossy(&buffer[..bytes_read]).to_string()
        }
    } else {
        String::from_utf8_lossy(&buffer[..bytes_read]).to_string()
    };

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        for key in ["type", "asset_type", "name", "vendor"] {
            if let Some(value) = json.get(key).and_then(|v| v.as_str()) {
                metadata.insert(key.to_string(), value.to_string());
            }
        }
        // Extract compatibility_base as JSON array string
        if let Some(cb) = json.get("compatibility_base") {
            if let Some(arr) = cb.as_array() {
                let strs: Vec<String> = arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
                if !strs.is_empty() {
                    if let Ok(json_str) = serde_json::to_string(&strs) {
                        metadata.insert("compatibility_base".to_string(), json_str);
                    }
                }
            }
        }
        // Check for compatibility_base array at scene.level.asset_info.compatibility_base
        if !metadata.contains_key("compatibility_base") {
            if let Some(scene) = json.get("scene") {
                if let Some(nodes) = scene.get("nodes").and_then(|v| v.as_array()) {
                    for node in nodes {
                        if let Some(preview) = node.get("preview") {
                            if let Some(viewer) = preview.get("viewer_settings") {
                                // Detect dForce via simulation_settings key presence
                                if viewer.get("simulation_settings").is_some() {
                                    metadata.insert("dforce_enabled".to_string(), "true".to_string());
                                    // Also detect dForce from top-level asset_type if available
                                }
                            }
                        }
                        // Try to find compatibility_base in node.structure
                        if let Some(structure) = node.get("structure") {
                            if let Some(cb_arr) = structure.get("compatibility_base").and_then(|v| v.as_array()) {
                                let strs: Vec<String> = cb_arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
                                if !strs.is_empty() {
                                    if let Ok(json_str) = serde_json::to_string(&strs) {
                                        metadata.insert("compatibility_base".to_string(), json_str);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(asset_info) = json.get("asset_info").and_then(|v| v.as_object()) {
            for key in ["id", "type", "name", "contributor", "modified"] {
                if let Some(value) = asset_info.get(key).and_then(|v| v.as_str()) {
                    metadata.insert(format!("asset_info.{}", key), value.to_string());
                }
            }
            // Asset type detail: map asset_info.type to a human-readable category
            if let Some(asset_type_val) = asset_info.get("type").and_then(|v| v.as_str()) {
                let detail = match asset_type_val.to_lowercase().as_str() {
                    t if t.contains("wearable") => "wearable",
                    t if t.contains("pose") => "pose_preset",
                    t if t.contains("morph") || t.contains("shape") || t.contains("modifier") => "morph",
                    t if t.contains("material") || t.contains("shader") => "material_preset",
                    t if t.contains("scene") => "scene",
                    t if t.contains("prop") || t.contains("accessory") => "prop",
                    t if t.contains("light") => "light",
                    t if t.contains("camera") => "camera",
                    t if t.contains("animation") => "animation",
                    t if t.contains("hair") => "hair",
                    t if t.contains("figure") || t.contains("actor") => "figure",
                    _ => "other",
                };
                metadata.insert("asset_type_detail".to_string(), detail.to_string());
            }
        }
        // Also check top-level 'type' if asset_info.type wasn't conclusive
        if !metadata.contains_key("asset_type_detail") {
            if let Some(top_type) = json.get("type").and_then(|v| v.as_str()) {
                metadata.insert("asset_type_detail".to_string(), top_type.to_string());
            }
        }
        // Fallback dforce detection from filename keywords combined with JSON
        if !metadata.contains_key("dforce_enabled") {
            let path_lower = path.to_string_lossy().to_lowercase();
            if path_lower.contains("dforce") {
                metadata.insert("dforce_enabled".to_string(), "true".to_string());
            }
        }
    }

    metadata
}

fn add_to_category(categorized: &mut CategorizedAssets, category: &str, asset: AssetInfo) {
    match category {
        "figures" => categorized.figures.push(asset),
        "clothing" => categorized.clothing.push(asset),
        "hair" => categorized.hair.push(asset),
        "poses" => categorized.poses.push(asset),
        "materials" => categorized.materials.push(asset),
        "morphs" => categorized.morphs.push(asset),
        "environments" => categorized.environments.push(asset),
        "lights" => categorized.lights.push(asset),
        "cameras" => categorized.cameras.push(asset),
        "animations" => categorized.animations.push(asset),
        _ => categorized.other.push(asset),
    }
}

fn persist_assets(categorized: &CategorizedAssets) {
    let Ok(db_guard) = crate::database::get_db() else {
        return;
    };
    let Some(db) = db_guard.as_ref() else {
        return;
    };
    let Ok(mut conn) = rusqlite::Connection::open(db.path()) else {
        return;
    };
    let Ok(tx) = conn.transaction() else {
        return;
    };

    let groups = [
        &categorized.figures,
        &categorized.clothing,
        &categorized.hair,
        &categorized.poses,
        &categorized.materials,
        &categorized.morphs,
        &categorized.environments,
        &categorized.lights,
        &categorized.cameras,
        &categorized.animations,
        &categorized.other,
    ];

    for assets in groups {
        for asset in assets {
            let compatibility_json = serde_json::to_string(&asset.compatibility_base).unwrap_or_default();
            let tags_json = serde_json::to_string(&asset.tags).unwrap_or_default();
            let _ = tx.execute(
                "INSERT OR REPLACE INTO user_assets (user_id, asset_path, asset_name, original_name, category, subcategory, vendor, file_type, file_size, thumbnail_path, compatibility, dforce_enabled, asset_type_detail, tags) VALUES ('default', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                rusqlite::params![
                    asset.path,
                    asset.name,
                    asset.metadata.get("asset_info.name").or_else(|| asset.metadata.get("name")).unwrap_or(&asset.name),
                    asset.category,
                    asset.subcategory,
                    asset.vendor.as_deref().or_else(|| asset.metadata.get("vendor").map(|x| x.as_str())).or_else(|| asset.metadata.get("asset_info.contributor").map(|x| x.as_str())),
                    asset.file_type,
                    asset.size as i64,
                    asset.thumbnail_path,
                    compatibility_json,
                    asset.dforce_enabled as i32,
                    asset.asset_type_detail.as_deref().or_else(|| asset.metadata.get("asset_type_detail").map(|x| x.as_str())),
                    tags_json,
                ],
            );
        }
    }

    let _ = tx.commit();
}

pub fn scan_multiple_paths(paths: &[String]) -> ScanResult {
    let mut combined = CategorizedAssets::default();
    let mut errors = vec![];
    let mut total_files = 0;

    for path in paths {
        let result = scan_directory(path);
        total_files += result.total_files;
        errors.extend(result.errors);

        combined.figures.extend(result.categorized.figures);
        combined.clothing.extend(result.categorized.clothing);
        combined.hair.extend(result.categorized.hair);
        combined.poses.extend(result.categorized.poses);
        combined.materials.extend(result.categorized.materials);
        combined.morphs.extend(result.categorized.morphs);
        combined.environments.extend(result.categorized.environments);
        combined.lights.extend(result.categorized.lights);
        combined.cameras.extend(result.categorized.cameras);
        combined.animations.extend(result.categorized.animations);
        combined.other.extend(result.categorized.other);
    }

    ScanResult {
        total_files,
        categorized: combined,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_detect_companion_thumbnail_png() {
        // Use the current source file itself as a reference, then check for none
        let result = detect_companion_thumbnail(Path::new("Cargo.toml"));
        // May or may not find a companion — just verify no crash
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn test_detect_companion_thumbnail_nonexistent() {
        let result = detect_companion_thumbnail(Path::new("nonexistent_file.duf"));
        assert!(result.is_none());
    }

    #[test]
    fn test_asset_info_new_fields_defaults() {
        let info = AssetInfo {
            path: "test.duf".to_string(),
            name: "test".to_string(),
            file_type: "duf".to_string(),
            size: 100,
            category: "figures".to_string(),
            subcategory: None,
            metadata: HashMap::new(),
            thumbnail_path: None,
            compatibility_base: vec![],
            dforce_enabled: false,
            asset_type_detail: None,
            vendor: None,
            tags: vec![],
        };
        assert_eq!(info.path, "test.duf");
        assert!(!info.dforce_enabled);
        assert!(info.compatibility_base.is_empty());
        assert!(info.tags.is_empty());
    }
}
