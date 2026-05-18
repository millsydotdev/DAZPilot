use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use regex::Regex;
use once_cell::sync::Lazy;
use walkdir::WalkDir;

static SDK_INDEX: Lazy<Mutex<Option<SdkIndex>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkMethod {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<String>,
    pub description: String,
    pub access: String,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkEnumValue {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkEnum {
    pub name: String,
    pub values: Vec<SdkEnumValue>,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkClass {
    pub name: String,
    pub file: String,
    pub line: usize,
    pub description: String,
    pub parents: Vec<String>,
    pub methods: Vec<SdkMethod>,
    pub enums: Vec<SdkEnum>,
    pub related_classes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkIndex {
    pub classes: Vec<SdkClass>,
    pub inheritance: HashMap<String, Vec<String>>,
}

pub fn get_sdk_include_path() -> String {
    // 1. Try DB first
    if let Ok(Some(path)) = crate::database::get_setting("daz_sdk_path") {
        if !path.is_empty() {
            return path;
        }
    }
    
    // 2. Try environment variable
    if let Ok(path) = std::env::var("DAZ_SDK_PATH") {
        if !path.is_empty() {
            return path;
        }
    }
    
    // 3. Try DIM (Daz Install Manager) common paths
    if let Some(dim_path) = discover_sdk_from_dim() {
        return dim_path;
    }
    
    // 4. Dynamic search: look for SDK directory relative to current executable
    if let Ok(exe_path) = std::env::current_exe() {
        let mut dir = exe_path.parent();
        while let Some(d) = dir {
            let candidate = d.join("DAZStudio4.5+ SDK").join("include");
            if candidate.exists() {
                return candidate.to_string_lossy().to_string();
            }
            // Also check if we are inside the SDK folder
            let candidate_workspace = d.join("include");
            if d.file_name().and_then(|n| n.to_str()).unwrap_or("").contains("DAZStudio4.5+ SDK") && candidate_workspace.exists() {
                return candidate_workspace.to_string_lossy().to_string();
            }
            dir = d.parent();
        }
    }
    
    // 5. Default fallback - return empty, user must configure via settings or DIM
    String::new()
}

fn discover_sdk_from_dim() -> Option<String> {
    // Common DIM install locations for DAZStudio SDK
    let candidates = vec![
        // Windows DIM default content locations
        dirs::home_dir()?.join("Documents/DAZ 3D/DAZStudio4.5+ SDK"),
        dirs::home_dir()?.join("My DAZ 3D Library/DAZStudio4.5+ SDK"),
        dirs::home_dir()?.join("Documents/DAZStudio4.5+ SDK"),
        // Public documents (Windows)
        dirs::document_dir()?.join("DAZ 3D/DAZStudio4.5+ SDK"),
        // ProgramData (Windows)
        PathBuf::from("C:/ProgramData/DAZ 3D/DAZStudio4.5+ SDK"),
        // macOS
        dirs::home_dir()?.join("Library/Application Support/DAZ 3D/DAZStudio4.5+ SDK"),
        // Linux
        dirs::home_dir()?.join(".local/share/DAZ 3D/DAZStudio4.5+ SDK"),
    ];
    
    for candidate in candidates {
        let include_path = candidate.join("include");
        if include_path.exists() {
            log::info!("Found Daz SDK via DIM discovery: {}", include_path.display());
            return Some(include_path.to_string_lossy().to_string());
        }
    }
    
    None
}

pub fn parse_all_headers() -> SdkIndex {
    let sdk_path = get_sdk_include_path();
    let mut classes = Vec::new();
    let mut inheritance: HashMap<String, Vec<String>> = HashMap::new();
    
    let header_pattern = Regex::new(r"^dz.*\.h$").unwrap();
    
    if Path::new(&sdk_path).exists() {
        let mut headers: Vec<_> = WalkDir::new(&sdk_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| header_pattern.is_match(n))
                    .unwrap_or(false)
            })
            .collect();
        
        headers.sort();
        
        println!("Found {} Daz SDK headers to parse", headers.len());
        
        for (i, header_path) in headers.iter().enumerate() {
            if i % 50 == 0 {
                println!("Parsing header {} of {}", i, headers.len());
            }
            
            if let Ok(class) = parse_header(&header_path) {
                if !class.name.is_empty() {
                    for parent in &class.parents {
                        inheritance
                            .entry(parent.clone())
                            .or_default()
                            .push(class.name.clone());
                    }
                    classes.push(class);
                }
            }
        }
    }
    
    println!("Parsed {} SDK classes", classes.len());
    
    let index = SdkIndex { classes, inheritance };
    if let Err(e) = persist_sdk_index(&index) {
        log::warn!("Failed to persist SDK index: {}", e);
    }
    index
}

fn parse_header(header_path: &Path) -> Result<SdkClass, String> {
    let content = fs::read_to_string(header_path)
        .map_err(|e| format!("Failed to read header: {}", e))?;
    
    let filename = header_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    let mut class = SdkClass {
        name: String::new(),
        file: filename.clone(),
        line: 0,
        description: String::new(),
        parents: Vec::new(),
        methods: Vec::new(),
        enums: Vec::new(),
        related_classes: Vec::new(),
    };
    
    let lines: Vec<&str> = content.lines().collect();
    
    // Find class definition
    let class_regex = Regex::new(r"class\s+(?:\w+_API\s+)?(Dz\w+)\s*:([^\\{;]+)").unwrap();
    let simple_class_regex = Regex::new(r"class\s+(?:\w+_API\s+)?(Dz\w+)").unwrap();
    
    for (i, line) in lines.iter().enumerate() {
        if let Some(caps) = class_regex.captures(line) {
            class.name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            class.line = i + 1;
            let parents = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            for parent in Regex::new(r"public\s+(Dz\w+)").unwrap().captures_iter(parents) {
                class.parents.push(parent[1].to_string());
            }
            class.description = extract_description(&lines, i);
            break;
        } else if let Some(caps) = simple_class_regex.captures(line) {
            class.name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            class.line = i + 1;
            class.description = extract_description(&lines, i);
            break;
        }
    }
    
    if class.name.is_empty() {
        return Err("No Dz class found".to_string());
    }
    
    // Find inheritance
    let inherit_regex = Regex::new(r"public\s+(Dz\w+)").unwrap();
    for line in &lines {
        if let Some(caps) = inherit_regex.captures(line) {
            if let Some(parent) = caps.get(1) {
                let parent_name = parent.as_str().to_string();
                if !class.parents.contains(&parent_name) {
                    class.parents.push(parent_name);
                }
            }
        }
    }
    
    // Parse methods - look for public methods
    let in_public_regex = Regex::new(r"^\s*public\s*:").unwrap();
    let method_regex = Regex::new(r"^\s*(?:virtual\s+|static\s+|inline\s+|DZ_\w+\s+)*([A-Za-z_]\w*(?:::\w+)?(?:\s*[*&])?(?:\s+const)?)\s+(\w+)\s*\(([^)]*)\)\s*(?:const)?\s*(?:=\s*0)?\s*;").unwrap();
    let slot_regex = Regex::new(r"(public|protected)\s+slots:").unwrap();
    
    let mut in_public_section = false;
    let mut in_slots = false;
    
    let mut pending_signature = String::new();
    let mut pending_line = 0usize;

    for (line_index, line) in lines.iter().enumerate() {
        if in_public_regex.is_match(line) {
            in_public_section = true;
            in_slots = false;
            continue;
        }
        
        if line.contains("private:") || line.contains("protected:") {
            in_public_section = false;
            in_slots = false;
            continue;
        }
        
        if slot_regex.is_match(line) {
            in_slots = true;
            in_public_section = true;
            continue;
        }
        
        if (in_public_section || in_slots) && !line.trim_start().starts_with("//") {
            if pending_signature.is_empty() {
                pending_line = line_index + 1;
            }
            pending_signature.push(' ');
            pending_signature.push_str(line.trim());

            if !pending_signature.contains(';') {
                continue;
            }

            let signature = pending_signature.clone();
            pending_signature.clear();

            if let Some(caps) = method_regex.captures(&signature) {
                let return_type = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("void").to_string();
                let name = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
                let params = caps.get(3).map(|m| m.as_str()).unwrap_or("");
                
                // Skip constructors/destructors and private methods
                if name.is_empty() || name == class.name || name.starts_with('_') {
                    continue;
                }
                
                let param_list: Vec<String> = params
                    .split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect();
                
                let access = if in_slots { "slot".to_string() } else { "public".to_string() };
                
                let description = extract_description(&lines, pending_line.saturating_sub(1));
                
                class.methods.push(SdkMethod {
                    name,
                    return_type,
                    parameters: param_list,
                    description,
                    access,
                    line: pending_line,
                });
            }
        }
    }
    
    // Parse enums
    let enum_regex = Regex::new(r"(?:Q_ENUM|Q_ENUMS)\s*\((\w+)\)").unwrap();
    for (line_index, line) in lines.iter().enumerate() {
        if let Some(caps) = enum_regex.captures(line) {
            if let Some(enum_name) = caps.get(1) {
                let values = extract_enum_values(&lines, enum_name.as_str());
                class.enums.push(SdkEnum {
                    name: enum_name.as_str().to_string(),
                    values,
                    line: line_index + 1,
                });
            }
        }
    }
    
    // Build related classes
    let mut related: std::collections::HashSet<String> = std::collections::HashSet::new();
    for parent in &class.parents {
        if parent.starts_with("Dz") {
            related.insert(parent.clone());
        }
    }
    for method in &class.methods {
        if method.return_type.starts_with("Dz") {
            related.insert(method.return_type.clone());
        }
        for param in &method.parameters {
            if param.starts_with("Dz") {
                related.insert(param.split_whitespace().next().unwrap_or("").to_string());
            }
        }
    }
    class.related_classes = related.into_iter().collect();
    class.related_classes.sort();
    
    Ok(class)
}

fn persist_sdk_index(index: &SdkIndex) -> Result<(), String> {
    let db_guard = crate::database::get_db()?;
    let Some(db) = db_guard.as_ref() else {
        return Ok(());
    };

    db.execute(
        r#"
        CREATE TABLE IF NOT EXISTS sdk_classes (
            name TEXT PRIMARY KEY,
            file TEXT NOT NULL,
            line INTEGER NOT NULL,
            description TEXT,
            parents TEXT,
            related_classes TEXT,
            indexed_at TEXT DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS sdk_methods (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            class_name TEXT NOT NULL,
            name TEXT NOT NULL,
            return_type TEXT,
            parameters TEXT,
            description TEXT,
            access TEXT,
            line INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS sdk_enums (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            class_name TEXT NOT NULL,
            name TEXT NOT NULL,
            values_json TEXT,
            line INTEGER NOT NULL
        );
        DELETE FROM sdk_methods;
        DELETE FROM sdk_enums;
        DELETE FROM sdk_classes;
        "#,
    )
    .map_err(|e| e.to_string())?;

    let mut conn = rusqlite::Connection::open(db.path())
        .map_err(|e| format!("Failed to open SDK index db: {}", e))?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    for class in &index.classes {
        tx.execute(
            "INSERT OR REPLACE INTO sdk_classes (name, file, line, description, parents, related_classes) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                class.name,
                class.file,
                class.line as i64,
                class.description,
                serde_json::to_string(&class.parents).unwrap_or_default(),
                serde_json::to_string(&class.related_classes).unwrap_or_default(),
            ],
        ).map_err(|e| e.to_string())?;
        for method in &class.methods {
            tx.execute(
                "INSERT INTO sdk_methods (class_name, name, return_type, parameters, description, access, line) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    class.name,
                    method.name,
                    method.return_type,
                    serde_json::to_string(&method.parameters).unwrap_or_default(),
                    method.description,
                    method.access,
                    method.line as i64,
                ],
            ).map_err(|e| e.to_string())?;
        }
        for sdk_enum in &class.enums {
            tx.execute(
                "INSERT INTO sdk_enums (class_name, name, values_json, line) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    class.name,
                    sdk_enum.name,
                    serde_json::to_string(&sdk_enum.values).unwrap_or_default(),
                    sdk_enum.line as i64,
                ],
            ).map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())
}

fn extract_description(lines: &[&str], pos: usize) -> String {
    if pos > 0 {
        for i in (0..pos.min(5)).rev() {
            let line = lines[pos - 1 - i];
            if line.contains("@brief") || line.contains("\\brief") {
                let start = line.find("@brief").unwrap_or(line.find("\\brief").unwrap()) + 6;
                return line[start..].trim().to_string();
            }
            if line.contains("///") || line.contains("/**") {
                let cleaned = line
                    .replace("///", "")
                    .replace("/**", "")
                    .replace("*/", "")
                    .trim()
                    .to_string();
                if !cleaned.is_empty() {
                    return cleaned;
                }
            }
        }
    }
    String::new()
}

fn extract_enum_values(lines: &[&str], enum_name: &str) -> Vec<SdkEnumValue> {
    let mut values = Vec::new();
    let in_enum_regex = Regex::new(&format!(r"enum\s+{}", enum_name)).unwrap();
    let value_regex = Regex::new(r"(\w+)\s*(?:=\s*(\w+))?").unwrap();
    
    let mut in_target_enum = false;
    let _brace_count = 0;
    
    for line in lines {
        if in_enum_regex.is_match(line) || line.contains(&format!("enum {}", enum_name)) {
            in_target_enum = true;
            continue;
        }
        
        if in_target_enum {
            if line.contains('}') {
                break;
            }
            
            if let Some(caps) = value_regex.captures(line) {
                let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let value = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                
                if !name.is_empty() && name != "enum" && name != "Q_ENUM" {
                    values.push(SdkEnumValue {
                        name: name.to_string(),
                        value: value.to_string(),
                    });
                }
            }
        }
    }
    
    values
}

pub fn load_or_build_index() -> SdkIndex {
    let mut guard = SDK_INDEX.lock().unwrap();
    
    if let Some(ref index) = *guard {
        return index.clone();
    }
    
    log::info!("Starting SDK header indexing - this may take a moment...");
    let index = parse_all_headers();
    log::info!("SDK indexing complete: {} classes, {} methods", 
        index.classes.len(),
        index.classes.iter().map(|c| c.methods.len()).sum::<usize>()
    );
    *guard = Some(index.clone());
    
    index
}

pub fn get_class(name: &str) -> Option<SdkClass> {
    let index = load_or_build_index();
    index.classes.into_iter().find(|c| c.name == name)
}

pub fn search_classes(query: &str) -> Vec<SdkClass> {
    let index = load_or_build_index();
    let query_lower = query.to_lowercase();
    
    index.classes
        .into_iter()
        .filter(|c| {
            c.name.to_lowercase().contains(&query_lower) ||
            c.description.to_lowercase().contains(&query_lower) ||
            c.file.to_lowercase().contains(&query_lower) ||
            c.methods.iter().any(|m| 
                m.name.to_lowercase().contains(&query_lower) ||
                m.return_type.to_lowercase().contains(&query_lower)
            )
        })
        .collect()
}

pub fn get_method_help(class_name: &str, method_name: &str) -> Option<SdkMethod> {
    get_class(class_name)
        .and_then(|c| c.methods.into_iter().find(|m| m.name == method_name))
}

pub fn get_related_classes(class_name: &str) -> Vec<String> {
    let index = load_or_build_index();
    
    let mut related = Vec::new();
    
    if let Some(class) = index.classes.iter().find(|c| c.name == class_name) {
        related.extend(class.related_classes.clone());
    }
    
    if let Some(children) = index.inheritance.get(class_name) {
        related.extend(children.clone());
    }
    
    related.sort();
    related.dedup();
    related
}

pub fn get_all_class_names() -> Vec<String> {
    let index = load_or_build_index();
    index.classes.iter().map(|c| c.name.clone()).collect()
}

#[tauri::command]
pub fn get_deep_sdk_index() -> SdkIndex {
    load_or_build_index()
}

#[tauri::command]
pub fn get_sdk_class_deep(name: String) -> Option<SdkClass> {
    get_class(&name)
}

#[tauri::command]
pub fn search_sdk_deep(query: String) -> Vec<SdkClass> {
    search_classes(&query)
}

#[tauri::command]
pub fn get_sdk_method_help(class_name: String, method_name: String) -> Option<SdkMethod> {
    get_method_help(&class_name, &method_name)
}

#[tauri::command]
pub fn get_sdk_related_classes(class_name: String) -> Vec<String> {
    get_related_classes(&class_name)
}

#[tauri::command]
pub fn get_all_sdk_classes() -> Vec<String> {
    get_all_class_names()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkIndexerStatus {
    pub sdk_path: String,
    pub classes_found: usize,
    pub methods_found: usize,
    pub enums_found: usize,
    pub is_loaded: bool,
}

#[tauri::command]
pub fn get_sdk_indexer_status() -> SdkIndexerStatus {
    let index = load_or_build_index();
    let methods_count: usize = index.classes.iter().map(|c| c.methods.len()).sum();
    let enums_count: usize = index.classes.iter().map(|c| c.enums.len()).sum();
    
    SdkIndexerStatus {
        sdk_path: get_sdk_include_path(),
        classes_found: index.classes.len(),
        methods_found: methods_count,
        enums_found: enums_count,
        is_loaded: true,
    }
}

#[tauri::command]
pub fn set_sdk_indexer_path(path: String) -> Result<String, String> {
    crate::database::save_setting("daz_sdk_path", &path)?;
    std::env::set_var("DAZ_SDK_PATH", &path);
    
    let mut guard = SDK_INDEX.lock().unwrap();
    *guard = None;
    
    let index = parse_all_headers();
    *guard = Some(index);
    
    Ok(format!("SDK path set to: {}, indexed {} classes", path, guard.as_ref().map(|i| i.classes.len()).unwrap_or(0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_path_is_reported() {
        assert!(!get_sdk_include_path().is_empty());
    }

    #[test]
    fn search_handles_empty_index() {
        let results = search_classes("definitely-not-a-daz-class-name");
        assert!(results.iter().all(|class| !class.name.is_empty()));
    }
}
