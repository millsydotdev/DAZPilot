use crate::agents::{AgentRequest, AgentResponse, AgentAction};
use crate::vision_service;
use crate::asset_fixer;

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

        match conflict.conflict_type.as_str() {
            "Multiple_Shells_Detected" | "MaterialZone" => {
                if let Some(first_asset) = conflict.assets.first() {
                    actions.push(AgentAction {
                        action_type: "fix_conflict".to_string(),
                        command: "fix_shell_zones".to_string(),
                        args: vec![first_asset.clone(), "AI_FIX_".to_string()],
                    });
                }
            }
            "MorphId" => {
                results.push("Morph ID conflict detected across assets. Recommendation: ensure each morph file uses unique morph IDs.".to_string());
                actions.push(AgentAction {
                    action_type: "warn_conflict".to_string(),
                    command: "chat".to_string(),
                    args: vec![format!("Morph ID conflict: {}", conflict.fix_suggestion)],
                });
            }
            "UVSet" => {
                results.push("UV set name overlap detected. Recommendation: check UV set naming in your assets.".to_string());
                actions.push(AgentAction {
                    action_type: "warn_conflict".to_string(),
                    command: "chat".to_string(),
                    args: vec![format!("UV conflict: {}", conflict.fix_suggestion)],
                });
            }
            _ => {
                actions.push(AgentAction {
                    action_type: "warn_conflict".to_string(),
                    command: "chat".to_string(),
                    args: vec![format!("Conflict detected: {}", conflict.fix_suggestion)],
                });
            }
        }
    }

    AgentResponse {
        success: true,
        result: Some(results.join("\n")),
        error: None,
        actions,
    }
}

/// Pre-screen an asset before loading it for known conflict types.
/// Called by the AI agent before executing load_asset.
pub fn check_before_load(asset_path: &str) -> asset_fixer::ConflictScanResult {
    // Scan the asset file directly for conflicts
    let path = std::path::Path::new(asset_path);
    let parent = path.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    if parent.is_empty() {
        return asset_fixer::ConflictScanResult {
            total_scanned: 0,
            conflicts: vec![],
            warnings: vec!["Cannot determine parent directory for pre-scan".to_string()],
        };
    }
    asset_fixer::scan_asset_conflicts(&parent)
}
