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

pub fn is_dev_mock_bridge_enabled() -> bool {
    std::env::var("DAZPILOT_DEV_MOCK_BRIDGE")
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
        .unwrap_or(false)
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
    if is_dev_mock_bridge_enabled() {
        let mut global = MCP_CLIENT.lock().unwrap();
        *global = None;
        return Ok("Connected to Daz3D dev mock bridge (DAZPILOT_DEV_MOCK_BRIDGE=1)".to_string());
    }

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
    is_dev_mock_bridge_enabled() || MCP_CLIENT.lock().unwrap().is_some()
}

pub fn check_connection_status() -> String {
    if is_dev_mock_bridge_enabled() {
        return "connected".to_string();
    }

    if MCP_CLIENT.lock().unwrap().is_some() {
        "connected".to_string()
    } else {
        "disconnected".to_string()
    }
}

pub fn send_mcp_request(command: &str, args: Value) -> Result<McpResponse, String> {
    validate_command(command, &args)?;

    if is_dev_mock_bridge_enabled() {
        return dev_mock_response(command, &args);
    }

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

fn dev_mock_response(command: &str, args: &Value) -> Result<McpResponse, String> {
    let data = match command {
        "get_commands" => serde_json::json!({ "commands": get_mcp_command_list() }),
        "get_scene_info" => serde_json::json!({
            "filename": "Dev Mock Scene.duf",
            "nodes": 3,
            "lights": 1,
            "cameras": 1,
            "primary_selection": "",
            "dev_mock": true
        }),
        "list_nodes" => serde_json::json!({
            "nodes": [
                {"name": "MockCamera", "type": "camera", "selected": false},
                {"name": "MockLight", "type": "light", "selected": false},
                {"name": "MockFigure", "type": "figure", "selected": true}
            ],
            "dev_mock": true
        }),
        "get_selected_nodes" => serde_json::json!({
            "nodes": [{"name": "MockFigure", "type": "figure", "selected": true}],
            "dev_mock": true
        }),
        "get_scene_assets" => serde_json::json!({
            "assets": ["/Mock/Figure.duf"],
            "dev_mock": true
        }),
        "get_cameras" => serde_json::json!({
            "cameras": [{"name": "MockCamera", "focal_length": 50.0}],
            "dev_mock": true
        }),
        "get_bounding_boxes" => serde_json::json!({
            "boxes": [{
                "node": "MockFigure",
                "min": [-0.5, 0.0, -0.5],
                "max": [0.5, 1.8, 0.5],
                "center": [0.0, 0.9, 0.0]
            }],
            "dev_mock": true
        }),
        "get_geoshells" => serde_json::json!({ "shells": [], "dev_mock": true }),
        "get_node_transform" => serde_json::json!({
            "position": [0.0, 0.0, 0.0],
            "rotation": [0.0, 0.0, 0.0, 1.0],
            "scale": [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            "dev_mock": true
        }),
        "get_node_properties" => serde_json::json!({
            "properties": [
                {"name": "xPos", "value": 0.0, "min": -100.0, "max": 100.0, "is_morph": false},
                {"name": "yPos", "value": 0.0, "min": -100.0, "max": 100.0, "is_morph": false}
            ],
            "dev_mock": true
        }),
        "get_figure_morphs" => serde_json::json!({
            "morphs": [
                {"id": "head_height", "label": "Head Height", "value": 0.0, "min": -1.0, "max": 1.0, "type": "morph"},
                {"id": "waist_width", "label": "Waist Width", "value": 0.0, "min": -1.0, "max": 1.0, "type": "morph"}
            ],
            "dev_mock": true
        }),
        "get_fitted_items" => serde_json::json!({ "items": [], "dev_mock": true }),
        "get_active_expressions" => serde_json::json!({ "expressions": [], "dev_mock": true }),
        "get_material_zones" => serde_json::json!({
            "materials": [{"name": "Skin", "label": "Skin"}, {"name": "Eyes", "label": "Eyes"}],
            "dev_mock": true
        }),
        "get_material_properties" => serde_json::json!({
            "properties": [
                {"name": "Opacity", "value": 1.0, "min": 0.0, "max": 1.0},
                {"name": "Glossiness", "value": 0.5, "min": 0.0, "max": 1.0}
            ],
            "dev_mock": true
        }),
        "get_material_channels" => serde_json::json!({
            "channels": [
                {"name": "Diffuse Color", "texture": "", "value": [0.8, 0.8, 0.8]},
                {"name": "Bump", "texture": "", "value": 0.0}
            ],
            "dev_mock": true
        }),
        "capture_viewport" => serde_json::json!({
            "result": "base64",
            "data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
            "dev_mock": true
        }),
        "play_timeline" => serde_json::json!({ "playing": true, "dev_mock": true }),
        "pause_timeline" => serde_json::json!({ "playing": false, "dev_mock": true }),
        "stop_timeline" => serde_json::json!({ "frame": 0, "dev_mock": true }),
        "get_timeline_state" => serde_json::json!({
            "current_frame": 0, "start_frame": 0, "end_frame": 300,
            "fps": 30.0, "is_playing": false, "dev_mock": true
        }),
        "list_bones" => serde_json::json!({
            "bones": [{"name": "hip", "parent": ""}, {"name": "abdomen", "parent": "hip"}],
            "dev_mock": true
        }),
        "list_keyframes" => serde_json::json!({
            "keyframes": [],
            "dev_mock": true
        }),
        "list_modifiers" => serde_json::json!({
            "modifiers": [],
            "dev_mock": true
        }),
        "add_node" | "add_figure" => serde_json::json!({
            "node_id": format!("Mock{}_Created", args.get("figure_type").or(args.get("type")).and_then(Value::as_str).unwrap_or("Node")),
            "dev_mock": true
        }),
        "select_node" => serde_json::json!({ "selected": true, "dev_mock": true }),
        "delete_node" => serde_json::json!({ "deleted": true, "dev_mock": true }),
        "set_property" | "set_morph" | "set_light" | "set_material_property" | "set_camera" => {
            serde_json::json!({ "set": true, "dev_mock": true })
        },
        "set_body_opacity" => serde_json::json!({
            "set": true,
            "matched_count": 2,
            "surfaces": ["Skin", "Torso"],
            "value": args.get("value"),
            "dev_mock": true
        }),
        "set_surface_opacity" => serde_json::json!({
            "set": true,
            "matched_count": 1,
            "surfaces": [args.get("surface_pattern").and_then(Value::as_str).unwrap_or("Surface")],
            "value": args.get("value"),
            "dev_mock": true
        }),
        "get_internal_surfaces" => serde_json::json!({
            "surfaces": ["Skull", "Ribcage", "Spine", "Pelvis"],
            "count": 4,
            "dev_mock": true
        }),
        "show_anatomy" => serde_json::json!({
            "shown": true,
            "matched_count": 4,
            "surfaces": ["Skull", "Ribcage", "Spine", "Pelvis"],
            "dev_mock": true
        }),
        "place_asset_inside" => serde_json::json!({
            "placed": true,
            "figure_id": args.get("figure_id"),
            "node_id": "MockPlacedAsset",
            "asset_path": args.get("asset_path"),
            "position": [0.0, 0.99, 0.0],
            "dev_mock": true
        }),
        "apply_morph" | "apply_expression" | "apply_pose" => {
            serde_json::json!({ "applied": true, "dev_mock": true })
        },
        "set_node_transform" => serde_json::json!({ "transformed": true, "dev_mock": true }),
        "set_bone_transform" => serde_json::json!({ "transformed": true, "dev_mock": true }),
        "set_keyframe" => serde_json::json!({ "keyframe_set": true, "dev_mock": true }),
        "set_timeline_range" => serde_json::json!({ "range_set": true, "dev_mock": true }),
        "seek_to_frame" => {
            serde_json::json!({ "seeked": true, "frame": args.get("frame"), "dev_mock": true })
        },
        "set_render_settings" => serde_json::json!({
            "width": args.get("width"), "height": args.get("height"),
            "applied": true, "dev_mock": true
        }),
        "set_render_options" => serde_json::json!({ "applied": true, "dev_mock": true }),
        "set_viewport_mode" => serde_json::json!({ "mode_set": true, "dev_mock": true }),
        "set_material_texture" => serde_json::json!({ "texture_set": true, "dev_mock": true }),
        "begin_undo_batch" => serde_json::json!({ "batch_started": true, "dev_mock": true }),
        "accept_undo_batch" => serde_json::json!({ "batch_accepted": true, "dev_mock": true }),
        "cancel_undo_batch" => serde_json::json!({ "batch_cancelled": true, "dev_mock": true }),
        "save_scene" => serde_json::json!({ "saved": true, "dev_mock": true }),
        "load_scene" => serde_json::json!({ "loaded": true, "dev_mock": true }),
        "clear_scene" => serde_json::json!({ "cleared": true, "dev_mock": true }),
        "render_preview" => serde_json::json!({ "requested": true, "dev_mock": true }),
        "run_script" => serde_json::json!({
            "result": "Script executed (dev mock)",
            "success": true,
            "dev_mock": true
        }),
        "load_asset" => serde_json::json!({
            "loaded": true,
            "path": args.get("path"),
            "dev_mock": true
        }),
        "import_model" => serde_json::json!({ "imported": true, "dev_mock": true }),
        "export_scene" => serde_json::json!({ "exported": true, "dev_mock": true }),
        "viewport_click" => serde_json::json!({
            "node": "MockFigure", "x": args.get("x"), "y": args.get("y"),
            "dev_mock": true
        }),
        "search_content" => serde_json::json!({
            "results": [{"name": "MockAsset", "path": "/Mock/MockAsset.duf", "type": "Figure"}],
            "dev_mock": true
        }),
        "run_dforce_simulation" => serde_json::json!({ "simulated": true, "dev_mock": true }),
        "apply_phy_modifier" => serde_json::json!({ "modifier_applied": true, "dev_mock": true }),
        "remove_phy_modifier" => serde_json::json!({ "modifier_removed": true, "dev_mock": true }),
        "set_phy_modifier_params" => serde_json::json!({ "params_set": true, "dev_mock": true }),
        _ => serde_json::json!({ "command": command, "args": args, "dev_mock": true }),
    };

    Ok(McpResponse {
        status: "ok".to_string(),
        result: Some(format!("Dev mock executed '{}'", command)),
        commands: if command == "get_commands" {
            Some(get_mcp_command_list())
        } else {
            None
        },
        data: Some(data),
        error: None,
    })
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
    #[serial]
    fn status_never_reports_mock() {
        std::env::remove_var("DAZPILOT_DEV_MOCK_BRIDGE");
        assert!(matches!(
            check_connection_status().as_str(),
            "connected" | "disconnected" | "error"
        ));
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
        std::env::remove_var("DAZPILOT_DEV_MOCK_BRIDGE");
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
    #[serial]
    fn mock_bridge_provides_valid_responses() {
        std::env::set_var("DAZPILOT_DEV_MOCK_BRIDGE", "1");
        let resp = send_mcp_request("get_scene_info", serde_json::json!({}));
        assert!(resp.is_ok());
        let resp = resp.unwrap();
        assert_eq!(resp.status, "ok");
        assert!(resp.data.is_some());
        std::env::remove_var("DAZPILOT_DEV_MOCK_BRIDGE");
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
    #[serial]
    fn acceptance_mock_bridge_core_commands() {
        std::env::set_var("DAZPILOT_DEV_MOCK_BRIDGE", "1");
        let commands = [
            ("get_scene_info", serde_json::json!({})),
            ("list_nodes", serde_json::json!({})),
            ("get_scene_assets", serde_json::json!({})),
            (
                "add_figure",
                serde_json::json!({ "figure_type": "genesis9" }),
            ),
            (
                "set_morph",
                serde_json::json!({
                    "node_id": "Genesis",
                    "morph": "Fitness",
                    "value": "0.5"
                }),
            ),
            (
                "set_light",
                serde_json::json!({
                    "node_id": "Light 1",
                    "property": "Intensity",
                    "value": "1.2"
                }),
            ),
            (
                "set_render_settings",
                serde_json::json!({
                    "width": "1920",
                    "height": "1080"
                }),
            ),
            ("play_timeline", serde_json::json!({})),
            ("pause_timeline", serde_json::json!({})),
            ("stop_timeline", serde_json::json!({})),
            ("get_timeline_state", serde_json::json!({})),
            (
                "get_figure_morphs",
                serde_json::json!({ "figure_id": "Genesis 9" }),
            ),
            (
                "apply_morph",
                serde_json::json!({ "figure_id": "Genesis 9", "morph_id": "test", "value": 0.5 }),
            ),
            (
                "set_body_opacity",
                serde_json::json!({ "node_id": "selected", "value": 0.15 }),
            ),
            (
                "set_surface_opacity",
                serde_json::json!({ "node_id": "selected", "surface_pattern": "torso", "value": 0.05 }),
            ),
            (
                "get_internal_surfaces",
                serde_json::json!({ "node_id": "selected" }),
            ),
            ("show_anatomy", serde_json::json!({ "node_id": "selected" })),
            (
                "place_asset_inside",
                serde_json::json!({ "figure_id": "selected", "asset_path": "/Mock/Alien.duf" }),
            ),
        ];
        for (cmd, args) in commands {
            let resp = send_mcp_request(cmd, args);
            assert!(resp.is_ok(), "acceptance failed for {}", cmd);
            assert_eq!(resp.unwrap().status, "ok");
        }
        std::env::remove_var("DAZPILOT_DEV_MOCK_BRIDGE");
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
            commands.len() >= 30,
            "Should have at least 30 commands, got {}",
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
        assert!(names.len() >= 55, "Should have at least 55 commands");
        assert!(names.contains(&"get_scene_info"));
        assert!(names.contains(&"list_nodes"));
        assert!(names.contains(&"load_asset"));
        assert!(names.contains(&"run_script"));
        assert!(names.contains(&"export_scene"));
    }
}
