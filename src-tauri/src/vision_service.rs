use serde::{Deserialize, Serialize};
use crate::mcp_client;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneAnalysis {
    pub description: String,
    pub detected_nodes: Vec<String>,
    pub lighting_style: String,
    pub primary_subject: Option<String>,
}

pub async fn analyze_current_viewport() -> Result<SceneAnalysis, String> {
    let temp_dir = std::env::temp_dir();
    let capture_path = temp_dir.join(format!("daz_capture_{}.png", chrono::Utc::now().timestamp()));
    let path_str = capture_path.to_string_lossy().to_string();

    // 1. Capture the viewport
    mcp_client::send_mcp_request("capture_viewport", serde_json::json!({ "path": path_str }))?;

    // 2. Perform AI Analysis with Ollama Vision Model
    let image_bytes = std::fs::read(&capture_path).map_err(|e| format!("Failed to read capture: {}", e))?;
    use base64::{Engine as _, engine::general_purpose};
    let base64_image = general_purpose::STANDARD.encode(&image_bytes);
    
    let messages = vec![crate::ollama_service::ChatMessage {
        role: "user".to_string(),
        content: "Analyze this 3D scene image. Describe the scene, list the detected nodes, specify the lighting style, and identify the primary subject. Return valid JSON matching this schema: {\"description\": \"...\", \"detected_nodes\": [\"...\"], \"lighting_style\": \"...\", \"primary_subject\": \"...\"}".to_string(),
        images: Some(vec![base64_image]),
    }];
    
    let ollama = crate::ollama_service::OllamaService::new();
    let response_result = ollama.chat("llava", messages, 0.7).await;
    
    let response = match response_result {
        Ok(res) => res,
        Err(_) => {
            return Ok(SceneAnalysis {
                description: "Vision backend (Ollama) is not running or the 'llava' model is not available. Please ensure Ollama is installed and running with the llava model to enable AI eyes.".to_string(),
                detected_nodes: vec![],
                lighting_style: "Unknown".to_string(),
                primary_subject: None,
            });
        }
    };
    
    // Attempt to parse JSON from the response.content
    let content = response.message.content;
    
    let json_str = if content.contains("```json") {
        content.split("```json").nth(1).unwrap_or(&content).split("```").next().unwrap_or(&content)
    } else if content.contains("```") {
        content.split("```").nth(1).unwrap_or(&content).split("```").next().unwrap_or(&content)
    } else {
        &content
    };
    
    match serde_json::from_str::<SceneAnalysis>(json_str.trim()) {
        Ok(analysis) => Ok(analysis),
        Err(_) => {
            // Fallback if parsing fails
            Ok(SceneAnalysis {
                description: content,
                detected_nodes: vec![],
                lighting_style: "Unknown".to_string(),
                primary_subject: None,
            })
        }
    }
}

pub fn get_capture_path() -> PathBuf {
    std::env::temp_dir().join("daz_latest_capture.png")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub node_id: String,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub center: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBoxesResponse {
    pub bounds: Vec<BoundingBox>,
}

pub fn generate_spatial_relationships(bounds: &[BoundingBox]) -> String {
    let mut relations = Vec::new();
    for i in 0..bounds.len() {
        for j in 0..bounds.len() {
            if i == j { continue; }
            let a = &bounds[i];
            let b = &bounds[j];
            
            let dx = a.center[0] - b.center[0];
            let dy = a.center[1] - b.center[1];
            let dz = a.center[2] - b.center[2];
            
            let dist = (dx*dx + dy*dy + dz*dz).sqrt();
            
            if dist < 300.0 {
                let mut rels = Vec::new();
                
                if dx > 15.0 {
                    rels.push("to the left of");
                } else if dx < -15.0 {
                    rels.push("to the right of");
                }
                
                if dz > 15.0 {
                    rels.push("in front of");
                } else if dz < -15.0 {
                    rels.push("behind");
                }
                
                if dy > 15.0 {
                    rels.push("above");
                } else if dy < -15.0 {
                    rels.push("below");
                }
                
                if !rels.is_empty() {
                    relations.push(format!("- '{}' is {} '{}' (distance: {:.1}cm)", a.node_id, rels.join(" and "), b.node_id, dist));
                }
            }
        }
    }
    relations.join("\n")
}

pub fn fetch_spatial_context() -> String {
    let result = mcp_client::send_mcp_request("get_bounding_boxes", serde_json::json!({}));
    let response = match result {
        Ok(res) => res,
        Err(_) => return "Spatial Awareness Error: Daz Studio is disconnected or not responding.".to_string(),
    };
    
    let raw_data = response.data.map(|d| d.to_string()).unwrap_or_else(|| "{}".to_string());
    let bounds_resp: BoundingBoxesResponse = match serde_json::from_str(&raw_data) {
        Ok(r) => r,
        Err(_) => return "Spatial Awareness Error: Failed to parse bounding box data.".to_string(),
    };
    
    if bounds_resp.bounds.is_empty() {
        return "Spatial Context: No objects detected in the current Daz Studio scene.".to_string();
    }
    
    let mut ctx = String::new();
    ctx.push_str("### 3D Spatial Layout Context\n\n");
    ctx.push_str("Here are the mathematically pre-calculated 3D positions, bounding boxes, and relative relationships between objects in the active Daz Studio scene:\n\n");
    
    ctx.push_str("| Node Name / ID | Center Position (X, Y, Z) | Size (Width, Height, Depth) |\n");
    ctx.push_str("| --- | --- | --- |\n");
    for box_item in &bounds_resp.bounds {
        let w = (box_item.max[0] - box_item.min[0]).abs();
        let h = (box_item.max[1] - box_item.min[1]).abs();
        let d = (box_item.max[2] - box_item.min[2]).abs();
        ctx.push_str(&format!(
            "| {} | ({:.1}, {:.1}, {:.1}) | ({:.1}cm x {:.1}cm x {:.1}cm) |\n",
            box_item.node_id, box_item.center[0], box_item.center[1], box_item.center[2], w, h, d
        ));
    }
    ctx.push_str("\n");
    
    let rels_str = generate_spatial_relationships(&bounds_resp.bounds);
    if !rels_str.is_empty() {
        ctx.push_str("#### Relative Object Directions:\n");
        ctx.push_str(&rels_str);
        ctx.push_str("\n");
    }
    
    ctx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_relationships() {
        let a = BoundingBox {
            node_id: "Genesis 8 Female".to_string(),
            min: [-10.0, 0.0, -10.0],
            max: [10.0, 180.0, 10.0],
            center: [0.0, 90.0, 0.0],
        };
        
        let b = BoundingBox {
            node_id: "Chair".to_string(),
            min: [-50.0, 0.0, 30.0],
            max: [-30.0, 60.0, 50.0],
            center: [-40.0, 30.0, 40.0],
        };
        
        let bounds = vec![a.clone(), b.clone()];
        let relations = generate_spatial_relationships(&bounds);
        
        println!("Calculated relations:\n{}", relations);
        
        assert!(relations.contains("'Genesis 8 Female' is to the left of and behind and above 'Chair'"));
        assert!(relations.contains("'Chair' is to the right of and in front of and below 'Genesis 8 Female'"));
    }
}
