#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportFormat {
    Obj,
    Fbx,
    Gltf,
    Dae,
    Ply,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSettings {
    pub format: ImportFormat,
    pub auto_skeleton: bool,
    pub skin_binding: bool,
    pub scale: f32,
    pub axis_conversion: AxisConvention,
    pub import_materials: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AxisConvention {
    YUp,
    ZUp,
    XUp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub success: bool,
    pub node_id: Option<String>,
    pub vertex_count: u32,
    pub face_count: u32,
    pub material_count: u32,
    pub bone_count: Option<u32>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Obj,
    Fbx,
    Gltf,
    Dae,
    Daz,
    Image,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    pub format: ExportFormat,
    pub quality: ExportQuality,
    pub include_materials: bool,
    pub include_animations: bool,
    pub bake_textures: bool,
    pub compression: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub success: bool,
    pub file_path: String,
    pub file_size: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExport {
    pub assets: Vec<String>,
    pub output_directory: String,
    pub format: ExportFormat,
    pub settings: ExportSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub total: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub results: Vec<ExportResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneAction {
    pub action_type: String,
    pub target: String,
    pub parameters: HashMap<String, String>,
    pub delay_frames: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneSequence {
    pub name: String,
    pub actions: Vec<SceneAction>,
    pub total_duration: f32,
    pub loop_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneComposition {
    pub name: String,
    pub description: String,
    pub sequences: Vec<SceneSequence>,
    pub camera_work: Vec<CameraAction>,
    pub transitions: Vec<Transition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraAction {
    pub camera_id: String,
    pub action_type: CameraActionType,
    pub start_frame: u32,
    pub end_frame: u32,
    pub parameters: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraActionType {
    Move,
    Rotate,
    Zoom,
    Pan,
    Cut,
    Dolly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub from_sequence: String,
    pub to_sequence: String,
    pub transition_type: TransitionType,
    pub duration: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionType {
    Cut,
    Fade,
    Dissolve,
    Wipe,
    Slide,
}

pub fn get_default_import_settings() -> ImportSettings {
    ImportSettings {
        format: ImportFormat::Obj,
        auto_skeleton: false,
        skin_binding: false,
        scale: 1.0,
        axis_conversion: AxisConvention::YUp,
        import_materials: true,
    }
}

pub fn get_default_export_settings() -> ExportSettings {
    ExportSettings {
        format: ExportFormat::Obj,
        quality: ExportQuality::High,
        include_materials: true,
        include_animations: true,
        bake_textures: false,
        compression: false,
    }
}

pub fn import_model(path: &str, settings: ImportSettings) -> ImportResult {
    log::info!("Importing model: {} as {:?}", path, settings.format);

    match crate::mcp_client::send_mcp_request(
        "import_model",
        serde_json::json!({
            "path": path,
            "settings": settings
        }),
    ) {
        Ok(resp) => ImportResult {
            success: true,
            node_id: resp
                .data
                .as_ref()
                .and_then(|d| d.get("node_id"))
                .and_then(|v| v.as_str())
                .map(ToString::to_string),
            vertex_count: 0,
            face_count: 0,
            material_count: 0,
            bone_count: None,
            message: resp
                .result
                .unwrap_or_else(|| format!("Daz import requested for {}", path)),
        },
        Err(e) => ImportResult {
            success: false,
            node_id: None,
            vertex_count: 0,
            face_count: 0,
            material_count: 0,
            bone_count: None,
            message: format!("Import not completed: {}", e),
        },
    }
}

pub fn export_scene(node_id: &str, path: &str, settings: ExportSettings) -> ExportResult {
    log::info!("Exporting node {} to {} as {:?}", node_id, path, settings.format);

    if let Some(parent) = std::path::Path::new(path).parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return ExportResult {
                success: false,
                file_path: path.to_string(),
                file_size: 0,
                message: format!("Failed to create export directory: {}", e),
            };
        }
    }

    // Try the bridge export_scene command first
    match crate::mcp_client::send_mcp_request(
        "export_scene",
        serde_json::json!({
            "node_id": node_id,
            "path": path,
            "settings": settings
        }),
    ) {
        Ok(resp) => ExportResult {
            success: true,
            file_path: path.to_string(),
            file_size: resp
                .data
                .as_ref()
                .and_then(|d| d.get("file_size"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            message: resp
                .result
                .unwrap_or_else(|| format!("Daz export requested for {}", path)),
        },
        Err(bridge_err) => {
            log::warn!("Bridge export_scene failed, trying DazScript fallback: {}", bridge_err);
            export_via_dazscript(node_id, path, &settings)
        }
    }
}

fn export_via_dazscript(node_id: &str, path: &str, settings: &ExportSettings) -> ExportResult {
    let ext = match settings.format {
        ExportFormat::Obj => "obj",
        ExportFormat::Fbx => "fbx",
        ExportFormat::Gltf => "glb",
        ExportFormat::Dae => "dae",
        ExportFormat::Daz => "duf",
        ExportFormat::Image => "png",
        ExportFormat::Video => "mp4",
    };

    let escaped_path = path.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        r#"
        var node = Scene.findNodeByLabel("{}");
        if (!node) {{
            node = Scene.getNode("{}");
        }}
        if (node) {{
            var exporter = App.createExporter("{}");
            if (exporter) {{
                exporter.setExportFilename("{}");
                exporter.exportObject(node);
                "Exported to {}"
            }} else {{
                "Exporter not available for format: {}"
            }}
        }} else {{
            "Node not found: {}"
        }}
        "#,
        node_id, node_id, ext, escaped_path, escaped_path, ext, node_id
    );

    match crate::mcp_client::send_mcp_request(
        "run_script",
        serde_json::json!({ "script": script, "args": {} }),
    ) {
        Ok(resp) => {
            let msg = resp.result.unwrap_or_else(|| format!("Export requested via DazScript for {}", path));
            ExportResult {
                success: true,
                file_path: path.to_string(),
                file_size: resp
                    .data
                    .as_ref()
                    .and_then(|d| d.get("file_size"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                message: msg,
            }
        }
        Err(e) => ExportResult {
            success: false,
            file_path: path.to_string(),
            file_size: 0,
            message: format!("Export not completed (bridge and DazScript both failed): {}", e),
        },
    }
}

pub fn batch_export(batch: BatchExport) -> BatchResult {
    log::info!("Batch exporting {} assets", batch.assets.len());
    
    if let Err(e) = std::fs::create_dir_all(&batch.output_directory) {
        return BatchResult {
            total: batch.assets.len() as u32,
            succeeded: 0,
            failed: batch.assets.len() as u32,
            results: batch.assets.iter().map(|asset| {
                ExportResult {
                    success: false,
                    file_path: format!("{}/{}.obj", batch.output_directory, asset),
                    file_size: 0,
                    message: format!("Failed to create batch export directory: {}", e),
                }
            }).collect(),
        };
    }
    
    let mut results = vec![];
    let mut succeeded = 0u32;
    let mut failed = 0u32;
    
    for asset in &batch.assets {
        let result = export_scene(asset, &format!("{}/{}.obj", batch.output_directory, asset), batch.settings.clone());
        if result.success {
            succeeded += 1;
        } else {
            failed += 1;
        }
        results.push(result);
    }
    
    BatchResult {
        total: batch.assets.len() as u32,
        succeeded,
        failed,
        results,
    }
}

pub fn create_scene_sequence(name: &str) -> SceneSequence {
    SceneSequence {
        name: name.to_string(),
        actions: vec![],
        total_duration: 0.0,
        loop_enabled: false,
    }
}

pub fn add_action_to_sequence(sequence: &mut SceneSequence, action: SceneAction) {
    if let Some(delay) = action.delay_frames {
        sequence.total_duration += delay as f32 / 30.0;
    }
    sequence.actions.push(action);
}

pub fn create_scene_composition(name: &str, description: &str) -> SceneComposition {
    SceneComposition {
        name: name.to_string(),
        description: description.to_string(),
        sequences: vec![],
        camera_work: vec![],
        transitions: vec![],
    }
}

pub fn create_camera_action(
    camera_id: &str,
    action_type: CameraActionType,
    start: u32,
    end: u32,
) -> CameraAction {
    CameraAction {
        camera_id: camera_id.to_string(),
        action_type,
        start_frame: start,
        end_frame: end,
        parameters: HashMap::new(),
    }
}

pub fn add_camera_action(composition: &mut SceneComposition, action: CameraAction) {
    composition.camera_work.push(action);
}

pub fn add_transition(composition: &mut SceneComposition, from: &str, to: &str, trans_type: TransitionType, duration: f32) {
    composition.transitions.push(Transition {
        from_sequence: from.to_string(),
        to_sequence: to.to_string(),
        transition_type: trans_type,
        duration,
    });
}

pub fn get_supported_import_formats() -> Vec<String> {
    vec!["OBJ".to_string(), "FBX".to_string(), "glTF".to_string(), "DAE".to_string(), "PLY".to_string()]
}

pub fn get_supported_export_formats() -> Vec<String> {
    vec!["OBJ".to_string(), "FBX".to_string(), "glTF".to_string(), "DAE".to_string(), "DAZ".to_string(), "PNG".to_string(), "JPEG".to_string(), "MP4".to_string()]
}

pub fn get_export_presets() -> HashMap<String, ExportSettings> {
    let mut presets = HashMap::new();
    
    presets.insert("web".to_string(), ExportSettings {
        format: ExportFormat::Gltf,
        quality: ExportQuality::Medium,
        include_materials: true,
        include_animations: true,
        bake_textures: false,
        compression: true,
    });
    
    presets.insert("archival".to_string(), ExportSettings {
        format: ExportFormat::Daz,
        quality: ExportQuality::Ultra,
        include_materials: true,
        include_animations: true,
        bake_textures: false,
        compression: false,
    });
    
    presets.insert("web3d".to_string(), ExportSettings {
        format: ExportFormat::Gltf,
        quality: ExportQuality::Low,
        include_materials: true,
        include_animations: false,
        bake_textures: true,
        compression: true,
    });
    
    presets
}
