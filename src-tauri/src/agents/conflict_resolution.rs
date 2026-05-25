use crate::agents::{AgentAction, AgentRequest, AgentResponse};
use crate::asset_fixer;
use crate::mcp_client;
use crate::vision_service;

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input_lower = request.input.to_lowercase();

    let wants_scan = input_lower.contains("scan")
        || input_lower.contains("detect")
        || input_lower.contains("check");
    let wants_fix = input_lower.contains("fix")
        || input_lower.contains("resolve")
        || input_lower.contains("repair")
        || input_lower.contains("auto-fix");
    let wants_status = input_lower.contains("status")
        || input_lower.contains("report")
        || input_lower.contains("summary");

    if wants_scan || (!wants_fix && !wants_status) {
        let scene_report = vision_service::detect_asset_conflicts_from_scene();

        if !scene_report.has_conflicts {
            return AgentResponse {
                success: true,
                result: Some("No conflicts detected in the current scene.".to_string()),
                error: None,
                actions: vec![],
                sub_results: vec![],
            };
        }

        let mut results = vec![];
        for conflict in &scene_report.conflicts {
            results.push(format!(
                "[{}] {}: {}",
                conflict.severity, conflict.conflict_type, conflict.fix_suggestion
            ));
        }

        return AgentResponse {
            success: true,
            result: Some(format!(
                "Conflicts detected ({}):\n{}",
                scene_report.conflicts.len(),
                results.join("\n")
            )),
            error: None,
            actions: vec![],
            sub_results: vec![],
        };
    }

    if wants_status {
        let scene_report = vision_service::detect_asset_conflicts_from_scene();
        if scene_report.has_conflicts {
            let conflict_list: Vec<String> = scene_report
                .conflicts
                .iter()
                .map(|c| {
                    format!(
                        "[{}] {} affecting {} asset(s)",
                        c.severity,
                        c.conflict_type,
                        c.assets.len()
                    )
                })
                .collect();
            return AgentResponse {
                success: true,
                result: Some(format!(
                    "Conflict report: {} conflict(s) found.\n{}",
                    scene_report.conflicts.len(),
                    conflict_list.join("\n")
                )),
                error: None,
                actions: vec![],
                sub_results: vec![],
            };
        }
        return AgentResponse {
            success: true,
            result: Some("No conflicts in the current scene.".to_string()),
            error: None,
            actions: vec![],
            sub_results: vec![],
        };
    }

    if wants_fix {
        return execute_fix(request);
    }

    let scene_report = vision_service::detect_asset_conflicts_from_scene();

    if !scene_report.has_conflicts {
        return AgentResponse {
            success: true,
            result: Some("No conflicts detected in the current scene.".to_string()),
            error: None,
            actions: vec![],
            sub_results: vec![],
        };
    }

    let mut actions = vec![];
    let mut results = vec![];

    for conflict in scene_report.conflicts {
        results.push(format!(
            "[{}] {}: {}",
            conflict.severity, conflict.conflict_type, conflict.fix_suggestion
        ));

        match conflict.conflict_type.as_str() {
            "Multiple_Shells_Detected" | "MaterialZone" => {
                if let Some(first_asset) = conflict.assets.first() {
                    let prefix = asset_fixer::detect_prefix_from_conflict(first_asset);
                    actions.push(AgentAction {
                        action_type: "fix_conflict".to_string(),
                        command: "fix_shell_zones".to_string(),
                        args: vec![first_asset.clone(), prefix],
                    });
                }
            },
            "MorphId" => {
                if let Some(first_asset) = conflict.assets.first() {
                    actions.push(AgentAction {
                        action_type: "fix_conflict".to_string(),
                        command: "fix_morph_ids".to_string(),
                        args: vec![first_asset.clone()],
                    });
                }
            },
            "UVSet" => {
                if let Some(first_asset) = conflict.assets.first() {
                    actions.push(AgentAction {
                        action_type: "fix_conflict".to_string(),
                        command: "fix_uv_sets".to_string(),
                        args: vec![first_asset.clone()],
                    });
                }
            },
            _ => {
                actions.push(AgentAction {
                    action_type: "warn_conflict".to_string(),
                    command: "chat".to_string(),
                    args: vec![format!("Conflict detected: {}", conflict.fix_suggestion)],
                });
            },
        }
    }

    AgentResponse {
        success: true,
        result: Some(results.join("\n")),
        error: None,
        actions,
        sub_results: vec![],
    }
}

fn execute_fix(_request: AgentRequest) -> AgentResponse {
    let scene_report = vision_service::detect_asset_conflicts_from_scene();

    if !scene_report.has_conflicts {
        return AgentResponse {
            success: true,
            result: Some("No conflicts to fix.".to_string()),
            error: None,
            actions: vec![],
            sub_results: vec![],
        };
    }

    let mut actions = vec![];
    let mut results = vec![];

    for conflict in scene_report.conflicts {
        match conflict.conflict_type.as_str() {
            "Multiple_Shells_Detected" | "MaterialZone" => {
                if let Some(first_asset) = conflict.assets.first() {
                    let prefix = asset_fixer::detect_prefix_from_conflict(first_asset);
                    actions.push(AgentAction {
                        action_type: "fix_conflict".to_string(),
                        command: "fix_shell_zones".to_string(),
                        args: vec![first_asset.clone(), prefix],
                    });
                    results.push(format!("Fixed shell zone conflict on {}", first_asset));
                }
            },
            "MorphId" => {
                if let Some(first_asset) = conflict.assets.first() {
                    actions.push(AgentAction {
                        action_type: "fix_conflict".to_string(),
                        command: "fix_morph_ids".to_string(),
                        args: vec![first_asset.clone()],
                    });
                    results.push(format!("Fixed morph ID conflict on {}", first_asset));
                }
            },
            "UVSet" => {
                if let Some(first_asset) = conflict.assets.first() {
                    actions.push(AgentAction {
                        action_type: "fix_conflict".to_string(),
                        command: "fix_uv_sets".to_string(),
                        args: vec![first_asset.clone()],
                    });
                    results.push(format!("Fixed UV set conflict on {}", first_asset));
                }
            },
            _ => {
                results.push(format!(
                    "Cannot auto-fix {} conflict: {}",
                    conflict.conflict_type, conflict.fix_suggestion
                ));
            },
        }
    }

    AgentResponse {
        success: true,
        result: Some(format!(
            "Applied {} fix(es):\n{}",
            results.len(),
            results.join("\n")
        )),
        error: None,
        actions,
        sub_results: vec![],
    }
}

/// Pre-screen an asset before loading it for known conflict types.
/// Called by the AI agent before executing load_asset.
pub fn check_before_load(asset_path: &str) -> asset_fixer::ConflictScanResult {
    let path = std::path::Path::new(asset_path);
    let parent = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    if parent.is_empty() {
        return asset_fixer::ConflictScanResult {
            total_scanned: 0,
            conflicts: vec![],
            warnings: vec!["Cannot determine parent directory for pre-scan".to_string()],
        };
    }
    asset_fixer::scan_asset_conflicts(&parent)
}

/// Detect geoshells in the scene via bridge command
pub fn detect_geoshells() -> Vec<String> {
    match mcp_client::send_mcp_request("get_geoshells", serde_json::json!({})) {
        Ok(resp) => {
            if let Some(data) = resp.data {
                if let Ok(shells) = serde_json::from_value::<Vec<String>>(data) {
                    return shells;
                }
            }
            vec![]
        },
        Err(_) => vec![],
    }
}
