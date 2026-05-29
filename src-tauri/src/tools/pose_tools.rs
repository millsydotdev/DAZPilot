use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "apply_pose_preset",
        "Applies a full-body pose preset to a figure from the content library. Searches by name, mood, or style.",
        ToolCategory::Pose,
        [
            tool_param("figure_id", "Figure node ID to apply the pose to", true, ToolParamType::String),
            tool_param("pose_name", "Name or description of the pose to search and apply", true, ToolParamType::String),
            tool_param("mirror", "Mirror the pose left-right (default false)", false, ToolParamType::Boolean),
            tool_param("blend", "Blend strength 0.0-1.0 for partial pose application (default 1.0)", false, ToolParamType::Number),
        ],
        "Result confirming pose applied to figure",
        [
            "Apply a relaxed standing pose to Genesis 9",
            "Search for and apply a fighting stance",
            "Apply a dancing pose at 50% blend",
        ],
        handle_apply_pose_preset
    );
    define_tool!(
        "list_pose_presets",
        "Lists available pose presets by category, mood, or figure compatibility.",
        ToolCategory::Pose,
        [
            tool_param(
                "category",
                "Category: standing, sitting, lying, action, dance, intimate, fantasy, sports",
                false,
                ToolParamType::String
            ),
            tool_param(
                "figure_type",
                "Filter by figure type (e.g., genesis_9, genesis_8)",
                false,
                ToolParamType::String
            ),
            tool_param(
                "mood",
                "Mood: casual, dramatic, romantic, aggressive, playful, elegant",
                false,
                ToolParamType::String
            ),
        ],
        "Result with list of matching pose presets",
        [
            "What standing poses are available?",
            "Show me action poses for Genesis 9",
            "Find romantic couple poses",
        ],
        handle_list_pose_presets
    );
    define_tool!(
        "save_pose_preset",
        "Saves the current figure pose as a reusable preset in the content library.",
        ToolCategory::Pose,
        [
            tool_param(
                "figure_id",
                "Figure node ID to capture pose from",
                true,
                ToolParamType::String
            ),
            tool_param(
                "preset_name",
                "Name for the new pose preset",
                true,
                ToolParamType::String
            ),
            tool_param(
                "category",
                "Category tag for organization",
                false,
                ToolParamType::String
            ),
            tool_param(
                "include_facial",
                "Include facial expression morphs (default true)",
                false,
                ToolParamType::Boolean
            ),
        ],
        "Result confirming pose preset saved",
        [
            "Save this pose as 'my_custom_pose'",
            "Save current standing pose as a preset",
        ],
        handle_save_pose_preset
    );
    define_tool!(
        "blend_poses",
        "Blends between two poses on a figure. Useful for creating transitions and hybrid poses.",
        ToolCategory::Pose,
        [
            tool_param("figure_id", "Figure node ID", true, ToolParamType::String),
            tool_param(
                "pose_a",
                "Name of the first pose preset or 'current' for current pose",
                true,
                ToolParamType::String
            ),
            tool_param(
                "pose_b",
                "Name of the second pose preset",
                true,
                ToolParamType::String
            ),
            tool_param(
                "blend_factor",
                "Blend between 0.0 (pose_a) and 1.0 (pose_b), default 0.5",
                false,
                ToolParamType::Number
            ),
        ],
        "Result with blend applied",
        [
            "Blend walking and running poses 50/50",
            "Create a hybrid between standing and leaning poses",
        ],
        handle_blend_poses
    );
    define_tool!(
        "mirror_pose",
        "Mirrors the current pose of a figure left-to-right. Useful for fixing asymmetric poses or creating mirrored pairs.",
        ToolCategory::Pose,
        [
            tool_param("figure_id", "Figure node ID to mirror the pose on", true, ToolParamType::String),
            tool_param("mirror_selected", "Only mirror selected body parts (default false = full body)", false, ToolParamType::Boolean),
        ],
        "Result confirming pose mirrored",
        [
            "Mirror the character's pose",
            "Mirror the full body pose",
        ],
        handle_mirror_pose
    );
    define_tool!(
        "apply_asymmetric_pose",
        "Applies different poses to the left and right sides of a figure. Useful for asymmetric stances and gestures.",
        ToolCategory::Pose,
        [
            tool_param("figure_id", "Figure node ID", true, ToolParamType::String),
            tool_param("left_pose", "Pose preset for the left side", true, ToolParamType::String),
            tool_param("right_pose", "Pose preset for the right side", true, ToolParamType::String),
        ],
        "Result confirming asymmetric pose applied",
        [
            "Apply different arm poses to each side",
            "Create a walking pose with mixed upper/lower body",
        ],
        handle_apply_asymmetric_pose
    );
    define_tool!(
        "reset_to_t_pose",
        "Resets a figure to the default T-pose or A-pose. Removes all pose transformations.",
        ToolCategory::Pose,
        [
            tool_param("figure_id", "Figure node ID", true, ToolParamType::String),
            tool_param(
                "pose_type",
                "Pose type: t_pose, a_pose (default t_pose)",
                false,
                ToolParamType::String
            ),
            tool_param(
                "preserve_facial",
                "Keep facial expression morphs (default false)",
                false,
                ToolParamType::Boolean
            ),
        ],
        "Result confirming pose reset",
        [
            "Reset the character to T-pose",
            "Reset to A-pose for easier clothing fitting",
        ],
        handle_reset_to_t_pose
    );
    define_tool!(
        "randomize_pose",
        "Applies a random pose to a figure. Can constrain to specific categories for controlled randomization.",
        ToolCategory::Pose,
        [
            tool_param("figure_id", "Figure node ID", true, ToolParamType::String),
            tool_param("category", "Constrain to category: standing, sitting, action, casual, dance", false, ToolParamType::String),
            tool_param("intensity", "Pose intensity 0.0-1.0, higher = more extreme (default 0.5)", false, ToolParamType::Number),
        ],
        "Result confirming random pose applied",
        [
            "Give this character a random pose",
            "Apply a random action pose",
        ],
        handle_randomize_pose
    );
}
fn handle_apply_pose_preset(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let pose_name = request.get_str("pose_name").unwrap_or_default();
    let mirror = request.get_bool("mirror").unwrap_or(false);
    let blend = request.get_f64("blend").unwrap_or(1.0);
    if figure_id.is_empty() || pose_name.is_empty() {
        return ToolResponse::err("apply_pose_preset", "figure_id and pose_name are required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "apply_pose",
        serde_json::json!({ "figure_id": figure_id, "pose": pose_name, "mirror": mirror, "blend": blend }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "apply_pose_preset",
            serde_json::json!({ "figure_id": figure_id, "pose": pose_name }),
            format!("Applied pose '{}' to figure", pose_name),
        ),
        Err(e) => ToolResponse::err("apply_pose_preset", e),
    }
}
fn handle_list_pose_presets(request: ToolRequest) -> ToolResponse {
    let category = request.get_str("category");
    let figure_type = request.get_str("figure_type");
    let mood = request.get_str("mood");
    let result = crate::mcp_client::send_mcp_request(
        "list_poses",
        serde_json::json!({ "category": category, "figure_type": figure_type, "mood": mood }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "list_pose_presets",
            serde_json::json!({ "presets": r.data }),
            "Pose presets listed",
        ),
        Err(e) => ToolResponse::err("list_pose_presets", e),
    }
}
fn handle_save_pose_preset(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let preset_name = request.get_str("preset_name").unwrap_or_default();
    let category = request.get_str("category");
    let include_facial = request.get_bool("include_facial").unwrap_or(true);
    if figure_id.is_empty() || preset_name.is_empty() {
        return ToolResponse::err("save_pose_preset", "figure_id and preset_name are required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "save_pose",
        serde_json::json!({ "figure_id": figure_id, "name": preset_name, "category": category, "include_facial": include_facial }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "save_pose_preset",
            serde_json::json!({ "preset_name": preset_name }),
            format!("Pose '{}' saved", preset_name),
        ),
        Err(e) => ToolResponse::err("save_pose_preset", e),
    }
}
fn handle_blend_poses(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let pose_a = request.get_str("pose_a").unwrap_or_default();
    let pose_b = request.get_str("pose_b").unwrap_or_default();
    let blend = request.get_f64("blend_factor").unwrap_or(0.5);
    if figure_id.is_empty() || pose_a.is_empty() || pose_b.is_empty() {
        return ToolResponse::err("blend_poses", "figure_id, pose_a, and pose_b are required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "blend_poses",
        serde_json::json!({ "figure_id": figure_id, "pose_a": pose_a, "pose_b": pose_b, "blend": blend }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "blend_poses",
            serde_json::json!({ "figure_id": figure_id, "blend": blend }),
            format!("Poses blended at {:.0}%", blend * 100.0),
        ),
        Err(e) => ToolResponse::err("blend_poses", e),
    }
}
fn handle_mirror_pose(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let mirror_selected = request.get_bool("mirror_selected").unwrap_or(false);
    if figure_id.is_empty() {
        return ToolResponse::err("mirror_pose", "figure_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "mirror_pose",
        serde_json::json!({ "figure_id": figure_id, "selected_only": mirror_selected }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "mirror_pose",
            serde_json::json!({ "figure_id": figure_id }),
            "Pose mirrored",
        ),
        Err(e) => ToolResponse::err("mirror_pose", e),
    }
}
fn handle_apply_asymmetric_pose(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let left_pose = request.get_str("left_pose").unwrap_or_default();
    let right_pose = request.get_str("right_pose").unwrap_or_default();
    if figure_id.is_empty() || left_pose.is_empty() || right_pose.is_empty() {
        return ToolResponse::err(
            "apply_asymmetric_pose",
            "figure_id, left_pose, and right_pose are required",
        );
    }
    let result = crate::mcp_client::send_mcp_request(
        "asymmetric_pose",
        serde_json::json!({ "figure_id": figure_id, "left": left_pose, "right": right_pose }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "apply_asymmetric_pose",
            serde_json::json!({ "figure_id": figure_id }),
            "Asymmetric pose applied",
        ),
        Err(e) => ToolResponse::err("apply_asymmetric_pose", e),
    }
}
fn handle_reset_to_t_pose(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let pose_type = request
        .get_str("pose_type")
        .unwrap_or_else(|| "t_pose".to_string());
    let preserve_facial = request.get_bool("preserve_facial").unwrap_or(false);
    if figure_id.is_empty() {
        return ToolResponse::err("reset_to_t_pose", "figure_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "reset_pose",
        serde_json::json!({ "figure_id": figure_id, "pose_type": pose_type, "preserve_facial": preserve_facial }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "reset_to_t_pose",
            serde_json::json!({ "figure_id": figure_id }),
            format!("Reset to {}", pose_type.replace('_', " ")),
        ),
        Err(e) => ToolResponse::err("reset_to_t_pose", e),
    }
}
fn handle_randomize_pose(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let category = request.get_str("category");
    let intensity = request.get_f64("intensity").unwrap_or(0.5);
    if figure_id.is_empty() {
        return ToolResponse::err("randomize_pose", "figure_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "random_pose",
        serde_json::json!({ "figure_id": figure_id, "category": category, "intensity": intensity }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "randomize_pose",
            serde_json::json!({ "figure_id": figure_id, "intensity": intensity }),
            "Random pose applied",
        ),
        Err(e) => ToolResponse::err("randomize_pose", e),
    }
}
