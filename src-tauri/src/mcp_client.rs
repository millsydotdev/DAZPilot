#![allow(dead_code)]

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

static MCP_CLIENT: Lazy<Arc<Mutex<Option<McpConnection>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCommand {
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub status: String,
    pub result: Option<String>,
    pub commands: Option<Vec<McpCommand>>,
    #[serde(default)]
    pub data: Option<Value>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DazRequest {
    pub id: String,
    pub command: String,
    #[serde(default)]
    pub args: Value,
}

#[derive(Debug, Clone)]
pub struct CommandSchema {
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub parameters: &'static [&'static str],
    pub high_risk: bool,
}

const COMMAND_SCHEMAS: &[CommandSchema] = &[
    CommandSchema {
        name: "get_commands",
        description: "List supported Daz bridge commands",
        category: "System",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "get_scene_info",
        description: "Get current Daz scene summary",
        category: "Scene",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "list_nodes",
        description: "List scene nodes",
        category: "Scene",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "get_selected_nodes",
        description: "List selected scene nodes",
        category: "Selection",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "select_node",
        description: "Select a Daz scene node by id or name",
        category: "Selection",
        parameters: &["node_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "get_cameras",
        description: "List scene cameras",
        category: "Camera",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "load_asset",
        description: "Load a Daz asset file into the current scene",
        category: "Assets",
        parameters: &["path"],
        high_risk: false,
    },
    CommandSchema {
        name: "apply_pose",
        description: "Apply a pose file to a figure",
        category: "Pose",
        parameters: &["pose_path", "figure_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "render_preview",
        description: "Trigger a Daz preview render",
        category: "Render",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "capture_viewport",
        description: "Capture the active Daz viewport",
        category: "Viewport",
        parameters: &["path"],
        high_risk: false,
    },
    CommandSchema {
        name: "import_model",
        description: "Import a model file through Daz",
        category: "Assets",
        parameters: &["path", "settings"],
        high_risk: false,
    },
    CommandSchema {
        name: "export_scene",
        description: "Export scene or node through Daz",
        category: "Assets",
        parameters: &["node_id", "path", "settings"],
        high_risk: true,
    },
    CommandSchema {
        name: "add_node",
        description: "Add a primitive node (point_light, spot_light, distant_light, camera, null)",
        category: "Scene",
        parameters: &["type", "name"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_property",
        description: "Set a node property",
        category: "Properties",
        parameters: &["node_id", "property", "value"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_material_property",
        description: "Set a material property",
        category: "Materials",
        parameters: &["node_id", "property", "value"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_body_opacity",
        description: "Set opacity across all body surfaces",
        category: "Materials",
        parameters: &["node_id", "value"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_surface_opacity",
        description: "Set opacity on matching material surfaces",
        category: "Materials",
        parameters: &["node_id", "surface_pattern", "value"],
        high_risk: false,
    },
    CommandSchema {
        name: "get_internal_surfaces",
        description: "List likely internal anatomy material surfaces",
        category: "Materials",
        parameters: &["node_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "show_anatomy",
        description: "Make internal anatomy surfaces fully opaque",
        category: "Materials",
        parameters: &["node_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "place_asset_inside",
        description: "Load and place an asset inside a figure",
        category: "Assets",
        parameters: &["figure_id", "asset_path"],
        high_risk: false,
    },
    CommandSchema {
        name: "get_node_properties",
        description: "Get animatable properties of a node",
        category: "Properties",
        parameters: &["node_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "delete_node",
        description: "Delete a node from the scene",
        category: "Scene",
        parameters: &["node_id"],
        high_risk: true,
    },
    CommandSchema {
        name: "get_geoshells",
        description: "Get all Geometry Shells in the scene",
        category: "Scene",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "begin_undo_batch",
        description: "Start a new undo batch in Daz Studio",
        category: "Scene",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "accept_undo_batch",
        description: "Accept the current undo batch with a caption",
        category: "Scene",
        parameters: &["caption"],
        high_risk: false,
    },
    CommandSchema {
        name: "cancel_undo_batch",
        description: "Cancel the current undo batch",
        category: "Scene",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "viewport_click",
        description: "Pick and select a node in the viewport at the given coordinates",
        category: "Viewport",
        parameters: &["x", "y"],
        high_risk: false,
    },
    // ── Animation Commands ────────────────────────────────────────────────────
    CommandSchema {
        name: "set_keyframe",
        description: "Set an animatable float property keyframe at a specific frame",
        category: "Animation",
        parameters: &["node_id", "property", "frame", "value", "interpolation"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_timeline_range",
        description: "Set the Daz Studio play range and animation range",
        category: "Animation",
        parameters: &["start_frame", "end_frame"],
        high_risk: false,
    },
    CommandSchema {
        name: "seek_to_frame",
        description: "Move the Daz Studio timeline cursor to a specific frame",
        category: "Animation",
        parameters: &["frame"],
        high_risk: false,
    },
    CommandSchema {
        name: "play_timeline",
        description: "Start Daz Studio timeline playback",
        category: "Animation",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "pause_timeline",
        description: "Pause Daz Studio timeline playback",
        category: "Animation",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "stop_timeline",
        description: "Stop playback and reset to frame 0",
        category: "Animation",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "get_timeline_state",
        description: "Query current Daz timeline frame, range, fps, and playback state",
        category: "Animation",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "run_dforce_simulation",
        description: "Run a dForce physics simulation via inline DAZ Script",
        category: "Animation",
        parameters: &["node_id", "start_frame", "end_frame"],
        high_risk: false,
    },
    CommandSchema {
        name: "get_material_properties",
        description: "Get material properties of a node",
        category: "Materials",
        parameters: &["node_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "apply_phy_modifier",
        description: "Apply DazPilot physics modifier to a node",
        category: "Physics",
        parameters: &["node_id", "stiffness", "damping", "mass"],
        high_risk: false,
    },
    CommandSchema {
        name: "remove_phy_modifier",
        description: "Remove DazPilot physics modifier from a node",
        category: "Physics",
        parameters: &["node_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_phy_modifier_params",
        description: "Update DazPilot physics modifier parameters",
        category: "Physics",
        parameters: &["node_id", "stiffness", "damping", "mass"],
        high_risk: false,
    },
    CommandSchema {
        name: "get_bounding_boxes",
        description: "Get world-space 3D bounding boxes of all scene nodes",
        category: "Vision",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "run_script",
        description: "Evaluate arbitrary DazScript on the main thread",
        category: "Scripting",
        parameters: &["script", "args"],
        high_risk: true,
    },
    CommandSchema {
        name: "get_scene_assets",
        description: "Get list of loaded asset labels currently in the Daz Studio scene",
        category: "Scene",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "add_figure",
        description: "Add a Genesis figure (8 or 9) to the scene. Use 'genesis8' or 'genesis9'.",
        category: "Scene",
        parameters: &["figure_type"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_morph",
        description: "Set a morph dial value on a figure (0.0–1.0)",
        category: "Properties",
        parameters: &["node_id", "morph", "value"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_light",
        description: "Set a light property (intensity, color, etc.)",
        category: "Lighting",
        parameters: &["node_id", "property", "value"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_render_settings",
        description: "Apply render resolution and quality presets",
        category: "Render",
        parameters: &["width", "height"],
        high_risk: false,
    },
    CommandSchema {
        name: "get_figure_morphs",
        description: "Get all morph dials and their values for a figure",
        category: "Properties",
        parameters: &["figure_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "get_fitted_items",
        description: "Get all fitted clothing/accessories on a figure",
        category: "Scene",
        parameters: &["figure_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "get_active_expressions",
        description: "Get all active expression dial values on a figure",
        category: "Properties",
        parameters: &["figure_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "get_material_zones",
        description: "Get material zone names on a figure",
        category: "Materials",
        parameters: &["figure_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "apply_morph",
        description: "Set a morph dial value on a figure (0.0–1.0)",
        category: "Properties",
        parameters: &["figure_id", "morph_id", "value"],
        high_risk: false,
    },
    CommandSchema {
        name: "apply_expression",
        description: "Set an expression dial value on a figure",
        category: "Properties",
        parameters: &["figure_id", "expression_id", "value"],
        high_risk: false,
    },
    CommandSchema {
        name: "save_scene",
        description: "Save the current scene to a file",
        category: "Scene",
        parameters: &["path"],
        high_risk: true,
    },
    CommandSchema {
        name: "load_scene",
        description: "Load a scene file (method: default/new/merge)",
        category: "Scene",
        parameters: &["path", "method"],
        high_risk: true,
    },
    CommandSchema {
        name: "clear_scene",
        description: "Clear the current scene",
        category: "Scene",
        parameters: &[],
        high_risk: true,
    },
    CommandSchema {
        name: "set_camera",
        description: "Set active camera or adjust camera properties",
        category: "Camera",
        parameters: &["camera", "focal_length", "focal_distance"],
        high_risk: false,
    },
    CommandSchema {
        name: "get_node_transform",
        description: "Get node world-space transform (pos/rot/scale)",
        category: "Scene",
        parameters: &["node_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_node_transform",
        description: "Set node world-space position, rotation, or scale",
        category: "Scene",
        parameters: &["node_id", "position", "rotation", "scale"],
        high_risk: true,
    },
    CommandSchema {
        name: "set_render_options",
        description: "Set render quality, resolution, and output options",
        category: "Render",
        parameters: &[
            "width",
            "height",
            "pixel_samples",
            "ray_trace_depth",
            "shading_rate",
            "gamma",
        ],
        high_risk: false,
    },
    CommandSchema {
        name: "search_content",
        description: "Search Daz content library for assets by name/type",
        category: "Assets",
        parameters: &["query", "type", "max_results"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_material_texture",
        description: "Assign a texture map file to a material surface channel",
        category: "Materials",
        parameters: &["node_id", "channel", "file_path"],
        high_risk: false,
    },
    CommandSchema {
        name: "get_material_channels",
        description: "Get all surface channels with texture paths and values",
        category: "Materials",
        parameters: &["node_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "list_bones",
        description: "List all bones in a figure's skeleton",
        category: "Animation",
        parameters: &["figure_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_bone_transform",
        description: "Set a bone's world-space position or rotation",
        category: "Animation",
        parameters: &["figure_id", "bone_name", "position", "rotation"],
        high_risk: true,
    },
    CommandSchema {
        name: "list_keyframes",
        description: "List all keyframes on a node property",
        category: "Animation",
        parameters: &["node_id", "property"],
        high_risk: false,
    },
    CommandSchema {
        name: "delete_keyframes",
        description: "Delete keyframes from a node property (range or all)",
        category: "Animation",
        parameters: &["node_id", "property", "start", "end"],
        high_risk: true,
    },
    CommandSchema {
        name: "list_modifiers",
        description: "List all modifiers on a node's geometry object",
        category: "Scene",
        parameters: &["node_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_viewport_mode",
        description: "Set viewport display mode (texture, shaded, wireframe, etc.)",
        category: "Viewport",
        parameters: &["mode"],
        high_risk: false,
    },
    // ── Environment Commands ──────────────────────────────────────────────────
    CommandSchema {
        name: "set_environment",
        description: "Set environment map or preset",
        category: "Environment",
        parameters: &["type", "preset", "intensity", "rotation"],
        high_risk: false,
    },
    CommandSchema {
        name: "add_ground",
        description: "Add a ground plane to the scene",
        category: "Environment",
        parameters: &["type", "color", "size"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_fog",
        description: "Enable/configure fog in the scene",
        category: "Environment",
        parameters: &["enabled", "density", "color", "distance"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_sun",
        description: "Set sun position and color",
        category: "Environment",
        parameters: &["direction", "intensity", "color"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_time_of_day",
        description: "Set scene time of day with automatic lighting",
        category: "Environment",
        parameters: &["time", "adjust_env"],
        high_risk: false,
    },
    CommandSchema {
        name: "add_env_light",
        description: "Add an environment fill/rim/bounce light",
        category: "Lighting",
        parameters: &["type", "intensity", "color", "direction"],
        high_risk: false,
    },
    CommandSchema {
        name: "rotate_environment",
        description: "Rotate the environment map",
        category: "Environment",
        parameters: &["rotation", "hdri_only"],
        high_risk: false,
    },
    CommandSchema {
        name: "get_environment_info",
        description: "Get current environment settings",
        category: "Environment",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "clear_environment",
        description: "Remove environment elements (hdri/ground/fog)",
        category: "Environment",
        parameters: &["hdri", "ground", "fog"],
        high_risk: false,
    },
    // ── Figure Commands ───────────────────────────────────────────────────────
    CommandSchema {
        name: "apply_figure_preset",
        description: "Apply a figure/character preset",
        category: "Figure",
        parameters: &[
            "figure_id",
            "preset_path",
            "apply_morphs",
            "apply_materials",
        ],
        high_risk: false,
    },
    CommandSchema {
        name: "list_figures",
        description: "List all figure nodes in the scene",
        category: "Figure",
        parameters: &["include_details"],
        high_risk: false,
    },
    CommandSchema {
        name: "remove_figure",
        description: "Remove a figure from the scene",
        category: "Figure",
        parameters: &["figure_id"],
        high_risk: true,
    },
    // ── Selection Commands ────────────────────────────────────────────────────
    CommandSchema {
        name: "select_by_type",
        description: "Select nodes by type and optional name filter",
        category: "Selection",
        parameters: &["type", "mode", "name_contains"],
        high_risk: false,
    },
    CommandSchema {
        name: "select_hierarchy",
        description: "Select a node hierarchy",
        category: "Selection",
        parameters: &["node_id", "include_parent"],
        high_risk: false,
    },
    CommandSchema {
        name: "invert_selection",
        description: "Invert the current node selection",
        category: "Selection",
        parameters: &["type"],
        high_risk: false,
    },
    CommandSchema {
        name: "save_selection",
        description: "Save current selection set",
        category: "Selection",
        parameters: &["name"],
        high_risk: false,
    },
    CommandSchema {
        name: "load_selection",
        description: "Load a saved selection set",
        category: "Selection",
        parameters: &["name"],
        high_risk: false,
    },
    // ── Camera Commands ──────────────────────────────────────────────────────
    CommandSchema {
        name: "create_camera",
        description: "Create a new camera node",
        category: "Camera",
        parameters: &["name", "position", "focal_length"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_camera_transform",
        description: "Position and orient a camera",
        category: "Camera",
        parameters: &["camera_id", "position", "target"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_focal_length",
        description: "Set camera focal length in mm",
        category: "Camera",
        parameters: &["camera_id", "focal_length"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_aperture",
        description: "Set camera aperture and depth of field",
        category: "Camera",
        parameters: &["camera_id", "f_stop", "enable_dof", "focus_distance"],
        high_risk: false,
    },
    CommandSchema {
        name: "focus_camera",
        description: "Point camera to look at a target node",
        category: "Camera",
        parameters: &["camera_id", "target", "offset"],
        high_risk: false,
    },
    // ── Viewport Commands ─────────────────────────────────────────────────────
    CommandSchema {
        name: "set_display_mode",
        description: "Set viewport display mode (textured, solid, wireframe, etc.)",
        category: "Viewport",
        parameters: &["mode", "viewport"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_viewport_quality",
        description: "Set viewport rendering quality",
        category: "Viewport",
        parameters: &["quality", "texture_resolution", "anti_aliasing"],
        high_risk: false,
    },
    CommandSchema {
        name: "toggle_guide",
        description: "Show/hide a viewport guide element",
        category: "Viewport",
        parameters: &["guide", "show"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_viewport_camera",
        description: "Set active viewport camera",
        category: "Viewport",
        parameters: &["camera", "viewport"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_viewport_lighting",
        description: "Set viewport lighting mode",
        category: "Viewport",
        parameters: &["lighting", "ambient_intensity"],
        high_risk: false,
    },
    CommandSchema {
        name: "center_view",
        description: "Center viewport on a node",
        category: "Viewport",
        parameters: &["node_id", "animate"],
        high_risk: false,
    },
    // ── Render Commands ───────────────────────────────────────────────────────
    CommandSchema {
        name: "render",
        description: "Render preview, region, or final output",
        category: "Render",
        parameters: &["quality", "width", "height", "mode"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_render_output",
        description: "Configure render output format and path",
        category: "Render",
        parameters: &["format", "path", "filename", "include_alpha"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_render_engine",
        description: "Select render engine and GPU options",
        category: "Render",
        parameters: &["engine", "use_gpu", "gpu_devices"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_resolution",
        description: "Set render output resolution",
        category: "Render",
        parameters: &["width", "height"],
        high_risk: false,
    },
    CommandSchema {
        name: "render_region",
        description: "Render a specific viewport region",
        category: "Render",
        parameters: &["x", "y", "width", "height", "quality"],
        high_risk: false,
    },
    CommandSchema {
        name: "queue_render",
        description: "Add a render pass to the queue",
        category: "Render",
        parameters: &["pass_name", "camera_name"],
        high_risk: false,
    },
    CommandSchema {
        name: "cancel_render",
        description: "Cancel the current render",
        category: "Render",
        parameters: &["clear_queue"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_denoising",
        description: "Configure render denoising settings",
        category: "Render",
        parameters: &["enabled", "strength", "mode"],
        high_risk: false,
    },
    // ── Export Commands ───────────────────────────────────────────────────────
    CommandSchema {
        name: "export_fbx",
        description: "Export scene to FBX format",
        category: "Export",
        parameters: &[
            "filepath",
            "selected_only",
            "scale",
            "axis_system",
            "embed_textures",
            "triangulate",
        ],
        high_risk: true,
    },
    CommandSchema {
        name: "export_obj",
        description: "Export scene to OBJ format",
        category: "Export",
        parameters: &["filepath", "selected_only", "export_materials"],
        high_risk: true,
    },
    CommandSchema {
        name: "export_gltf",
        description: "Export scene to glTF format",
        category: "Export",
        parameters: &[
            "filepath",
            "selected_only",
            "binary",
            "compress",
            "include_animations",
        ],
        high_risk: true,
    },
    CommandSchema {
        name: "export_collada",
        description: "Export scene to Collada DAE format",
        category: "Export",
        parameters: &["filepath", "selected_only", "export_normals", "export_uvs"],
        high_risk: true,
    },
    CommandSchema {
        name: "export_usd",
        description: "Export scene to USD/USDZ format",
        category: "Export",
        parameters: &["filepath", "selected_only", "include_animations", "flatten"],
        high_risk: true,
    },
    CommandSchema {
        name: "export_selected",
        description: "Export selected nodes to a file",
        category: "Export",
        parameters: &["filepath", "format"],
        high_risk: true,
    },
    CommandSchema {
        name: "batch_export",
        description: "Batch export multiple items to files",
        category: "Export",
        parameters: &["dir", "items", "format", "name_pattern"],
        high_risk: true,
    },
    CommandSchema {
        name: "export_animation",
        description: "Export animation data to a file",
        category: "Export",
        parameters: &[
            "filepath",
            "figure_id",
            "format",
            "bake_keyframes",
            "frame_rate",
        ],
        high_risk: true,
    },
    // ── Pose Commands ─────────────────────────────────────────────────────────
    CommandSchema {
        name: "list_poses",
        description: "List available pose presets",
        category: "Pose",
        parameters: &["category", "figure_type", "mood"],
        high_risk: false,
    },
    CommandSchema {
        name: "save_pose",
        description: "Save current pose as a preset",
        category: "Pose",
        parameters: &["figure_id", "name", "category", "include_facial"],
        high_risk: false,
    },
    CommandSchema {
        name: "blend_poses",
        description: "Blend between two poses",
        category: "Pose",
        parameters: &["figure_id", "pose_a", "pose_b", "blend"],
        high_risk: false,
    },
    CommandSchema {
        name: "mirror_pose",
        description: "Mirror the current pose left-right",
        category: "Pose",
        parameters: &["figure_id", "selected_only"],
        high_risk: false,
    },
    CommandSchema {
        name: "asymmetric_pose",
        description: "Apply different poses to left and right sides",
        category: "Pose",
        parameters: &["figure_id", "left", "right"],
        high_risk: false,
    },
    CommandSchema {
        name: "reset_pose",
        description: "Reset a figure to T-pose or A-pose",
        category: "Pose",
        parameters: &["figure_id", "pose_type", "preserve_facial"],
        high_risk: false,
    },
    CommandSchema {
        name: "random_pose",
        description: "Apply a random pose to a figure",
        category: "Pose",
        parameters: &["figure_id", "category", "intensity"],
        high_risk: false,
    },
    // ── Hair Commands ─────────────────────────────────────────────────────────
    CommandSchema {
        name: "load_hair",
        description: "Load a hair asset from the content library",
        category: "Hair",
        parameters: &["name", "figure_id", "color"],
        high_risk: false,
    },
    CommandSchema {
        name: "style_hair",
        description: "Apply a hair styling preset",
        category: "Hair",
        parameters: &["hair_id", "preset"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_hair_color",
        description: "Change hair color with optional highlights/roots",
        category: "Hair",
        parameters: &["hair_id", "color", "highlights", "root_color"],
        high_risk: false,
    },
    CommandSchema {
        name: "apply_hair_physics",
        description: "Configure dForce physics on hair",
        category: "Hair",
        parameters: &["hair_id", "enable", "stiffness", "gravity_scale", "wind"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_hair_length",
        description: "Adjust hair length preset",
        category: "Hair",
        parameters: &["hair_id", "length", "scale_factor"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_hair_volume",
        description: "Adjust hair volume/thickness",
        category: "Hair",
        parameters: &["hair_id", "volume"],
        high_risk: false,
    },
    CommandSchema {
        name: "list_hair_presets",
        description: "List available hair presets",
        category: "Hair",
        parameters: &["figure_type"],
        high_risk: false,
    },
    CommandSchema {
        name: "remove_hair",
        description: "Remove a hair asset from the scene",
        category: "Hair",
        parameters: &["hair_id", "keep_textures"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_hair_shader",
        description: "Change hair shader settings (glossy, matte, etc.)",
        category: "Hair",
        parameters: &["hair_id", "shader_type", "gloss", "specular"],
        high_risk: false,
    },
    CommandSchema {
        name: "apply_hair_preset",
        description: "Apply a complete hair style preset",
        category: "Hair",
        parameters: &["hair_id", "preset"],
        high_risk: false,
    },
    // ── Clothing/Fitting Commands ─────────────────────────────────────────────
    CommandSchema {
        name: "load_clothing",
        description: "Load a clothing asset onto a figure",
        category: "Fitting",
        parameters: &["name", "fit_mode", "figure_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "fit_clothing",
        description: "Fit clothing to a figure",
        category: "Fitting",
        parameters: &["clothing_id", "figure_id", "fit_type", "clear_morphs"],
        high_risk: false,
    },
    CommandSchema {
        name: "remove_clothing",
        description: "Remove clothing from a figure or scene",
        category: "Fitting",
        parameters: &["clothing_id", "remove_from_scene"],
        high_risk: false,
    },
    CommandSchema {
        name: "list_worn_items",
        description: "List items worn by a figure",
        category: "Fitting",
        parameters: &["figure_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_clothing_params",
        description: "Set a clothing parameter value",
        category: "Fitting",
        parameters: &["clothing_id", "parameter", "value"],
        high_risk: false,
    },
    CommandSchema {
        name: "suggest_outfit",
        description: "Suggest an outfit for a figure",
        category: "Fitting",
        parameters: &["figure_id", "style"],
        high_risk: false,
    },
    // ── Props Commands ────────────────────────────────────────────────────────
    CommandSchema {
        name: "load_prop",
        description: "Load a prop from the content library",
        category: "Props",
        parameters: &["name", "category", "position"],
        high_risk: false,
    },
    CommandSchema {
        name: "position_prop",
        description: "Position a prop in the scene",
        category: "Props",
        parameters: &["prop_id", "position", "relative"],
        high_risk: false,
    },
    CommandSchema {
        name: "rotate_prop",
        description: "Rotate a prop",
        category: "Props",
        parameters: &["prop_id", "rotation", "relative"],
        high_risk: false,
    },
    CommandSchema {
        name: "scale_prop",
        description: "Scale a prop uniformly or per-axis",
        category: "Props",
        parameters: &["prop_id", "scale", "relative"],
        high_risk: false,
    },
    CommandSchema {
        name: "list_props",
        description: "List props in the scene",
        category: "Props",
        parameters: &["category"],
        high_risk: false,
    },
    // ── Morph Commands ────────────────────────────────────────────────────────
    CommandSchema {
        name: "batch_set_morphs",
        description: "Set multiple morph values at once",
        category: "Morphs",
        parameters: &["figure_id", "morphs"],
        high_risk: false,
    },
    CommandSchema {
        name: "symmetry_morphs",
        description: "Mirror morph values symmetrically left-right",
        category: "Morphs",
        parameters: &["figure_id", "direction", "morph_group"],
        high_risk: false,
    },
    CommandSchema {
        name: "randomize_morphs",
        description: "Randomize morph values on a figure",
        category: "Morphs",
        parameters: &["figure_id", "intensity", "morph_group"],
        high_risk: false,
    },
    CommandSchema {
        name: "save_morph_preset",
        description: "Save current morph values as a preset",
        category: "Morphs",
        parameters: &["figure_id", "preset_name", "morph_group"],
        high_risk: false,
    },
    CommandSchema {
        name: "load_morph_preset",
        description: "Load a morph preset onto a figure",
        category: "Morphs",
        parameters: &["figure_id", "preset_name", "blend"],
        high_risk: false,
    },
    CommandSchema {
        name: "reset_morphs",
        description: "Reset all morphs on a figure to zero",
        category: "Morphs",
        parameters: &["figure_id", "morph_group"],
        high_risk: false,
    },
    // ── Physics Commands ──────────────────────────────────────────────────────
    CommandSchema {
        name: "simulate_physics",
        description: "Run a physics simulation on nodes",
        category: "Physics",
        parameters: &["node_ids", "frames", "start_frame", "real_time"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_wind",
        description: "Set wind parameters for physics simulation",
        category: "Physics",
        parameters: &["direction", "speed", "turbulence", "gust_strength"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_gravity",
        description: "Set gravity for physics simulation",
        category: "Physics",
        parameters: &["strength", "direction", "node_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "add_collision",
        description: "Add a collision object for physics",
        category: "Physics",
        parameters: &["node_id", "shape", "friction"],
        high_risk: false,
    },
    CommandSchema {
        name: "bake_physics",
        description: "Bake physics simulation to keyframes",
        category: "Animation",
        parameters: &["node_id", "range_start", "range_end", "sample_rate"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_physics_props",
        description: "Set physics properties on a node (mass, stiffness, etc.)",
        category: "Physics",
        parameters: &["node_id", "mass", "stiffness", "damping", "collision"],
        high_risk: false,
    },
    CommandSchema {
        name: "remove_physics",
        description: "Remove physics modifiers from a node",
        category: "Physics",
        parameters: &["node_id", "remove_modifiers"],
        high_risk: false,
    },
    // ── Rigging Commands ──────────────────────────────────────────────────────
    CommandSchema {
        name: "get_joint_list",
        description: "List all joints/bones in a figure",
        category: "Rigging",
        parameters: &["figure_id", "include_hidden"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_joint_rotation",
        description: "Set a joint rotation",
        category: "Rigging",
        parameters: &["figure_id", "joint", "rotation", "space"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_ik_fk_blend",
        description: "Set IK/FK blend on a limb",
        category: "Rigging",
        parameters: &["figure_id", "limb", "blend"],
        high_risk: false,
    },
    CommandSchema {
        name: "add_joint",
        description: "Add a new joint to a figure",
        category: "Rigging",
        parameters: &["figure_id", "joint_name", "parent_joint", "position"],
        high_risk: true,
    },
    // ── Transform Commands ────────────────────────────────────────────────────
    CommandSchema {
        name: "set_transform",
        description: "Set node transform (position/rotation/scale)",
        category: "Transform",
        parameters: &["node_id", "position", "rotation", "scale", "space"],
        high_risk: false,
    },
    CommandSchema {
        name: "align_nodes",
        description: "Align nodes to a target node",
        category: "Transform",
        parameters: &["target_node", "node_ids", "axes", "alignment"],
        high_risk: false,
    },
    CommandSchema {
        name: "distribute_nodes",
        description: "Distribute nodes evenly along an axis",
        category: "Transform",
        parameters: &["node_ids", "axis", "spacing", "bounds"],
        high_risk: false,
    },
    CommandSchema {
        name: "snap_to_ground",
        description: "Snap a node to the ground or another node",
        category: "Transform",
        parameters: &["node_id", "offset_y", "snap_to_node"],
        high_risk: false,
    },
    CommandSchema {
        name: "reset_transform",
        description: "Reset node transform to defaults",
        category: "Transform",
        parameters: &["node_id", "position", "rotation", "scale"],
        high_risk: false,
    },
    // ── Scene Commands ────────────────────────────────────────────────────────
    CommandSchema {
        name: "set_visibility",
        description: "Show or hide a node",
        category: "Scene",
        parameters: &["node_id", "visible", "recursive"],
        high_risk: false,
    },
    CommandSchema {
        name: "delete_nodes",
        description: "Delete multiple nodes from the scene",
        category: "Scene",
        parameters: &["node_ids"],
        high_risk: true,
    },
    CommandSchema {
        name: "duplicate_nodes",
        description: "Duplicate one or more nodes",
        category: "Scene",
        parameters: &["node_ids", "copies", "offset"],
        high_risk: false,
    },
    CommandSchema {
        name: "rename_node",
        description: "Rename a scene node",
        category: "Scene",
        parameters: &["node_id", "new_name"],
        high_risk: false,
    },
    CommandSchema {
        name: "group_nodes",
        description: "Group nodes under a new parent",
        category: "Scene",
        parameters: &["node_ids", "group_name"],
        high_risk: false,
    },
    CommandSchema {
        name: "merge_scene",
        description: "Merge a scene file into the current scene",
        category: "Scene",
        parameters: &["filepath", "import_location"],
        high_risk: true,
    },
    CommandSchema {
        name: "get_scene_stats",
        description: "Get scene statistics",
        category: "Scene",
        parameters: &["detailed"],
        high_risk: false,
    },
    CommandSchema {
        name: "select_all",
        description: "Select all nodes in the scene",
        category: "Selection",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "deselect_all",
        description: "Deselect all nodes",
        category: "Selection",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "select_children",
        description: "Select all children of a node",
        category: "Selection",
        parameters: &["node_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "select_parent",
        description: "Select the parent of the current selection",
        category: "Selection",
        parameters: &["node_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "get_selection_count",
        description: "Get the number of selected nodes",
        category: "Selection",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "delete_camera",
        description: "Delete a camera from the scene",
        category: "Camera",
        parameters: &["camera_name"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_camera_target",
        description: "Set camera aim/focus point",
        category: "Camera",
        parameters: &["camera_name", "target"],
        high_risk: false,
    },
    CommandSchema {
        name: "get_camera_properties",
        description: "Get detailed properties of a camera",
        category: "Camera",
        parameters: &["camera_name"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_pivot",
        description: "Set a node's pivot point",
        category: "Transform",
        parameters: &["node_id", "pivot"],
        high_risk: false,
    },
    CommandSchema {
        name: "selection_map_list",
        description: "List selection set maps",
        category: "Selection",
        parameters: &[],
        high_risk: false,
    },
    CommandSchema {
        name: "selection_map_get_pairs",
        description: "Get all node-group pairs in a selection map",
        category: "Selection",
        parameters: &["map_index"],
        high_risk: false,
    },
    CommandSchema {
        name: "selection_map_add_pair",
        description: "Add a node-group pair to a selection map",
        category: "Selection",
        parameters: &["map_index", "node_id", "group_name"],
        high_risk: false,
    },
    CommandSchema {
        name: "selection_map_remove_pair",
        description: "Remove a node-group pair from a selection map by index",
        category: "Selection",
        parameters: &["map_index", "pair_index"],
        high_risk: false,
    },
    CommandSchema {
        name: "selection_map_clear",
        description: "Clear all pairs from a selection map",
        category: "Selection",
        parameters: &["map_index"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_node_selectable",
        description: "Control whether a node can be selected in the viewport",
        category: "Node",
        parameters: &["node_id", "selectable"],
        high_risk: false,
    },
    CommandSchema {
        name: "set_render_visible",
        description: "Control whether a node is visible in renders",
        category: "Node",
        parameters: &["node_id", "visible"],
        high_risk: false,
    },
    CommandSchema {
        name: "parent_node",
        description: "Reparent a node under a new parent",
        category: "Node",
        parameters: &["node_id", "parent_id"],
        high_risk: true,
    },
    CommandSchema {
        name: "unparent_node",
        description: "Remove a node from its parent",
        category: "Node",
        parameters: &["node_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "mesh_get_vertex_count",
        description: "Get vertex count of a mesh node",
        category: "Mesh",
        parameters: &["node_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "mesh_get_face_count",
        description: "Get face/polygon count of a mesh node",
        category: "Mesh",
        parameters: &["node_id"],
        high_risk: false,
    },
    CommandSchema {
        name: "get_shape_materials",
        description: "List material names on a shape",
        category: "Material",
        parameters: &["node_id", "shape_index"],
        high_risk: false,
    },
    CommandSchema {
        name: "lock_property",
        description: "Lock or unlock a property",
        category: "Property",
        parameters: &["node_id", "property_name", "locked"],
        high_risk: false,
    },
];

pub struct McpConnection {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    host: String,
    port: u16,
}

impl McpConnection {
    pub fn connect(host: &str, port: u16) -> Result<Self, String> {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect_timeout(
            &addr
                .parse()
                .map_err(|e| format!("Invalid address: {}", e))?,
            Duration::from_secs(5),
        )
        .map_err(|e| format!("Bridge connection failed: {}", e))?;

        stream
            .set_read_timeout(Some(Duration::from_secs(20)))
            .map_err(|e| format!("Failed to set read timeout: {}", e))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(10)))
            .map_err(|e| format!("Failed to set write timeout: {}", e))?;

        let reader = BufReader::new(
            stream
                .try_clone()
                .map_err(|e| format!("Failed to clone stream: {}", e))?,
        );

        Ok(Self {
            stream,
            reader,
            host: host.to_string(),
            port,
        })
    }

    pub fn send_json(&mut self, request: &DazRequest) -> Result<McpResponse, String> {
        let json = serde_json::to_string(request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;
        let wire = format!("{}\n", json);
        self.stream
            .write_all(wire.as_bytes())
            .map_err(|e| format!("Failed to send bridge request: {}", e))?;
        self.stream
            .flush()
            .map_err(|e| format!("Failed to flush bridge request: {}", e))?;

        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => Err("Bridge closed connection without response".to_string()),
            Ok(_) => {
                let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
                if trimmed.is_empty() {
                    return Err("Bridge returned an empty response".to_string());
                }
                parse_bridge_response(trimmed)
            },
            Err(e) => Err(format!("Failed to read bridge response: {}", e)),
        }
    }

    pub fn reconnect(&mut self) -> Result<(), String> {
        *self = Self::connect(&self.host, self.port)?;
        Ok(())
    }
}

fn parse_bridge_response(raw: &str) -> Result<McpResponse, String> {
    let value: Value = serde_json::from_str(raw)
        .map_err(|e| format!("Failed to parse bridge response: {} - raw: {}", e, raw))?;

    let status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("error")
        .to_string();

    let error = match status.as_str() {
        "error" => value
            .get("error")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        _ => None,
    };

    let result = value
        .get("result")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let commands = value
        .get("commands")
        .cloned()
        .and_then(|v| serde_json::from_value(v).ok());

    let data = value.get("data").cloned().or_else(|| {
        let mut object = value.as_object()?.clone();
        object.remove("id");
        object.remove("status");
        object.remove("error");
        object.remove("result");
        object.remove("commands");
        (!object.is_empty()).then_some(Value::Object(object))
    });

    Ok(McpResponse {
        status,
        result,
        commands,
        data,
        error,
    })
}

pub fn get_command_schemas() -> Vec<CommandSchema> {
    COMMAND_SCHEMAS.to_vec()
}

pub fn get_mcp_command_list() -> Vec<McpCommand> {
    COMMAND_SCHEMAS
        .iter()
        .map(|schema| McpCommand {
            name: schema.name.to_string(),
            description: schema.description.to_string(),
            category: schema.category.to_string(),
            parameters: schema.parameters.iter().map(|p| p.to_string()).collect(),
        })
        .collect()
}

pub fn command_requires_confirmation(command: &str) -> bool {
    COMMAND_SCHEMAS
        .iter()
        .find(|schema| schema.name == command)
        .map(|schema| schema.high_risk)
        .unwrap_or(true)
}

pub fn validate_command(command: &str, args: &Value) -> Result<(), String> {
    let Some(schema) = COMMAND_SCHEMAS.iter().find(|schema| schema.name == command) else {
        return Err(format!("Unsupported Daz bridge command: {}", command));
    };

    let provided: HashSet<&str> = args
        .as_object()
        .map(|obj| obj.keys().map(|k| k.as_str()).collect())
        .unwrap_or_default();

    for required in schema.parameters {
        if !provided.contains(required) {
            return Err(format!(
                "Command '{}' is missing required argument '{}'",
                command, required
            ));
        }
    }

    Ok(())
}

pub fn set_daz3d_connection(host: &str, port: u16) -> Result<String, String> {
    let conn = McpConnection::connect(host, port)?;
    let mut global = MCP_CLIENT.lock().unwrap();
    *global = Some(conn);
    Ok(format!("Connected to Daz3D bridge at {}:{}", host, port))
}

pub fn disconnect_daz3d() -> String {
    let mut global = MCP_CLIENT.lock().unwrap();
    *global = None;
    "Disconnected".to_string()
}

pub fn is_connected() -> bool {
    MCP_CLIENT.lock().unwrap().is_some()
}

pub fn check_connection_status() -> String {
    if MCP_CLIENT.lock().unwrap().is_some() {
        "connected".to_string()
    } else {
        "disconnected".to_string()
    }
}

pub fn send_mcp_request(command: &str, args: Value) -> Result<McpResponse, String> {
    validate_command(command, &args)?;

    let mut global = MCP_CLIENT.lock().unwrap();
    let Some(ref mut conn) = *global else {
        return Err(
            "Not connected to Daz3D. Start Daz Studio with the bridge plugin loaded, then connect."
                .to_string(),
        );
    };

    let request = DazRequest {
        id: uuid_simple(),
        command: command.to_string(),
        args,
    };

    match conn.send_json(&request) {
        Ok(resp) => {
            if resp.status == "error" {
                Err(resp
                    .error
                    .clone()
                    .unwrap_or_else(|| "Daz bridge command failed".to_string()))
            } else {
                Ok(resp)
            }
        },
        Err(e) => {
            if conn.reconnect().is_ok() {
                conn.send_json(&request)
            } else {
                *global = None;
                Err(format!("Daz bridge connection lost: {}", e))
            }
        },
    }
}

fn uuid_simple() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{:x}{:x}{:04x}", now.as_secs(), now.subsec_nanos(), seq)
}

#[tauri::command]
pub fn connect_to_daz3d(host: String, port: u16) -> Result<String, String> {
    set_daz3d_connection(&host, port)
}

#[tauri::command]
pub fn disconnect_from_daz3d() -> String {
    disconnect_daz3d()
}

#[tauri::command]
pub fn check_daz3d_connection_status() -> String {
    check_connection_status()
}

#[tauri::command]
pub fn send_daz3d_command(command: String, args: Value) -> Result<McpResponse, String> {
    send_mcp_request(&command, args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn status_reports_disconnected() {
        assert_eq!(check_connection_status(), "disconnected");
    }

    #[test]
    fn validates_required_arguments() {
        assert!(validate_command("select_node", &serde_json::json!({})).is_err());
        assert!(
            validate_command("select_node", &serde_json::json!({ "node_id": "Genesis" })).is_ok()
        );
    }

    #[test]
    fn rejects_unknown_commands() {
        assert!(validate_command("pretend_success", &serde_json::json!({})).is_err());
    }

    #[test]
    #[serial]
    fn connection_fails_with_useful_error_when_bridge_not_running() {
        let result = McpConnection::connect("127.0.0.1", 19999);
        assert!(result.is_err());
        match result {
            Err(ref err) => {
                assert!(
                    err.contains("Bridge connection failed"),
                    "Error should mention bridge connection: {}",
                    err
                );
            },
            Ok(_) => panic!("Expected connection to fail"),
        }
    }

    #[test]
    fn bridge_response_parser_handles_valid_json() {
        let raw = r#"{"status":"ok","result":"test","data":{"key":"value"}}"#;
        let resp = parse_bridge_response(raw).unwrap();
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.result, Some("test".to_string()));
        assert!(resp.data.is_some());
    }

    #[test]
    fn bridge_response_parser_handles_error_json() {
        let raw = r#"{"status":"error","error":"something broke"}"#;
        let resp = parse_bridge_response(raw).unwrap();
        assert_eq!(resp.status, "error");
    }

    #[test]
    fn acceptance_schema_includes_workflow_commands() {
        let names: Vec<&str> = COMMAND_SCHEMAS.iter().map(|s| s.name).collect();
        for required in [
            "get_scene_assets",
            "add_figure",
            "set_morph",
            "set_light",
            "set_render_settings",
            "set_body_opacity",
            "set_surface_opacity",
            "get_internal_surfaces",
            "show_anatomy",
            "place_asset_inside",
        ] {
            assert!(names.contains(&required), "missing schema {}", required);
        }
    }

    #[test]
    fn material_opacity_commands_validate_required_arguments() {
        assert!(validate_command(
            "set_body_opacity",
            &serde_json::json!({ "node_id": "selected", "value": 0.2 })
        )
        .is_ok());
        assert!(validate_command(
            "set_surface_opacity",
            &serde_json::json!({ "node_id": "selected", "surface_pattern": "torso" })
        )
        .is_err());
        assert!(validate_command(
            "place_asset_inside",
            &serde_json::json!({ "figure_id": "selected", "asset_path": "/Mock/Alien.duf" })
        )
        .is_ok());
    }

    #[test]
    fn bridge_response_parser_error_has_message() {
        let raw = r#"{"status":"error","error":"something broke"}"#;
        let resp = parse_bridge_response(raw).unwrap();
        assert_eq!(resp.error, Some("something broke".to_string()));
    }

    #[test]
    fn command_schemas_are_complete() {
        let commands = get_mcp_command_list();
        assert!(
            commands.len() >= 178,
            "Should have at least 178 commands, got {}",
            commands.len()
        );
        assert!(commands.iter().any(|c| c.name == "get_scene_info"));
        assert!(commands.iter().any(|c| c.name == "load_asset"));
        assert!(commands.iter().any(|c| c.name == "run_script"));
    }

    #[test]
    fn animation_commands_in_schema() {
        let names: Vec<&str> = COMMAND_SCHEMAS.iter().map(|s| s.name).collect();
        for cmd in &[
            "play_timeline",
            "pause_timeline",
            "stop_timeline",
            "get_timeline_state",
        ] {
            assert!(names.contains(cmd), "missing animation schema: {}", cmd);
        }
    }

    #[test]
    fn scene_property_commands_in_schema() {
        let names: Vec<&str> = COMMAND_SCHEMAS.iter().map(|s| s.name).collect();
        for cmd in &[
            "get_figure_morphs",
            "get_fitted_items",
            "get_active_expressions",
            "get_material_zones",
            "apply_morph",
            "apply_expression",
        ] {
            assert!(
                names.contains(cmd),
                "missing scene property schema: {}",
                cmd
            );
        }
    }

    #[test]
    fn schema_consistency_check() {
        let names: Vec<&str> = COMMAND_SCHEMAS.iter().map(|s| s.name).collect();
        assert!(names.len() >= 178, "Should have at least 178 commands");
        assert!(names.contains(&"get_scene_info"));
        assert!(names.contains(&"list_nodes"));
        assert!(names.contains(&"load_asset"));
        assert!(names.contains(&"run_script"));
        assert!(names.contains(&"export_scene"));
    }
}
