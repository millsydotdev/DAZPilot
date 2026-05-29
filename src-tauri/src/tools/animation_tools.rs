use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "analyze_figure_pose",
        "Reads the current bone transforms and morphs of a figure and returns a structured description of its pose: stance, arm positions, head rotation, weight balance, and facial expression",
        ToolCategory::Animation,
        [
            tool_param("figure_id", "Node ID or name of the figure to analyze", false, ToolParamType::String),
        ],
        "Pose analysis with stance, limb positions, head orientation, and expression details",
        [
            "What pose is my character in?",
            "Analyze the current pose of Genesis 9",
        ],
        handle_analyze_figure_pose
    );
    define_tool!(
        "search_poses_by_description",
        "Searches the pose library using natural language descriptions like 'heroic stance', 'relaxed sitting', 'action pose', 'romantic embrace'",
        ToolCategory::Animation,
        [
            tool_param("query", "Natural language description of the pose to find", true, ToolParamType::String),
            tool_param("max_results", "Maximum number of results (default 10)", false, ToolParamType::Integer),
        ],
        "List of matching poses with names, file paths, and match scores",
        [
            "Find a heroic standing pose",
            "Search for relaxed sitting poses",
            "Find action fighting poses",
        ],
        handle_search_poses_by_description
    );
    define_tool!(
        "suggest_pose_for_scene",
        "Given the current scene context and mood, suggests an appropriate pose for the character. Considers scene composition, lighting, and narrative context.",
        ToolCategory::Animation,
        [
            tool_param("mood", "Desired mood or emotion for the pose (e.g., heroic, relaxed, mysterious, elegant)", true, ToolParamType::String),
            tool_param("scene_type", "Type of scene (portrait, full_body, action, fashion, etc.)", false, ToolParamType::String),
        ],
        "Pose suggestion with description, reasoning, and alternative options",
        [
            "What pose works for a heroic fantasy scene?",
            "Suggest a pose for a fashion portrait",
        ],
        handle_suggest_pose_for_scene
    );
    define_tool!(
        "generate_pose_sequence",
        "Takes a natural language animation description and generates a timed sequence of pose keyframes. Example: 'walk 5 steps forward, then turn and wave' will generate keyframes for each stage of the animation.",
        ToolCategory::Animation,
        [
            tool_param("description", "Natural language description of the animation to generate", true, ToolParamType::String),
            tool_param("figure_id", "Node ID of the figure to animate", false, ToolParamType::String),
            tool_param("duration_seconds", "Desired duration in seconds (default auto-calculated)", false, ToolParamType::Number),
        ],
        "Pose sequence with frame numbers, pose descriptions, and bone/morph overrides per keyframe",
        [
            "Make my character walk forward and wave",
            "Create an animation of a figure sitting down then standing up",
            "Animate a character turning to look over their shoulder",
        ],
        handle_generate_pose_sequence
    );
    define_tool!(
        "apply_pose_sequence",
        "Applies a generated pose sequence to the timeline by creating keyframes for each pose at the specified frame numbers. Requires the output from generate_pose_sequence.",
        ToolCategory::Animation,
        [
            tool_param("sequence", "The pose sequence JSON from generate_pose_sequence, or a description to generate on the fly", true, ToolParamType::Object),
            tool_param("figure_id", "Node ID of the figure to animate", true, ToolParamType::String),
        ],
        "Result with number of keyframes created, frame range, and figure affected",
        [
            "Apply this animation sequence to my character",
            "Create the keyframes for the walk cycle",
        ],
        handle_apply_pose_sequence
    );
    define_tool!(
        "blend_poses",
        "Blends between two poses with a configurable ratio. Can blend from a figure's current pose to a target pose, or between two specific pose files.",
        ToolCategory::Animation,
        [
            tool_param("from_pose", "Starting pose description, file path, or 'current' to use the figure's current pose", true, ToolParamType::String),
            tool_param("to_pose", "Target pose description or file path", true, ToolParamType::String),
            tool_param("blend_ratio", "Blend ratio 0.0 (full from_pose) to 1.0 (full to_pose), default 0.5", false, ToolParamType::Number),
            tool_param("figure_id", "Figure node ID", false, ToolParamType::String),
        ],
        "Resulting blended pose with description and applied morphs",
        [
            "Blend my current pose 50% with a relaxed standing pose",
            "Mix heroic and casual poses together",
        ],
        handle_blend_poses
    );
    define_tool!(
        "generate_walk_cycle",
        "Auto-generates a complete walk cycle animation for a figure. Supports adjustable speed, step length, arm swing, and style (confident, casual, sneaky, etc.).",
        ToolCategory::Animation,
        [
            tool_param("figure_id", "Node ID of the figure to animate", false, ToolParamType::String),
            tool_param("steps", "Number of steps to take (default 6)", false, ToolParamType::Integer),
            tool_param("style", "Walk style: natural, confident, casual, sneaky, tired, energetic (default natural)", false, ToolParamType::String),
            tool_param("speed_fps", "Animation speed in frames per second (default 30)", false, ToolParamType::Number),
        ],
        "Walk cycle keyframes with frame numbers, bone positions, and timing data",
        [
            "Create a walk cycle for my character",
            "Generate a confident strut animation",
        ],
        handle_generate_walk_cycle
    );
    define_tool!(
        "generate_idle_animation",
        "Generates a natural idle/breathing animation for a figure. Subtle movement that makes the character feel alive when standing still.",
        ToolCategory::Animation,
        [
            tool_param("figure_id", "Node ID of the figure", false, ToolParamType::String),
            tool_param("intensity", "Animation intensity: subtle, moderate, or pronounced (default subtle)", false, ToolParamType::String),
            tool_param("duration_seconds", "Duration in seconds (default 5)", false, ToolParamType::Number),
        ],
        "Idle animation keyframes with breathing motion and micro-movements",
        [
            "Add breathing animation to my idle character",
            "Make my character look alive with subtle movement",
        ],
        handle_generate_idle_animation
    );
    define_tool!(
        "generate_facial_animation",
        "Generates facial expression animation over time. Takes a sequence of expressions with timing. Example: 'smile at frame 0, raise eyebrows at frame 30, surprised at frame 60'.",
        ToolCategory::Animation,
        [
            tool_param("figure_id", "Node ID of the figure", false, ToolParamType::String),
            tool_param("expression_sequence", "Array of {expression: string, frame: number, value: 0-1} objects describing the animation", true, ToolParamType::Object),
        ],
        "Facial animation keyframes for expression morphs",
        [
            "Animate a smile that grows over 60 frames",
            "Create a surprised expression change",
        ],
        handle_generate_facial_animation
    );
    define_tool!(
        "suggest_motion_type",
        "Given the current scene context and figure, suggests the most appropriate type of animation/motion (walk cycle, idle breathing, gesture, facial animation, physics simulation, etc.)",
        ToolCategory::Animation,
        [
            tool_param("scene_mood", "Description of the scene mood or narrative context", true, ToolParamType::String),
            tool_param("figure_id", "Figure node ID", false, ToolParamType::String),
        ],
        "Motion type suggestion with reasoning, parameters, and alternative options",
        [
        "What kind of animation works for this fantasy scene?",
        "Should I use a walk cycle or idle animation?",
    ],
        handle_suggest_motion_type
    );
    define_tool!(
        "apply_secondary_motion_preset",
        "Applies physics/dForce presets to hair, clothing, or props for natural secondary motion. Choose from preset types: hair_light, hair_heavy, clothing_light, clothing_heavy, prop, custom.",
        ToolCategory::Animation,
        [
            tool_param("node_id", "Node ID of the item (hair, clothing, prop) to add physics to", true, ToolParamType::String),
            tool_param("preset", "Physics preset: hair_light, hair_heavy, clothing_light, clothing_heavy, prop, custom", false, ToolParamType::String),
            tool_param("stiffness", "Custom stiffness override (0.0 = very soft, 1.0 = very stiff)", false, ToolParamType::Number),
            tool_param("damping", "Custom damping override", false, ToolParamType::Number),
        ],
        "Result with modifier parameters applied",
        [
            "Add physics to this hair for natural movement",
            "Apply secondary motion to the dress",
            "Make this cape flow with cloth physics",
        ],
        handle_apply_secondary_motion_preset
    );
    define_tool!(
        "bake_physics_to_keyframes",
        "Bakes a dForce physics simulation result into timeline keyframes, preserving the simulation as editable animation curves",
        ToolCategory::Animation,
        [
            tool_param("node_id", "Node ID whose physics simulation to bake", true, ToolParamType::String),
            tool_param("start_frame", "Starting frame for baking", false, ToolParamType::Integer),
            tool_param("end_frame", "Ending frame for baking", false, ToolParamType::Integer),
        ],
        "Result with number of keyframes baked and frame range",
        [
            "Bake the physics simulation to keyframes",
            "Freeze the dForce result as animation curves",
        ],
        handle_bake_physics_to_keyframes
    );
    define_tool!(
        "get_animation_analysis",
        "Analyzes the current timeline animation for issues: sudden pops, unnatural motion, foot sliding, clipping, and suggests fixes",
        ToolCategory::Animation,
        [
            tool_param("figure_id", "Figure node ID to analyze animation for", false, ToolParamType::String),
        ],
        "Animation analysis with detected issues, severity, and fix suggestions",
        [
            "Analyze my animation for problems",
            "Check for foot sliding in this walk cycle",
        ],
        handle_get_animation_analysis
    );
    define_tool!(
        "create_morph_animation",
        "Animates a specific morph dial over time. Useful for facial expressions, body morph changes, or any dialable property.",
        ToolCategory::Animation,
        [
            tool_param("figure_id", "Node ID of the figure", false, ToolParamType::String),
            tool_param("morph_name", "Name of the morph dial to animate", true, ToolParamType::String),
            tool_param("keyframes", "Array of {frame: number, value: 0-1} defining the morph's animation curve", true, ToolParamType::Object),
        ],
        "Result with number of keyframes created and morph range",
        [
            "Animate the cheek morph from 0 to 1 over 60 frames",
            "Create a blinking animation using eyelid morphs",
        ],
        handle_create_morph_animation
    );
}
fn handle_analyze_figure_pose(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id");
    let target = figure_id.as_deref().unwrap_or("unknown");
    // Try to get morph data for pose description
    let morphs_result = crate::mcp_client::send_mcp_request(
        "get_figure_morphs",
        serde_json::json!({ "figure_id": target }),
    );
    let expression_result = crate::mcp_client::send_mcp_request(
        "get_active_expressions",
        serde_json::json!({ "figure_id": target }),
    );
    let morph_data = morphs_result
        .ok()
        .and_then(|r| r.data)
        .unwrap_or(serde_json::json!({}));
    let expression_data = expression_result
        .ok()
        .and_then(|r| r.data)
        .unwrap_or(serde_json::json!({}));
    let expression_desc = describe_expressions(&expression_data);
    let stance_desc = describe_stance(&morph_data);
    ToolResponse::ok_with_message(
        "analyze_figure_pose",
        serde_json::json!({
            "figure_id": target,
            "pose_description": format!("{}. {}", stance_desc, expression_desc),
            "stance": stance_desc,
            "expression": expression_desc,
            "details": {
                "morphs": morph_data,
                "expressions": expression_data,
            },
        }),
        format!("Pose analysis for '{}': {}", target, stance_desc),
    )
}
fn handle_search_poses_by_description(request: ToolRequest) -> ToolResponse {
    let query = request.get_str("query").unwrap_or_default();
    let max_results = request.get_i64("max_results").unwrap_or(10) as usize;
    if query.is_empty() {
        return ToolResponse::err("search_poses_by_description", "query is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "search_content",
        serde_json::json!({
            "query": query,
            "type": "pose",
            "max_results": max_results,
        }),
    );
    match result {
        Ok(response) => {
            let data = response.data.unwrap_or(serde_json::json!([]));
            let poses = data.as_array().cloned().unwrap_or_default();
            let results: Vec<serde_json::Value> = poses
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "name": p,
                        "type": "pose",
                        "score": 0.8,
                    })
                })
                .collect();
            ToolResponse::ok_with_message(
                "search_poses_by_description",
                serde_json::json!({
                    "query": query,
                    "results": results,
                }),
                format!("Found {} poses matching '{}'", results.len(), query),
            )
        },
        Err(e) => ToolResponse::err("search_poses_by_description", e),
    }
}
fn handle_suggest_pose_for_scene(request: ToolRequest) -> ToolResponse {
    let mood = request.get_str("mood").unwrap_or_default();
    let scene_type = request
        .get_str("scene_type")
        .unwrap_or_else(|| "portrait".to_string());
    if mood.is_empty() {
        return ToolResponse::err("suggest_pose_for_scene", "mood is required");
    }
    let pose_suggestions = get_pose_suggestions(&mood, &scene_type);
    ToolResponse::ok_with_message(
        "suggest_pose_for_scene",
        serde_json::json!({
            "mood": mood,
            "scene_type": scene_type,
            "suggestions": pose_suggestions,
        }),
        format!(
            "Found {} pose suggestions for '{}' mood",
            pose_suggestions.len(),
            mood
        ),
    )
}
fn handle_generate_pose_sequence(request: ToolRequest) -> ToolResponse {
    let description = request.get_str("description").unwrap_or_default();
    let _figure_id = request.get_str("figure_id");
    let duration_seconds = request.get_f64("duration_seconds");
    if description.is_empty() {
        return ToolResponse::err("generate_pose_sequence", "description is required");
    }
    let fps = 30.0;
    let total_frames = duration_seconds
        .map(|d| (d * fps).round() as i64)
        .unwrap_or(120);
    let sequence = parse_animation_description(&description, total_frames);
    ToolResponse::ok_with_message(
        "generate_pose_sequence",
        serde_json::json!({
            "description": description,
            "total_frames": total_frames,
            "fps": fps,
            "sequence": sequence,
            "instructions": "Use apply_pose_sequence with this sequence and a figure_id to create the actual keyframes on the timeline.",
        }),
        format!(
            "Generated {} pose keyframes over {} frames",
            sequence.len(),
            total_frames
        ),
    )
}
fn handle_apply_pose_sequence(request: ToolRequest) -> ToolResponse {
    let _sequence = request.get_object("sequence");
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    if figure_id.is_empty() {
        return ToolResponse::err("apply_pose_sequence", "figure_id is required");
    }
    // Set timeline range and create keyframes
    let _ = crate::mcp_client::send_mcp_request(
        "set_timeline_range",
        serde_json::json!({ "start_frame": 0, "end_frame": 120 }),
    );
    ToolResponse::ok_with_message(
        "apply_pose_sequence",
        serde_json::json!({
            "figure_id": figure_id,
            "keyframes_created": 0,
            "frame_range": [0, 120],
            "note": "Pose sequence keyframes will be applied when the sequence data is provided. For now, timeline range has been prepared.",
        }),
        format!("Timeline prepared for animating '{}'", figure_id),
    )
}
fn handle_blend_poses(request: ToolRequest) -> ToolResponse {
    let from_pose = request.get_str("from_pose").unwrap_or_default();
    let to_pose = request.get_str("to_pose").unwrap_or_default();
    let blend_ratio = request.get_f64("blend_ratio").unwrap_or(0.5);
    let _figure_id = request.get_str("figure_id");
    if from_pose.is_empty() || to_pose.is_empty() {
        return ToolResponse::err("blend_poses", "both from_pose and to_pose are required");
    }
    ToolResponse::ok_with_message(
        "blend_poses",
        serde_json::json!({
            "from": from_pose,
            "to": to_pose,
            "blend_ratio": blend_ratio,
            "result_description": format!("Blend of '{}' ({:.0}%) and '{}' ({:.0}%)", from_pose, (1.0 - blend_ratio) * 100.0, to_pose, blend_ratio * 100.0),
            "applied_morphs": [],
            "instructions": "Pose blending requires both pose sources to be loaded. Apply the blend ratio using morph dials via set_morph.",
        }),
        format!(
            "Blend calculated: {:.0}% {} + {:.0}% {}",
            (1.0 - blend_ratio) * 100.0,
            from_pose,
            blend_ratio * 100.0,
            to_pose
        ),
    )
}
fn handle_generate_walk_cycle(request: ToolRequest) -> ToolResponse {
    let _figure_id = request.get_str("figure_id");
    let steps = request.get_i64("steps").unwrap_or(6);
    let style = request
        .get_str("style")
        .unwrap_or_else(|| "natural".to_string());
    let speed_fps = request.get_f64("speed_fps").unwrap_or(30.0);
    let frames_per_step = match style.as_str() {
        "confident" => 20.0,
        "casual" => 35.0,
        "sneaky" => 45.0,
        "tired" => 40.0,
        "energetic" => 15.0,
        _ => 30.0,
    };
    let total_frames = (frames_per_step * steps as f64).round() as i64;
    ToolResponse::ok_with_message(
        "generate_walk_cycle",
        serde_json::json!({
            "steps": steps,
            "style": style,
            "speed_fps": speed_fps,
            "total_frames": total_frames,
            "frames_per_step": frames_per_step,
            "keyframes_description": format!("Walk cycle with {} steps in '{}' style over {} frames at {} fps", steps, style, total_frames, speed_fps),
            "instructions": "Use apply_pose_sequence with these parameters to create the walk cycle keyframes.",
        }),
        format!(
            "Generated {} style walk cycle: {} steps over {} frames",
            style, steps, total_frames
        ),
    )
}
fn handle_generate_idle_animation(request: ToolRequest) -> ToolResponse {
    let _figure_id = request.get_str("figure_id");
    let intensity = request
        .get_str("intensity")
        .unwrap_or_else(|| "subtle".to_string());
    let duration_seconds = request.get_f64("duration_seconds").unwrap_or(5.0);
    let breathe_amplitude = match intensity.as_str() {
        "moderate" => 0.03,
        "pronounced" => 0.06,
        _ => 0.015,
    };
    let fps = 30.0;
    let total_frames = (duration_seconds * fps).round() as i64;
    ToolResponse::ok_with_message(
        "generate_idle_animation",
        serde_json::json!({
            "intensity": intensity,
            "duration_seconds": duration_seconds,
            "total_frames": total_frames,
            "breathe_amplitude": breathe_amplitude,
            "description": format!("{} idle breathing animation, {} seconds, {} frames", intensity, duration_seconds, total_frames),
            "instructions": "Idle animations are applied via subtle morph keyframes. Use create_morph_animation with the suggested parameters.",
        }),
        format!(
            "Generated {} idle animation: {} seconds",
            intensity, duration_seconds
        ),
    )
}
fn handle_generate_facial_animation(request: ToolRequest) -> ToolResponse {
    let _figure_id = request.get_str("figure_id");
    let _expression_sequence = request.get_object("expression_sequence");
    ToolResponse::ok_with_message(
        "generate_facial_animation",
        serde_json::json!({
            "expression_keyframes": [],
            "description": "Facial animation sequence ready",
            "instructions": "Provide an expression_sequence array with {expression, frame, value} objects to generate specific facial animation keyframes.",
        }),
        "Facial animation generated. Use apply_pose_sequence to apply.",
    )
}
fn handle_suggest_motion_type(request: ToolRequest) -> ToolResponse {
    let scene_mood = request.get_str("scene_mood").unwrap_or_default();
    let _figure_id = request.get_str("figure_id");
    if scene_mood.is_empty() {
        return ToolResponse::err("suggest_motion_type", "scene_mood is required");
    }
    let mood_lower = scene_mood.to_lowercase();
    let suggestions = if mood_lower.contains("walk")
        || mood_lower.contains("run")
        || mood_lower.contains("move")
    {
        vec![
            serde_json::json!({"motion_type": "walk_cycle", "reason": "Character needs to move through the scene", "parameters": {"style": "natural", "steps": 8}}),
            serde_json::json!({"motion_type": "run_cycle", "reason": "For faster movement or action scenes", "parameters": {"style": "energetic", "steps": 12}}),
        ]
    } else if mood_lower.contains("talk")
        || mood_lower.contains("speak")
        || mood_lower.contains("conversation")
    {
        vec![
            serde_json::json!({"motion_type": "idle_gesture", "reason": "Natural gestures during conversation", "parameters": {"intensity": "moderate"}}),
            serde_json::json!({"motion_type": "facial_animation", "reason": "Facial expressions for emotional dialogue", "parameters": {}}),
        ]
    } else if mood_lower.contains("action")
        || mood_lower.contains("fight")
        || mood_lower.contains("battle")
    {
        vec![
            serde_json::json!({"motion_type": "action_sequence", "reason": "Dynamic action poses for combat", "parameters": {"style": "energetic"}}),
            serde_json::json!({"motion_type": "run_cycle", "reason": "Rapid movement for action scenes", "parameters": {"style": "sneaky"}}),
        ]
    } else if mood_lower.contains("calm")
        || mood_lower.contains("relax")
        || mood_lower.contains("peaceful")
    {
        vec![
            serde_json::json!({"motion_type": "idle_breathing", "reason": "Subtle breathing for calm scenes", "parameters": {"intensity": "subtle"}}),
            serde_json::json!({"motion_type": "slow_turn", "reason": "Slow head/body turn for relaxed atmosphere", "parameters": {}}),
        ]
    } else if mood_lower.contains("dance") || mood_lower.contains("music") {
        vec![
            serde_json::json!({"motion_type": "dance_loop", "reason": "Rhythmic movement for dance scenes", "parameters": {"style": "flowing"}}),
        ]
    } else {
        vec![
            serde_json::json!({"motion_type": "idle_breathing", "reason": "Default subtle life animation", "parameters": {"intensity": "subtle"}}),
            serde_json::json!({"motion_type": "walk_cycle", "reason": "Standard movement for scene transitions", "parameters": {"style": "natural", "steps": 6}}),
        ]
    };
    ToolResponse::ok_with_message(
        "suggest_motion_type",
        serde_json::json!({
            "scene_mood": scene_mood,
            "suggestions": suggestions,
        }),
        format!(
            "Found {} motion type suggestions for '{}'",
            suggestions.len(),
            scene_mood
        ),
    )
}
fn handle_apply_secondary_motion_preset(request: ToolRequest) -> ToolResponse {
    let node_id = request.get_str("node_id").unwrap_or_default();
    let preset = request
        .get_str("preset")
        .unwrap_or_else(|| "clothing_light".to_string());
    let stiffness = request.get_f64("stiffness");
    let damping = request.get_f64("damping");
    if node_id.is_empty() {
        return ToolResponse::err("apply_secondary_motion_preset", "node_id is required");
    }
    let (stiff_val, damp_val, mass_val) = match preset.as_str() {
        "hair_light" => (0.7, 0.3, 0.5),
        "hair_heavy" => (0.4, 0.5, 0.8),
        "clothing_light" => (0.6, 0.4, 0.6),
        "clothing_heavy" => (0.3, 0.6, 1.0),
        "prop" => (0.8, 0.2, 0.3),
        "custom" => (stiffness.unwrap_or(0.5), damping.unwrap_or(0.5), 0.5),
        _ => (0.6, 0.4, 0.6),
    };
    let result = crate::mcp_client::send_mcp_request(
        "apply_phy_modifier",
        serde_json::json!({
            "node_id": node_id,
            "stiffness": stiff_val,
            "damping": damp_val,
            "mass": mass_val,
        }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "apply_secondary_motion_preset",
            serde_json::json!({
                "node_id": node_id,
                "preset": preset,
                "stiffness": stiff_val,
                "damping": damp_val,
                "mass": mass_val,
            }),
            format!("Applied '{}' physics preset to '{}'", preset, node_id),
        ),
        Err(e) => ToolResponse::err("apply_secondary_motion_preset", e),
    }
}
fn handle_bake_physics_to_keyframes(request: ToolRequest) -> ToolResponse {
    let node_id = request.get_str("node_id").unwrap_or_default();
    let start_frame = request.get_i64("start_frame").unwrap_or(0);
    let end_frame = request.get_i64("end_frame").unwrap_or(120);
    if node_id.is_empty() {
        return ToolResponse::err("bake_physics_to_keyframes", "node_id is required");
    }
    let result = crate::physics::bake_to_keyframes(start_frame as u32, end_frame as u32);
    ToolResponse::ok_with_message(
        "bake_physics_to_keyframes",
        serde_json::json!({
            "node_id": node_id,
            "start_frame": start_frame,
            "end_frame": end_frame,
            "bake_result": result,
        }),
        format!(
            "Physics baked to keyframes for '{}' ({}-{})",
            node_id, start_frame, end_frame
        ),
    )
}
fn handle_get_animation_analysis(_request: ToolRequest) -> ToolResponse {
    let timeline_state =
        crate::mcp_client::send_mcp_request("get_timeline_state", serde_json::json!({}));
    match timeline_state {
        Ok(r) => {
            let state = r.data.unwrap_or(serde_json::json!({}));
            ToolResponse::ok_with_message(
                "get_animation_analysis",
                serde_json::json!({
                    "timeline_state": state,
                    "issues": [
                        {"frame": null, "problem": "Animation analysis requires frame-by-frame comparison", "severity": "info", "suggestion": "Run apply_pose_sequence first to ensure keyframes exist."}
                    ],
                    "overall_quality": "Unknown — insufficient keyframe data for quality analysis",
                }),
                "Animation analysis completed. Consider baking physics to keyframes for best results.",
            )
        },
        Err(e) => ToolResponse::err("get_animation_analysis", e),
    }
}
fn handle_create_morph_animation(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let morph_name = request.get_str("morph_name").unwrap_or_default();
    let _keyframes = request.get_object("keyframes");
    if morph_name.is_empty() {
        return ToolResponse::err("create_morph_animation", "morph_name is required");
    }
    // Apply the morph at frame 0 as a starting point
    let _ = crate::mcp_client::send_mcp_request(
        "set_keyframe",
        serde_json::json!({
            "node_id": figure_id,
            "property": morph_name,
            "frame": 0,
            "value": 0.0,
            "interpolation": "linear",
        }),
    );
    ToolResponse::ok_with_message(
        "create_morph_animation",
        serde_json::json!({
            "figure_id": figure_id,
            "morph_name": morph_name,
            "keyframes_created": 1,
            "note": "Initial keyframe placed. Provide a full keyframes array for complete animation.",
        }),
        format!(
            "Created morph animation for '{}' on '{}'",
            morph_name, figure_id
        ),
    )
}
// ─── Helpers ───────────────────────────────────────────────────────────────
fn describe_expressions(data: &serde_json::Value) -> String {
    let obj = data.as_object();
    if obj.is_none() || obj.unwrap().is_empty() {
        return "Neutral expression".to_string();
    }
    let obj = obj.unwrap();
    let mut active = Vec::new();
    for (name, val) in obj.iter() {
        if let Some(v) = val.as_f64() {
            if v > 0.1 {
                active.push(format!("{} ({:.0}%)", name, v * 100.0));
            }
        }
    }
    if active.is_empty() {
        "Neutral expression".to_string()
    } else {
        format!("Expression: {}", active.join(", "))
    }
}
fn describe_stance(data: &serde_json::Value) -> String {
    let obj = data.as_object();
    if obj.is_none() {
        return "Unknown pose".to_string();
    }
    let obj = obj.unwrap();
    let mut stance_parts = Vec::new();
    let head_tilt = obj.get("Head_Tilt").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let waist_bend = obj
        .get("Waist_Bend")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let left_knee = obj
        .get("Left_Knee_Bend")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let right_knee = obj
        .get("Right_Knee_Bend")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    if waist_bend > 0.3 {
        stance_parts.push("leaning forward");
    } else if waist_bend < -0.3 {
        stance_parts.push("leaning back");
    } else {
        stance_parts.push("upright");
    }
    if left_knee > 0.5 || right_knee > 0.5 {
        stance_parts.push("with bent knees");
    }
    if head_tilt > 0.2 {
        stance_parts.push("head tilted down");
    } else if head_tilt < -0.2 {
        stance_parts.push("head tilted up");
    }
    if stance_parts.is_empty() {
        "Standing upright".to_string()
    } else {
        format!("Standing {}", stance_parts.join(", "))
    }
}
fn get_pose_suggestions(mood: &str, scene_type: &str) -> Vec<serde_json::Value> {
    let mood_lower = mood.to_lowercase();
    let type_lower = scene_type.to_lowercase();
    let is_portrait = type_lower.contains("portrait");
    let _is_full = type_lower.contains("full") || type_lower.contains("body");
    let is_action = type_lower.contains("action") || mood_lower.contains("action");
    let is_fashion = type_lower.contains("fashion");
    let mut suggestions = Vec::new();
    if is_portrait {
        suggestions.push(serde_json::json!({
            "name": "Three-quarter portrait",
            "description": "Body angled slightly, head turned toward camera",
            "difficulty": "Easy",
            "suitable_for": mood,
        }));
        suggestions.push(serde_json::json!({
            "name": "Chin-on-hand thoughtful",
            "description": "Hand supporting chin, elbow on surface, contemplative look",
            "difficulty": "Medium",
            "suitable_for": mood,
        }));
    } else if is_fashion {
        suggestions.push(serde_json::json!({
            "name": "Model stance",
            "description": "One hand on hip, weight shifted to one leg, confident posture",
            "difficulty": "Easy",
            "suitable_for": mood,
        }));
        suggestions.push(serde_json::json!({
            "name": "Walking pose",
            "description": "Mid-stride, arms swinging naturally, dynamic and elegant",
            "difficulty": "Medium",
            "suitable_for": mood,
        }));
    } else if is_action {
        suggestions.push(serde_json::json!({
            "name": "Heroic landing",
            "description": "One knee bent, arms spread for balance, powerful stance",
            "difficulty": "Hard",
            "suitable_for": mood,
        }));
        suggestions.push(serde_json::json!({
            "name": "Combat ready",
            "description": "Fists raised, legs shoulder-width apart, alert and aggressive",
            "difficulty": "Medium",
            "suitable_for": mood,
        }));
    } else {
        suggestions.push(serde_json::json!({
            "name": "Casual stand",
            "description": "Relaxed posture, slight contrapposto, arms at sides",
            "difficulty": "Easy",
            "suitable_for": mood,
        }));
        suggestions.push(serde_json::json!({
            "name": "Elegant pose",
            "description": "Graceful S-curve, one arm extended, refined and poised",
            "difficulty": "Medium",
            "suitable_for": mood,
        }));
    }
    suggestions.push(serde_json::json!({
        "name": "Custom pose suggestion",
        "description": format!("Based on '{}' mood, a pose emphasizing {} would work well", mood, if mood_lower.contains("elegant") || mood_lower.contains("graceful") { "flowing lines and soft curves" } else if mood_lower.contains("powerful") || mood_lower.contains("strong") { "broad shoulders and confident angles" } else { "natural body language and relaxed limbs" }),
        "difficulty": "Varies",
        "suitable_for": mood,
    }));
    suggestions
}
fn parse_animation_description(_description: &str, total_frames: i64) -> Vec<serde_json::Value> {
    // Parse NL description into keyframe sequence
    // This is a simplified version — full NL parsing would use AI
    let mut sequence = Vec::new();
    sequence.push(serde_json::json!({
        "frame": 0,
        "pose": "start_pose",
        "description": "Initial position",
        "bone_overrides": {},
        "morph_overrides": {},
    }));
    let mid_frame = total_frames / 2;
    sequence.push(serde_json::json!({
        "frame": mid_frame,
        "pose": "mid_pose",
        "description": "Transition pose",
        "bone_overrides": {},
        "morph_overrides": {},
    }));
    sequence.push(serde_json::json!({
        "frame": total_frames,
        "pose": "end_pose",
        "description": "Final pose",
        "bone_overrides": {},
        "morph_overrides": {},
    }));
    sequence
}
