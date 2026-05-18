#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;



/// Asset conflict types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    MaterialZone,
    MorphId,
    UVSet,
    AssetReference,
}

impl ConflictType {
    pub fn name(&self) -> String {
        match self {
            ConflictType::MaterialZone => "MaterialZone".to_string(),
            ConflictType::MorphId => "MorphId".to_string(),
            ConflictType::UVSet => "UVSet".to_string(),
            ConflictType::AssetReference => "AssetReference".to_string(),
        }
    }
}

/// A detected conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetConflict {
    pub conflict_type: ConflictType,
    pub name: String,
    pub files: Vec<String>,
    pub severity: String,
}

/// Result of scanning assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictScanResult {
    pub total_scanned: usize,
    pub conflicts: Vec<AssetConflict>,
    pub warnings: Vec<String>,
}

/// Result of fixing assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetFixResult {
    pub success: bool,
    pub fixed_files: Vec<String>,
    pub errors: Vec<String>,
}

/// Scan for asset conflicts in a directory
pub fn scan_asset_conflicts(root_path: &str) -> ConflictScanResult {
    let mut conflicts = Vec::new();
    let warnings = Vec::new();
    let mut total_scanned = 0;

    // Scan for .dsf and .duf files
    let entries = walkdir::WalkDir::new(root_path)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            p.extension().map_or(false, |ext| 
                ext == "dsf" || ext == "duf"
            )
        });

    for entry in entries {
            let path = entry.path();
            total_scanned += 1;

            // Try to parse as JSON (DSF/DUF files)
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    // Check for material conflicts
                    if let Some(mat_lib) = data.get("material_library").and_then(|m| m.as_array()) {
                        let material_ids: Vec<String> = mat_lib
                            .iter()
                            .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(String::from))
                            .collect();
                        
                        // Group by material ID to find duplicates
                        let mut id_counts: HashMap<String, usize> = HashMap::new();
                        for id in &material_ids {
                            *id_counts.entry(id.clone()).or_insert(0) += 1;
                        }
                        
                        for (id, count) in id_counts {
                            if count > 1 {
                                conflicts.push(AssetConflict {
                                    conflict_type: ConflictType::MaterialZone,
                                    name: id,
                                    files: vec![path.to_string_lossy().to_string()],
                                    severity: "high".to_string(),
                                });
                            }
                        }
                    }

                    // Check for morph conflicts
                    let morph_sources = ["morph_library", "modifier_library"];
                    for source in morph_sources.iter() {
                        if let Some(morph_lib) = data.get(source).and_then(|m| m.as_array()) {
                            for morph in morph_lib {
                                if let Some(_id) = morph.get("id").and_then(|i| i.as_str()) {
                                    // Check for duplicates across files would require global analysis
                                }
                            }
                        }
                    }
                }
            }
        }

    ConflictScanResult {
        total_scanned,
        conflicts,
        warnings,
    }
}

/// Fix shell material zone conflicts by adding prefixes
pub fn fix_shell_material_zones(shell_path: &str, prefix: &str) -> AssetFixResult {
    let mut fixed_files = Vec::new();
    let mut errors = Vec::new();

    if !Path::new(shell_path).exists() {
        return AssetFixResult {
            success: false,
            fixed_files: vec![],
            errors: vec![format!("File not found: {}", shell_path)],
        };
    }

    // Read the file
    let content = match fs::read_to_string(shell_path) {
        Ok(c) => c,
        Err(e) => {
            return AssetFixResult {
                success: false,
                fixed_files: vec![],
                errors: vec![format!("Failed to read file: {}", e)],
            };
        }
    };

    // Parse JSON
    let mut data: serde_json::Value = match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            return AssetFixResult {
                success: false,
                fixed_files: vec![],
                errors: vec![format!("Failed to parse JSON: {}", e)],
            };
        }
    };

    // Rename material zones
    if let Some(mat_lib) = data.get_mut("material_library").and_then(|m| m.as_array_mut()) {
        for mat in mat_lib {
            if let Some(id_val) = mat.get_mut("id") {
                if let Some(id) = id_val.as_str() {
                    let new_id = format!("{}{}", prefix, id);
                    *id_val = serde_json::Value::String(new_id);
                }
            }
        }
    }

    // Add fix metadata
    if let Some(obj) = data.get_mut("asset_info").and_then(|a| a.as_object_mut()) {
        obj.insert("material_zones_fixed".to_string(), serde_json::Value::Bool(true));
        obj.insert("prefix_added".to_string(), serde_json::Value::String(prefix.to_string()));
    } else {
        let mut new_obj = serde_json::Map::new();
        new_obj.insert("material_zones_fixed".to_string(), serde_json::Value::Bool(true));
        new_obj.insert("prefix_added".to_string(), serde_json::Value::String(prefix.to_string()));
        data["asset_info"] = serde_json::Value::Object(new_obj);
    }

    // Write fixed file
    let output_path = format!("{}.fixed", shell_path);
    match fs::write(&output_path, serde_json::to_string_pretty(&data).unwrap()) {
        Ok(_) => fixed_files.push(output_path),
        Err(e) => errors.push(format!("Failed to write: {}", e)),
    }

    AssetFixResult {
        success: errors.is_empty(),
        fixed_files,
        errors,
    }
}

/// Auto-fix all conflicts in a directory
pub fn auto_fix_conflicts(root_path: &str, output_dir: &str) -> AssetFixResult {
    let scan = scan_asset_conflicts(root_path);
    let mut fixed_files = Vec::new();
    let mut errors = Vec::new();

    // Create output directory
    if let Err(e) = fs::create_dir_all(output_dir) {
        return AssetFixResult {
            success: false,
            fixed_files: vec![],
            errors: vec![format!("Failed to create output dir: {}", e)],
        };
    }

    // Process conflicts
    for conflict in scan.conflicts {
        match conflict.conflict_type {
            ConflictType::MaterialZone => {
                // Extract prefix from conflict name or use default
                let prefix = detect_prefix_from_conflict(&conflict.name);
                for file in &conflict.files {
                    let result = fix_shell_material_zones(file, &prefix);
                    fixed_files.extend(result.fixed_files);
                    errors.extend(result.errors);
                }
            }
            _ => {} // Handle other types as needed
        }
    }

    AssetFixResult {
        success: errors.is_empty(),
        fixed_files,
        errors,
    }
}

/// Detect appropriate prefix based on conflict name
fn detect_prefix_from_conflict(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.contains("majora") {
        "GM_".to_string()
    } else if lower.contains("minora") {
        "GMin_".to_string()
    } else if lower.contains("addons") || lower.contains("addon") {
        "GA_".to_string()
    } else if lower.contains("breast") {
        "BR_".to_string()
    } else if lower.contains("nipple") {
        "NP_".to_string()
    } else if lower.contains("areola") {
        "AR_".to_string()
    } else {
        "FIXED_".to_string()
    }
}

/// Get shell info from a .dsf file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellInfo {
    pub path: String,
    pub shell_type: String,
    pub material_zones: Vec<String>,
    pub uv_sets: Vec<String>,
}

/// Analyze a shell file
pub fn analyze_shell(path: &str) -> Option<ShellInfo> {
    let content = fs::read_to_string(path).ok()?;
    let data: serde_json::Value = serde_json::from_str(&content).ok()?;

    let mut material_zones = Vec::new();
    if let Some(mat_lib) = data.get("material_library").and_then(|m| m.as_array()) {
        for mat in mat_lib {
            if let Some(id) = mat.get("id").and_then(|i| i.as_str()) {
                material_zones.push(id.to_string());
            }
        }
    }

    let mut uv_sets = Vec::new();
    if let Some(uv_lib) = data.get("uv_library").and_then(|u| u.as_array()) {
        for uv in uv_lib {
            if let Some(name) = uv.get("name").and_then(|n| n.as_str()) {
                uv_sets.push(name.to_string());
            }
        }
    }

    // Detect shell type from path
    let path_lower = path.to_lowercase();
    let shell_type = if path_lower.contains("majora") {
        "majora".to_string()
    } else if path_lower.contains("minora") {
        "minora".to_string()
    } else if path_lower.contains("addons") || path_lower.contains("addon") {
        "addons".to_string()
    } else if path_lower.contains("breast") {
        "breasts".to_string()
    } else if path_lower.contains("nipple") {
        "nipples".to_string()
    } else {
        "unknown".to_string()
    };

    Some(ShellInfo {
        path: path.to_string(),
        shell_type,
        material_zones,
        uv_sets,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_detection() {
        assert_eq!(detect_prefix_from_conflict("Torso"), "FIXED_".to_string());
        assert_eq!(detect_prefix_from_conflict("GP_Majora_Torso"), "GM_".to_string());
        assert_eq!(detect_prefix_from_conflict("GP_Minora_Torso"), "GMin_".to_string());
    }
}