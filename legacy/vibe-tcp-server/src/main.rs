use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use serde::Serialize;

type SharedState = Arc<Mutex<ServerState>>;

#[derive(Default)]
struct ServerState {
    connected_clients: usize,
    current_scene: Option<String>,
}

#[derive(Serialize)]
struct Response {
    status: String,
    message: Option<String>,
    commands: Option<Vec<Command>>,
}

#[derive(Serialize)]
struct Command {
    name: String,
    description: String,
    category: String,
}

fn get_commands() -> Vec<Command> {
    vec![
        Command { name: "get_scene_info".into(), description: "Get current scene info".into(), category: "Scene".into() },
        Command { name: "list_nodes".into(), description: "List all nodes".into(), category: "Scene".into() },
        Command { name: "select_node".into(), description: "Select a node".into(), category: "Selection".into() },
        Command { name: "get_selected_nodes".into(), description: "Get selected nodes".into(), category: "Selection".into() },
        Command { name: "load_asset".into(), description: "Load an asset".into(), category: "Assets".into() },
        Command { name: "apply_pose".into(), description: "Apply pose to figure".into(), category: "Figure".into() },
        Command { name: "render_preview".into(), description: "Render preview".into(), category: "Render".into() },
        Command { name: "get_cameras".into(), description: "List cameras".into(), category: "Camera".into() },
    ]
}

async fn handle_client(mut stream: TcpStream, state: SharedState) {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => {
                let cmd = line.trim();
                println!("Received: {}", cmd);
                
                let response = match cmd {
                    "ping" => Response {
                        status: "ok".into(),
                        message: Some("pong".into()),
                        commands: None,
                    },
                    "get_commands" => Response {
                        status: "ok".into(),
                        message: None,
                        commands: Some(get_commands()),
                    },
                    "get_scene_info" => Response {
                        status: "ok".into(),
                        message: Some("Scene: Genesis 8 Female, 3 lights, 1 camera".into()),
                        commands: None,
                    },
                    _ => Response {
                        status: "ok".into(),
                        message: Some(format!("Executed: {}", cmd)),
                        commands: None,
                    },
                };
                
                let json = serde_json::to_string(&response).unwrap();
                writer.write_all(json.as_bytes()).await.unwrap();
                writer.write_all(b"\n").await.unwrap();
            }
            Err(e) => {
                println!("Error reading: {}", e);
                break;
            }
        }
    }
    
    let mut s = state.lock().await;
    s.connected_clients -= 1;
    println!("Client disconnected");
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let addr = "127.0.0.1:8765";
    let listener = TcpListener::bind(addr).await?;
    
    let state: SharedState = Arc::new(Mutex::new(ServerState::default()));
    
    println!("Vibe TCP Server listening on {}", addr);
    println!("Waiting for connections from Daz3D Vibe app...");
    
    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                println!("Client connected from {}", peer_addr);
                
                let state = state.clone();
                let mut s = state.lock().await;
                s.connected_clients += 1;
                drop(s);
                
                tokio::spawn(async move {
                    handle_client(stream, state).await;
                });
            }
            Err(e) => {
                eprintln!("Accept error: {}", e);
            }
        }
    }
}