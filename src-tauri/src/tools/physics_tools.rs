use super::{tool_param, ToolCategory, ToolParamType, ToolRequest, ToolResponse};
use crate::define_tool;
pub fn register_tools() {
    define_tool!(
        "simulate_physics",
        "Runs a dForce physics simulation on selected scene nodes. Simulates fabric, hair, and soft body dynamics.",
        ToolCategory::Physics,
        [
            tool_param("node_ids", "Array of node IDs to simulate, or 'all' for all physics-enabled nodes. Default 'all'", false, ToolParamType::String),
            tool_param("frames", "Number of frames to simulate (default 100)", false, ToolParamType::Integer),
            tool_param("start_frame", "Start frame for simulation (default current frame)", false, ToolParamType::Integer),
            tool_param("real_time", "Simulate in real-time (default false, faster = false)", false, ToolParamType::Boolean),
        ],
        "Result with simulation status and frame range",
        [
            "Run physics simulation on all dynamic items",
            "Simulate clothing physics for 50 frames",
            "Run a real-time physics simulation",
        ],
        handle_simulate_physics
    );
    define_tool!(
        "set_wind",
        "Sets wind parameters for physics simulations. Controls direction, speed, and turbulence.",
        ToolCategory::Physics,
        [
            tool_param(
                "direction",
                "Wind direction as [x, y, z] vector (default [0, 0, 1])",
                false,
                ToolParamType::FloatArray
            ),
            tool_param(
                "speed",
                "Wind speed in Daz units (0-100, default 10)",
                false,
                ToolParamType::Number
            ),
            tool_param(
                "turbulence",
                "Wind turbulence 0.0-1.0 (default 0.3)",
                false,
                ToolParamType::Number
            ),
            tool_param(
                "gust_strength",
                "Gust strength multiplier 0.0-2.0 (default 0.5)",
                false,
                ToolParamType::Number
            ),
        ],
        "Result confirming wind settings",
        [
            "Set a gentle breeze from the left",
            "Create strong wind from behind",
            "Add turbulent windy conditions for hair simulation",
        ],
        handle_set_wind
    );
    define_tool!(
        "set_gravity",
        "Configures gravity settings for physics simulations. Can override global gravity for selected nodes.",
        ToolCategory::Physics,
        [
            tool_param("strength", "Gravity strength (0 = zero gravity, 1.0 = normal, default 1.0)", false, ToolParamType::Number),
            tool_param("direction", "Gravity direction as [x, y, z] (default [0, -1, 0] for downward)", false, ToolParamType::FloatArray),
            tool_param("node_id", "Apply to specific node only (default = global setting)", false, ToolParamType::String),
        ],
        "Result confirming gravity settings",
        [
            "Reduce gravity for floaty hair movement",
            "Set zero gravity for space scene",
            "Double gravity for heavy cloth effect",
        ],
        handle_set_gravity
    );
    define_tool!(
        "add_collision_object",
        "Adds a collision object for physics simulations. Ensures clothing/hair doesn't clip through scene objects.",
        ToolCategory::Physics,
        [
            tool_param("node_id", "Node ID of the object to act as collision body", true, ToolParamType::String),
            tool_param("shape", "Collision shape: auto, sphere, box, capsule, mesh (default auto)", false, ToolParamType::String),
            tool_param("friction", "Surface friction 0.0-1.0 (default 0.5)", false, ToolParamType::Number),
        ],
        "Result confirming collision object added",
        [
            "Add the chair as a collision object for clothing",
            "Make the ground plane a collision surface",
            "Set the table as a collision body",
        ],
        handle_add_collision_object
    );
    define_tool!(
        "bake_physics_simulation",
        "Bakes a completed physics simulation to keyframes. Makes the simulation permanent and editable on the timeline.",
        ToolCategory::Physics,
        [
            tool_param("node_id", "Node ID to bake physics on (default = all simulated nodes)", false, ToolParamType::String),
            tool_param("range_start", "Start frame to bake (default = simulation start)", false, ToolParamType::Integer),
            tool_param("range_end", "End frame to bake (default = simulation end)", false, ToolParamType::Integer),
            tool_param("sample_rate", "Keyframe sample rate: 1 = every frame, 2 = every other (default 1)", false, ToolParamType::Integer),
        ],
        "Result confirming bake and keyframe count",
        [
            "Bake the hair physics simulation to keyframes",
            "Bake clothing simulation to every other frame",
        ],
        handle_bake_physics_simulation
    );
    define_tool!(
        "set_physics_properties",
        "Adjusts physical properties of a simulated node: mass, stiffness, damping, and collision settings.",
        ToolCategory::Physics,
        [
            tool_param("node_id", "Node ID to adjust physics properties on", true, ToolParamType::String),
            tool_param("mass", "Mass multiplier 0.1-10.0 (default 1.0)", false, ToolParamType::Number),
            tool_param("stiffness", "Stiffness 0.0-1.0 (default 0.5)", false, ToolParamType::Number),
            tool_param("damping", "Damping 0.0-1.0 (default 0.1)", false, ToolParamType::Number),
            tool_param("collision", "Enable collision for this node (default true)", false, ToolParamType::Boolean),
        ],
        "Result confirming physics properties applied",
        [
            "Make the fabric stiffer with less damping",
            "Reduce mass for lighter physics behavior",
            "Disable collision on this accessory",
        ],
        handle_set_physics_properties
    );
    define_tool!(
        "remove_physics",
        "Removes physics simulation data or dForce modifiers from selected nodes.",
        ToolCategory::Physics,
        [
            tool_param(
                "node_id",
                "Node ID to remove physics from",
                true,
                ToolParamType::String
            ),
            tool_param(
                "remove_modifiers",
                "Remove dForce modifiers completely (default false = just disable)",
                false,
                ToolParamType::Boolean
            ),
        ],
        "Result confirming physics removal",
        [
            "Remove physics from this clothing item",
            "Disable and remove dForce from the hair",
        ],
        handle_remove_physics
    );
}
fn handle_simulate_physics(request: ToolRequest) -> ToolResponse {
    let node_ids = request
        .get_str("node_ids")
        .unwrap_or_else(|| "all".to_string());
    let frames = request.get_i64("frames").unwrap_or(100);
    let start_frame = request.get_i64("start_frame");
    let real_time = request.get_bool("real_time").unwrap_or(false);
    let result = crate::mcp_client::send_mcp_request(
        "simulate_physics",
        serde_json::json!({ "node_ids": node_ids, "frames": frames, "start_frame": start_frame, "real_time": real_time }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "simulate_physics",
            serde_json::json!({ "node_ids": node_ids, "frames": frames, "result": r.data }),
            format!("Physics simulation for {} frames", frames),
        ),
        Err(e) => ToolResponse::err("simulate_physics", e),
    }
}
fn handle_set_wind(request: ToolRequest) -> ToolResponse {
    let direction = request.get_array("direction");
    let speed = request.get_f64("speed").unwrap_or(10.0);
    let turbulence = request.get_f64("turbulence").unwrap_or(0.3);
    let gust = request.get_f64("gust_strength").unwrap_or(0.5);
    let dir = if direction.len() >= 3 {
        direction
    } else {
        vec![
            serde_json::json!(0.0),
            serde_json::json!(0.0),
            serde_json::json!(1.0),
        ]
    };
    let result = crate::mcp_client::send_mcp_request(
        "set_wind",
        serde_json::json!({ "direction": dir, "speed": speed, "turbulence": turbulence, "gust_strength": gust }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_wind",
            serde_json::json!({ "speed": speed, "turbulence": turbulence }),
            format!("Wind: speed={:.0}, turbulence={:.1}", speed, turbulence),
        ),
        Err(e) => ToolResponse::err("set_wind", e),
    }
}
fn handle_set_gravity(request: ToolRequest) -> ToolResponse {
    let strength = request.get_f64("strength").unwrap_or(1.0);
    let direction = request.get_array("direction");
    let node_id = request.get_str("node_id");
    let dir = if direction.len() >= 3 {
        direction
    } else {
        vec![
            serde_json::json!(0.0),
            serde_json::json!(-1.0),
            serde_json::json!(0.0),
        ]
    };
    let result = crate::mcp_client::send_mcp_request(
        "set_gravity",
        serde_json::json!({ "strength": strength, "direction": dir, "node_id": node_id }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_gravity",
            serde_json::json!({ "strength": strength }),
            format!("Gravity set to {:.1}x", strength),
        ),
        Err(e) => ToolResponse::err("set_gravity", e),
    }
}
fn handle_add_collision_object(request: ToolRequest) -> ToolResponse {
    let node_id = request.get_str("node_id").unwrap_or_default();
    let shape = request
        .get_str("shape")
        .unwrap_or_else(|| "auto".to_string());
    let friction = request.get_f64("friction").unwrap_or(0.5);
    if node_id.is_empty() {
        return ToolResponse::err("add_collision_object", "node_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "add_collision",
        serde_json::json!({ "node_id": node_id, "shape": shape, "friction": friction }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "add_collision_object",
            serde_json::json!({ "node_id": node_id }),
            format!("Collision object added: {}", node_id),
        ),
        Err(e) => ToolResponse::err("add_collision_object", e),
    }
}
fn handle_bake_physics_simulation(request: ToolRequest) -> ToolResponse {
    let node_id = request.get_str("node_id");
    let range_start = request.get_i64("range_start");
    let range_end = request.get_i64("range_end");
    let sample_rate = request.get_i64("sample_rate").unwrap_or(1);
    let result = crate::mcp_client::send_mcp_request(
        "bake_physics",
        serde_json::json!({ "node_id": node_id, "range_start": range_start, "range_end": range_end, "sample_rate": sample_rate }),
    );
    match result {
        Ok(r) => ToolResponse::ok_with_message(
            "bake_physics_simulation",
            serde_json::json!({ "result": r.data }),
            "Physics baked to keyframes",
        ),
        Err(e) => ToolResponse::err("bake_physics_simulation", e),
    }
}
fn handle_set_physics_properties(request: ToolRequest) -> ToolResponse {
    let node_id = request.get_str("node_id").unwrap_or_default();
    let mass = request.get_f64("mass").unwrap_or(1.0);
    let stiffness = request.get_f64("stiffness").unwrap_or(0.5);
    let damping = request.get_f64("damping").unwrap_or(0.1);
    let collision = request.get_bool("collision").unwrap_or(true);
    if node_id.is_empty() {
        return ToolResponse::err("set_physics_properties", "node_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "set_physics_props",
        serde_json::json!({ "node_id": node_id, "mass": mass, "stiffness": stiffness, "damping": damping, "collision": collision }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "set_physics_properties",
            serde_json::json!({ "node_id": node_id, "mass": mass, "stiffness": stiffness }),
            format!("Physics properties set on '{}'", node_id),
        ),
        Err(e) => ToolResponse::err("set_physics_properties", e),
    }
}
fn handle_remove_physics(request: ToolRequest) -> ToolResponse {
    let node_id = request.get_str("node_id").unwrap_or_default();
    let remove_mods = request.get_bool("remove_modifiers").unwrap_or(false);
    if node_id.is_empty() {
        return ToolResponse::err("remove_physics", "node_id is required");
    }
    let result = crate::mcp_client::send_mcp_request(
        "remove_physics",
        serde_json::json!({ "node_id": node_id, "remove_modifiers": remove_mods }),
    );
    match result {
        Ok(_) => ToolResponse::ok_with_message(
            "remove_physics",
            serde_json::json!({ "node_id": node_id }),
            if remove_mods {
                "Physics modifiers removed"
            } else {
                "Physics disabled"
            },
        ),
        Err(e) => ToolResponse::err("remove_physics", e),
    }
}
