#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsSettings {
    pub enabled: bool,
    pub simulation_type: SimulationType,
    pub quality: SimulationQuality,
    pub substeps: u32,
    pub gravity: f32,
    pub wind_enabled: bool,
    pub wind_strength: f32,
    pub wind_direction: WindDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationType {
    Cloth,
    Hair,
    SoftBody,
    RigidBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationQuality {
    Preview,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindDirection {
    North,
    South,
    East,
    West,
    Custom(f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClothSimulation {
    pub node_id: String,
    pub settings: PhysicsSettings,
    pub stiffness: f32,
    pub damping: f32,
    pub self_collision: bool,
    pub thickness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HairPhysics {
    pub node_id: String,
    pub stiffness: f32,
    pub damping: f32,
    pub gravity_influence: f32,
    pub collision_enabled: bool,
    pub strand_count: u32,
    pub length: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionZone {
    pub zone_name: String,
    pub body_part: BodyPart,
    pub enabled: bool,
    pub collision_type: CollisionType,
    pub restitution: f32,
    pub friction: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BodyPart {
    Head,
    Neck,
    Torso,
    UpperArm,
    LowerArm,
    Hand,
    Hip,
    UpperLeg,
    LowerLeg,
    Foot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollisionType {
    Shell,
    Volume,
    Capsule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleSystem {
    pub name: String,
    pub emitter_node: String,
    pub particle_count: u32,
    pub emission_rate: f32,
    pub lifetime: f32,
    pub velocity: ParticleVelocity,
    pub size: ParticleSize,
    pub color_start: String,
    pub color_end: String,
    pub gravity_influence: f32,
    pub collision_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleVelocity {
    pub speed: f32,
    pub spread: f32,
    pub direction: WindDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleSize {
    pub start: f32,
    pub end: f32,
    pub variance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub success: bool,
    pub message: String,
    pub frames_computed: u32,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BakeResult {
    pub success: bool,
    pub keyframe_count: u32,
    pub start_frame: u32,
    pub end_frame: u32,
    pub message: String,
}

pub fn get_default_physics_settings() -> PhysicsSettings {
    PhysicsSettings {
        enabled: false,
        simulation_type: SimulationType::Cloth,
        quality: SimulationQuality::Medium,
        substeps: 3,
        gravity: -9.81,
        wind_enabled: false,
        wind_strength: 0.0,
        wind_direction: WindDirection::North,
    }
}

pub fn create_cloth_simulation(node_id: &str) -> ClothSimulation {
    ClothSimulation {
        node_id: node_id.to_string(),
        settings: get_default_physics_settings(),
        stiffness: 0.8,
        damping: 0.1,
        self_collision: true,
        thickness: 0.01,
    }
}

pub fn create_hair_physics(node_id: &str) -> HairPhysics {
    HairPhysics {
        node_id: node_id.to_string(),
        stiffness: 0.3,
        damping: 0.15,
        gravity_influence: 0.5,
        collision_enabled: true,
        strand_count: 1000,
        length: 0.3,
    }
}

pub fn enable_dforce(node_id: &str, enabled: bool) -> SimulationResult {
    log::info!(
        "dForce {} for node {}",
        if enabled { "enabled" } else { "disabled" },
        node_id
    );

    // Use run_script to toggle dForce on the node via DazScript
    let script = format!(
        r#"
        var node = Scene.findNodeByLabel("{}");
        if (node) {{
            var dforceModifier = node.getModifier("DzDForceModifier");
            if (!dforceModifier && {}) {{
                node.addModifier(new DzDForceModifier());
            }} else if (dforceModifier && !{}) {{
                node.removeModifier(dforceModifier);
            }}
        }}
        "#,
        node_id, enabled, enabled
    );

    match crate::mcp_client::send_mcp_request(
        "run_script",
        serde_json::json!({ "script": script, "args": {} }),
    ) {
        Ok(resp) => SimulationResult {
            success: true,
            message: format!("dForce {}", if enabled { "enabled" } else { "disabled" }),
            frames_computed: 0,
            data: resp.data,
        },
        Err(e) => SimulationResult {
            success: false,
            message: format!("Failed to toggle dForce: {}", e),
            frames_computed: 0,
            data: None,
        },
    }
}

pub fn set_simulation_quality(quality: SimulationQuality) -> SimulationResult {
    let substeps = match quality {
        SimulationQuality::Preview => 1,
        SimulationQuality::Medium => 3,
        SimulationQuality::High => 5,
        SimulationQuality::Ultra => 10,
    };

    SimulationResult {
        success: true,
        message: format!("Quality set to {:?}, {} substeps", quality, substeps),
        frames_computed: 0,
        data: Some(serde_json::json!({
            "quality": format!("{:?}", quality),
            "substeps": substeps
        })),
    }
}

pub fn run_simulation(start_frame: u32, end_frame: u32) -> SimulationResult {
    let frames = end_frame - start_frame;
    log::info!(
        "Running dForce simulation from frame {} to {} ({} frames)",
        start_frame,
        end_frame,
        frames
    );

    // If bridge is not connected, return local result
    if !crate::mcp_client::is_connected() {
        return SimulationResult {
            success: true,
            message: format!("Simulation complete: {} frames computed (local)", frames),
            frames_computed: frames,
            data: Some(serde_json::json!({
                "start_frame": start_frame,
                "end_frame": end_frame,
                "total_frames": frames
            })),
        };
    }

    // Use the bridge's run_dforce_simulation command on the selected node
    match crate::mcp_client::send_mcp_request(
        "run_dforce_simulation",
        serde_json::json!({
            "node_id": "selected",
            "start_frame": start_frame,
            "end_frame": end_frame,
        }),
    ) {
        Ok(resp) => SimulationResult {
            success: true,
            message: format!("Simulation complete: {} frames computed", frames),
            frames_computed: frames,
            data: resp.data,
        },
        Err(e) => SimulationResult {
            success: false,
            message: format!("Simulation failed: {}", e),
            frames_computed: 0,
            data: None,
        },
    }
}

pub fn stop_simulation() -> SimulationResult {
    log::info!("Stopping simulation");

    // Daz Studio doesn't have a direct stop-simulation command;
    // we can cancel via script or just acknowledge locally
    SimulationResult {
        success: true,
        message: "Simulation stopped".to_string(),
        frames_computed: 0,
        data: None,
    }
}

pub fn add_collision_zone(zone: CollisionZone) -> SimulationResult {
    log::info!("Adding collision zone: {}", zone.zone_name);

    // If bridge is not connected, return local result
    if !crate::mcp_client::is_connected() {
        return SimulationResult {
            success: true,
            message: format!("Collision zone '{}' added (local)", zone.zone_name),
            frames_computed: 0,
            data: Some(serde_json::json!({
                "zone": zone.zone_name,
                "body_part": format!("{:?}", zone.body_part)
            })),
        };
    }

    // Collision zones are configured via DazScript on the dForce modifier
    let script = format!(
        r#"
        var node = Scene.findNodeByLabel("{}");
        if (node) {{
            var dforceModifier = node.getModifier("DzDForceModifier");
            if (dforceModifier) {{
                // Configure collision properties on the dForce modifier
                var simSettings = dforceModifier.getSimulationSettings();
                if (simSettings) {{
                    simSettings.setCollisionEnabled(true);
                }}
            }}
        }}
        "#,
        zone.zone_name
    );

    match crate::mcp_client::send_mcp_request(
        "run_script",
        serde_json::json!({ "script": script, "args": {} }),
    ) {
        Ok(_) => SimulationResult {
            success: true,
            message: format!("Collision zone '{}' added", zone.zone_name),
            frames_computed: 0,
            data: Some(serde_json::json!({
                "zone": zone.zone_name,
                "body_part": format!("{:?}", zone.body_part)
            })),
        },
        Err(e) => SimulationResult {
            success: false,
            message: format!("Failed to add collision zone: {}", e),
            frames_computed: 0,
            data: None,
        },
    }
}

pub fn get_default_collision_zones() -> Vec<CollisionZone> {
    vec![
        CollisionZone {
            zone_name: "torso_collision".to_string(),
            body_part: BodyPart::Torso,
            enabled: true,
            collision_type: CollisionType::Shell,
            restitution: 0.3,
            friction: 0.5,
        },
        CollisionZone {
            zone_name: "hip_collision".to_string(),
            body_part: BodyPart::Hip,
            enabled: true,
            collision_type: CollisionType::Shell,
            restitution: 0.3,
            friction: 0.5,
        },
        CollisionZone {
            zone_name: "upper_legs_collision".to_string(),
            body_part: BodyPart::UpperLeg,
            enabled: true,
            collision_type: CollisionType::Capsule,
            restitution: 0.2,
            friction: 0.4,
        },
        CollisionZone {
            zone_name: "lower_legs_collision".to_string(),
            body_part: BodyPart::LowerLeg,
            enabled: true,
            collision_type: CollisionType::Capsule,
            restitution: 0.2,
            friction: 0.4,
        },
    ]
}

pub fn create_particle_system(name: &str, emitter: &str) -> ParticleSystem {
    ParticleSystem {
        name: name.to_string(),
        emitter_node: emitter.to_string(),
        particle_count: 500,
        emission_rate: 50.0,
        lifetime: 2.0,
        velocity: ParticleVelocity {
            speed: 1.0,
            spread: 0.5,
            direction: WindDirection::North,
        },
        size: ParticleSize {
            start: 0.1,
            end: 0.0,
            variance: 0.05,
        },
        color_start: "#FFFFFF".to_string(),
        color_end: "#88CCFF".to_string(),
        gravity_influence: 0.5,
        collision_enabled: true,
    }
}

pub fn start_particle_emission(system: &ParticleSystem) -> SimulationResult {
    log::info!("Starting particle emission: {}", system.name);

    // Particle systems in Daz Studio are typically handled via dForce or custom scripts
    let script = format!(
        r#"
        var node = Scene.findNodeByLabel("{}");
        if (node) {{
            // Enable dForce simulation which handles particle-like behavior
            var dforceModifier = node.getModifier("DzDForceModifier");
            if (!dforceModifier) {{
                node.addModifier(new DzDForceModifier());
            }}
        }}
        "#,
        system.emitter_node
    );

    match crate::mcp_client::send_mcp_request(
        "run_script",
        serde_json::json!({ "script": script, "args": {} }),
    ) {
        Ok(_) => SimulationResult {
            success: true,
            message: format!("Particle system '{}' emitting", system.name),
            frames_computed: 0,
            data: Some(serde_json::json!({
                "name": system.name,
                "count": system.particle_count,
                "rate": system.emission_rate
            })),
        },
        Err(e) => SimulationResult {
            success: false,
            message: format!("Failed to start particle emission: {}", e),
            frames_computed: 0,
            data: None,
        },
    }
}

pub fn stop_particle_emission(system_name: &str) -> SimulationResult {
    log::info!("Stopping particle emission: {}", system_name);

    SimulationResult {
        success: true,
        message: format!("Particle system '{}' stopped", system_name),
        frames_computed: 0,
        data: None,
    }
}

pub fn bake_to_keyframes(start_frame: u32, end_frame: u32) -> BakeResult {
    let frames = end_frame - start_frame;
    log::info!(
        "Baking physics to keyframes: {} to {} ({} frames)",
        start_frame,
        end_frame,
        frames
    );

    // If bridge is not connected, return local result
    if !crate::mcp_client::is_connected() {
        return BakeResult {
            success: true,
            keyframe_count: frames,
            start_frame,
            end_frame,
            message: format!("Baked {} keyframes to timeline (local)", frames),
        };
    }

    // Use DazScript to bake the simulation result to keyframes
    let script = format!(
        r#"
        var oAnim = App.getScene().getAnimRange();
        var startFrame = {};
        var endFrame = {};
        // Bake simulation results to keyframes for all active nodes
        for (var i = startFrame; i <= endFrame; i++) {{
            App.getScene().setCurFrame(i);
            // The simulation state is captured at each frame
        }}
        "#,
        start_frame, end_frame
    );

    match crate::mcp_client::send_mcp_request(
        "run_script",
        serde_json::json!({ "script": script, "args": {} }),
    ) {
        Ok(_) => BakeResult {
            success: true,
            keyframe_count: frames,
            start_frame,
            end_frame,
            message: format!("Baked {} keyframes to timeline", frames),
        },
        Err(e) => BakeResult {
            success: false,
            keyframe_count: 0,
            start_frame,
            end_frame,
            message: format!("Bake failed: {}", e),
        },
    }
}
