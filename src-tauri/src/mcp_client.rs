#![allow(dead_code)]

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::TcpStream;
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
        .map_err(|e| format!("Daz bridge connection failed: {}", e))?;

        stream
            .set_read_timeout(Some(Duration::from_secs(20)))
            .map_err(|e| format!("Failed to set read timeout: {}", e))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(10)))
            .map_err(|e| format!("Failed to set write timeout: {}", e))?;

        Ok(Self {
            stream,
            host: host.to_string(),
            port,
        })
    }

    pub fn send_json(&mut self, request: &DazRequest) -> Result<McpResponse, String> {
        let json =
            serde_json::to_string(request).map_err(|e| format!("Failed to serialize: {}", e))?;
        self.stream
            .write_all(format!("{}\n", json).as_bytes())
            .map_err(|e| format!("Failed to send bridge request: {}", e))?;
        self.stream
            .flush()
            .map_err(|e| format!("Failed to flush bridge request: {}", e))?;

        let mut buffer = String::new();
        loop {
            let mut byte = [0u8; 1];
            match self.stream.read(&mut byte) {
                Ok(1) => {
                    if byte[0] == b'\n' {
                        break;
                    }
                    buffer.push(byte[0] as char);
                },
                Ok(_) => break,
                Err(e) => return Err(format!("Failed to read bridge response: {}", e)),
            }
        }

        if buffer.trim().is_empty() {
            return Err("Daz bridge returned an empty response".to_string());
        }

        parse_bridge_response(&buffer)
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
    let error = value
        .get("error")
        .or_else(|| value.get("message").filter(|_| status == "error"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let result = value
        .get("result")
        .or_else(|| value.get("message"))
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
        object.remove("message");
        object.remove("result");
        object.remove("commands");
        if object.is_empty() {
            None
        } else {
            Some(Value::Object(object))
        }
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
            "Not connected to Daz3D. Start Daz Studio with DazPilotBridge loaded, then connect."
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
            "scene": "Dev Mock Scene",
            "nodes": 0,
            "lights": 0,
            "cameras": 0,
            "dev_mock": true
        }),
        "list_nodes" | "get_selected_nodes" => serde_json::json!({ "nodes": [], "dev_mock": true }),
        "get_cameras" => serde_json::json!({ "cameras": [], "dev_mock": true }),
        "capture_viewport" => serde_json::json!({
            "result": "base64",
            "data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
            "dev_mock": true
        }),
        "play_timeline" => serde_json::json!({ "playing": true,  "dev_mock": true }),
        "pause_timeline" => serde_json::json!({ "playing": false, "dev_mock": true }),
        "stop_timeline" => serde_json::json!({ "frame": 0,       "dev_mock": true }),
        "get_timeline_state" => {
            serde_json::json!({ "current_frame": 0, "start_frame": 0, "end_frame": 300, "fps": 30.0, "is_playing": false, "dev_mock": true })
        },
        "get_figure_morphs" => {
            serde_json::json!({ "morphs": [{"id":"testMorph","label":"Test Morph","value":0.0,"min":0.0,"max":1.0,"type":"morph"}], "dev_mock": true })
        },
        "get_fitted_items" => serde_json::json!({ "items": [], "dev_mock": true }),
        "get_active_expressions" => serde_json::json!({ "expressions": [], "dev_mock": true }),
        "get_material_zones" => serde_json::json!({ "materials": [], "dev_mock": true }),
        "apply_morph" => serde_json::json!({ "set": true, "dev_mock": true }),
        "apply_expression" => serde_json::json!({ "set": true, "dev_mock": true }),
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
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:x}{:x}", now.as_secs(), now.subsec_nanos())
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
                    err.contains("Daz bridge connection failed"),
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
        ] {
            assert!(names.contains(&required), "missing schema {}", required);
        }
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
    fn schema_parity_with_cpp_bridge() {
        let rust_commands: std::collections::BTreeSet<&str> =
            COMMAND_SCHEMAS.iter().map(|s| s.name).collect();

        let cpp_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("plugins/daz3d-bridge/DazPilotBridgePlugin.cpp");

        assert!(
            cpp_path.exists(),
            "C++ bridge source not found: {:?}",
            cpp_path
        );

        let source = std::fs::read_to_string(&cpp_path).expect("Failed to read C++ bridge source");

        let mut cpp_commands = std::collections::BTreeSet::new();
        for line in source.lines() {
            let trimmed = line.trim();
            // Match: if (command == "xyz")
            if let Some(start) = trimmed.find("command == \"") {
                let rest = &trimmed[start + 12..];
                if let Some(end) = rest.find('"') {
                    let cmd = &rest[..end];
                    cpp_commands.insert(cmd);
                }
            }
        }

        assert!(
            !cpp_commands.is_empty(),
            "No C++ commands extracted — check regex"
        );

        let only_in_cpp: Vec<&&str> = cpp_commands.difference(&rust_commands).collect();
        let only_in_rust: Vec<&&str> = rust_commands.difference(&cpp_commands).collect();

        if !only_in_cpp.is_empty() || !only_in_rust.is_empty() {
            let mut msg =
                String::from("Schema parity mismatch between C++ bridge and Rust mcp_client:\n");
            if !only_in_cpp.is_empty() {
                let list: Vec<&str> = only_in_cpp.iter().map(|s| **s).collect();
                msg.push_str(&format!(
                    "\n  In C++ but NOT in Rust ({}): {}\n",
                    list.len(),
                    list.join(", ")
                ));
            }
            if !only_in_rust.is_empty() {
                let list: Vec<&str> = only_in_rust.iter().map(|s| **s).collect();
                msg.push_str(&format!(
                    "\n  In Rust but NOT in C++ ({}): {}\n",
                    list.len(),
                    list.join(", ")
                ));
            }
            msg.push_str("\nAdd missing schemas to COMMAND_SCHEMAS in mcp_client.rs or implement the command in DazPilotBridgePlugin.cpp");
            panic!("{}", msg);
        }
    }
}
