use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "export_fbx",
        "Exports selected scene nodes to FBX format with configurable scale, axis system, and embedding options.",
        ToolCategory::Export,
        [
            tool_param("filepath", "Full output path including .fbx extension", true, ToolParamType::String),
            tool_param("selected_only", "Export only selected nodes (default true)", false, ToolParamType::Boolean),
            tool_param("scale", "Scale multiplier: 0.01 for game engines, 1.0 for Daz-native (default 0.01)", false, ToolParamType::Number),
            tool_param("axis_system", "Axis system: z_up (Unreal/Unity), y_up (Blender/Maya), daz (default z_up)", false, ToolParamType::String),
            tool_param("embed_textures", "Embed textures in the FBX file (default false)", false, ToolParamType::Boolean),
            tool_param("triangulate", "Triangulate all meshes (default false)", false, ToolParamType::Boolean),
        ],
        "Result with export file path, node count, and file size",
        [
            "Export the figure as FBX for Unity",
            "Export scene as FBX for Blender with Y-up",
            "Export triangulated FBX with embedded textures",
        ],
        handle_export_fbx
    );
    define_tool!(
        "export_obj",
        "Exports selected nodes to Wavefront OBJ format. Best for static meshes and universal compatibility.",
        ToolCategory::Export,
        [
            tool_param("filepath", "Full output path (.obj file)", true, ToolParamType::String),
            tool_param("selected_only", "Export only selected nodes (default true)", false, ToolParamType::Boolean),
            tool_param("export_materials", "Export .mtl material file (default true)", false, ToolParamType::Boolean),
        ],
        "Result with export file paths",
        [
            "Export the prop as OBJ with materials",
            "Export selected nodes as OBJ",
        ],
        handle_export_obj
    );
    define_tool!(
        "export_gltf",
        "Exports scene to glTF or GLB format. Best for web, AR/VR, and modern game engines.",
        ToolCategory::Export,
        [
            tool_param(
                "filepath",
                "Full output path (.gltf or .glb extension)",
                true,
                ToolParamType::String
            ),
            tool_param(
                "selected_only",
                "Export only selected nodes (default true)",
                false,
                ToolParamType::Boolean
            ),
            tool_param(
                "binary",
                "Export as binary .glb (default false)",
                false,
                ToolParamType::Boolean
            ),
            tool_param(
                "compress",
                "Apply Draco mesh compression (default false)",
                false,
                ToolParamType::Boolean
            ),
            tool_param(
                "include_animations",
                "Include animation data (default true)",
                false,
                ToolParamType::Boolean
            ),
        ],
        "Result with export file path and format info",
        [
            "Export the scene as GLB for web viewing",
            "Export with Draco compression",
            "Export animation as glTF",
        ],
        handle_export_gltf
    );
    define_tool!(
        "get_export_recommendations",
        "Recommends optimal export settings (format, scale, axis system, compression) based on target use case.",
        ToolCategory::Export,
        [
            tool_param("target", "Target: unity, unreal, blender, maya, web, sketchfab, printing", false, ToolParamType::String),
            tool_param("has_animations", "Whether export includes animations (default false)", false, ToolParamType::Boolean),
        ],
        "Result with recommended format and settings",
        [
            "What export settings for Unity?",
            "Recommend export for Sketchfab with animations",
            "Best format for 3D printing",
        ],
        handle_get_export_recommendations
    );
    define_tool!(
        "export_collada",
        "Exports scene nodes to Collada DAE format. Good for legacy pipeline compatibility.",
        ToolCategory::Export,
        [
            tool_param(
                "filepath",
                "Full output path including .dae extension",
                true,
                ToolParamType::String
            ),
            tool_param(
                "selected_only",
                "Export selected nodes only (default true)",
                false,
                ToolParamType::Boolean
            ),
            tool_param(
                "export_normals",
                "Export normals (default true)",
                false,
                ToolParamType::Boolean
            ),
            tool_param(
                "export_uvs",
                "Export UV coordinates (default true)",
                false,
                ToolParamType::Boolean
            ),
        ],
        "Result confirming Collada export",
        [
            "Export as Collada DAE for compatibility",
            "Export the model as DAE with normals",
        ],
        handle_export_collada
    );
    define_tool!(
        "export_usd",
        "Exports scene to Universal Scene Description (USD/USDZ) format. Best for Pixar-compatible pipelines and ARKit.",
        ToolCategory::Export,
        [
            tool_param("filepath", "Full output path (.usd, .usda, .usdc, or .usdz)", true, ToolParamType::String),
            tool_param("selected_only", "Export selected nodes only (default true)", false, ToolParamType::Boolean),
            tool_param("include_animations", "Include animation data (default true)", false, ToolParamType::Boolean),
            tool_param("flatten", "Flatten into single file (default true)", false, ToolParamType::Boolean),
        ],
        "Result confirming USD export",
        [
            "Export as USDZ for ARKit",
            "Export scene as USDA for Pixar pipeline",
        ],
        handle_export_usd
    );
    define_tool!(
        "export_selected_only",
        "Exports only the currently selected nodes in the most appropriate format based on content type.",
        ToolCategory::Export,
        [
            tool_param("filepath", "Full output path (extension determines format)", true, ToolParamType::String),
            tool_param("format", "Force specific format: fbx, obj, gltf, glb, dae, usd (auto-detected from extension if omitted)", false, ToolParamType::String),
        ],
        "Result confirming selected export",
        [
            "Export the selected items to my desktop as FBX",
            "Export selection as OBJ",
        ],
        handle_export_selected_only
    );
    define_tool!(
        "batch_export",
        "Exports multiple scene items or cameras in batch, each to separate files with naming convention.",
        ToolCategory::Export,
        [
            tool_param("export_dir", "Directory to export files to", true, ToolParamType::String),
            tool_param("items", "Array of node IDs to export, or 'all' for entire scene", true, ToolParamType::Object),
            tool_param("format", "Export format: fbx, obj, gltf, glb, dae (default fbx)", false, ToolParamType::String),
            tool_param("name_pattern", "Naming pattern: {name}, {index}, {date} supported (default '{name}')", false, ToolParamType::String),
        ],
        "Result with list of exported files",
        [
            "Export all figures as individual FBX files",
            "Batch export selected props as OBJ",
        ],
        handle_batch_export
    );
    define_tool!(
        "export_animation",
        "Exports animation data for a figure. Supports FBX, BVH, and glTF animation export formats.",
        ToolCategory::Export,
        [
            tool_param("filepath", "Full output path", true, ToolParamType::String),
            tool_param("figure_id", "Figure node ID to export animation from", true, ToolParamType::String),
            tool_param("format", "Animation format: fbx, bvh, gltf (default fbx)", false, ToolParamType::String),
            tool_param("bake_keyframes", "Bake procedural animation to keyframes (default true)", false, ToolParamType::Boolean),
            tool_param("frame_rate", "Target frame rate (default 30)", false, ToolParamType::Integer),
        ],
        "Result confirming animation export",
        [
            "Export the walk animation as BVH",
            "Export character animation as FBX at 60fps",
        ],
        handle_export_animation
    );
}
fn handle_export_fbx(request: ToolRequest) -> ToolResponse {
    let filepath = request.get_str("filepath").unwrap_or_default();
    if filepath.is_empty() {
        return ToolResponse::err("export_fbx", "filepath is required");
    }
    let selected_only = request.get_bool("selected_only").unwrap_or(true);
    let scale = request.get_f64("scale").unwrap_or(0.01);
    let axis = request
        .get_str("axis_system")
        .unwrap_or_else(|| "z_up".to_string());
    let embed = request.get_bool("embed_textures").unwrap_or(false);
    let tri = request.get_bool("triangulate").unwrap_or(false);
    let result = crate::mcp_client::send_mcp_request(
        "export_fbx",
        serde_json::json!({ "filepath": filepath, "selected_only": selected_only, "scale": scale, "axis_system": axis, "embed_textures": embed, "triangulate": tri }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "export_fbx",
            serde_json::json!({ "filepath": filepath, "result": r.data }),
            format!("Exported FBX to '{}'", filepath),
        ),
        Err(e) => ToolResponse::err("export_fbx", e),
    }
}
fn handle_export_obj(request: ToolRequest) -> ToolResponse {
    let filepath = request.get_str("filepath").unwrap_or_default();
    if filepath.is_empty() {
        return ToolResponse::err("export_obj", "filepath is required");
    }
    let selected_only = request.get_bool("selected_only").unwrap_or(true);
    let mat = request.get_bool("export_materials").unwrap_or(true);
    let result = crate::mcp_client::send_mcp_request(
        "export_obj",
        serde_json::json!({ "filepath": filepath, "selected_only": selected_only, "export_materials": mat }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "export_obj",
            serde_json::json!({ "filepath": filepath, "result": r.data }),
            format!("Exported OBJ to '{}'", filepath),
        ),
        Err(e) => ToolResponse::err("export_obj", e),
    }
}
fn handle_export_gltf(request: ToolRequest) -> ToolResponse {
    let filepath = request.get_str("filepath").unwrap_or_default();
    if filepath.is_empty() {
        return ToolResponse::err("export_gltf", "filepath is required");
    }
    let selected_only = request.get_bool("selected_only").unwrap_or(true);
    let binary = request.get_bool("binary").unwrap_or(false);
    let compress = request.get_bool("compress").unwrap_or(false);
    let anim = request.get_bool("include_animations").unwrap_or(true);
    let result = crate::mcp_client::send_mcp_request(
        "export_gltf",
        serde_json::json!({ "filepath": filepath, "selected_only": selected_only, "binary": binary, "compress": compress, "include_animations": anim }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "export_gltf",
            serde_json::json!({ "filepath": filepath, "result": r.data }),
            format!(
                "Exported {} to '{}'",
                if binary { "GLB" } else { "glTF" },
                filepath
            ),
        ),
        Err(e) => ToolResponse::err("export_gltf", e),
    }
}
fn handle_get_export_recommendations(request: ToolRequest) -> ToolResponse {
    let target = request
        .get_str("target")
        .unwrap_or_else(|| "blender".to_string());
    let has_anim = request.get_bool("has_animations").unwrap_or(false);
    let (format, scale, axis, notes) = match target.to_lowercase().as_str() {
        "unity" => (
            "FBX",
            0.01,
            "Z-Up",
            "Triangulate meshes, embed textures for Standard shader",
        ),
        "unreal" => (
            "FBX",
            0.01,
            "Z-Up",
            "Use FBX 2018, enable skeletal meshes for characters",
        ),
        "blender" => ("FBX", 1.0, "Y-Up", "OBJ also works well for static meshes"),
        "maya" => ("FBX", 1.0, "Y-Up", "Enable smoothing groups"),
        "web" => (
            "glTF",
            1.0,
            "Y-Up",
            "GLB binary format for single-file delivery",
        ),
        "sketchfab" => ("glTF", 1.0, "Y-Up", "GLB preferred, max 50MB recommended"),
        "printing" => (
            "OBJ",
            1.0,
            "Z-Up",
            "Export as manifold solid, no materials needed",
        ),
        _ => ("FBX", 1.0, "Y-Up", "Adjust settings based on target"),
    };
    ToolResponse::ok_with_message(
        "get_export_recommendations",
        serde_json::json!({ "target": target, "recommended_format": format, "scale": scale, "axis_system": axis, "include_animations": has_anim, "notes": notes }),
        format!(
            "Recommendation for {}: {} format, {:.2}x scale, {}",
            target, format, scale, notes
        ),
    )
}
fn handle_export_collada(request: ToolRequest) -> ToolResponse {
    let filepath = request.get_str("filepath").unwrap_or_default();
    if filepath.is_empty() {
        return ToolResponse::err("export_collada", "filepath is required");
    }
    let selected_only = request.get_bool("selected_only").unwrap_or(true);
    let normals = request.get_bool("export_normals").unwrap_or(true);
    let uvs = request.get_bool("export_uvs").unwrap_or(true);
    let result = crate::mcp_client::send_mcp_request(
        "export_collada",
        serde_json::json!({ "filepath": filepath, "selected_only": selected_only, "export_normals": normals, "export_uvs": uvs }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "export_collada",
            serde_json::json!({ "filepath": filepath, "result": r.data }),
            format!("Exported DAE to '{}'", filepath),
        ),
        Err(e) => ToolResponse::err("export_collada", e),
    }
}
fn handle_export_usd(request: ToolRequest) -> ToolResponse {
    let filepath = request.get_str("filepath").unwrap_or_default();
    if filepath.is_empty() {
        return ToolResponse::err("export_usd", "filepath is required");
    }
    let selected_only = request.get_bool("selected_only").unwrap_or(true);
    let anim = request.get_bool("include_animations").unwrap_or(true);
    let flatten = request.get_bool("flatten").unwrap_or(true);
    let result = crate::mcp_client::send_mcp_request(
        "export_usd",
        serde_json::json!({ "filepath": filepath, "selected_only": selected_only, "include_animations": anim, "flatten": flatten }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "export_usd",
            serde_json::json!({ "filepath": filepath, "result": r.data }),
            format!("Exported USD to '{}'", filepath),
        ),
        Err(e) => ToolResponse::err("export_usd", e),
    }
}
fn handle_export_selected_only(request: ToolRequest) -> ToolResponse {
    let filepath = request.get_str("filepath").unwrap_or_default();
    let format = request.get_str("format");
    if filepath.is_empty() {
        return ToolResponse::err("export_selected_only", "filepath is required");
    }
    let ext = filepath.split('.').next_back().unwrap_or("").to_lowercase();
    let fmt = format.unwrap_or_else(|| match ext.as_str() {
        "fbx" => "fbx".to_string(),
        "obj" => "obj".to_string(),
        "gltf" => "gltf".to_string(),
        "glb" => "glb".to_string(),
        "dae" => "dae".to_string(),
        "usd" | "usda" | "usdc" | "usdz" => "usd".to_string(),
        _ => "fbx".to_string(),
    });
    let result = crate::mcp_client::send_mcp_request(
        "export_selected",
        serde_json::json!({ "filepath": filepath, "format": fmt }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "export_selected_only",
            serde_json::json!({ "filepath": filepath, "format": fmt, "result": r.data }),
            format!(
                "Exported selection as {} to '{}'",
                fmt.to_uppercase(),
                filepath
            ),
        ),
        Err(e) => ToolResponse::err("export_selected_only", e),
    }
}
fn handle_batch_export(request: ToolRequest) -> ToolResponse {
    let export_dir = request.get_str("export_dir").unwrap_or_default();
    let items = request.get_array("items");
    let format = request
        .get_str("format")
        .unwrap_or_else(|| "fbx".to_string());
    let name_pattern = request
        .get_str("name_pattern")
        .unwrap_or_else(|| "{name}".to_string());
    if export_dir.is_empty() {
        return ToolResponse::err("batch_export", "export_dir is required");
    }
    if items.is_empty() {
        return ToolResponse::err("batch_export", "items array is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "batch_export",
        serde_json::json!({ "dir": export_dir, "items": items, "format": format, "name_pattern": name_pattern }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "batch_export",
            serde_json::json!({ "dir": export_dir, "format": format, "result": r.data }),
            format!("Batch export to '{}'", export_dir),
        ),
        Err(e) => ToolResponse::err("batch_export", e),
    }
}
fn handle_export_animation(request: ToolRequest) -> ToolResponse {
    let filepath = request.get_str("filepath").unwrap_or_default();
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let format = request
        .get_str("format")
        .unwrap_or_else(|| "fbx".to_string());
    let bake = request.get_bool("bake_keyframes").unwrap_or(true);
    let frame_rate = request.get_i64("frame_rate").unwrap_or(30);
    if filepath.is_empty() || figure_id.is_empty() {
        return ToolResponse::err("export_animation", "filepath and figure_id are required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "export_animation",
        serde_json::json!({ "filepath": filepath, "figure_id": figure_id, "format": format, "bake_keyframes": bake, "frame_rate": frame_rate }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "export_animation",
            serde_json::json!({ "filepath": filepath, "figure_id": figure_id, "result": r.data }),
            format!("Animation exported to '{}'", filepath),
        ),
        Err(e) => ToolResponse::err("export_animation", e),
    }
}
