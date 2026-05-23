#[derive(Debug, Clone)]
pub struct CommandKnowledge {
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub parameters: &'static [&'static str],
    pub high_risk: bool,
    pub usage_notes: &'static [&'static str],
    pub sdk_refs: &'static [&'static str],
}

#[derive(Debug, Clone)]
pub struct CommandKnowledgeBase {
    commands: Vec<CommandKnowledge>,
}

impl CommandKnowledgeBase {
    pub fn new() -> Self {
        Self { commands: build_command_catalog() }
    }

    pub fn get_command(&self, name: &str) -> Option<&CommandKnowledge> {
        self.commands.iter().find(|c| c.name == name)
    }

    pub fn get_commands_by_category(&self, category: &str) -> Vec<&CommandKnowledge> {
        self.commands.iter().filter(|c| c.category == category).collect()
    }

    pub fn get_all_commands(&self) -> &[CommandKnowledge] {
        &self.commands
    }

    pub fn find_commands_for_purpose(&self, purpose: &str) -> Vec<&CommandKnowledge> {
        let lower = purpose.to_lowercase();
        self.commands.iter().filter(|c| {
            c.name.to_lowercase().contains(&lower)
                || c.description.to_lowercase().contains(&lower)
                || c.category.to_lowercase().contains(&lower)
                || c.usage_notes.iter().any(|n| n.to_lowercase().contains(&lower))
        }).collect()
    }

    pub fn get_scene_workflow_commands(&self) -> Vec<&CommandKnowledge> {
        let scene_flow: &[&str] = &["add_figure", "load_asset", "add_node", "set_light", "set_camera",
            "set_morph", "apply_pose", "apply_expression", "set_material_property",
            "set_material_texture", "set_render_options", "render_preview"];
        scene_flow.iter().filter_map(|name| self.get_command(name)).collect()
    }

    pub fn build_command_catalog_prompt(&self) -> String {
        let mut prompt = String::from("Available Daz Bridge commands:\n");
        for cmd in &self.commands {
            let risk = if cmd.high_risk { " [HIGH RISK]" } else { "" };
            prompt.push_str(&format!("\n- {} ({}){}. {}", cmd.name, cmd.category, risk, cmd.description));
            prompt.push_str(&format!("\n  params: {}", cmd.parameters.join(", ")));
            if !cmd.usage_notes.is_empty() {
                prompt.push_str(&format!("\n  notes: {}", cmd.usage_notes.join("; ")));
            }
        }
        prompt
    }
}

fn build_command_catalog() -> Vec<CommandKnowledge> {
    vec![
        CommandKnowledge {
            name: "get_commands",
            description: "List supported Daz bridge commands",
            category: "System",
            parameters: &[],
            high_risk: false,
            usage_notes: &["Use to discover available commands at runtime"],
            sdk_refs: &[],
        },
        CommandKnowledge {
            name: "get_scene_info",
            description: "Get current Daz scene summary",
            category: "Scene",
            parameters: &[],
            high_risk: false,
            usage_notes: &["Call first to understand the current scene state before planning"],
            sdk_refs: &["DzScene"],
        },
        CommandKnowledge {
            name: "list_nodes",
            description: "List all scene nodes with their IDs and types",
            category: "Scene",
            parameters: &[],
            high_risk: false,
            usage_notes: &["Returns node hierarchy including figures, lights, cameras, props"],
            sdk_refs: &["DzNode", "DzScene"],
        },
        CommandKnowledge {
            name: "add_node",
            description: "Add a primitive node (point_light, spot_light, distant_light, camera, null)",
            category: "Scene",
            parameters: &["type", "name"],
            high_risk: false,
            usage_notes: &[
                "Use for creating lights and cameras from scratch",
                "For figures use add_figure instead",
                "Light types: point_light, spot_light, distant_light, ambient_light",
            ],
            sdk_refs: &["DzLight", "DzCamera", "DzNull"],
        },
        CommandKnowledge {
            name: "delete_node",
            description: "Delete a node from the scene",
            category: "Scene",
            parameters: &["node_id"],
            high_risk: true,
            usage_notes: &["Cannot be undone. Use with caution."],
            sdk_refs: &["DzScene"],
        },
        CommandKnowledge {
            name: "get_selected_nodes",
            description: "List currently selected scene nodes",
            category: "Selection",
            parameters: &[],
            high_risk: false,
            usage_notes: &["Use to see what the user has selected before acting on selection"],
            sdk_refs: &["DzSelectionMgr"],
        },
        CommandKnowledge {
            name: "select_node",
            description: "Select a Daz scene node by id or name",
            category: "Selection",
            parameters: &["node_id"],
            high_risk: false,
            usage_notes: &["Use before commands that operate on 'selected' node"],
            sdk_refs: &["DzSelectionMgr"],
        },
        CommandKnowledge {
            name: "get_cameras",
            description: "List all scene cameras",
            category: "Camera",
            parameters: &[],
            high_risk: false,
            usage_notes: &["Use to find camera names before calling set_camera"],
            sdk_refs: &["DzCamera"],
        },
        CommandKnowledge {
            name: "set_camera",
            description: "Set active camera or adjust camera properties (focal length/distance)",
            category: "Camera",
            parameters: &["camera", "focal_length", "focal_distance"],
            high_risk: false,
            usage_notes: &[
                "Switch camera by name (e.g. 'Perspective View')",
                "Omit focal params to leave them unchanged",
                "focal_length: 35mm (wide) to 200mm (tele), typical portrait: 85mm",
                "focal_distance: distance to focal plane in scene units",
            ],
            sdk_refs: &["DzCamera"],
        },
        CommandKnowledge {
            name: "load_asset",
            description: "Load a Daz asset file into the current scene",
            category: "Assets",
            parameters: &["path"],
            high_risk: false,
            usage_notes: &[
                "Path can be relative to content directory or absolute",
                "Use search_content to find assets before loading",
                "Supports .duf, .dsf, .pz2, .bmp, etc.",
            ],
            sdk_refs: &["DzContentMgr", "DzAsset"],
        },
        CommandKnowledge {
            name: "search_content",
            description: "Search Daz content library for assets by name/type",
            category: "Assets",
            parameters: &["query", "type", "max_results"],
            high_risk: false,
            usage_notes: &[
                "Use before load_asset to find exact paths",
                "type: figure, clothing, hair, pose, material, prop, environment",
                "max_results: default 10, max 100",
            ],
            sdk_refs: &["DzContentMgr"],
        },
        CommandKnowledge {
            name: "add_figure",
            description: "Add a Genesis figure (8 or 9) to the scene",
            category: "Scene",
            parameters: &["figure_type"],
            high_risk: false,
            usage_notes: &[
                "figure_type: 'genesis8' or 'genesis9'",
                "Creates a base figure with no morphs applied",
                "Use set_morph or apply_morph to customize appearance",
            ],
            sdk_refs: &["DzFigure"],
        },
        CommandKnowledge {
            name: "apply_pose",
            description: "Apply a pose file to a figure",
            category: "Pose",
            parameters: &["pose_path", "figure_id"],
            high_risk: false,
            usage_notes: &[
                "pose_path: path to .duf pose file",
                "figure_id: optional, applies to selected if omitted",
                "Use search_content to find poses first",
            ],
            sdk_refs: &["DzPose"],
        },
        CommandKnowledge {
            name: "set_morph",
            description: "Set a morph dial value on a figure (0.0–1.0)",
            category: "Properties",
            parameters: &["node_id", "morph", "value"],
            high_risk: false,
            usage_notes: &[
                "node_id: the figure node",
                "morph: exact morph dial name (e.g. 'Head_Height')",
                "value: 0.0 (minimum) to 1.0 (maximum)",
                "Use get_figure_morphs to list available morphs",
            ],
            sdk_refs: &["DzModifier", "DzFigure"],
        },
        CommandKnowledge {
            name: "get_figure_morphs",
            description: "Get all morph dials and their values for a figure",
            category: "Properties",
            parameters: &["figure_id"],
            high_risk: false,
            usage_notes: &["Use to discover available morphs before calling set_morph or apply_morph"],
            sdk_refs: &["DzModifier", "DzFigure"],
        },
        CommandKnowledge {
            name: "apply_morph",
            description: "Set a morph dial value on a figure (0.0–1.0)",
            category: "Properties",
            parameters: &["figure_id", "morph_id", "value"],
            high_risk: false,
            usage_notes: &[
                "Alternative to set_morph with different parameter format",
                "figure_id: the figure identifier",
                "morph_id: the morph identifier from get_figure_morphs",
            ],
            sdk_refs: &["DzModifier"],
        },
        CommandKnowledge {
            name: "apply_expression",
            description: "Set an expression dial value on a figure",
            category: "Properties",
            parameters: &["figure_id", "expression_id", "value"],
            high_risk: false,
            usage_notes: &[
                "expression_id: expression name from get_active_expressions",
                "value: 0.0 to 1.0",
                "Use get_active_expressions to list available expressions",
            ],
            sdk_refs: &["DzModifier"],
        },
        CommandKnowledge {
            name: "get_active_expressions",
            description: "Get all active expression dial values on a figure",
            category: "Properties",
            parameters: &["figure_id"],
            high_risk: false,
            usage_notes: &["Use before apply_expression to see current expression state"],
            sdk_refs: &["DzModifier"],
        },
        CommandKnowledge {
            name: "set_property",
            description: "Set a node property (transforms, visibility, etc.)",
            category: "Properties",
            parameters: &["node_id", "property", "value"],
            high_risk: false,
            usage_notes: &[
                "For morphs use set_morph instead",
                "For materials use set_material_property instead",
                "Common properties: xpos, ypos, zpos, xrot, yrot, zrot, xscale, yscale, zscale",
            ],
            sdk_refs: &["DzProperty"],
        },
        CommandKnowledge {
            name: "get_node_properties",
            description: "Get animatable properties of a node",
            category: "Properties",
            parameters: &["node_id"],
            high_risk: false,
            usage_notes: &[
                "Returns list of animatable properties with their current values",
                "Use before set_property or set_keyframe to discover available properties",
            ],
            sdk_refs: &["DzProperty"],
        },
        CommandKnowledge {
            name: "set_light",
            description: "Set a light property (intensity, color, etc.)",
            category: "Lighting",
            parameters: &["node_id", "property", "value"],
            high_risk: false,
            usage_notes: &[
                "node_id: the light node (created via add_node)",
                "property: 'intensity', 'color', 'enable', 'shadow'",
                "value: for color use 'R,G,B' format (0-255), for intensity use float",
                "Create lights first with add_node, then configure with set_light",
            ],
            sdk_refs: &["DzLight"],
        },
        CommandKnowledge {
            name: "set_material_property",
            description: "Set a material property (color, roughness, etc.)",
            category: "Materials",
            parameters: &["node_id", "property", "value"],
            high_risk: false,
            usage_notes: &[
                "Common properties: 'Base Color', 'Roughness', 'Metallic', 'Opacity'",
                "For textures use set_material_texture",
            ],
            sdk_refs: &["DzMaterial"],
        },
        CommandKnowledge {
            name: "get_material_properties",
            description: "Get material properties of a node (shader parameters)",
            category: "Materials",
            parameters: &["node_id"],
            high_risk: false,
            usage_notes: &["Use before set_material_property to see current material state"],
            sdk_refs: &["DzMaterial"],
        },
        CommandKnowledge {
            name: "get_material_zones",
            description: "Get material zone names on a figure",
            category: "Materials",
            parameters: &["figure_id"],
            high_risk: false,
            usage_notes: &["Use to discover surface zones before setting material properties per zone"],
            sdk_refs: &["DzMaterial"],
        },
        CommandKnowledge {
            name: "set_material_texture",
            description: "Assign a texture map file to a material surface channel",
            category: "Materials",
            parameters: &["node_id", "channel", "file_path"],
            high_risk: false,
            usage_notes: &[
                "channel: 'Base Color', 'Roughness', 'Normal', 'Metallic', 'Opacity', 'Subsurface'",
                "file_path: absolute path to texture image file",
                "Use get_material_channels to see current texture assignments",
            ],
            sdk_refs: &["DzImageProperty", "DzDefaultMaterial"],
        },
        CommandKnowledge {
            name: "get_material_channels",
            description: "Get all surface channels with texture paths and values",
            category: "Materials",
            parameters: &["node_id"],
            high_risk: false,
            usage_notes: &["Use before set_material_texture to see current texture state"],
            sdk_refs: &["DzImageProperty", "DzDefaultMaterial"],
        },
        CommandKnowledge {
            name: "get_fitted_items",
            description: "Get all fitted clothing/accessories on a figure",
            category: "Scene",
            parameters: &["figure_id"],
            high_risk: false,
            usage_notes: &["Use to see what clothing is already fitted before adding more"],
            sdk_refs: &["DzFigure"],
        },
        CommandKnowledge {
            name: "get_scene_assets",
            description: "Get list of loaded asset labels currently in the Daz Studio scene",
            category: "Scene",
            parameters: &[],
            high_risk: false,
            usage_notes: &["Returns asset labels for all loaded assets in the scene"],
            sdk_refs: &["DzContentMgr"],
        },
        CommandKnowledge {
            name: "get_geoshells",
            description: "Get all Geometry Shells in the scene",
            category: "Scene",
            parameters: &[],
            high_risk: false,
            usage_notes: &["Geometry shells are generated by some clothing items"],
            sdk_refs: &["DzGeoShell"],
        },
        CommandKnowledge {
            name: "get_node_transform",
            description: "Get node world-space transform (position, rotation, scale)",
            category: "Scene",
            parameters: &["node_id"],
            high_risk: false,
            usage_notes: &[
                "Returns position as [x, y, z] array",
                "Returns rotation as Euler angles [x, y, z] in degrees",
                "Returns scale as [x, y, z] array",
            ],
            sdk_refs: &["DzNode"],
        },
        CommandKnowledge {
            name: "set_node_transform",
            description: "Set node world-space position, rotation, or scale",
            category: "Scene",
            parameters: &["node_id", "position", "rotation", "scale"],
            high_risk: true,
            usage_notes: &[
                "position: [x, y, z] float array",
                "rotation: [x, y, z] float array in degrees",
                "scale: [x, y, z] float array",
                "Omit arrays you don't want to change",
            ],
            sdk_refs: &["DzNode"],
        },
        CommandKnowledge {
            name: "begin_undo_batch",
            description: "Start a new undo batch in Daz Studio",
            category: "Scene",
            parameters: &[],
            high_risk: false,
            usage_notes: &["Always call before making multiple changes so user can undo as one action"],
            sdk_refs: &["DzUndoMgr"],
        },
        CommandKnowledge {
            name: "accept_undo_batch",
            description: "Accept the current undo batch with a caption",
            category: "Scene",
            parameters: &["caption"],
            high_risk: false,
            usage_notes: &["caption: human-readable description for the undo history entry"],
            sdk_refs: &["DzUndoMgr"],
        },
        CommandKnowledge {
            name: "cancel_undo_batch",
            description: "Cancel the current undo batch",
            category: "Scene",
            parameters: &[],
            high_risk: false,
            usage_notes: &["Use to roll back changes if something goes wrong mid-batch"],
            sdk_refs: &["DzUndoMgr"],
        },
        CommandKnowledge {
            name: "save_scene",
            description: "Save the current scene to a file",
            category: "Scene",
            parameters: &["path"],
            high_risk: true,
            usage_notes: &["Overwrites existing file. Use with caution."],
            sdk_refs: &["DzScene"],
        },
        CommandKnowledge {
            name: "load_scene",
            description: "Load a scene file with method (default/new/merge)",
            category: "Scene",
            parameters: &["path", "method"],
            high_risk: true,
            usage_notes: &["method: 'default', 'new' (clears first), or 'merge'"],
            sdk_refs: &["DzScene"],
        },
        CommandKnowledge {
            name: "clear_scene",
            description: "Clear the current scene",
            category: "Scene",
            parameters: &[],
            high_risk: true,
            usage_notes: &["Cannot be undone. All unsaved work will be lost."],
            sdk_refs: &["DzScene"],
        },
        CommandKnowledge {
            name: "render_preview",
            description: "Trigger a Daz preview render using current settings",
            category: "Render",
            parameters: &[],
            high_risk: false,
            usage_notes: &[
                "Set render options first with set_render_settings or set_render_options",
                "Renders using the active viewport camera",
            ],
            sdk_refs: &["DzRenderMgr"],
        },
        CommandKnowledge {
            name: "set_render_settings",
            description: "Apply render resolution and quality presets quickly",
            category: "Render",
            parameters: &["width", "height"],
            high_risk: false,
            usage_notes: &[
                "Quick preset-based render config",
                "For full control use set_render_options instead",
                "Common presets: 1920x1080 (HD), 3840x2160 (4K), 800x600 (preview)",
            ],
            sdk_refs: &["DzRenderMgr"],
        },
        CommandKnowledge {
            name: "set_render_options",
            description: "Set render quality, resolution, and output options in detail",
            category: "Render",
            parameters: &["width", "height", "pixel_samples", "ray_trace_depth", "shading_rate", "gamma"],
            high_risk: false,
            usage_notes: &[
                "For quick setup use set_render_settings instead",
                "pixel_samples: 16 (draft) to 4096 (final quality)",
                "ray_trace_depth: 2 (draft) to 8 (final quality, default: 4)",
                "shading_rate: 1.0 (default)",
                "gamma: 2.2 (sRGB default)",
            ],
            sdk_refs: &["DzRenderOptions", "DzRenderMgr"],
        },
        CommandKnowledge {
            name: "capture_viewport",
            description: "Capture the active Daz viewport to an image file",
            category: "Viewport",
            parameters: &["path"],
            high_risk: false,
            usage_notes: &["path: absolute path to save the PNG image"],
            sdk_refs: &["DzViewport"],
        },
        CommandKnowledge {
            name: "set_viewport_mode",
            description: "Set viewport display mode (texture, shaded, wireframe, etc.)",
            category: "Viewport",
            parameters: &["mode"],
            high_risk: false,
            usage_notes: &[
                "mode: 'texture' (full textured), 'shaded' (smooth), 'wireframe', 'lit_wireframe', 'smooth_flat'",
                "Useful for preview performance vs quality control",
            ],
            sdk_refs: &["DzViewport"],
        },
        CommandKnowledge {
            name: "viewport_click",
            description: "Pick and select a node in the viewport at given coordinates",
            category: "Viewport",
            parameters: &["x", "y"],
            high_risk: false,
            usage_notes: &["x, y: pixel coordinates in the viewport"],
            sdk_refs: &["DzViewport"],
        },
        CommandKnowledge {
            name: "set_keyframe",
            description: "Set an animatable float property keyframe at a specific frame",
            category: "Animation",
            parameters: &["node_id", "property", "frame", "value", "interpolation"],
            high_risk: false,
            usage_notes: &[
                "property: animatable property name (see get_node_properties)",
                "frame: timeline frame number",
                "interpolation: 'linear', 'bezier', 'smooth', 'step', or 'auto'",
            ],
            sdk_refs: &["DzKeyframe", "DzFloatKeyframe"],
        },
        CommandKnowledge {
            name: "set_timeline_range",
            description: "Set the Daz Studio play range and animation range",
            category: "Animation",
            parameters: &["start_frame", "end_frame"],
            high_risk: false,
            usage_notes: &["Sets both the playback range and animation range"],
            sdk_refs: &["DzTime"],
        },
        CommandKnowledge {
            name: "seek_to_frame",
            description: "Move the timeline cursor to a specific frame",
            category: "Animation",
            parameters: &["frame"],
            high_risk: false,
            usage_notes: &["Use before setting keyframes to position the timeline"],
            sdk_refs: &["DzTime"],
        },
        CommandKnowledge {
            name: "play_timeline",
            description: "Start Daz Studio timeline playback",
            category: "Animation",
            parameters: &[],
            high_risk: false,
            usage_notes: &["Starts playback from current frame"],
            sdk_refs: &["DzTime"],
        },
        CommandKnowledge {
            name: "pause_timeline",
            description: "Pause Daz Studio timeline playback",
            category: "Animation",
            parameters: &[],
            high_risk: false,
            usage_notes: &[],
            sdk_refs: &["DzTime"],
        },
        CommandKnowledge {
            name: "stop_timeline",
            description: "Stop playback and reset to frame 0",
            category: "Animation",
            parameters: &[],
            high_risk: false,
            usage_notes: &["Stops and resets the timeline cursor to frame 0"],
            sdk_refs: &["DzTime"],
        },
        CommandKnowledge {
            name: "get_timeline_state",
            description: "Query current timeline frame, range, fps, and playback state",
            category: "Animation",
            parameters: &[],
            high_risk: false,
            usage_notes: &["Returns current frame, start/end range, FPS, and whether playing"],
            sdk_refs: &["DzTime"],
        },
        CommandKnowledge {
            name: "list_bones",
            description: "List all bones in a figure's skeleton",
            category: "Animation",
            parameters: &["figure_id"],
            high_risk: false,
            usage_notes: &[
                "Returns hierarchical bone names (e.g. 'hip', 'abdomen', 'chest', 'lShldr', 'rShldr', 'neck', 'head')",
                "Use before set_bone_transform to discover bone names",
            ],
            sdk_refs: &["DzBone", "DzSkeleton"],
        },
        CommandKnowledge {
            name: "set_bone_transform",
            description: "Set a bone's world-space position or rotation",
            category: "Animation",
            parameters: &["figure_id", "bone_name", "position", "rotation"],
            high_risk: true,
            usage_notes: &[
                "bone_name: from list_bones output",
                "position: [x, y, z] or omit",
                "rotation: [x, y, z] Euler angles in degrees or omit",
                "Can break figure posing if used incorrectly",
            ],
            sdk_refs: &["DzBone", "DzSkeleton"],
        },
        CommandKnowledge {
            name: "list_keyframes",
            description: "List all keyframes on a node property",
            category: "Animation",
            parameters: &["node_id", "property"],
            high_risk: false,
            usage_notes: &["Use to inspect existing animation data before modifying keyframes"],
            sdk_refs: &["DzKeyframe"],
        },
        CommandKnowledge {
            name: "delete_keyframes",
            description: "Delete keyframes from a node property (range or all)",
            category: "Animation",
            parameters: &["node_id", "property", "start", "end"],
            high_risk: true,
            usage_notes: &["Omitting start/end deletes all keyframes on that property"],
            sdk_refs: &["DzKeyframe"],
        },
        CommandKnowledge {
            name: "run_dforce_simulation",
            description: "Run a dForce physics simulation via inline DazScript",
            category: "Animation",
            parameters: &["node_id", "start_frame", "end_frame"],
            high_risk: false,
            usage_notes: &[
                "Simulates dynamic clothing/hair physics",
                "Requires dForce-enabled clothing to be fitted to figure",
                "start_frame/end_frame: frame range for simulation",
            ],
            sdk_refs: &["DzSimulator"],
        },
        CommandKnowledge {
            name: "run_script",
            description: "Evaluate arbitrary DazScript on the main thread",
            category: "Scripting",
            parameters: &["script", "args"],
            high_risk: true,
            usage_notes: &[
                "Can execute any DazScript API call",
                "Use as last resort when no bridge command exists for a task",
                "script: DazScript code as a string",
            ],
            sdk_refs: &["DazScript"],
        },
        CommandKnowledge {
            name: "import_model",
            description: "Import a model file through Daz (OBJ, FBX, etc.)",
            category: "Assets",
            parameters: &["path", "settings"],
            high_risk: false,
            usage_notes: &[
                "Supported formats: OBJ, FBX, glTF",
                "settings: JSON object with import options",
            ],
            sdk_refs: &["DzImportMgr"],
        },
        CommandKnowledge {
            name: "export_scene",
            description: "Export scene or node through Daz",
            category: "Assets",
            parameters: &["node_id", "path", "settings"],
            high_risk: true,
            usage_notes: &["node_id: empty for full scene, specific node for single export"],
            sdk_refs: &["DzExportMgr", "DzExporter"],
        },
        CommandKnowledge {
            name: "apply_phy_modifier",
            description: "Apply DazPilot physics modifier to a node (cloth/jelly/hair)",
            category: "Physics",
            parameters: &["node_id", "stiffness", "damping", "mass"],
            high_risk: false,
            usage_notes: &[
                "stiffness: 0.0 (floppy) to 1.0 (rigid)",
                "damping: 0.0 to 1.0 (energy dissipation)",
                "mass: > 0.0 (heavier = more gravity influence)",
            ],
            sdk_refs: &["DzModifier"],
        },
        CommandKnowledge {
            name: "remove_phy_modifier",
            description: "Remove DazPilot physics modifier from a node",
            category: "Physics",
            parameters: &["node_id"],
            high_risk: false,
            usage_notes: &["Removes the DazPilot physics modifier if present"],
            sdk_refs: &["DzModifier"],
        },
        CommandKnowledge {
            name: "set_phy_modifier_params",
            description: "Update DazPilot physics modifier parameters",
            category: "Physics",
            parameters: &["node_id", "stiffness", "damping", "mass"],
            high_risk: false,
            usage_notes: &["Updates existing modifier without reapplying"],
            sdk_refs: &["DzModifier"],
        },
        CommandKnowledge {
            name: "get_bounding_boxes",
            description: "Get world-space 3D bounding boxes of all scene nodes",
            category: "Vision",
            parameters: &[],
            high_risk: false,
            usage_notes: &["Returns center, size, and min/max for each node's bounding box"],
            sdk_refs: &["DzNode"],
        },
        CommandKnowledge {
            name: "list_modifiers",
            description: "List all modifiers on a node's geometry object",
            category: "Scene",
            parameters: &["node_id"],
            high_risk: false,
            usage_notes: &[
                "Returns modifier names and types for the given node",
                "Useful for inspecting figure morphs, smoothing, subdivision modifiers",
            ],
            sdk_refs: &["DzModifier"],
        },
    ]
}
