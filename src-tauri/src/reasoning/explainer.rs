use crate::reasoning::planner::PlanningContext;
use crate::reasoning::planner::{Plan, PlanStep};

/// Explains plans and decisions to the user in understandable terms
pub struct Explainer {
    pub knowledge_base: crate::knowledge::daz_concepts::DazKnowledgeBase,
}

impl Explainer {
    pub fn new() -> Self {
        Self {
            knowledge_base: crate::knowledge::daz_concepts::DazKnowledgeBase::new(),
        }
    }

    /// Explain why a plan was chosen
    pub fn explain_plan_selection(
        &self,
        plan: &Plan,
        alternatives: &[Plan],
        _context: &PlanningContext,
    ) -> String {
        let mut explanation = String::new();

        explanation.push_str("I chose this plan because:\n");
        explanation.push_str(&format!(
            "• Overall confidence: {:.0}%\n",
            plan.confidence * 100.0
        ));
        explanation.push_str(&format!(
            "• Estimated time: {} seconds\n",
            plan.estimated_total_time_seconds
        ));
        explanation.push_str(&format!("• Risk level: {:?}\n", plan.risk_level));

        if let Some(ref fallback) = plan.fallback_plan {
            explanation.push_str(&format!(
                "• Has fallback plan: Yes ({} steps)\n",
                fallback.steps.len()
            ));
        } else {
            explanation.push_str("• Has fallback plan: No\n");
        }

        explanation.push_str("\nPlan details:\n");
        for (i, step) in plan.steps.iter().enumerate() {
            explanation.push_str(&format!(
                "{}. {} ({}s, {:.0}% confidence)\n",
                i + 1,
                step.description,
                step.estimated_time_seconds,
                step.confidence * 100.0
            ));
        }

        if !alternatives.is_empty() {
            explanation.push_str(&format!(
                "\nConsidered {} alternative plans:\n",
                alternatives.len()
            ));
            for (i, alt) in alternatives.iter().enumerate().take(3) {
                // Show top 3
                explanation.push_str(&format!(
                    "{}. {} (confidence: {:.0}%)\n",
                    i + 1,
                    alt.description,
                    alt.confidence * 100.0
                ));
            }
            if alternatives.len() > 3 {
                explanation.push_str(&format!("  ... and {} more\n", alternatives.len() - 3));
            }
        }

        explanation
    }

    /// Explain what a plan step does
    pub fn explain_step(&self, step: &PlanStep, _context: &PlanningContext) -> String {
        let mut explanation = String::new();

        explanation.push_str(&format!("Step: {}\n", step.description));
        explanation.push_str(&format!("Command: {}\n", step.action.command));
        explanation.push_str(&format!(
            "Estimated time: {} seconds\n",
            step.estimated_time_seconds
        ));
        explanation.push_str(&format!("Confidence: {:.0}%\n", step.confidence * 100.0));

        // Explain what the command does
        explanation.push_str(&format!(
            "Effect: {}\n",
            self.describe_command_effect(&step.action.command, &step.action.args)
        ));

        // Explain prerequisites
        if !step.prerequisites.is_empty() {
            explanation.push_str(&format!(
                "Prerequisites: {}\n",
                step.prerequisites.join(", ")
            ));
        } else {
            explanation.push_str("Prerequisites: None (can start immediately)\n");
        }

        // Explain alternatives if any
        if !step.alternatives.is_empty() {
            explanation.push_str(&format!(
                "Alternatives available: {} options\n",
                step.alternatives.len()
            ));
        }

        explanation
    }

    /// Explain why a plan failed
    pub fn explain_failure(
        &self,
        _plan: &Plan,
        failed_step: &PlanStep,
        error: &str,
        context: &PlanningContext,
    ) -> String {
        let mut explanation = String::new();

        explanation.push_str(&format!(
            "The plan failed at step: {}\n",
            failed_step.description
        ));
        explanation.push_str(&format!("Error: {}\n", error));

        // Provide context
        explanation.push_str("Context when failure occurred:\n");
        explanation.push_str(&format!(
            "• Attempted command: {}\n",
            failed_step.action.command
        ));
        explanation.push_str(&format!("• Command args: {}\n", failed_step.action.args));

        // Provide suggestions based on failure type
        explanation.push_str("Suggestions:\n");
        let suggestions = self.generate_failure_suggestions(failed_step, error, context);
        for suggestion in suggestions {
            explanation.push_str(&format!("• {}\n", suggestion));
        }

        explanation
    }

    /// Explain why an alternative approach might be better
    pub fn explain_alternative(
        &self,
        original: &Plan,
        alternative: &Plan,
        _context: &PlanningContext,
    ) -> String {
        let mut explanation = String::new();

        explanation.push_str(&format!(
            "Alternative plan '{}' might be better because:\n",
            alternative.description
        ));
        explanation.push_str(&format!(
            "• Higher confidence: {:.0}% vs {:.0}%\n",
            alternative.confidence * 100.0,
            original.confidence * 100.0
        ));
        explanation.push_str(&format!(
            "• Lower estimated time: {}s vs {}s\n",
            alternative.estimated_total_time_seconds, original.estimated_total_time_seconds
        ));
        explanation.push_str(&format!(
            "• Lower risk: {:?} vs {:?}\n",
            alternative.risk_level, original.risk_level
        ));

        explanation
    }

    /// Describe what a command does in plain language
    fn describe_command_effect(&self, command: &str, args: &serde_json::Value) -> String {
        match command {
            "add_figure" => {
                let figure_type = args
                    .get("figure_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let figure_name = if figure_type == "genesis9" {
                    "Genesis 9 Female"
                } else {
                    "Genesis 8 Female"
                };
                format!("Add a {} to the scene", figure_name)
            },
            "load_asset" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                format!("Load asset from {}", path)
            },
            "apply_pose" => {
                let pose = args
                    .get("pose")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                format!("Apply the {} pose", pose)
            },
            "set_keyframe" => {
                let node_id = args
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let property = args
                    .get("property")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let frame = args.get("frame").and_then(|v| v.as_i64()).unwrap_or(0);
                let value = args.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                format!(
                    "Set {} on {} to {} at frame {}",
                    property, node_id, value, frame
                )
            },
            "add_node" => {
                let node_type = args
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unnamed");
                format!("Add a {} named {}", node_type, name)
            },
            "set_property" => {
                let node_id = args
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let property = args
                    .get("property")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let value = args
                    .get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                format!("Set {} of {} to {}", property, node_id, value)
            },
            "set_light" => {
                let node_id = args
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let property = args
                    .get("property")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let value = args
                    .get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                format!("Set light {}'s {} to {}", node_id, property, value)
            },
            "render_preview" => "Generate a preview render of the current scene".to_string(),
            "export_scene" => {
                let format = args
                    .get("settings")
                    .and_then(|s| s.get("format").and_then(|v| v.as_str()))
                    .unwrap_or("unknown");
                format!("Export the scene as a {} file", format)
            },
            "run_dforce_simulation" => {
                let node_id = args
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let start = args
                    .get("start_frame")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let end = args.get("end_frame").and_then(|v| v.as_i64()).unwrap_or(30);
                format!(
                    "Run a dForce physics simulation on {} from frame {} to {}",
                    node_id, start, end
                )
            },
            "begin_undo_batch" => "Start recording changes for undo/redo".to_string(),
            "accept_undo_batch" => "Accept and name the recorded changes".to_string(),
            "cancel_undo_batch" => "Discard the recorded changes".to_string(),
            _ => {
                format!("Execute the {} command", command)
            },
        }
    }

    /// Generate suggestions for fixing a failed step
    fn generate_failure_suggestions(
        &self,
        step: &PlanStep,
        error: &str,
        _context: &PlanningContext,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        if error.contains("not found") || error.contains("failed to find") {
            suggestions.push("Check that the asset name or path is correct".to_string());
            suggestions.push("Try searching for the asset using different keywords".to_string());
            suggestions
                .push("Make sure the asset is installed and indexed in your library".to_string());
        }

        if error.contains("invalid") || error.contains("unsupported") {
            suggestions.push("Verify that the command parameters are correct".to_string());
            suggestions
                .push("Check if the asset type is compatible with the operation".to_string());
        }

        if error.contains("permission") || error.contains("denied") {
            suggestions.push("This operation may require explicit permission".to_string());
            suggestions.push("Try running a similar, lower-risk action first".to_string());
        }

        if step.action.command == "load_asset" && error.contains("not found") {
            suggestions.push("Try using the asset's exact filename from your library".to_string());
            suggestions.push("Search for similar assets that might work instead".to_string());
        }

        if step.action.command == "add_figure" {
            suggestions.push("Make sure the figure is installed in your Daz3D library".to_string());
            suggestions
                .push("Try checking the exact figure name in your content library".to_string());
        }

        if step.action.command.starts_with("set_") && error.contains("property") {
            suggestions.push(
                "Verify that the property name is correct for the selected node type".to_string(),
            );
            suggestions
                .push("Try checking what properties are available for this node type".to_string());
        }

        // If we have no specific suggestions, give general ones
        if suggestions.is_empty() {
            suggestions.push("Double-check the command parameters and try again".to_string());
            suggestions.push("Consider breaking this down into smaller steps".to_string());
            suggestions.push("Look at similar successful actions in your history".to_string());
        }

        suggestions
    }

    /// Generate an educational "why" snippet for a given command and its arguments.
    /// Returns None when no specific teaching is available for the command.
    pub fn explain_teaching_concept(
        &self,
        command: &str,
        args: &serde_json::Value,
    ) -> Option<String> {
        match command {
            "add_figure" => {
                let figure_type = args
                    .get("figure_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("figure");
                Some(format!(
                    "In DAZ Studio, figures are the base characters you build upon. \
                     '{}' is part of the Genesis ecosystem — a fully rigged 3D character \
                     with joints, morphs, and material zones. Genesis 8/9 figures share \
                     a common骨架 (skeleton) so clothing and poses transfer between them. \
                     \n\n💡 Try this: After adding a figure, explore the Parameters tab \
                     in DAZ Studio to see all the morphs and properties available.",
                    figure_type
                ))
            },
            "load_asset" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("an asset");
                Some(format!(
                    "DAZ Studio uses .duf files (DAZ Universal Format) for most content. \
                     Your content library is organized into categories like People, Clothing, \
                     Props, and Environments. Assets can include figures, clothing, hair, \
                     poses, materials, and more.\n\nLoading '{}'.\
                     \n\n💡 Try this: Open the Content Library pane in DAZ Studio to browse \
                     your installed assets manually. Right-click any asset to see its metadata.",
                    path
                ))
            },
            "apply_pose" => {
                let pose = args
                    .get("pose")
                    .and_then(|v| v.as_str())
                    .unwrap_or("this pose");
                Some(format!(
                    "Poses in DAZ Studio are stored as rotation values for each joint \
                     in a figure's skeleton. Each bone has X/Y/Z rotation properties \
                     that define its orientation. Poses can be applied as a whole or mixed.\
                     \n\n💡 Try this: After applying '{}', go to the Parameters tab and \
                     adjust individual joint rotations to customise the pose.",
                    pose
                ))
            },
            "set_keyframe" => {
                let node_id = args
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("a node");
                let frame = args.get("frame").and_then(|v| v.as_i64()).unwrap_or(0);
                Some(format!(
                    "Keyframes are the foundation of animation in DAZ Studio. A keyframe \
                     records the state of a property (position, rotation, scale) at a specific \
                     point in time. The Timeline panel shows you all keyframes as diamonds \
                     that you can drag to adjust timing.\
                     \n\nSetting keyframe on '{}' at frame {}.\
                     \n\n💡 Try this: Open the Timeline tab in DAZ Studio to see all \
                     keyframes. Shift-click to select multiple and drag to adjust timing.",
                    node_id, frame
                ))
            },
            "add_node" => {
                let node_type = args.get("type").and_then(|v| v.as_str()).unwrap_or("node");
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unnamed");
                Some(format!(
                    "Every object in a DAZ Studio scene is a 'node' in a hierarchical \
                     scene tree. Nodes can be figures, cameras, lights, props, or nulls. \
                     Child nodes inherit the transformations of their parent.\
                     \n\nAdding a '{}' node named '{}'.\
                     \n\n💡 Try this: Open the Scene tab to see the node hierarchy. \
                     You can parent nodes by dragging one onto another.",
                    node_type, name
                ))
            },
            "set_property" => {
                let node_id = args
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("node");
                let property = args
                    .get("property")
                    .and_then(|v| v.as_str())
                    .unwrap_or("property");
                let value = args
                    .get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("value");
                Some(format!(
                    "Properties control every aspect of a node — its translation, \
                     rotation, scale, visibility, and more. Each property can be keyframed \
                     for animation. The Parameters tab in DAZ Studio lists all editable \
                     properties for the selected node.\
                     \n\nSetting '{}' on '{}' to '{}'.\
                     \n\n💡 Try this: Select the node in the Viewport, then look at the \
                     Parameters tab to see all available properties.",
                    property, node_id, value
                ))
            },
            "set_light" => {
                let light_type = args
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("a light");
                let property = args
                    .get("property")
                    .and_then(|v| v.as_str())
                    .unwrap_or("intensity");
                Some(format!(
                    "DAZ Studio supports four light types:\n\
                     • Point lights — emit from a single point in all directions\n\
                     • Spot lights — emit a cone of light (like a stage spotlight)\n\
                     • Distant lights — parallel rays (simulates sunlight)\n\
                     • Area lights — emit from a rectangular surface (soft light)\n\n\
                     Adjusting '{}' on '{}'.\
                     \n\n💡 Try this: In the Viewport, select the light and use the \
                     manipulator tools (Move, Rotate, Scale) to position it manually.",
                    property, light_type
                ))
            },
            "render_preview" | "render" => Some(
                "Rendering converts your 3D scene into a final 2D image. DAZ Studio \
                     uses Iray (physically-based, NVIDIA GPU-accelerated) as its primary \
                     render engine. Iray simulates real-world light behavior — photons \
                     bouncing off surfaces based on material properties.\
                     \n\n💡 Try this: Before rendering, check the Render Settings tab. \
                     Higher samples = less noise but slower renders. Start with Draft mode \
                     for quick previews."
                    .to_string(),
            ),
            "export_scene" => {
                let format = args
                    .get("settings")
                    .and_then(|s| s.get("format").and_then(|v| v.as_str()))
                    .unwrap_or("the chosen format");
                Some(format!(
                    "DAZ Studio can export to many formats for use in other applications. \
                     Common export formats include:\n\
                     • FBX — for game engines (Unity, Unreal) and animation software\n\
                     • OBJ — universal mesh format (geometry only)\n\
                     • Collada (DAE) — for interoperability\n\n\
                     Exporting as '{}'.\
                     \n\n💡 Try this: Before exporting, make sure your figure is in the \
                     desired pose — some formats bake the pose into the mesh.",
                    format
                ))
            },
            "run_dforce_simulation" => {
                let node_id = args
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("selected node");
                Some(format!(
                    "dForce is DAZ Studio's physics simulation system. It simulates \
                     realistic cloth, hair, and soft-body dynamics. When you run a dForce \
                     simulation, the software calculates how vertices move based on gravity, \
                     collision, wind, and material stiffness over a range of frames.\
                     \n\nSimulating on '{}'.\
                     \n\n💡 Try this: Before simulating, adjust the dForce modifiers in \
                     the Parameters tab — increase Stiffness for heavier fabrics or \
                     decrease for flowing silk.",
                    node_id
                ))
            },
            "set_morph" => {
                let morph_name = args
                    .get("morph")
                    .and_then(|v| v.as_str())
                    .unwrap_or("this morph");
                let value = args.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                Some(format!(
                    "Morphs (also called 'shape keys' or 'blend shapes') deform a mesh \
                     by moving vertices to predefined positions. They are how you customise \
                     a figure's body shape, facial features, and expressions in DAZ Studio. \
                     Morphs range from 0.0 (off) to 1.0 (full effect).\
                     \n\nSetting '{}' to {}.\
                     \n\n💡 Try this: In the Parameters tab, search for morph names. \
                     Most figure packs include dozens of face and body morphs you can \
                     combine for unique looks.",
                    morph_name, value
                ))
            },
            "set_material" | "set_surface" => {
                let material_name = args
                    .get("material")
                    .and_then(|v| v.as_str())
                    .unwrap_or("a material");
                Some(format!(
                    "Materials (also called 'surfaces') define how an object looks when \
                     rendered — its color, roughness, shininess, transparency, and textures. \
                     DAZ Studio uses the UberSurface shader which supports:\n\
                     • Base Color — the main color/albedo\n\
                     • Roughness — how rough or smooth the surface is\n\
                     • Metallic — whether it behaves like metal\n\
                     • Normal map — adds surface detail without geometry\n\n\
                     Adjusting '{}'.\
                     \n\n💡 Try this: Open the Surfaces tab, select a surface, and \
                     experiment with different presets from the Content Library.",
                    material_name
                ))
            },
            "set_camera" | "create_camera" => Some(
                "Cameras in DAZ Studio define the viewpoint for rendering. Key camera \
                     properties include:\n\
                     • Focal length — wide angle (short) vs telephoto (long)\n\
                     • Aperture — depth of field blur\n\
                     • Distance — how far the camera is from the subject\n\n\
                     \n\n💡 Try this: Use the Viewport navigation (Alt+MMB to orbit, \
                     Alt+RMB to zoom) to find your ideal angle, then click Edit > \
                     Create New Camera to save that view."
                    .to_string(),
            ),
            "begin_undo_batch" => Some(
                "DAZ Studio has a multi-level Undo/Redo system. DazPilot uses \
                     undo batching to group related changes so a single Ctrl+Z undoes \
                     the entire operation instead of each small step.\
                     \n\n💡 Try this: Press Ctrl+Z in DAZ Studio to undo the last action, \
                     or use Edit > Undo/Redo History to jump to any previous state."
                    .to_string(),
            ),
            "select_node" => Some(
                "Selection is fundamental in DAZ Studio — most operations act on \
                     the currently selected node(s). You can select nodes in the Viewport \
                     (click) or in the Scene tab (Ctrl+click for multiple).\
                     \n\n💡 Try this: In the Scene tab, you can search for nodes by name, \
                     and use the filter to show only figures, lights, or cameras."
                    .to_string(),
            ),
            _ => None,
        }
    }

    /// Generate step-by-step DAZ Studio UI navigation instructions for a given command.
    /// Returns None when no manual instructions are available.
    pub fn manual_steps_for_command(
        &self,
        command: &str,
        _args: &serde_json::Value,
    ) -> Option<String> {
        match command {
            "add_figure" => Some(
                "1. Open the Content Library pane (Windows > Panes > Content Library)\n\
                 2. Navigate to People > Genesis 9 > Figures\n\
                 3. Double-click 'Genesis 9 Female' to load\n\
                 4. The figure appears in the Viewport and Scene tab"
                    .to_string(),
            ),
            "load_asset" => Some(
                "1. Open the Content Library pane\n\
                 2. Browse to the asset category (People, Clothing, Props, etc.)\n\
                 3. Find the asset you want and double-click it\n\
                 4. It will load into the current scene"
                    .to_string(),
            ),
            "apply_pose" => Some(
                "1. Select the figure in the Viewport or Scene tab\n\
                 2. Open Content Library > People > Genesis 9 > Poses\n\
                 3. Double-click a pose preset (.duf) to apply it\n\
                 4. The figure's joints update to match the pose"
                    .to_string(),
            ),
            "set_keyframe" => Some(
                "1. Select the node/property you want to animate\n\
                 2. Set the Timeline playhead to the desired frame\n\
                 3. In Parameters tab, right-click a property value\n\
                 4. Choose 'Create Keyframe' from the context menu\n\
                 5. Move to another frame, change the value, and keyframe again"
                    .to_string(),
            ),
            "add_node" => Some(
                "1. Go to Create menu in the top toolbar\n\
                 2. Choose New Primitive > Cube (or Sphere, Plane, etc.)\n\
                 3. The node appears in the Scene tab\n\
                 4. Use Edit > Figure > Rigging > Parent Node to reparent"
                    .to_string(),
            ),
            "set_property" => Some(
                "1. Select the node in the Viewport or Scene tab\n\
                 2. Open the Parameters tab (Windows > Panes > Parameters)\n\
                 3. Find the property in the list (search by name if needed)\n\
                 4. Drag the slider or type a value directly"
                    .to_string(),
            ),
            "set_light" => Some(
                "1. Select the light in the Viewport or Scene tab\n\
                 2. Open the Parameters tab\n\
                 3. Find the property (Intensity, Color, etc.)\n\
                 4. Adjust the value — Intensity affects brightness, Color changes tint\n\
                 5. Use the ActivePose tool to rotate/reposition the light in the Viewport"
                    .to_string(),
            ),
            "render_preview" | "render" => Some(
                "1. Open the Render Settings tab (Windows > Panes > Render Settings)\n\
                 2. Choose Iray or 3Delight as your render engine\n\
                 3. Select 'Draft' mode for a quick preview\n\
                 4. Click the 'Render' button in the Viewport toolbar (or press Ctrl+R)\n\
                 5. Wait for the render to complete — more samples = less noise"
                    .to_string(),
            ),
            "export_scene" => Some(
                "1. Go to File > Export\n\
                 2. Choose the format (FBX, OBJ, Collada, etc.)\n\
                 3. In the export dialog, select which nodes to include\n\
                 4. Configure export options (bake poses, embed textures, etc.)\n\
                 5. Click Accept/Export and choose a destination"
                    .to_string(),
            ),
            "run_dforce_simulation" => Some(
                "1. Select the clothing/hair item in the Scene tab\n\
                 2. Open the Parameters tab and find the dForce modifiers\n\
                 3. Adjust Stiffness, Gravity, and Wind settings\n\
                 4. Open the Timeline tab and set frame range (e.g., 1-30)\n\
                 5. Click the dForce button (cloth icon) in the Timeline toolbar\n\
                 6. Wait for simulation to calculate — green progress bar appears"
                    .to_string(),
            ),
            "set_morph" => Some(
                "1. Select the figure in the Viewport\n\
                 2. Open the Parameters tab\n\
                 3. Type the morph name in the search box (e.g., 'Fitness')\n\
                 4. Drag the slider to adjust the morph value (0-100%)\n\
                 5. Multiple morphs can be combined for unique looks"
                    .to_string(),
            ),
            "set_material" | "set_surface" => Some(
                "1. Select the figure or object in the Viewport\n\
                 2. Open the Surfaces tab (Windows > Panes > Surfaces)\n\
                 3. Select the surface/material from the list (e.g., 'Skin', 'Shirt')\n\
                 4. Find the property to edit (Base Color, Roughness, etc.)\n\
                 5. Click the color swatch or drag the slider to adjust"
                    .to_string(),
            ),
            "set_camera" | "create_camera" => Some(
                "1. Navigate to your desired view in the Viewport\n\
                 2. Go to Edit > Create New Camera to save this viewpoint\n\
                 3. Select the camera in the Scene tab\n\
                 4. In Parameters, adjust Focal Length for zoom (24mm=wide, 85mm=portrait)\n\
                 5. Enable Depth of Field in Render Settings, adjust Aperture for blur"
                    .to_string(),
            ),
            "select_node" => Some(
                "1. Click directly on an object in the Viewport\n\
                 2. Or use the Scene tab to find and click on nodes\n\
                 3. Ctrl+click to select multiple nodes\n\
                 4. The selected node's properties appear in Parameters tab"
                    .to_string(),
            ),
            "begin_undo_batch" => Some(
                "1. Press Ctrl+Z to undo the last action\n\
                 2. Press Ctrl+Y to redo\n\
                 3. For history, go to Edit > Undo/Redo History\n\
                 4. Click any entry to jump to that state"
                    .to_string(),
            ),
            _ => None,
        }
    }

    /// Explain what a plan achieves
    pub fn explain_plan_outcome(&self, plan: &Plan) -> String {
        let mut explanation = String::new();

        explanation.push_str("If successful, this plan will:\n");

        // Group steps by type to give a higher-level explanation
        let mut figures_added = 0;
        let mut assets_loaded = 0;
        let mut lights_added = 0;
        let mut poses_applied = 0;
        let mut renders_done = 0;

        for step in &plan.steps {
            match step.action.command.as_str() {
                "add_figure" => figures_added += 1,
                "load_asset" => assets_loaded += 1,
                "add_node" => {
                    if let Some(node_type) = step.action.args.get("type").and_then(|v| v.as_str()) {
                        if node_type.contains("_light") {
                            lights_added += 1;
                        }
                    }
                },
                "apply_pose" => poses_applied += 1,
                "render_preview" | "render" => renders_done += 1,
                _ => {},
            }
        }

        if figures_added > 0 {
            explanation.push_str(&format!(
                "• Add {} figure{}\n",
                figures_added,
                if figures_added == 1 { "" } else { "s" }
            ));
        }
        if assets_loaded > 0 {
            explanation.push_str(&format!(
                "• Load {} asset{}\n",
                assets_loaded,
                if assets_loaded == 1 { "" } else { "s" }
            ));
        }
        if lights_added > 0 {
            explanation.push_str(&format!(
                "• Add {} light{}\n",
                lights_added,
                if lights_added == 1 { "" } else { "s" }
            ));
        }
        if poses_applied > 0 {
            explanation.push_str(&format!(
                "• Apply {} pose{}\n",
                poses_applied,
                if poses_applied == 1 { "" } else { "s" }
            ));
        }
        if renders_done > 0 {
            explanation.push_str(&format!(
                "• Generate {} render{}\n",
                renders_done,
                if renders_done == 1 { "" } else { "s" }
            ));
        }

        if explanation.ends_with(":\n") {
            explanation.push_str("• No significant changes to the scene\n");
        }

        explanation
    }
}

impl Default for Explainer {
    fn default() -> Self {
        Self::new()
    }
}
