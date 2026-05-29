use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "query_daz_knowledge",
        "Searches the DazPilot knowledge bases (concepts, commands, assets, scenes, workflows, failures) for information about any Daz Studio topic. Use this when you need Daz-specific knowledge the AI wasn't trained on.",
        ToolCategory::Knowledge,
        [
            tool_param("query", "Your question about Daz Studio, its features, or workflows", true, ToolParamType::String),
            tool_param("max_results", "Maximum number of knowledge results to return (default 5)", false, ToolParamType::Integer),
            tool_param("knowledge_base", "Optional: filter to specific knowledge base (daz_concepts, command_knowledge, asset_knowledge, scene_knowledge, workflow_knowledge, failure_knowledge)", false, ToolParamType::String),
        ],
        "Knowledge results with source, relevant content, and source module name",
        [
            "How do Genesis figure morphs work?",
            "What's the best way to set up three-point lighting?",
            "Explain how dForce physics works in Daz Studio",
        ],
        handle_query_daz_knowledge
    );
    define_tool!(
        "get_step_by_step_guide",
        "Given a task description, returns a step-by-step procedural guide using DazPilot's workflow knowledge. Covers: creating scenes, setting up characters, applying poses, lighting, rendering, animation, and fixing common issues.",
        ToolCategory::Knowledge,
        [
            tool_param("task", "What task do you need help with? (e.g., 'create a fantasy scene', 'set up a character', 'render an animation')", true, ToolParamType::String),
            tool_param("difficulty", "Skill level: beginner, intermediate, or advanced (default beginner)", false, ToolParamType::String),
        ],
        "Step-by-step guide with numbered instructions, Daz-specific tips, and tool suggestions per step",
        [
            "How do I create a character in Daz Studio?",
            "Step-by-step guide for rendering a still image",
            "Guide me through setting up animation",
        ],
        handle_get_step_by_step_guide
    );
    define_tool!(
        "explain_daz_element",
        "Explains what a Daz Studio concept, node, property, or term means in plain language. Handles Daz-specific terminology like 'geoshell', 'morph dial', 'dForce modifier', 'UV set', 'LIE', 'Iray Uber', etc.",
        ToolCategory::Knowledge,
        [
            tool_param("term", "The Daz Studio term, property name, or concept to explain", true, ToolParamType::String),
        ],
        "Explanation with definition, usage context, related terms, and practical tips",
        [
            "What is a geoshell in Daz Studio?",
            "Explain morph dials and how they work",
            "What does Iray Uber mean?",
        ],
        handle_explain_daz_element
    );
    define_tool!(
        "suggest_next_action",
        "Given the current scene state, recent conversation history, and available tools, suggests what the user might want to do next. This is the AI's proactive assistance — offering help before being asked.",
        ToolCategory::Knowledge,
        [
            tool_param("recent_actions", "Brief description of what the user just did or asked about", true, ToolParamType::String),
            tool_param("user_goal", "Optional known user goal to guide the suggestion", false, ToolParamType::String),
        ],
        "Next action suggestion with reasoning, tools to use, and expected outcome",
        [
            "I just loaded a figure, what next?",
            "I've set up lighting, what should I do now?",
        ],
        handle_suggest_next_action
    );
    define_tool!(
        "get_workflow_plan",
        "Given a high-level goal, generates a complete multi-step plan using DazPilot's workflow templates. This is the main tool for planning complex multi-step tasks.",
        ToolCategory::Knowledge,
        [
            tool_param("goal", "What do you want to achieve? (e.g., 'create a fantasy character render', 'animate a walk cycle', 'export for Unity')", true, ToolParamType::String),
        ],
        "Workflow plan with numbered steps, each with tool recommendations and expected outcomes",
        [
            "Plan out creating a full fantasy scene render",
            "I want to create an animated character for export",
        ],
        handle_get_workflow_plan
    );
    define_tool!(
        "explain_last_action",
        "Explains what the most recently executed tool action did, why it was done, what effect to expect in Daz Studio, and how to undo it if needed",
        ToolCategory::Knowledge,
        [
            tool_param("action_name", "Name of the action to explain", false, ToolParamType::String),
        ],
        "Action explanation with purpose, expected effect, and undo instructions",
        [
            "What did that last action do?",
            "Explain what applying the morph did",
        ],
        handle_explain_last_action
    );
    define_tool!(
        "suggest_tutorial",
        "Given the user's apparent skill level and goal, suggests a relevant tutorial or learning resource from the built-in tutorial library",
        ToolCategory::Knowledge,
        [
            tool_param("topic", "What topic do you want to learn about?", true, ToolParamType::String),
            tool_param("skill_level", "Current skill level: beginner, intermediate, or advanced (auto-detected if omitted)", false, ToolParamType::String),
        ],
        "Tutorial suggestion with title, topics covered, estimated time, and difficulty level",
        [
            "I'm new to Daz, where should I start?",
            "Teach me about advanced lighting techniques",
        ],
        handle_suggest_tutorial
    );
    define_tool!(
        "troubleshoot_issue",
        "Given a description of an error, bug, or unexpected behavior in Daz Studio, suggests likely causes and step-by-step fixes using the failure knowledge base",
        ToolCategory::Knowledge,
        [
            tool_param("issue_description", "Describe what went wrong or what unexpected behavior you're seeing", true, ToolParamType::String),
        ],
        "Troubleshooting results with likely causes, fix steps, and prevention tips",
        [
            "My asset won't load, it says 'conflict detected'",
            "The figure looks distorted after applying a pose",
            "dForce simulation isn't working on this dress",
        ],
        handle_troubleshoot_issue
    );
}
fn handle_query_daz_knowledge(request: ToolRequest) -> ToolResponse {
    let query = request.get_str("query").unwrap_or_default();
    let max_results = request.get_i64("max_results").unwrap_or(5) as usize;
    let kb_filter = request.get_str("knowledge_base");
    if query.is_empty() {
        return ToolResponse::err("query_daz_knowledge", "query is required");
    }
    let lower = query.to_lowercase();
    let mut results = Vec::new();
    // Search through knowledge bases
    // 1. Daz Concepts KB
    if kb_filter
        .as_deref()
        .map_or(true, |f| f == "daz_concepts" || f.is_empty())
    {
        let concepts = get_daz_concept_knowledge(&lower);
        for c in concepts.iter().take(max_results) {
            results.push(c.clone());
        }
    }
    // 2. Command Knowledge KB
    if kb_filter
        .as_deref()
        .map_or(true, |f| f == "command_knowledge" || f.is_empty())
    {
        let commands = get_command_knowledge(&lower);
        for c in commands.iter().take(max_results) {
            results.push(c.clone());
        }
    }
    // 3. Workflow Knowledge KB
    if kb_filter
        .as_deref()
        .map_or(true, |f| f == "workflow_knowledge" || f.is_empty())
    {
        let workflows = get_workflow_knowledge(&lower);
        for w in workflows.iter().take(max_results) {
            results.push(w.clone());
        }
    }
    results.truncate(max_results);
    if results.is_empty() {
        results.push(serde_json::json!({
            "source": "daz_concepts",
            "content": format!("I don't have specific knowledge about '{}' in my knowledge bases. Try rephrasing or asking about a more general Daz concept.", query),
            "relevance": 0.5,
        }));
    }
    ToolResponse::ok_with_message(
        "query_daz_knowledge",
        serde_json::json!({
            "query": query,
            "results": results,
            "total_results": results.len(),
        }),
        format!("Found {} knowledge results for '{}'", results.len(), query),
    )
}
fn handle_get_step_by_step_guide(request: ToolRequest) -> ToolResponse {
    let task = request.get_str("task").unwrap_or_default();
    let difficulty = request
        .get_str("difficulty")
        .unwrap_or_else(|| "beginner".to_string());
    if task.is_empty() {
        return ToolResponse::err("get_step_by_step_guide", "task is required");
    }
    let lower = task.to_lowercase();
    let steps = generate_workflow_steps(&lower, &difficulty);
    ToolResponse::ok_with_message(
        "get_step_by_step_guide",
        serde_json::json!({
            "task": task,
            "difficulty": difficulty,
            "steps": steps,
            "total_steps": steps.len(),
            "tip": "Each step can be executed using the available tools. Ask me to perform any step for you.",
        }),
        format!(
            "Generated {} step guide for '{}' (difficulty: {})",
            steps.len(),
            task,
            difficulty
        ),
    )
}
fn handle_explain_daz_element(request: ToolRequest) -> ToolResponse {
    let term = request.get_str("term").unwrap_or_default();
    if term.is_empty() {
        return ToolResponse::err("explain_daz_element", "term is required");
    }
    let lower = term.to_lowercase();
    let explanation = get_term_explanation(&lower);
    ToolResponse::ok_with_message(
        "explain_daz_element",
        explanation,
        format!("Explanation of '{}'", term),
    )
}
fn handle_suggest_next_action(request: ToolRequest) -> ToolResponse {
    let recent_actions = request.get_str("recent_actions").unwrap_or_default();
    let user_goal = request.get_str("user_goal");
    let lower = recent_actions.to_lowercase();
    let suggestions = generate_next_action_suggestions(&lower, user_goal.as_deref());
    ToolResponse::ok_with_message(
        "suggest_next_action",
        serde_json::json!({
            "context": recent_actions,
            "suggestion": suggestions.primary,
            "reason": suggestions.reason,
            "tool_to_use": suggestions.tool,
            "alternative_suggestions": suggestions.alternatives,
            "priority": suggestions.priority,
        }),
        format!("Suggestion: {}", suggestions.primary),
    )
}
fn handle_get_workflow_plan(request: ToolRequest) -> ToolResponse {
    let goal = request.get_str("goal").unwrap_or_default();
    if goal.is_empty() {
        return ToolResponse::err("get_workflow_plan", "goal is required");
    }
    let lower = goal.to_lowercase();
    let plan = generate_workflow_plan(&lower);
    ToolResponse::ok_with_message(
        "get_workflow_plan",
        serde_json::json!({
            "goal": goal,
            "plan_name": plan.name,
            "steps": plan.steps,
            "total_steps": plan.steps.len(),
            "estimated_complexity": plan.complexity,
            "requirements": plan.requirements,
        }),
        format!(
            "Created '{}' plan with {} steps",
            plan.name,
            plan.steps.len()
        ),
    )
}
fn handle_explain_last_action(request: ToolRequest) -> ToolResponse {
    let action_name = request
        .get_str("action_name")
        .unwrap_or_else(|| "last".to_string());
    let explanation: String = match action_name.as_str() {
        "load_asset" | "load" => "Loads a Daz asset file (figure, clothing, prop, etc.) into the scene from your content library. The asset will appear at the scene origin or wherever it was designed to load. Undo: use Edit > Undo or call clear_scene if nothing else was done.".to_string(),
        "apply_pose" => "Applies a pose file to a figure, setting all bone rotations and body morphs to match the pose preset. The figure will immediately assume the pose. Undo: apply a different pose or use the Pose Control > Zero Pose option.".to_string(),
        "set_morph" => "Adjusts a specific morph dial on a figure (e.g., Head_Height, Breast_Size, Waist_Width). Values range from 0.0 (neutral) to 1.0 (full). Undo: set the same morph back to 0.0.".to_string(),
        "set_light" => "Modifies a light's properties: intensity (brightness), color (RGB), or enable/disable. Changes are immediate in the viewport. Undo: set the previous values back.".to_string(),
        "add_figure" => "Adds a Genesis base figure to the scene. The figure appears at the origin in a default T-pose or A-pose. Undo: delete the figure node.".to_string(),
        "set_keyframe" => "Sets a keyframe on an animatable property at a specific timeline frame. This records the property's value at that point in time for animation playback. Undo: delete the keyframe.".to_string(),
        "render_preview" => "Triggers a preview render of the current viewport. This uses the current render settings and displays the result. Undo: simply close the render window.".to_string(),
        _ => {
            if action_name == "last" {
                "No recent action to explain, or action name not recognized. Try specifying an action name like 'load_asset', 'apply_pose', or 'set_morph'.".to_string()
            } else {
                format!("Action '{}' is not in the quick-reference list, but generally:\n- Read actions query scene data without changing anything\n- Write actions modify the scene and can be undone via Edit > Undo or using the begin_undo_batch/accept_undo_batch tools", action_name)
            }
        },
    };
    ToolResponse::ok_with_message(
        "explain_last_action",
        serde_json::json!({
            "action": action_name,
            "explanation": explanation,
            "undo_available": true,
        }),
        explanation,
    )
}
fn handle_suggest_tutorial(request: ToolRequest) -> ToolResponse {
    let topic = request.get_str("topic").unwrap_or_default();
    let skill_level = request
        .get_str("skill_level")
        .unwrap_or_else(|| "beginner".to_string());
    if topic.is_empty() {
        return ToolResponse::err("suggest_tutorial", "topic is required");
    }
    let lower = topic.to_lowercase();
    let tutorials = get_tutorial_suggestions(&lower, &skill_level);
    ToolResponse::ok_with_message(
        "suggest_tutorial",
        serde_json::json!({
            "topic": topic,
            "skill_level": skill_level,
            "suggestions": tutorials,
        }),
        format!("Found {} tutorial suggestions", tutorials.len()),
    )
}
fn handle_troubleshoot_issue(request: ToolRequest) -> ToolResponse {
    let issue = request.get_str("issue_description").unwrap_or_default();
    if issue.is_empty() {
        return ToolResponse::err("troubleshoot_issue", "issue_description is required");
    }
    let lower = issue.to_lowercase();
    let fixes = get_troubleshooting_fixes(&lower);
    ToolResponse::ok_with_message(
        "troubleshoot_issue",
        serde_json::json!({
            "issue": issue,
            "likely_causes": fixes,
            "auto_fix_available": false,
            "prevention_tip": "To avoid this in the future, always check asset compatibility before loading using check_asset_conflicts.",
        }),
        format!("Found {} possible causes and fixes", fixes.len()),
    )
}
// ─── Internal Knowledge Helpers ────────────────────────────────────────────
fn get_daz_concept_knowledge(query: &str) -> Vec<serde_json::Value> {
    let mut results = Vec::new();
    let entries: Vec<(&str, &str, &str, f64)> = vec![
        ("geoshell", "geoshell", "A Geometry Shell (geoshell) is a thin offset mesh that duplicates an existing surface. Used for fitting clothing tightly to a figure without full re-meshing. Geoshells can cause conflicts when multiple clothing items try to use the same shell zones.", 0.9),
        ("morph", "morph dial", "Morph dials are sliders that control specific shape deformations on a figure. Each morph targets a specific area (e.g., Head_Height, Waist_Width). Values range from 0.0 (neutral) to 1.0 (full), and can go negative or beyond 1.0 for extreme ranges. Daz figures have thousands of morph dials organized by body region and function.", 0.9),
        ("dforce", "dForce physics", "dForce is Daz Studio's built-in physics simulation engine for cloth, hair, and soft body dynamics. It simulates realistic movement based on gravity, collision, stiffness, damping, and mass parameters. Access the dForce controls in the Simulation Settings pane. Requires the Simulation → dForce menu to be enabled.", 0.85),
        ("iray", "Iray Uber", "Iray Uber is the default shader for Iray rendering in Daz Studio. It's a physically-based rendering (PBR) shader that simulates real-world light behavior. Key parameters include Base Color, Roughness, Metallic, Normal Map, Opacity, and Emission. Iray Uber surfaces respond to lighting just like real materials.", 0.85),
        ("uv", "UV set", "A UV set defines how a 2D texture map wraps around a 3D model. Daz figures can have multiple UV sets (e.g., for different clothing layers or body parts). UV conflicts occur when two assets try to use the same UV channels, causing texture stretching or misalignment.", 0.8),
        ("genesis", "Genesis figure", "Genesis is Daz's unified figure platform. Genesis 8 and 9 are the current major versions. Each generation has Male, Female, and Base variants. Genesis 9 introduced a new mesh topology and improved morph handling. Assets are NOT cross-compatible between Genesis 8 and 9.", 0.9),
        ("pose", "pose preset", "A pose preset (.duf file) stores the complete bone rotation and body morph data for a figure. Poses can be applied to any figure of the correct generation and gender. Poses range from casual standing to complex action poses and can be blended with morph sliders.", 0.85),
        ("light", "lighting preset", "Lighting in Daz Studio uses three main light types: Point (omnidirectional), Spot (cone-shaped with falloff), and Distant (parallel rays simulating sun). Professional setups typically use 3-point lighting: Key light (main), Fill light (shadows), Rim light (edge definition). HDRI environment maps can also provide lighting.", 0.85),
    ];
    for (keyword, name, content, relevance) in &entries {
        if query.contains(keyword) || query.contains(name) {
            results.push(serde_json::json!({
                "source": "daz_concepts",
                "term": name,
                "content": content,
                "relevance": relevance,
            }));
        }
    }
    results
}
fn get_command_knowledge(query: &str) -> Vec<serde_json::Value> {
    let mut results = Vec::new();
    let entries: Vec<(&str, &str, &str, f64)> = vec![
        ("load", "Loading Assets", "Use search_content to find assets in your library, then load_asset with the full path to load into the scene. For figures specifically, you can use add_figure with 'genesis9' or 'genesis8' to add base figures.", 0.85),
        ("render", "Rendering", "Use set_render_options for quality settings (pixel_samples: 16=draft to 4096=final), set_render_settings for output dimensions, then render_preview to render. For final quality, use at least 64 pixel samples for preview and 512+ for final.", 0.9),
        ("pose", "Applying Poses", "Use search_content(type='pose') to find pose files, then apply_pose with the pose_path and figure_id. You can also use set_bone_transform for individual bone adjustments.", 0.85),
        ("morph", "Using Morphs", "Use get_figure_morphs to see all available morph dials on a figure, then set_morph with the node_id, morph name, and value (0.0-1.0). For expressions, use apply_expression with expression names from get_active_expressions.", 0.85),
        ("light", "Setting Up Lights", "Use add_node(type='point_light'|'spot_light'|'distant_light') to create lights, then set_light to configure intensity, color, and enable/disable. Multiple lights create richer scenes.", 0.85),
        ("animation", "Animation", "Use set_timeline_range to set the frame range, set_keyframe to place keyframes on properties, then play_timeline to preview. For physics animation, use run_dforce_simulation after setting up dForce modifiers.", 0.8),
        ("export", "Exporting", "Use export_scene with node_id, path, and format settings. Common export formats: OBJ (universal), FBX (animation), Collada (compatibility). Set export options before calling.", 0.8),
    ];
    for (keyword, name, content, relevance) in &entries {
        if query.contains(keyword) {
            results.push(serde_json::json!({
                "source": "command_knowledge",
                "term": name,
                "content": content,
                "relevance": relevance,
            }));
        }
    }
    results
}
fn get_workflow_knowledge(query: &str) -> Vec<serde_json::Value> {
    let mut results = Vec::new();
    if query.contains("scene") || query.contains("create") || query.contains("setup") {
        results.push(serde_json::json!({
            "source": "workflow_knowledge",
            "workflow": "CreateScene",
            "content": "Scene creation workflow: (1) Load environment/background, (2) Add figure(s), (3) Apply character morphs, (4) Apply pose, (5) Add clothing/hair, (6) Set up lighting, (7) Position camera, (8) Adjust materials, (9) Render or export.",
            "relevance": 0.9,
        }));
    }
    if query.contains("character") || query.contains("figure") || query.contains("genesis") {
        results.push(serde_json::json!({
            "source": "workflow_knowledge",
            "workflow": "CreateCharacter",
            "content": "Character creation: (1) add_figure with desired base, (2) Apply body morphs for proportions, (3) Apply facial morphs for features, (4) Apply skin material, (5) Add hair, (6) Save as character preset for reuse.",
            "relevance": 0.85,
        }));
    }
    if query.contains("light") || query.contains("lighting") {
        results.push(serde_json::json!({
            "source": "workflow_knowledge",
            "workflow": "SetupLighting",
            "content": "3-point lighting setup: (1) Key light: spot_light at 45° above and side, intensity 2.0, warm color. (2) Fill light: point_light opposite side, intensity 1.0, cool color. (3) Rim light: spot_light from behind, intensity 1.5. Adjust all for desired mood.",
            "relevance": 0.9,
        }));
    }
    if query.contains("animate") || query.contains("animation") {
        results.push(serde_json::json!({
            "source": "workflow_knowledge",
            "workflow": "AnimateCharacter",
            "content": "Animation workflow: (1) Set timeline range, (2) Pose figure at start frame → set_keyframe, (3) Pose at next key position → set_keyframe, (4) Repeat for all key poses, (5) Add dForce physics for cloth/hair, (6) Play to preview, (7) Run physics simulation, (8) Render animation.",
            "relevance": 0.85,
        }));
    }
    results
}
fn generate_workflow_steps(task: &str, difficulty: &str) -> Vec<serde_json::Value> {
    let is_beginner = difficulty == "beginner";
    if task.contains("create") && task.contains("scene") {
        vec![
            serde_json::json!({"step": 1, "title": "Set up the environment", "details": "Load a background or environment asset. Use 'load_asset' with an HDRI or environment prop.", "tip": "Start with a simple background to establish the mood.", "tool": "load_asset"}),
            serde_json::json!({"step": 2, "title": "Add a figure", "details": "Use 'add_figure' to add a Genesis 8 or 9 base figure to the scene.", "tip": "Genesis 9 is the latest and recommended for new projects.", "tool": "add_figure"}),
            serde_json::json!({"step": 3, "title": if is_beginner { "Apply character morphs"} else {"Customize character morphs"}, "details": if is_beginner {"Use basic morph presets for body and face shapes."} else {"Use 'set_morph' to adjust individual morph dials for body proportions, face shape, and features."}, "tip": "Save your character as a preset once you're happy with the look.", "tool": "set_morph"}),
            serde_json::json!({"step": 4, "title": "Add clothing and hair", "details": "Use 'search_assets_by_description' to find compatible clothing, then 'load_asset' or 'add_figure' to apply them.", "tip": "Always check figure compatibility before loading assets.", "tool": "search_assets_by_description"}),
            serde_json::json!({"step": 5, "title": "Pose the character", "details": "Use 'search_poses_by_description' to find a pose, then 'apply_pose' or use 'set_bone_transform' for custom posing.", "tip": "Start with a pose preset and fine-tune with individual bone adjustments.", "tool": "apply_pose"}),
            serde_json::json!({"step": 6, "title": "Set up lighting", "details": "Create lights with 'add_node' and configure with 'set_light'. Start with a 3-point lighting setup.", "tip": "Use 'suggest_lighting_for_mood' to get lighting recommendations.", "tool": "add_node"}),
            serde_json::json!({"step": 7, "title": "Position the camera", "details": "Use 'set_camera' to choose and position the camera for the best view of your scene.", "tip": "Use 'suggest_camera_angle' for composition recommendations.", "tool": "set_camera"}),
            serde_json::json!({"step": 8, "title": "Render the scene", "details": "Set render options with 'set_render_options' (quality, resolution) then 'render_preview' to render.", "tip": "Use low quality (16 samples) for test renders, high quality (2000+ samples) for final.", "tool": "render_preview"}),
        ]
    } else if task.contains("animate") || task.contains("animation") {
        vec![
            serde_json::json!({"step": 1, "title": "Prepare the timeline", "details": "Use 'set_timeline_range' to set the frame range for your animation.", "tip": "A 30-second animation at 30fps = 900 frames.", "tool": "set_timeline_range"}),
            serde_json::json!({"step": 2, "title": "Set the starting pose", "details": "Pose your figure as desired, then use 'set_keyframe' to capture the pose at frame 0.", "tip": "Use 'analyze_figure_pose' to describe the starting pose.", "tool": "set_keyframe"}),
            serde_json::json!({"step": 3, "title": "Create key poses", "details": "Move to the next key frame, change the figure's pose, and set another keyframe. Repeat for all major pose changes.", "tip": "Use 'generate_pose_sequence' to automatically create key poses from a description.", "tool": "generate_pose_sequence"}),
            serde_json::json!({"step": 4, "title": "Add secondary motion", "details": "Use 'apply_secondary_motion_preset' to add dForce physics to hair and clothing for natural movement.", "tip": "Hair needs different physics settings than clothing.", "tool": "apply_secondary_motion_preset"}),
            serde_json::json!({"step": 5, "title": "Run physics simulation", "details": "Use 'run_dforce_simulation' to simulate the physics for the animation range.", "tip": "Bake to keyframes first if you want to edit the physics result.", "tool": "run_dforce_simulation"}),
            serde_json::json!({"step": 6, "title": "Preview and refine", "details": "Use 'play_timeline' to preview the animation. Adjust keyframes as needed.", "tip": "Use 'get_animation_analysis' to detect issues like foot sliding.", "tool": "play_timeline"}),
            serde_json::json!({"step": 7, "title": "Render the animation", "details": "Set render options and render each frame. Use 'queue_render_job' for batch rendering.", "tip": "Animation rendering takes much longer than stills — start with low quality for preview.", "tool": "render_preview"}),
        ]
    } else if task.contains("render") || task.contains("image") || task.contains("still") {
        vec![
            serde_json::json!({"step": 1, "title": "Finalize the scene", "details": "Ensure all assets are loaded, positioned, and posed correctly.", "tip": "Use 'get_comprehensive_scene_report' to review the full scene state.", "tool": "get_comprehensive_scene_report"}),
            serde_json::json!({"step": 2, "title": "Optimize lighting", "details": "Fine-tune lights for the desired mood. Use 'suggest_lighting_for_mood' for recommendations.", "tip": "Preview lighting with 'render_preview' at low quality.", "tool": "set_light"}),
            serde_json::json!({"step": 3, "title": "Set camera composition", "details": "Use 'set_camera' to frame the shot perfectly. Consider rule of thirds.", "tip": "Use 'analyze_scene_composition' to evaluate your framing.", "tool": "set_camera"}),
            serde_json::json!({"step": 4, "title": "Configure render settings", "details": "Set resolution, quality, and output options with 'set_render_options'.", "tip": "1920x1080 is standard HD. 3840x2160 for 4K. Use 512+ pixel samples for final quality.", "tool": "set_render_options"}),
            serde_json::json!({"step": 5, "title": "Render", "details": "Use 'render_preview' to render the final image.", "tip": "Close other applications during rendering for faster results.", "tool": "render_preview"}),
        ]
    } else if task.contains("fix")
        || task.contains("issue")
        || task.contains("problem")
        || task.contains("error")
        || task.contains("conflict")
    {
        vec![
            serde_json::json!({"step": 1, "title": "Diagnose the issue", "details": "Use 'troubleshoot_issue' to describe the problem and get likely causes.", "tip": "Be specific about what's happening vs what you expected.", "tool": "troubleshoot_issue"}),
            serde_json::json!({"step": 2, "title": "Check for conflicts", "details": "Use 'check_asset_conflicts' to scan for known conflict types.", "tip": "Common conflicts: geoshells, morph IDs, UV sets.", "tool": "check_asset_conflicts"}),
            serde_json::json!({"step": 3, "title": "Apply fixes", "details": "Use the recommended fix from the diagnosis. This may involve removing conflicting items, reloading assets, or adjusting properties.", "tip": "Always undo before retrying a failed operation.", "tool": "apply_pose"}),
            serde_json::json!({"step": 4, "title": "Verify the fix", "details": "Re-run 'troubleshoot_issue' or use 'get_comprehensive_scene_report' to confirm the issue is resolved.", "tip": "If the issue persists, try loading assets one at a time to isolate the problem.", "tool": "get_comprehensive_scene_report"}),
        ]
    } else {
        vec![
            serde_json::json!({"step": 1, "title": "Define your goal", "details": format!("You want to: {}. Let's break this down into manageable steps.", task), "tip": "Be as specific as possible about what you want to achieve.", "tool": "get_workflow_plan"}),
            serde_json::json!({"step": 2, "title": "Start with the basics", "details": "Load or create the fundamental elements of the scene first (figures, environment).", "tip": "Build up from simple to complex — start with one figure.", "tool": "add_figure"}),
            serde_json::json!({"step": 3, "title": "Refine and detail", "details": "Add details, adjust materials, fine-tune positioning.", "tip": "Small adjustments make big differences in the final result.", "tool": "set_morph"}),
            serde_json::json!({"step": 4, "title": "Finalize and output", "details": "Set up rendering or export settings and produce the final output.", "tip": "Save your work before rendering in case of crashes.", "tool": "render_preview"}),
        ]
    }
}
fn get_term_explanation(term: &str) -> serde_json::Value {
    let definition: &str;
    let usage: &str;
    let related: Vec<&str>;
    if term.contains("geoshell") || term.contains("shell") {
        definition = "A Geometry Shell (geoshell) is a thin offset mesh that duplicates the surface of another object. It's commonly used in Daz clothing to create a layer that follows the base figure's shape without needing a custom mesh.";
        usage = "Geoshells are often used for tight-fitting clothing like bodysuits, leggings, and gloves. They are generated automatically when you fit certain clothing types and can be adjusted using the 'Adjust Geoshell' controls.";
        related = vec!["morph", "UV set", "fitting"];
    } else if term.contains("morph") {
        definition = "Morph dials are sliders that control shape deformations on Daz figures. Each morph targets a specific part of the mesh (e.g., 'Head_Height' controls the height of the head). Values typically range from 0.0 to 1.0.";
        usage = "Use get_figure_morphs to list available morphs, then set_morph to apply values. Morphs can be combined for unlimited variation. HD (High Definition) morphs have finer detail.";
        related = vec!["figure", "expression", "pose"];
    } else if term.contains("iray") || term.contains("uber") || term.contains("shader") {
        definition = "Iray Uber is Daz Studio's primary physically-based shader for Iray rendering. It simulates real-world materials by controlling Base Color, Roughness, Metallic, Normal, Opacity, and other physical properties.";
        usage = "Materials using Iray Uber respond naturally to lighting. Set Base Color for the surface color, Roughness (0=mirror, 1=matte), Metallic (0=dielectric, 1=metal), and connect texture maps for detail.";
        related = vec!["render", "light", "texture"];
    } else if term.contains("dforce") || term.contains("physics") || term.contains("simulation") {
        definition = "dForce is Daz Studio's integrated physics simulation system for cloth, hair, and soft-body dynamics. It calculates realistic movement based on physical properties like gravity, wind, collision, stiffness, and mass.";
        usage = "Apply dForce modifiers to clothing or hair, set simulation parameters (stiffness, damping, mass), then run the simulation. The result can be baked to keyframes for editing or rendering.";
        related = vec!["hair", "clothing", "animation"];
    } else if term.contains("genesis") {
        definition = "Genesis is Daz's unified figure platform. Each generation (8, 9) provides base male and female figures with standardized morphs, rigging, and UV mapping. Genesis 9 is the current generation with improved mesh topology and morph handling.";
        usage = "Choose your figure generation based on asset compatibility. Genesis 8 has the largest library of compatible assets. Genesis 9 offers better quality and newer features but has a smaller compatible asset library.";
        related = vec!["morph", "figure", "UV set"];
    } else if term.contains("pose") {
        definition = "A pose preset stores the complete bone rotation and morph data for a figure's body position. Poses are saved as .duf files and can include partial poses (upper body only, hand poses, etc.).";
        usage = "Apply poses to your figure using the 'Pose' tab or via apply_pose in DazPilot. Poses can be blended and combined using morph sliders for custom results.";
        related = vec!["morph", "animation", "bone"];
    } else if term.contains("light") || term.contains("lighting") {
        definition = "Lighting in Daz Studio uses Point (omnidirectional), Spot (directional cone), and Distant (parallel rays) light types. Professional setups typically use multiple lights for optimal results.";
        usage = "A basic 3-point lighting setup consists of: Key light (main illumination), Fill light (reduces shadows), and Rim/Back light (separates subject from background). HDRI environments can also provide lighting.";
        related = vec!["render", "Iray", "camera"];
    } else if term.contains("uv") || term.contains("texture") || term.contains("map") {
        definition = "UV mapping defines how 2D textures wrap around 3D surfaces. Each vertex on the 3D mesh is assigned a UV coordinate that maps to a specific point on the texture image.";
        usage = "UV conflicts occur when multiple assets try to use overlapping UV space. Daz figures have standard UV sets that clothing and texture assets are designed to match. Use the UV Editor to inspect and adjust UV layouts.";
        related = vec!["material", "geoshell", "texture"];
    } else if term.contains("render") || term.contains("iray") || term.contains("render engine") {
        definition = "Iray is Daz Studio's primary photo-realistic render engine. It uses NVIDIA Iray technology for physically-based rendering, simulating how light interacts with materials to produce realistic images.";
        usage = "Set render quality (pixel samples), resolution, and output options before rendering. Higher pixel samples = less noise but longer render times. Iray supports NVIDIA GPU acceleration for faster rendering.";
        related = vec!["light", "material", "camera"];
    } else {
        definition = "I don't have a specific explanation for this term in my Daz knowledge base. Try query_daz_knowledge for a broader search, or rephrase your question.";
        usage = "If this is a technical Daz property, try looking it up in the Daz Studio documentation or use search_content to find related assets.";
        related = vec!["query_daz_knowledge", "get_step_by_step_guide"];
    }
    serde_json::json!({
        "term": term,
        "definition": definition,
        "usage_context": usage,
        "related_terms": related,
        "practical_tip": "Ask me to show you how to use this in your current scene.",
    })
}
struct SuggestionResult {
    primary: String,
    reason: String,
    tool: String,
    alternatives: Vec<String>,
    priority: String,
}
fn generate_next_action_suggestions(recent: &str, goal: Option<&str>) -> SuggestionResult {
    if let Some(g) = goal {
        let g_lower = g.to_lowercase();
        if g_lower.contains("render") || g_lower.contains("image") {
            return SuggestionResult {
                primary: "Set up for final rendering".to_string(),
                reason: "You mentioned rendering as your goal. Let me help you optimize the scene and configure render settings.".to_string(),
                tool: "set_render_options".to_string(),
                alternatives: vec![
                    "Optimize lighting with suggest_lighting_for_mood".to_string(),
                    "Frame the shot with suggest_camera_angle".to_string(),
                    "Run a test render first to check quality".to_string(),
                ],
                priority: "high".to_string(),
            };
        }
        if g_lower.contains("animate") || g_lower.contains("animation") {
            return SuggestionResult {
                primary: "Set up the animation timeline".to_string(),
                reason: "Your goal is animation. Let's start by configuring the timeline and planning your keyframes.".to_string(),
                tool: "set_timeline_range".to_string(),
                alternatives: vec![
                    "Generate a pose sequence with generate_pose_sequence".to_string(),
                    "Add secondary motion with apply_secondary_motion_preset".to_string(),
                    "Use suggest_motion_type for animation recommendations".to_string(),
                ],
                priority: "high".to_string(),
            };
        }
    }
    if recent.contains("load") || recent.contains("add figure") || recent.contains("genesis") {
        SuggestionResult {
            primary: "Customize the figure with morphs and materials".to_string(),
            reason: "Now that you have a figure in the scene, the next natural step is to customize its appearance. I can help with body morphs, skin materials, and finding compatible clothing.".to_string(),
            tool: "suggest_morphs_for_look".to_string(),
            alternatives: vec![
                "Search for compatible clothing with search_assets_by_description".to_string(),
                "Apply a pose with search_poses_by_description".to_string(),
                "Set up lighting with suggest_lighting_for_mood".to_string(),
            ],
            priority: "medium".to_string(),
        }
    } else if recent.contains("pose")
        || recent.contains("morph")
        || recent.contains("dress")
        || recent.contains("clothing")
    {
        SuggestionResult {
            primary: "Set up lighting to showcase the character".to_string(),
            reason: "Your character is coming together nicely. Good lighting will make all your customization work shine. I can suggest a lighting setup that complements the scene.".to_string(),
            tool: "suggest_lighting_for_mood".to_string(),
            alternatives: vec![
                "Add a background environment".to_string(),
                "Position the camera for the best view".to_string(),
                "Check for any remaining outfit gaps with recommend_outfit_completion".to_string(),
            ],
            priority: "medium".to_string(),
        }
    } else if recent.contains("light") || recent.contains("camera") {
        SuggestionResult {
            primary: "Do a test render to see how everything looks".to_string(),
            reason: "Lighting and camera are set up. A quick test render will show if you need to make any adjustments before the final output.".to_string(),
            tool: "render_preview".to_string(),
            alternatives: vec![
                "Fine-tune materials with suggest_material_improvements".to_string(),
                "Analyze scene composition with analyze_scene_composition".to_string(),
                "Add props to enrich the scene".to_string(),
            ],
            priority: "low".to_string(),
        }
    } else if recent.contains("render") || recent.contains("image") {
        SuggestionResult {
            primary: "Review and save your work".to_string(),
            reason: "You've done a render. If you're happy with the result, I recommend saving the scene and perhaps trying different camera angles or render settings.".to_string(),
            tool: "save_scene".to_string(),
            alternatives: vec![
                "Try different render settings for higher quality".to_string(),
                "Queue batch renders from multiple angles".to_string(),
                "Export the scene for use in other applications".to_string(),
            ],
            priority: "low".to_string(),
        }
    } else {
        SuggestionResult {
            primary: "Explore what's possible with the current scene".to_string(),
            reason: "I can help you analyze what's currently in your scene and suggest creative next steps.".to_string(),
            tool: "get_comprehensive_scene_report".to_string(),
            alternatives: vec![
                "Tell me what you want to create and I'll make a plan".to_string(),
                "Browse your asset library for inspiration".to_string(),
                "Try the 'Surprise me' feature for creative suggestions".to_string(),
            ],
            priority: "low".to_string(),
        }
    }
}
struct WorkflowPlanOutput {
    name: String,
    steps: Vec<serde_json::Value>,
    complexity: String,
    requirements: Vec<String>,
}
fn generate_workflow_plan(goal: &str) -> WorkflowPlanOutput {
    if goal.contains("fantasy") || goal.contains("character") && goal.contains("render") {
        WorkflowPlanOutput {
            name: "Fantasy Character Render".to_string(),
            steps: vec![
                serde_json::json!({"action": "Add figure", "params": {"figure_type": "genesis9"}, "expected_outcome": "Base figure in scene", "tool": "add_figure"}),
                serde_json::json!({"action": "Customize character", "params": {"goal": "fantasy elf character"}, "expected_outcome": "Character with fantasy morphs", "tool": "suggest_morphs_for_look"}),
                serde_json::json!({"action": "Find and load outfit", "params": {"style": "fantasy", "category": "clothing"}, "expected_outcome": "Character dressed in fantasy outfit", "tool": "search_assets_by_description"}),
                serde_json::json!({"action": "Find and load hair", "params": {"style": "fantasy", "category": "hair"}, "expected_outcome": "Character with fantasy hairstyle", "tool": "search_assets_by_description"}),
                serde_json::json!({"action": "Apply pose", "params": {"mood": "heroic", "scene_type": "full_body"}, "expected_outcome": "Character in heroic pose", "tool": "suggest_pose_for_scene"}),
                serde_json::json!({"action": "Set up lighting", "params": {"mood": "dramatic"}, "expected_outcome": "Dramatic 3-point lighting", "tool": "suggest_lighting_for_mood"}),
                serde_json::json!({"action": "Position camera", "params": {"scene_type": "portrait"}, "expected_outcome": "Well-framed camera angle", "tool": "suggest_camera_angle"}),
                serde_json::json!({"action": "Render final image", "params": {"quality": "high", "resolution": "3840x2160"}, "expected_outcome": "High-quality fantasy render", "tool": "set_render_options"}),
            ],
            complexity: "Medium-High".to_string(),
            requirements: vec![
                "Genesis 9 base figure".to_string(),
                "Fantasy clothing/hair assets in library".to_string(),
                "Ollama with vision model for recommendations".to_string(),
            ],
        }
    } else if goal.contains("animate") || goal.contains("walk") || goal.contains("cycle") {
        WorkflowPlanOutput {
            name: "Character Walk Cycle Animation".to_string(),
            steps: vec![
                serde_json::json!({"action": "Add figure", "params": {"figure_type": "genesis9"}, "expected_outcome": "Figure ready for animation", "tool": "add_figure"}),
                serde_json::json!({"action": "Set timeline range", "params": {"start_frame": 0, "end_frame": 180}, "expected_outcome": "6-second animation timeline", "tool": "set_timeline_range"}),
                serde_json::json!({"action": "Generate walk cycle", "params": {"steps": 6, "style": "natural"}, "expected_outcome": "Walk cycle pose sequence", "tool": "generate_walk_cycle"}),
                serde_json::json!({"action": "Apply pose sequence", "params": {"figure_id": "figure"}, "expected_outcome": "Keyframes on timeline", "tool": "apply_pose_sequence"}),
                serde_json::json!({"action": "Add secondary motion", "params": {"preset": "clothing_light"}, "expected_outcome": "Clothing physics applied", "tool": "apply_secondary_motion_preset"}),
                serde_json::json!({"action": "Preview and refine", "params": {}, "expected_outcome": "Animation previewed and adjusted", "tool": "play_timeline"}),
                serde_json::json!({"action": "Render animation", "params": {"quality": "draft"}, "expected_outcome": "Rendered animation frames", "tool": "render_preview"}),
            ],
            complexity: "Medium".to_string(),
            requirements: vec![
                "Genesis figure".to_string(),
                "Pose library available".to_string(),
                "dForce-compatible clothing".to_string(),
            ],
        }
    } else if goal.contains("export")
        || goal.contains("unity")
        || goal.contains("unreal")
        || goal.contains("game")
    {
        WorkflowPlanOutput {
            name: "Export for Game Engine".to_string(),
            steps: vec![
                serde_json::json!({"action": "Prepare scene", "params": {"goal": "export for game engine"}, "expected_outcome": "Optimized scene", "tool": "get_step_by_step_guide"}),
                serde_json::json!({"action": "Check export recommendations", "params": {"usage": "game_engine"}, "expected_outcome": "Export format guidance", "tool": "suggest_export_format"}),
                serde_json::json!({"action": "Optimize assets", "params": {"target": "unity"}, "expected_outcome": "Optimized meshes and textures", "tool": "prepare_for_game_engine"}),
                serde_json::json!({"action": "Export scene", "params": {"format": "fbx"}, "expected_outcome": "Exported FBX file", "tool": "export_scene"}),
            ],
            complexity: "Medium".to_string(),
            requirements: vec![
                "Complete scene".to_string(),
                "Export format knowledge".to_string(),
                "Target engine specifications".to_string(),
            ],
        }
    } else {
        WorkflowPlanOutput {
            name: "General Scene Workflow".to_string(),
            steps: vec![
                serde_json::json!({"action": "Analyze current scene", "params": {}, "expected_outcome": "Full scene understanding", "tool": "get_comprehensive_scene_report"}),
                serde_json::json!({"action": "Define specific goal", "params": {"goal": goal}, "expected_outcome": "Refined plan from AI", "tool": "get_step_by_step_guide"}),
                serde_json::json!({"action": "Execute primary step", "params": {}, "expected_outcome": "Progress toward goal", "tool": "suggest_next_action"}),
            ],
            complexity: "Variable".to_string(),
            requirements: vec!["Daz Studio with DazPilot bridge connected".to_string()],
        }
    }
}
fn get_tutorial_suggestions(topic: &str, _skill_level: &str) -> Vec<serde_json::Value> {
    let mut suggestions = Vec::new();
    if topic.contains("beginner") || topic.contains("start") || topic.contains("new") {
        suggestions.push(serde_json::json!({
            "title": "Getting Started with Daz Studio",
            "topics": ["Interface overview", "Loading assets", "Basic posing", "Simple rendering"],
            "estimated_time": "30 minutes",
            "difficulty": "Beginner",
        }));
        suggestions.push(serde_json::json!({
            "title": "Your First Character",
            "topics": ["Adding a figure", "Applying morphs", "Loading clothing", "Basic lighting"],
            "estimated_time": "45 minutes",
            "difficulty": "Beginner",
        }));
    }
    if topic.contains("light") || topic.contains("lighting") {
        suggestions.push(serde_json::json!({
            "title": "Mastering 3-Point Lighting",
            "topics": ["Key light placement", "Fill light balance", "Rim light effects", "HDRI environments"],
            "estimated_time": "45 minutes",
            "difficulty": "Intermediate",
        }));
        suggestions.push(serde_json::json!({
            "title": "Advanced Lighting for Mood",
            "topics": ["Color temperature", "Light intensity ratios", "Shadow control", "Multiple light setups"],
            "estimated_time": "60 minutes",
            "difficulty": "Advanced",
        }));
    }
    if topic.contains("animate") || topic.contains("animation") || topic.contains("motion") {
        suggestions.push(serde_json::json!({
            "title": "Animation Basics: Keyframes and Timeline",
            "topics": ["Timeline setup", "Keyframe creation", "Pose-to-pose animation", "Playback controls"],
            "estimated_time": "40 minutes",
            "difficulty": "Intermediate",
        }));
        suggestions.push(serde_json::json!({
            "title": "Walk Cycles and Character Movement",
            "topics": ["Walk cycle mechanics", "Foot placement", "Arm swing", "Timing and weight"],
            "estimated_time": "60 minutes",
            "difficulty": "Advanced",
        }));
    }
    if topic.contains("render") || topic.contains("iray") || topic.contains("quality") {
        suggestions.push(serde_json::json!({
            "title": "Iray Rendering Fundamentals",
            "topics": ["Render settings explained", "Quality vs speed tradeoffs", "GPU acceleration", "Output formats"],
            "estimated_time": "35 minutes",
            "difficulty": "Intermediate",
        }));
        suggestions.push(serde_json::json!({
            "title": "Photorealistic Rendering Techniques",
            "topics": ["Lighting for realism", "Material optimization", "Texture resolution", "Post-processing"],
            "estimated_time": "55 minutes",
            "difficulty": "Advanced",
        }));
    }
    if topic.contains("material") || topic.contains("texture") || topic.contains("shader") {
        suggestions.push(serde_json::json!({
            "title": "Iray Uber Shader Deep Dive",
            "topics": ["Base Color", "Roughness/Metallic", "Normal maps", "Opacity and transparency"],
            "estimated_time": "45 minutes",
            "difficulty": "Intermediate",
        }));
    }
    if suggestions.is_empty() {
        suggestions.push(serde_json::json!({
            "title": "DazPilot Built-in Tutorials",
            "topics": ["Using AI tools", "Scene composition", "Asset management", "Workflow automation"],
            "estimated_time": "20 minutes",
            "difficulty": "All levels",
        }));
    }
    suggestions
}
fn get_troubleshooting_fixes(issue: &str) -> Vec<serde_json::Value> {
    let mut causes = Vec::new();
    if issue.contains("load") || issue.contains("conflict") || issue.contains("fit") {
        causes.push(serde_json::json!({
            "cause": "Asset-figure compatibility mismatch",
            "probability": "High",
            "fix": "Use check_asset_conflicts to identify the specific conflict type. Common fixes: use a different figure generation, or manually resolve geoshell/morph conflicts using the conflict resolution tools.",
            "steps": [
                "Run check_asset_conflicts on the asset",
                "Check if the figure generation matches (Genesis 8 vs 9)",
                "Try a different version of the asset if available",
            ],
        }));
        causes.push(serde_json::json!({
            "cause": "Missing content path or library not scanned",
            "probability": "Medium",
            "fix": "Ensure your Daz content library paths are configured correctly and have been scanned.",
            "steps": [
                "Check content paths in DazPilot settings",
                "Run scan_library to refresh the asset index",
                "Verify the asset exists at the expected path",
            ],
        }));
    }
    if issue.contains("pose")
        && (issue.contains("distort") || issue.contains("weird") || issue.contains("wrong"))
    {
        causes.push(serde_json::json!({
            "cause": "Pose applied to wrong figure generation or gender",
            "probability": "High",
            "fix": "Poses are generation and gender-specific. A Genesis 8 Female pose won't work correctly on a Genesis 9 or Male figure.",
            "steps": [
                "Verify the figure type matches the pose source",
                "Search for poses specifically for your figure type",
                "Use set_bone_transform for manual adjustments if needed",
            ],
        }));
        causes.push(serde_json::json!({
            "cause": "Morph dials from pose conflicting with existing morphs",
            "probability": "Medium",
            "fix": "Some poses include morph values that may conflict with your character's custom morphs.",
            "steps": [
                "Zero the figure's morphs before applying the pose",
                "Apply the pose gradually using morph blending",
                "Use selective pose application (upper body only)",
            ],
        }));
    }
    if issue.contains("dforce") || issue.contains("physics") || issue.contains("simulation") {
        causes.push(serde_json::json!({
            "cause": "dForce modifier not applied to the clothing/hair node",
            "probability": "High",
            "fix": "You need to apply a dForce modifier to each item you want to simulate. Use apply_secondary_motion_preset to add the right physics modifier.",
            "steps": [
                "Select the clothing or hair node",
                "Use apply_secondary_motion_preset with the appropriate preset",
                "Adjust stiffness, damping, and mass parameters",
                "Run the simulation again",
            ],
        }));
        causes.push(serde_json::json!({
            "cause": "Figure not in the correct pose before simulation",
            "probability": "Medium",
            "fix": "dForce simulations should be run after the figure is in its final pose, not before.",
            "steps": [
                "Apply the final pose to your figure first",
                "Zero out any simulation frames before the pose",
                "Run the simulation from the keyframe after the pose change",
            ],
        }));
    }
    if issue.contains("render")
        && (issue.contains("slow") || issue.contains("long") || issue.contains("time"))
    {
        causes.push(serde_json::json!({
            "cause": "Render quality settings too high for preview",
            "probability": "High",
            "fix": "Use lower pixel samples (16-64) for test renders, and save high quality (512-2000) for final renders.",
            "steps": [
                "Set pixel_samples to 64 for quick previews",
                "Disable features like caustics for test renders",
                "Use a lower resolution for preview (1280x720)",
            ],
        }));
    }
    if causes.is_empty() {
        causes.push(serde_json::json!({
            "cause": "Unknown issue pattern",
            "probability": "Low",
            "fix": "I don't have specific knowledge about this issue. Try using get_comprehensive_scene_report to check the current scene state, or consult Daz Studio's documentation.",
            "steps": [
                "Run get_comprehensive_scene_report",
                "Check Daz Studio's error log",
                "Try isolating the issue by undoing recent changes",
            ],
        }));
    }
    causes
}
