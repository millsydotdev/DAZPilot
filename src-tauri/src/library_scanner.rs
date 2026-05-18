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
}

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "duf", "dsf", "obj", "fbx", "dae", "pth", "fit", "hr2", "cr2", "mc6", "dsa", "dsb",
];

const CATEGORY_PATTERNS: &[(&str, &[&str])] = &[
    ("figures", &["figure", "genesis", "victoria", "michael", "david", "youth", "person"]),
    ("clothing", &["clothing", "outfit", "shirt", "pants", "dress", "jacket", "top", "bottom", " skirt", "wear"]),
    ("hair", &["hair", " hairstyle", "ponytail", "braid"]),
    ("poses", &["pose", "poseable"]),
    ("materials", &["material", "shader", "texture", "skin", " makeup"]),
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
            .args(&["query", "HKCU\\Software\\DAZ\\Studio4"])
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
    let errors = vec![];
    let mut total_files = 0;

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

        let asset = AssetInfo {
            path: file_path.to_string_lossy().to_string(),
            name: file_name,
            file_type: extension.clone(),
            size: file_size,
            category: category.clone(),
            subcategory,
            metadata,
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

fn read_asset_metadata(path: &std::path::Path) -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    let Ok(bytes) = std::fs::read(path) else {
        return metadata;
    };

    let sample_len = bytes.len().min(256 * 1024);
    
    // Check for Gzip header (1f 8b)
    let content = if bytes.len() > 2 && bytes[0] == 0x1f && bytes[1] == 0x8b {
        let mut decoder = GzDecoder::new(&bytes[..]);
        let mut decompressed = String::new();
        // Read up to 256KB of decompressed data
        match decoder.read_to_string(&mut decompressed) {
            Ok(_) => decompressed,
            Err(_) => String::from_utf8_lossy(&bytes[..sample_len]).to_string(),
        }
    } else {
        String::from_utf8_lossy(&bytes[..sample_len]).to_string()
    };

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        for key in ["type", "asset_type", "name", "vendor", "compatibility_base"] {
            if let Some(value) = json.get(key).and_then(|v| v.as_str()) {
                metadata.insert(key.to_string(), value.to_string());
            }
        }
        if let Some(asset_info) = json.get("asset_info").and_then(|v| v.as_object()) {
            for key in ["id", "type", "name", "contributor"] {
                if let Some(value) = asset_info.get(key).and_then(|v| v.as_str()) {
                    metadata.insert(format!("asset_info.{}", key), value.to_string());
                }
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
            let _ = tx.execute(
                "INSERT OR REPLACE INTO user_assets (user_id, asset_path, asset_name, original_name, category, subcategory, vendor, file_type, file_size) VALUES ('default', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    asset.path,
                    asset.name,
                    asset.metadata.get("asset_info.name").or_else(|| asset.metadata.get("name")).unwrap_or(&asset.name),
                    asset.category,
                    asset.subcategory,
                    asset.metadata.get("vendor").or_else(|| asset.metadata.get("asset_info.contributor")),
                    asset.file_type,
                    asset.size as i64,
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
