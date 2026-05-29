use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "list_joints",
        "Lists all joints/bones for a given figure with their names, hierarchy, and current rotation values.",
        ToolCategory::Rigging,
        [
            tool_param("figure_id", "Node ID of the figure to inspect", true, ToolParamType::String),
            tool_param("include_hidden", "Include hidden/auxiliary joints (default false)", false, ToolParamType::Boolean),
        ],
        "Result with list of joints, hierarchy depth, and rotation values",
        [
            "List all joints on Genesis 9",
            "Show me the bone structure of my figure",
        ],
        handle_list_joints
    );
    define_tool!(
        "set_joint_rotation",
        "Sets the rotation of a specific joint/bone using local Euler angles. Useful for fine-tuning poses at the joint level.",
        ToolCategory::Rigging,
        [
            tool_param("figure_id", "Node ID of the figure", true, ToolParamType::String),
            tool_param("joint_name", "Name of the joint/bone to rotate", true, ToolParamType::String),
            tool_param("x", "X-axis rotation in degrees", false, ToolParamType::Number),
            tool_param("y", "Y-axis rotation in degrees", false, ToolParamType::Number),
            tool_param("z", "Z-axis rotation in degrees", false, ToolParamType::Number),
            tool_param("space", "Rotation space: local, world, parent (default local)", false, ToolParamType::String),
        ],
        "Result confirming the joint rotation was applied",
        [
            "Rotate the left elbow 45 degrees on the X axis",
            "Bend the right knee -30 degrees",
            "Tilt the head 15 degrees on X and -10 on Z",
        ],
        handle_set_joint_rotation
    );
    define_tool!(
        "mirror_pose",
        "Mirrors the current pose of a figure from one side to the other. Automatically maps left/right joints.",
        ToolCategory::Rigging,
        [
            tool_param("figure_id", "Node ID of the figure to mirror", true, ToolParamType::String),
            tool_param("direction", "Mirror direction: left_to_right, right_to_left (default left_to_right)", false, ToolParamType::String),
            tool_param("include_morphs", "Also mirror facial/body morphs (default true)", false, ToolParamType::Boolean),
        ],
        "Result with count of joints and morphs mirrored",
        [
            "Mirror the pose from left to right on my character",
            "Mirror the right arm pose to the left arm",
        ],
        handle_mirror_pose
    );
    define_tool!(
        "set_ik_fk_blend",
        "Controls the blend between IK (Inverse Kinematics) and FK (Forward Kinematics) for a figure's limbs. IK is useful for foot/hand placement, FK for natural swinging motion.",
        ToolCategory::Rigging,
        [
            tool_param("figure_id", "Node ID of the figure", true, ToolParamType::String),
            tool_param("limb", "Limb to configure: left_arm, right_arm, both_arms, left_leg, right_leg, both_legs, all (default all)", false, ToolParamType::String),
            tool_param("blend", "IK blend 0.0 (full FK) to 1.0 (full IK), default 0.0", false, ToolParamType::Number),
        ],
        "Result confirming IK/FK blend applied to the specified limbs",
        [
            "Enable IK on both legs for foot planting",
            "Set left arm to 50% IK blend",
            "Switch to full FK on both arms",
        ],
        handle_set_ik_fk_blend
    );
    define_tool!(
        "add_joint",
        "Adds a custom joint/bone to a figure at a specified position. Useful for custom rigging or adding accessories that need to follow specific animations.",
        ToolCategory::Rigging,
        [
            tool_param("figure_id", "Node ID of the parent figure", true, ToolParamType::String),
            tool_param("joint_name", "Name for the new joint", true, ToolParamType::String),
            tool_param("parent_joint", "Name of the parent joint to attach under", true, ToolParamType::String),
            tool_param("position", "Local position [x, y, z] relative to parent", false, ToolParamType::FloatArray),
        ],
        "Result with the new joint's node ID",
        [
            "Add a custom weapon bone to the right hand",
            "Create an accessory joint under the head",
        ],
        handle_add_joint
    );
}
fn handle_list_joints(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let include_hidden = request.get_bool("include_hidden").unwrap_or(false);
    if figure_id.is_empty() {
        return ToolResponse::err("list_joints", "figure_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "get_joint_list",
        serde_json::json!({
            "figure_id": figure_id,
            "include_hidden": include_hidden,
        }),
    );
    match result {
        Ok(r) => {
            let joints = r
                .data
                .as_ref()
                .and_then(|d| d.get("joints"))
                .cloned()
                .unwrap_or(serde_json::json!([]));
            ToolResponse::ok_with_message(
                "list_joints",
                serde_json::json!({ "figure_id": figure_id, "joints": joints }),
                format!("Found joints for '{}'", figure_id),
            )
        },
        Err(e) => ToolResponse::err("list_joints", e),
    }
}
fn handle_set_joint_rotation(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let joint_name = request.get_str("joint_name").unwrap_or_default();
    let x = request.get_f64("x").unwrap_or(0.0);
    let y = request.get_f64("y").unwrap_or(0.0);
    let z = request.get_f64("z").unwrap_or(0.0);
    let space = request
        .get_str("space")
        .unwrap_or_else(|| "local".to_string());
    if figure_id.is_empty() || joint_name.is_empty() {
        return ToolResponse::err(
            "set_joint_rotation",
            "figure_id and joint_name are required",
        );
    }
    let result = crate::mcp_client::send_mcp_request(
        "set_joint_rotation",
        serde_json::json!({
            "figure_id": figure_id,
            "joint": joint_name,
            "rotation": { "x": x, "y": y, "z": z },
            "space": space,
        }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_joint_rotation",
            serde_json::json!({ "figure_id": figure_id, "joint": joint_name, "rotation": { "x": x, "y": y, "z": z } }),
            format!(
                "Rotated '{}' on '{}' to ({:.1}, {:.1}, {:.1})",
                joint_name, figure_id, x, y, z
            ),
        ),
        Err(e) => ToolResponse::err("set_joint_rotation", e),
    }
}
fn handle_mirror_pose(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let direction = request
        .get_str("direction")
        .unwrap_or_else(|| "left_to_right".to_string());
    let include_morphs = request.get_bool("include_morphs").unwrap_or(true);
    if figure_id.is_empty() {
        return ToolResponse::err("mirror_pose", "figure_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "mirror_pose",
        serde_json::json!({
            "figure_id": figure_id,
            "direction": direction,
            "include_morphs": include_morphs,
        }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "mirror_pose",
            serde_json::json!({ "figure_id": figure_id, "result": r.data }),
            format!("Mirrored pose on '{}'", figure_id),
        ),
        Err(e) => ToolResponse::err("mirror_pose", e),
    }
}
fn handle_set_ik_fk_blend(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let limb = request.get_str("limb").unwrap_or_else(|| "all".to_string());
    let blend = request.get_f64("blend").unwrap_or(0.0);
    if figure_id.is_empty() {
        return ToolResponse::err("set_ik_fk_blend", "figure_id is required");
    }
    let blend_clamped = blend.clamp(0.0, 1.0);
    let result = crate::mcp_client::send_mcp_request(
        "set_ik_fk_blend",
        serde_json::json!({
            "figure_id": figure_id,
            "limb": limb,
            "blend": blend_clamped,
        }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_ik_fk_blend",
            serde_json::json!({ "figure_id": figure_id, "limb": limb, "blend": blend_clamped }),
            format!(
                "Set IK/FK blend to {:.0}% IK on '{}' {}",
                blend_clamped * 100.0,
                figure_id,
                limb
            ),
        ),
        Err(e) => ToolResponse::err("set_ik_fk_blend", e),
    }
}
fn handle_add_joint(request: ToolRequest) -> ToolResponse {
    let figure_id = request.get_str("figure_id").unwrap_or_default();
    let joint_name = request.get_str("joint_name").unwrap_or_default();
    let parent_joint = request.get_str("parent_joint").unwrap_or_default();
    let position = request.get_array("position");
    if figure_id.is_empty() || joint_name.is_empty() || parent_joint.is_empty() {
        return ToolResponse::err(
            "add_joint",
            "figure_id, joint_name, and parent_joint are required",
        );
    }
    let result = crate::mcp_client::send_mcp_request(
        "add_joint",
        serde_json::json!({
            "figure_id": figure_id,
            "joint_name": joint_name,
            "parent_joint": parent_joint,
            "position": position,
        }),
    );
    match result {
        Ok(r) => {
            let node_id = r.data.as_ref().and_then(|d| d.get("node_id")).cloned();
            ToolResponse::ok_with_message(
                "add_joint",
                serde_json::json!({ "joint_name": joint_name, "node_id": node_id }),
                format!(
                    "Added joint '{}' under '{}' on '{}'",
                    joint_name, parent_joint, figure_id
                ),
            )
        },
        Err(e) => ToolResponse::err("add_joint", e),
    }
}
