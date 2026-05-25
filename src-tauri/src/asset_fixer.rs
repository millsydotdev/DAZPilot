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

impl ConflictScanResult {
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }
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
    // Track morph IDs globally across all files to detect cross-file duplicates
    let mut global_morph_ids: HashMap<String, Vec<String>> = HashMap::new();

    // Scan for .dsf and .duf files
    let entries = walkdir::WalkDir::new(root_path)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            p.extension()
                .is_some_and(|ext| ext == "dsf" || ext == "duf")
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

                    // Group by material ID to find duplicates within the same file
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

                // Check for morph conflicts — global dedup across files
                let morph_sources = ["morph_library", "modifier_library"];
                for source in morph_sources.iter() {
                    if let Some(morph_lib) = data.get(source).and_then(|m| m.as_array()) {
                        for morph in morph_lib {
                            if let Some(id) = morph.get("id").and_then(|i| i.as_str()) {
                                let path_str = path.to_string_lossy().to_string();
                                let entry = global_morph_ids.entry(id.to_string()).or_default();
                                if !entry.contains(&path_str) {
                                    entry.push(path_str);
                                }
                            }
                        }
                    }
                }

                // Check for UV conflicts
                if let Some(uv_lib) = data.get("uv_library").and_then(|u| u.as_array()) {
                    let uv_names: Vec<String> = uv_lib
                        .iter()
                        .filter_map(|u| u.get("name").and_then(|n| n.as_str()).map(String::from))
                        .collect();
                    let mut uv_counts: HashMap<String, Vec<String>> = HashMap::new();
                    for uv_name in &uv_names {
                        uv_counts
                            .entry(uv_name.clone())
                            .or_default()
                            .push(path.to_string_lossy().to_string());
                    }
                    for (name, paths) in uv_counts {
                        if paths.len() > 1 {
                            conflicts.push(AssetConflict {
                                conflict_type: ConflictType::UVSet,
                                name,
                                files: paths,
                                severity: "medium".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    // Cross-file morph conflict detection
    for (morph_id, files) in &global_morph_ids {
        if files.len() > 1 {
            conflicts.push(AssetConflict {
                conflict_type: ConflictType::MorphId,
                name: morph_id.clone(),
                files: files.clone(),
                severity: "medium".to_string(),
            });
        }
    }

    ConflictScanResult {
        total_scanned,
        conflicts,
        warnings,
    }
}

/// Detect assets with overlapping UV set names
pub fn detect_uv_conflicts(assets: &[crate::library_scanner::AssetInfo]) -> Vec<AssetConflict> {
    let mut conflicts = Vec::new();
    let mut uv_map: HashMap<String, Vec<String>> = HashMap::new();
    for asset in assets {
        if let Some(uv_str) = asset.metadata.get("uv_sets") {
            if let Ok(uvs) = serde_json::from_str::<Vec<String>>(uv_str) {
                for uv in uvs {
                    uv_map.entry(uv).or_default().push(asset.path.clone());
                }
            }
        }
    }
    for (name, paths) in uv_map {
        if paths.len() > 1 {
            conflicts.push(AssetConflict {
                conflict_type: ConflictType::UVSet,
                name,
                files: paths,
                severity: "medium".to_string(),
            });
        }
    }
    conflicts
}

/// Check if an asset is compatible with a loaded figure
pub fn check_compatibility_mismatch(
    asset: &crate::library_scanner::AssetInfo,
    loaded_figure: &str,
) -> bool {
    if asset.compatibility_base.is_empty() {
        // No compatibility info means we can't determine — assume compatible
        return false;
    }
    // If the loaded figure name appears in any compatibility_base entry, it's compatible
    let figure_lower = loaded_figure.to_lowercase();
    !asset.compatibility_base.iter().any(|base| {
        figure_lower.contains(&base.to_lowercase()) || base.to_lowercase().contains(&figure_lower)
    })
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
        },
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
        },
    };

    // Rename material zones
    if let Some(mat_lib) = data
        .get_mut("material_library")
        .and_then(|m| m.as_array_mut())
    {
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
        obj.insert(
            "material_zones_fixed".to_string(),
            serde_json::Value::Bool(true),
        );
        obj.insert(
            "prefix_added".to_string(),
            serde_json::Value::String(prefix.to_string()),
        );
    } else {
        let mut new_obj = serde_json::Map::new();
        new_obj.insert(
            "material_zones_fixed".to_string(),
            serde_json::Value::Bool(true),
        );
        new_obj.insert(
            "prefix_added".to_string(),
            serde_json::Value::String(prefix.to_string()),
        );
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

/// Fix duplicate morph IDs by prefixing them per file
pub fn fix_morph_ids(root_path: &str, output_dir: &str) -> AssetFixResult {
    let scan = scan_asset_conflicts(root_path);
    let mut fixed_files = Vec::new();
    let mut errors = Vec::new();

    if let Err(e) = fs::create_dir_all(output_dir) {
        return AssetFixResult {
            success: false,
            fixed_files: vec![],
            errors: vec![format!("Failed to create output dir: {}", e)],
        };
    }

    let mut file_prefix_map: HashMap<String, usize> = HashMap::new();

    for conflict in scan.conflicts {
        if let ConflictType::MorphId = conflict.conflict_type {
            for file in &conflict.files {
                let count = file_prefix_map.entry(file.clone()).or_insert(0);
                *count += 1;
                let prefix = format!("FX_{}_", count);

                let content = match fs::read_to_string(file) {
                    Ok(c) => c,
                    Err(e) => {
                        errors.push(format!("Failed to read {}: {}", file, e));
                        continue;
                    },
                };

                let mut data: serde_json::Value = match serde_json::from_str(&content) {
                    Ok(d) => d,
                    Err(e) => {
                        errors.push(format!("Failed to parse {}: {}", file, e));
                        continue;
                    },
                };

                let morph_sources = ["morph_library", "modifier_library"];
                for source in morph_sources.iter() {
                    if let Some(morph_lib) = data.get_mut(source).and_then(|m| m.as_array_mut()) {
                        for morph in morph_lib.iter_mut() {
                            if let Some(id_val) = morph.get_mut("id") {
                                if let Some(id) = id_val.as_str() {
                                    if id == conflict.name {
                                        *id_val =
                                            serde_json::Value::String(format!("{}{}", prefix, id));
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(obj) = data.get_mut("asset_info").and_then(|a| a.as_object_mut()) {
                    obj.insert("morph_ids_fixed".to_string(), serde_json::Value::Bool(true));
                }

                let output_path = format!("{}.fixed", file);
                match fs::write(&output_path, serde_json::to_string_pretty(&data).unwrap()) {
                    Ok(_) => {
                        if !fixed_files.contains(&output_path) {
                            fixed_files.push(output_path);
                        }
                    },
                    Err(e) => errors.push(format!("Failed to write {}: {}", output_path, e)),
                }
            }
        }
    }

    AssetFixResult {
        success: errors.is_empty(),
        fixed_files,
        errors,
    }
}

/// Fix duplicate UV set names by prefixing them within each file
pub fn fix_uv_sets(root_path: &str, output_dir: &str) -> AssetFixResult {
    let scan = scan_asset_conflicts(root_path);
    let mut fixed_files = Vec::new();
    let mut errors = Vec::new();

    if let Err(e) = fs::create_dir_all(output_dir) {
        return AssetFixResult {
            success: false,
            fixed_files: vec![],
            errors: vec![format!("Failed to create output dir: {}", e)],
        };
    }

    for conflict in scan.conflicts {
        if let ConflictType::UVSet = conflict.conflict_type {
            for file in &conflict.files {
                let prefix = detect_prefix_from_conflict(&conflict.name);

                let content = match fs::read_to_string(file) {
                    Ok(c) => c,
                    Err(e) => {
                        errors.push(format!("Failed to read {}: {}", file, e));
                        continue;
                    },
                };

                let mut data: serde_json::Value = match serde_json::from_str(&content) {
                    Ok(d) => d,
                    Err(e) => {
                        errors.push(format!("Failed to parse {}: {}", file, e));
                        continue;
                    },
                };

                if let Some(uv_lib) = data.get_mut("uv_library").and_then(|u| u.as_array_mut()) {
                    for uv in uv_lib.iter_mut() {
                        if let Some(name_val) = uv.get_mut("name") {
                            if let Some(name) = name_val.as_str() {
                                if name == conflict.name {
                                    *name_val =
                                        serde_json::Value::String(format!("{}{}", prefix, name));
                                }
                            }
                        }
                    }
                }

                if let Some(obj) = data.get_mut("asset_info").and_then(|a| a.as_object_mut()) {
                    obj.insert("uv_sets_fixed".to_string(), serde_json::Value::Bool(true));
                }

                let output_path = format!("{}.fixed", file);
                match fs::write(&output_path, serde_json::to_string_pretty(&data).unwrap()) {
                    Ok(_) => {
                        if !fixed_files.contains(&output_path) {
                            fixed_files.push(output_path);
                        }
                    },
                    Err(e) => errors.push(format!("Failed to write {}: {}", output_path, e)),
                }
            }
        }
    }

    AssetFixResult {
        success: errors.is_empty(),
        fixed_files,
        errors,
    }
}

/// Auto-fix all conflicts (MaterialZone, MorphId, UVSet) in a directory
pub fn auto_fix_conflicts(root_path: &str, output_dir: &str) -> AssetFixResult {
    let mut all_fixed = Vec::new();
    let mut all_errors = Vec::new();

    let material = auto_fix_material_zones(root_path, output_dir);
    all_fixed.extend(material.fixed_files);
    all_errors.extend(material.errors);

    let morph = fix_morph_ids(root_path, output_dir);
    all_fixed.extend(morph.fixed_files);
    all_errors.extend(morph.errors);

    let uv = fix_uv_sets(root_path, output_dir);
    all_fixed.extend(uv.fixed_files);
    all_errors.extend(uv.errors);

    all_fixed.sort();
    all_fixed.dedup();

    AssetFixResult {
        success: all_errors.is_empty(),
        fixed_files: all_fixed,
        errors: all_errors,
    }
}

/// Fix material zone conflicts only
fn auto_fix_material_zones(root_path: &str, output_dir: &str) -> AssetFixResult {
    let scan = scan_asset_conflicts(root_path);
    let mut fixed_files = Vec::new();
    let mut errors = Vec::new();

    if let Err(e) = fs::create_dir_all(output_dir) {
        return AssetFixResult {
            success: false,
            fixed_files: vec![],
            errors: vec![format!("Failed to create output dir: {}", e)],
        };
    }

    for conflict in scan.conflicts {
        if let ConflictType::MaterialZone = conflict.conflict_type {
            let prefix = detect_prefix_from_conflict(&conflict.name);
            for file in &conflict.files {
                let result = fix_shell_material_zones(file, &prefix);
                fixed_files.extend(result.fixed_files);
                errors.extend(result.errors);
            }
        }
    }

    AssetFixResult {
        success: errors.is_empty(),
        fixed_files,
        errors,
    }
}

/// Detect appropriate prefix based on conflict name
pub fn detect_prefix_from_conflict(name: &str) -> String {
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
    fn test_has_conflicts() {
        let result = ConflictScanResult {
            total_scanned: 0,
            conflicts: vec![],
            warnings: vec![],
        };
        assert!(!result.has_conflicts());

        let result = ConflictScanResult {
            total_scanned: 1,
            conflicts: vec![AssetConflict {
                conflict_type: ConflictType::MaterialZone,
                name: "test".to_string(),
                files: vec!["test.dsf".to_string()],
                severity: "high".to_string(),
            }],
            warnings: vec![],
        };
        assert!(result.has_conflicts());
    }

    #[test]
    fn test_fix_morph_ids_creates_fixed_file() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        // Morph conflicts are detected cross-file: same morph ID in different files
        let content_a = serde_json::json!({
            "morph_library": [{"id": "Shared_Morph"}],
            "asset_info": {"name": "test_a"}
        });
        let content_b = serde_json::json!({
            "morph_library": [{"id": "Shared_Morph"}],
            "asset_info": {"name": "test_b"}
        });
        let path_a = dir.path().join("test_a.dsf");
        let path_b = dir.path().join("test_b.dsf");
        let mut fa = std::fs::File::create(&path_a).unwrap();
        fa.write_all(content_a.to_string().as_bytes()).unwrap();
        drop(fa);
        let mut fb = std::fs::File::create(&path_b).unwrap();
        fb.write_all(content_b.to_string().as_bytes()).unwrap();
        drop(fb);

        let result = fix_morph_ids(&dir.path().to_string_lossy(), &dir.path().to_string_lossy());
        assert!(result.success);
        assert!(!result.fixed_files.is_empty());

        let fixed_a = format!("{}.fixed", path_a.to_string_lossy());
        let fixed_b = format!("{}.fixed", path_b.to_string_lossy());
        assert!(std::path::Path::new(&fixed_a).exists() || std::path::Path::new(&fixed_b).exists());
    }

    #[test]
    fn test_fix_uv_sets_creates_fixed_file() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_uv.dsf");
        let mut file = std::fs::File::create(&file_path).unwrap();
        let content = serde_json::json!({
            "uv_library": [
                {"name": "UVSet1"},
                {"name": "UVSet1"}
            ],
            "asset_info": {"name": "test"}
        });
        file.write_all(content.to_string().as_bytes()).unwrap();
        drop(file);

        let result = fix_uv_sets(&dir.path().to_string_lossy(), &dir.path().to_string_lossy());
        assert!(result.success);
        assert!(!result.fixed_files.is_empty());

        let fixed_path = format!("{}.fixed", file_path.to_string_lossy());
        assert!(std::path::Path::new(&fixed_path).exists());
    }

    #[test]
    fn test_auto_fix_handles_all_conflict_types() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();

        // File with material zone conflict
        let mat_path = dir.path().join("test_mat.dsf");
        let mut mat_file = std::fs::File::create(&mat_path).unwrap();
        let mat_content = serde_json::json!({
            "material_library": [
                {"id": "Torso"},
                {"id": "Torso"}
            ],
            "asset_info": {"name": "test"}
        });
        mat_file
            .write_all(mat_content.to_string().as_bytes())
            .unwrap();
        drop(mat_file);

        // File with morph ID conflict
        let morph_path = dir.path().join("test_morph.dsf");
        let mut morph_file = std::fs::File::create(&morph_path).unwrap();
        let morph_content = serde_json::json!({
            "morph_library": [
                {"id": "Smile"},
                {"id": "Smile"}
            ],
            "asset_info": {"name": "test"}
        });
        morph_file
            .write_all(morph_content.to_string().as_bytes())
            .unwrap();
        drop(morph_file);

        // File with UV set conflict
        let uv_path = dir.path().join("test_uv.dsf");
        let mut uv_file = std::fs::File::create(&uv_path).unwrap();
        let uv_content = serde_json::json!({
            "uv_library": [
                {"name": "BaseUV"},
                {"name": "BaseUV"}
            ],
            "asset_info": {"name": "test"}
        });
        uv_file
            .write_all(uv_content.to_string().as_bytes())
            .unwrap();
        drop(uv_file);

        let result =
            auto_fix_conflicts(&dir.path().to_string_lossy(), &dir.path().to_string_lossy());
        assert!(result.success);
        assert!(!result.fixed_files.is_empty());
    }

    #[test]
    fn test_prefix_detection() {
        assert_eq!(detect_prefix_from_conflict("Torso"), "FIXED_".to_string());
        assert_eq!(
            detect_prefix_from_conflict("GP_Majora_Torso"),
            "GM_".to_string()
        );
        assert_eq!(
            detect_prefix_from_conflict("GP_Minora_Torso"),
            "GMin_".to_string()
        );
    }

    #[test]
    fn test_compatibility_mismatch_no_info() {
        let asset = crate::library_scanner::AssetInfo {
            path: "test.duf".to_string(),
            name: "test".to_string(),
            file_type: "duf".to_string(),
            size: 100,
            category: "clothing".to_string(),
            subcategory: None,
            metadata: std::collections::HashMap::new(),
            thumbnail_path: None,
            compatibility_base: vec![],
            dforce_enabled: false,
            asset_type_detail: None,
            vendor: None,
            tags: vec![],
            visual_properties: None,
            visual_description: None,
        };
        // Empty compatibility_base means no mismatch detected
        assert!(!check_compatibility_mismatch(&asset, "Genesis 9"));
    }

    #[test]
    fn test_compatibility_mismatch_detected() {
        let asset = crate::library_scanner::AssetInfo {
            path: "test.duf".to_string(),
            name: "test".to_string(),
            file_type: "duf".to_string(),
            size: 100,
            category: "clothing".to_string(),
            subcategory: None,
            metadata: std::collections::HashMap::new(),
            thumbnail_path: None,
            compatibility_base: vec!["Genesis 8 Female".to_string()],
            dforce_enabled: false,
            asset_type_detail: None,
            vendor: None,
            tags: vec![],
            visual_properties: None,
            visual_description: None,
        };
        // Genesis 8 clothing on Genesis 9 should be a mismatch
        assert!(check_compatibility_mismatch(&asset, "Genesis 9"));
    }

    #[test]
    fn test_compatibility_mismatch_compatible() {
        let asset = crate::library_scanner::AssetInfo {
            path: "test.duf".to_string(),
            name: "test".to_string(),
            file_type: "duf".to_string(),
            size: 100,
            category: "clothing".to_string(),
            subcategory: None,
            metadata: std::collections::HashMap::new(),
            thumbnail_path: None,
            compatibility_base: vec!["Genesis 9".to_string(), "Genesis 8 Female".to_string()],
            dforce_enabled: false,
            asset_type_detail: None,
            vendor: None,
            tags: vec![],
            visual_properties: None,
            visual_description: None,
        };
        // Genesis 9 is in compatibility_base, so no mismatch
        assert!(!check_compatibility_mismatch(&asset, "Genesis 9"));
    }
}
