//! Conflict Resolution Agent: detects and proposes fixes for scene/asset conflicts.

use crate::agents::{AgentRequest, AgentResponse, AgentAction};
use crate::vision_service;

pub fn execute(_request: AgentRequest) -> AgentResponse {
    let report = vision_service::detect_asset_conflicts_from_scene();
    
    if !report.has_conflicts {
        return AgentResponse {
            success: true,
            result: Some("No conflicts detected in the current scene.".to_string()),
            error: None,
            actions: vec![],
        };
    }

    let mut actions = vec![];
    let mut results = vec![];

    for conflict in report.conflicts {
        results.push(format!("[{}] {}: {}", conflict.severity, conflict.conflict_type, conflict.fix_suggestion));
        
        // If it's a shell conflict, we can propose a fix action
        if conflict.conflict_type == "Multiple_Shells_Detected" {
            actions.push(AgentAction {
                action_type: "fix_conflict".to_string(),
                command: "fix_shell_zones".to_string(),
                args: vec![conflict.assets.first().cloned().unwrap_or_default(), "AI_FIX_".to_string()],
            });
        }
    }

    AgentResponse {
        success: true,
        result: Some(results.join("\n")),
        error: None,
        actions,
    }
}
