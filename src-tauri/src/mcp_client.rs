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
        name: "run_dforce_simulation",
        description: "Run a dForce physics simulation via inline DAZ Script",
        category: "Animation",
        parameters: &["node_id", "start_frame", "end_frame"],
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
        description: "Get list of loaded assets in the current scene",
        category: "Scene",
        parameters: &[],
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
            &addr.parse().map_err(|e| format!("Invalid address: {}", e))?,
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
                }
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
        return Err("Not connected to Daz3D. Start Daz Studio with DazPilotBridge loaded, then connect.".to_string());
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
        }
        Err(e) => {
            if conn.reconnect().is_ok() {
                conn.send_json(&request)
            } else {
                *global = None;
                Err(format!("Daz bridge connection lost: {}", e))
            }
        }
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

    #[test]
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
        assert!(validate_command("select_node", &serde_json::json!({ "node_id": "Genesis" })).is_ok());
    }

    #[test]
    fn rejects_unknown_commands() {
        assert!(validate_command("pretend_success", &serde_json::json!({})).is_err());
    }
}
