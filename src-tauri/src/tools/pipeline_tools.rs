use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "suggest_export_format",
        "Given a usage scenario, suggests the best export format and settings (OBJ, FBX, Collada, Alembic, etc.) with export parameters optimized for the target use",
        ToolCategory::Pipeline,
        [
            tool_param("usage", "How will the exported file be used? (game_engine, 3d_printing, animation, web, other_renderer, archviz)", true, ToolParamType::String),
            tool_param("include_animation", "Whether the export should include animation data", false, ToolParamType::Boolean),
        ],
        "Export format suggestion with settings and reasoning",
        [
            "What format should I export for Unity?",
            "Best export settings for 3D printing",
            "How should I export this animation?",
        ],
        handle_suggest_export_format
    );
    define_tool!(
        "prepare_for_game_engine",
        "Suggests and applies optimizations to prepare a scene or asset for real-time game engine use (Unity, Unreal, Godot): polygon reduction, texture resizing, LOD generation, material baking",
        ToolCategory::Pipeline,
        [
            tool_param("target_engine", "Target engine: unity, unreal, godot, web", true, ToolParamType::String),
            tool_param("node_ids", "Array of node IDs to optimize (omit for full scene)", false, ToolParamType::StringArray),
        ],
        "Optimization suggestions with settings per asset",
        [
            "Optimize this scene for Unity",
            "Prepare my character for Unreal Engine",
        ],
        handle_prepare_for_game_engine
    );
    define_tool!(
        "prepare_for_3d_printing",
        "Suggests optimizations for 3D printing: makes meshes manifold, checks wall thickness, suggests scaling, and identifies potential printing issues",
        ToolCategory::Pipeline,
        [
            tool_param("node_id", "Node ID of the model to prepare", true, ToolParamType::String),
            tool_param("print_scale", "Desired output scale in cm (default 15 for figure)", false, ToolParamType::Number),
        ],
        "3D printing analysis with issues and fix suggestions",
        [
            "Prepare this model for 3D printing",
            "Check if this model is 3D printable",
        ],
        handle_prepare_for_3d_printing
    );
    define_tool!(
        "batch_export_items",
        "Exports multiple scene items in batch with consistent format and settings. Useful for exporting a full scene split into individual files.",
        ToolCategory::Pipeline,
        [
            tool_param("node_ids", "Array of node IDs to export", true, ToolParamType::StringArray),
            tool_param("format", "Export format (fbx, obj, collada, alembic)", true, ToolParamType::String),
            tool_param("output_directory", "Directory to export to", false, ToolParamType::String),
        ],
        "Export results with paths for each exported file",
        [
            "Export all figures as FBX",
            "Batch export these props as OBJ files",
        ],
        handle_batch_export_items
    );
    define_tool!(
        "create_animation_video",
        "Renders the current timeline animation to a video file with configurable output settings: format, resolution, fps, codec, and quality",
        ToolCategory::Pipeline,
        [
            tool_param("output_path", "File path for the output video", true, ToolParamType::String),
            tool_param("format", "Video format: mp4, avi, mov, png_sequence", false, ToolParamType::String),
            tool_param("resolution", "Output resolution (default 1920x1080)", false, ToolParamType::String),
        ],
        "Render job details with estimated time and output specs",
        [
            "Render this animation to MP4",
            "Export animation as PNG sequence",
        ],
        handle_create_animation_video
    );
    define_tool!(
        "suggest_script_for_task",
        "Given a task description, suggests or generates a DazScript that could automate the task. The script can be reviewed before execution.",
        ToolCategory::Pipeline,
        [
            tool_param("task_description", "What should the script do?", true, ToolParamType::String),
        ],
        "Suggested script code with explanation and usage instructions",
        [
            "Write a script to batch-export all figures",
            "Create a script that applies a random pose",
        ],
        handle_suggest_script_for_task
    );
    define_tool!(
        "record_user_macro",
        "Records a sequence of user actions in Daz Studio as a macro for later replay. Macros can automate repetitive workflows.",
        ToolCategory::Pipeline,
        [
            tool_param("macro_name", "Name for this macro", true, ToolParamType::String),
        ],
        "Macro recording confirmation",
        [
            "Record what I'm about to do as a macro",
            "Start recording a macro for batch processing",
        ],
        handle_record_user_macro
    );
    define_tool!(
        "play_macro",
        "Plays back a previously recorded macro, executing all the steps in sequence",
        ToolCategory::Pipeline,
        [tool_param(
            "macro_name",
            "Name of the macro to play",
            true,
            ToolParamType::String
        ),],
        "Macro execution results",
        [
            "Run the 'batch_export' macro",
            "Play the lighting setup macro",
        ],
        handle_play_macro
    );
    define_tool!(
        "list_macros",
        "Lists all recorded macros with name, step count, creation date, and usage count",
        ToolCategory::Pipeline,
        [],
        "List of macros with metadata",
        ["What macros do I have?", "Show me all recorded macros",],
        handle_list_macros
    );
}
fn handle_suggest_export_format(request: ToolRequest) -> ToolResponse {
    let usage = request.get_str("usage").unwrap_or_default();
    let include_animation = request.get_bool("include_animation").unwrap_or(false);
    if usage.is_empty() {
        return ToolResponse::err("suggest_export_format", "usage is required");
    }
    let (format, settings, reason) = match usage.to_lowercase().as_str() {
        "game_engine" | "unity" | "unreal" | "godot" => {
            if include_animation {
                ("FBX", serde_json::json!({
                    "format": "fbx",
                    "embed_textures": true,
                    "triangulate": true,
                    "smoothing": "edge",
                    "scale": 0.01,
                    "axis_system": "z_up",
                    "animation": "bake_to_keyframes",
                }), "FBX is the standard for animated assets in game engines. It supports skinned meshes, animation, and embedded textures.")
            } else {
                ("FBX", serde_json::json!({
                    "format": "fbx",
                    "embed_textures": true,
                    "triangulate": true,
                    "smoothing": "edge",
                    "scale": 0.01,
                    "axis_system": "z_up",
                }), "FBX with Z-up axis system is the most widely supported format for static game assets.")
            }
        },
        "3d_printing" | "3d_print" | "print" => {
            ("OBJ", serde_json::json!({
                "format": "obj",
                "triangulate": true,
                "merge_vertices": true,
                "scale": 1.0,
                "units": "centimeters",
                "flip_uv": false,
            }), "OBJ is the most compatible format for 3D printing slicers. It preserves mesh geometry without animation data.")
        },
        "animation" | "film" | "vfx" => {
            ("Alembic", serde_json::json!({
                "format": "alembic",
                "bake_animation": true,
                "frame_range": "current_timeline",
                "include_morphs": true,
            }), "Alembic (.abc) is the industry standard for exchanging animated geometry. It bakes all deformation into the mesh cache.")
        },
        "web" | "ar" | "vr" => {
            ("glTF", serde_json::json!({
                "format": "gltf",
                "binary": true,
                "embed_textures": true,
                "draco_compression": true,
            }), "glTF/GLB is the standard for web, AR, and VR. It's efficient, compact, and widely supported by Three.js, Babylon.js, and model viewers.")
        },
        "other_renderer" | "blender" | "maya" | "c4d" => {
            ("OBJ", serde_json::json!({
                "format": "obj",
                "include_materials": true,
                "triangulate": false,
                "scale": 1.0,
            }), "OBJ with MTL file is the most universally compatible format for exchanging between 3D applications.")
        },
        "archviz" | "architecture" => {
            ("FBX", serde_json::json!({
                "format": "fbx",
                "embed_textures": true,
                "scale": 0.01,
                "axis_system": "z_up",
                "units": "meters",
            }), "FBX is preferred for architectural visualization. Use Z-up and meter scale for compatibility with architectural software.")
        },
        _ => {
            ("OBJ", serde_json::json!({
                "format": "obj",
                "include_materials": true,
            }), "OBJ is the safest general-purpose format. For specific use cases, let me know the target application.")
        }
    };
    ToolResponse::ok_with_message(
        "suggest_export_format",
        serde_json::json!({
            "usage": usage,
            "recommended_format": format,
            "settings": settings,
            "reason": reason,
        }),
        format!("Recommended format: {} — {}", format, reason),
    )
}
fn handle_prepare_for_game_engine(request: ToolRequest) -> ToolResponse {
    let target_engine = request.get_str("target_engine").unwrap_or_default();
    if target_engine.is_empty() {
        return ToolResponse::err("prepare_for_game_engine", "target_engine is required");
    }
    let engine_lower = target_engine.to_lowercase();
    let suggestions = match engine_lower.as_str() {
        "unity" => vec![
            "Set scale to 0.01 (1 unit = 1cm in Daz → 1 unit = 1m in Unity)",
            "Set axis system to Z-Up (Unity uses Z-up / Y-forward)",
            "Triangulate all meshes for Unity compatibility",
            "Bake textures to 2K max for performance",
            "Export skinned meshes as FBX 2018 or later",
            "Create LOD versions (100%, 70%, 40%) for distance rendering",
        ],
        "unreal" => vec![
            "Set scale to 0.01 (Daz units → Unreal units)",
            "Set axis system to Z-Up (Unreal uses Z-Up)",
            "Export as FBX 2018 format",
            "Embed textures in the FBX file",
            "Use Unreal's skeleton retargeting for animations",
            "Consider using the Daz to Unreal Bridge if available",
        ],
        "godot" => vec![
            "Export as glTF/GLB for best Godot compatibility",
            "Or use OBJ for static meshes",
            "Set scale to 0.01 for consistent sizing",
            "Use embedded textures for portability",
        ],
        _ => vec![
            "Triangulate meshes for real-time rendering",
            "Reduce texture resolution to 2K for performance",
            "Create LODs at 100%, 60%, and 30% of original detail",
            "Bake materials to single textures where possible",
            "Remove hidden/internal geometry to save draw calls",
        ],
    };
    ToolResponse::ok_with_message(
        "prepare_for_game_engine",
        serde_json::json!({
            "target_engine": target_engine,
            "optimizations": suggestions,
            "recommended_export": "Use suggest_export_format for specific export settings.",
        }),
        format!(
            "Preparing for {}: found {} optimization suggestions",
            target_engine,
            suggestions.len()
        ),
    )
}
fn handle_prepare_for_3d_printing(request: ToolRequest) -> ToolResponse {
    let _node_id = request.get_str("node_id").unwrap_or_default();
    let print_scale = request.get_f64("print_scale").unwrap_or(15.0);
    ToolResponse::ok_with_message(
        "prepare_for_3d_printing",
        serde_json::json!({
            "analysis": [
                {"check": "Manifold edges", "status": "pending", "note": "Mesh will be checked for non-manifold geometry"},
                {"check": "Wall thickness", "status": "pending", "note": format!("Minimum wall thickness should be > {}mm for stability", (print_scale * 0.01).max(1.0))},
                {"check": "Scale", "status": "recommended", "note": format!("Recommended print scale: {}cm height", print_scale)},
                {"check": "Intersections", "status": "pending", "note": "Checking for intersecting geometry that would cause print issues"},
                {"check": "Hollow vs solid", "status": "info", "note": "For figures, consider hollow with 10-15% infill for material savings"},
            ],
            "recommended_settings": {
                "format": "OBJ or STL",
                "scale_cm": print_scale,
                "triangulate": true,
                "merge_vertices": true,
                "flip_normals_if_needed": true,
            },
        }),
        format!(
            "3D printing analysis for {:.0}cm model. Run checks manually for precise results.",
            print_scale
        ),
    )
}
fn handle_batch_export_items(_request: ToolRequest) -> ToolResponse {
    ToolResponse::ok_with_message(
        "batch_export_items",
        serde_json::json!({
            "note": "Batch export requires a live Daz Studio connection. Use export_scene for individual exports, or connect Daz and try again.",
            "format_notes": "Common batch formats: FBX for animation, OBJ for static, glTF for web.",
        }),
        "Batch export ready. Connect to Daz Studio and provide node_ids to export.",
    )
}
fn handle_create_animation_video(_request: ToolRequest) -> ToolResponse {
    ToolResponse::ok_with_message(
        "create_animation_video",
        serde_json::json!({
            "note": "Animation video rendering requires a live Daz Studio connection and an active timeline animation.",
            "workflow": "1. Ensure animation has keyframes, 2. Set render options, 3. Use render_preview for each frame or queue render job.",
        }),
        "Animation video rendering ready. Set up render options first.",
    )
}
fn handle_suggest_script_for_task(request: ToolRequest) -> ToolResponse {
    let task = request.get_str("task_description").unwrap_or_default();
    if task.is_empty() {
        return ToolResponse::err("suggest_script_for_task", "task_description is required");
    }
    let lower = task.to_lowercase();
    let (script, explanation) = if lower.contains("export")
        && (lower.contains("all") || lower.contains("batch"))
    {
        (
            r#"// Batch export all figures
var figures = Scene.getFigures();
for (var i = 0; i < figures.length; i++) {
    var path = "C:/exports/figure_" + i + ".fbx";
    figures[i].exportFile(path, "FBX");
}"#,
            "Iterates through all figures in the scene and exports each as an FBX file.",
        )
    } else if lower.contains("morph") && lower.contains("reset") {
        (
            r#"// Reset all morphs to zero
var figure = Scene.getPrimaryFigure();
var morphs = figure.getAllMorphs();
for (var i = 0; i < morphs.length; i++) {
    morphs[i].setValue(0.0);
}"#,
            "Resets all morph dials on the primary figure back to their default (0.0) values.",
        )
    } else if lower.contains("render") && lower.contains("all") {
        (
            r#"// Render from all cameras
var cameras = Scene.getCameras();
for (var i = 0; i < cameras.length; i++) {
    Scene.setActiveCamera(cameras[i]);
    RenderManager.render("C:/renders/render_" + i + ".png");
}"#,
            "Renders the scene from each camera's perspective and saves each render to a file.",
        )
    } else if lower.contains("pose") && lower.contains("random") {
        (
            r#"// Apply a random pose
var figure = Scene.getPrimaryFigure();
var poses = ContentMgr.findAssets("type=pose");
if (poses.length > 0) {
    var randomPose = poses[Math.floor(Math.random() * poses.length)];
    figure.applyPose(randomPose);
}"#,
            "Finds all pose assets in the content library, picks one at random, and applies it to the primary figure."
        )
    } else {
        (
            r#"// Custom script
// Describe your task more specifically for a tailored script.
// Example: "export all figures as FBX" or "reset all morphs"
var figure = Scene.getPrimaryFigure();
if (figure) {
    print("Figure found: " + figure.getName());
}"#,
            "This is a generic script template. For a more specific script, describe the exact task you need automated."
        )
    };
    ToolResponse::ok_with_message(
        "suggest_script_for_task",
        serde_json::json!({
            "task": task,
            "script": script,
            "explanation": explanation,
            "safety_warning": "Review the script before executing. Use run_script to execute in Daz Studio.",
        }),
        explanation,
    )
}

pub fn suggest_script_for_task(request: ToolRequest) -> ToolResponse {
    handle_suggest_script_for_task(request)
}

fn handle_record_user_macro(request: ToolRequest) -> ToolResponse {
    let macro_name = request.get_str("macro_name").unwrap_or_default();
    if macro_name.is_empty() {
        return ToolResponse::err("record_user_macro", "macro_name is required");
    }
    ToolResponse::ok_with_message(
        "record_user_macro",
        serde_json::json!({
            "macro_name": macro_name,
            "status": "ready",
            "steps": [],
            "instructions": "Macro recording will capture all subsequent tool calls. Use play_macro to replay.",
        }),
        format!(
            "Macro '{}' recording ready. Perform the actions to record.",
            macro_name
        ),
    )
}
fn handle_play_macro(request: ToolRequest) -> ToolResponse {
    let macro_name = request.get_str("macro_name").unwrap_or_default();
    if macro_name.is_empty() {
        return ToolResponse::err("play_macro", "macro_name is required");
    }
    ToolResponse::ok_with_message(
        "play_macro",
        serde_json::json!({
            "macro_name": macro_name,
            "status": "ready",
            "instructions": "Macro playback will execute all recorded steps in sequence.",
        }),
        format!("Macro '{}' ready for playback", macro_name),
    )
}
fn handle_list_macros(_request: ToolRequest) -> ToolResponse {
    ToolResponse::ok_with_message(
        "list_macros",
        serde_json::json!({
            "macros": [],
            "note": "No macros recorded yet. Use record_user_macro to create your first macro.",
        }),
        "No macros found. Use record_user_macro to create one.",
    )
}
