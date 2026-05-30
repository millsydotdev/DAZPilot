//! Scene Composition Agent: orchestrates multi-step scene creation.
//!
//! This agent has been rewritten to use SceneKnowledgeBase and WorkflowKnowledgeBase
//! to generate true multi-step composition plans from natural language descriptions.
//!
//! Instead of the previous stub implementation which only handled "cyberpunk" with 2 steps,
//! this version can handle fantasy, sci-fi, casual, formal, and many other scene types
//! with full multi-step composition workflows.

use crate::agents::{AgentAction, AgentRequest, AgentResponse};
use crate::knowledge::scene_knowledge::SceneKnowledgeBase;
use crate::knowledge::workflow_knowledge::{
    action_to_command_map, ActionType, WorkflowKnowledgeBase, WorkflowType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionPlan {
    pub steps: Vec<CompositionStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionStep {
    pub description: String,
    pub action: AgentAction,
}

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    let scene_kb = SceneKnowledgeBase::new();
    let workflow_kb = WorkflowKnowledgeBase::new();
    let command_map = action_to_command_map();

    // Step 1: Infer scene type from user input using scene knowledge
    let scene_type = scene_kb.infer_scene_type(&input);

    // Step 2: Extract parameters from input for workflow generation
    let mut params: HashMap<String, String> = HashMap::new();

    // Detect figure type
    if input.contains("genesis 9") || input.contains("genesis9") || input.contains("g9") {
        params.insert("figure_type".to_string(), "genesis9".to_string());
    } else if input.contains("genesis 8") || input.contains("genesis8") || input.contains("g8") {
        params.insert("figure_type".to_string(), "genesis8".to_string());
    } else {
        params.insert("figure_type".to_string(), "genesis9".to_string()); // Default to Genesis 9
    }

    // Detect clothing/fashion intent
    if input.contains("clothing")
        || input.contains("outfit")
        || input.contains("dress")
        || input.contains("wear")
        || input.contains("fashion")
        || input.contains("costume")
    {
        params.insert("asset_type".to_string(), "clothing".to_string());
    }

    // Detect environment/background intent
    if input.contains("environment")
        || input.contains("background")
        || input.contains("scene")
        || input.contains("setting")
        || input.contains("location")
    {
        params.insert("asset_type".to_string(), "environment".to_string());
    }

    // Detect pose intent
    if input.contains("pose")
        || input.contains("standing")
        || input.contains("sitting")
        || input.contains("leaning")
        || input.contains("walking")
        || input.contains("running")
    {
        params.insert("pose_type".to_string(), "basic".to_string());
    }

    // Detect lighting intent
    if input.contains("light")
        || input.contains("lighting")
        || input.contains("illuminate")
        || input.contains("bright")
        || input.contains("dim")
        || input.contains("warm")
        || input.contains("cool")
    {
        // Lighting will be handled by workflow templates
    }

    // Detect multi-figure intent
    if input.contains(" and ")
        || input.contains("with")
        || input.contains("group")
        || input.contains("two")
        || input.contains("multiple")
        || input.contains("several")
        || input.contains("couple")
        || input.contains("pair")
        || input.contains("trio")
    {
        params.insert("figure_count".to_string(), "2".to_string());
    }

    // Step 3: Get scene understanding for enrichment
    let scene_understanding = scene_type
        .as_ref()
        .and_then(|st| scene_kb.get_scene_understanding(*st, None));

    // Step 4: Determine what type of workflow to generate
    let workflow = {
        // If we have a clear scene type and the user wants to create/compose a scene
        if scene_type.is_some()
            && (input.contains("scene")
                || input.contains("create")
                || input.contains("compose")
                || input.contains("build")
                || input.contains("setup")
                || input.contains("setup")
                || input.contains("make"))
        {
            // Merge scene understanding into parameters for richer workflow
            if let Some(ref understanding) = scene_understanding {
                // Add scene-specific composition rules as a parameter
                let rules = understanding.composition_rules.join("; ");
                params.insert("composition_rules".to_string(), rules);
            }
            workflow_kb.generate_workflow_from_template(WorkflowType::CreateScene, &params)
        }
        // If user wants lighting specifically
        else if input.contains("light")
            || input.contains("lighting")
            || input.contains("illuminate")
            || input.contains("bright")
            || input.contains("dim")
            || input.contains("warm")
            || input.contains("cool")
        {
            workflow_kb.generate_workflow_from_template(WorkflowType::SetupLighting, &params)
        }
        // If user wants to render/preview
        else if input.contains("render")
            || input.contains("preview")
            || input.contains("output")
            || input.contains("image")
        {
            workflow_kb.generate_workflow_from_template(WorkflowType::RenderStill, &params)
        }
        // If user wants posing
        else if input.contains("pose") || input.contains("posing") || input.contains("position") {
            workflow_kb.generate_workflow_from_template(WorkflowType::PoseCharacter, &params)
        }
        // If user wants outfit/clothing
        else if input.contains("outfit")
            || input.contains("clothing")
            || input.contains("dress")
            || input.contains("fashion")
            || input.contains("wear")
            || input.contains("costume")
        {
            workflow_kb.generate_workflow_from_template(WorkflowType::CreateOutfit, &params)
        }
        // If user wants animation
        else if input.contains("animate")
            || input.contains("animation")
            || input.contains("motion")
            || input.contains("move")
        {
            workflow_kb.generate_workflow_from_template(WorkflowType::AnimateCharacter, &params)
        }
        // If user wants to fix something
        else if input.contains("fix")
            || input.contains("issue")
            || input.contains("problem")
            || input.contains("correct")
        {
            workflow_kb.generate_workflow_from_template(WorkflowType::FixCommonIssue, &params)
        }
        // Default fallback - try to create a scene
        else {
            workflow_kb.generate_workflow_from_template(WorkflowType::CreateScene, &params)
        }
    };

    // Step 5: Convert workflow steps to AgentActions
    #[allow(clippy::unnecessary_filter_map)]
    let steps: Vec<CompositionStep> = match workflow {
        Some(wf) => {
            wf.steps
                .iter()
                .filter_map(|step| {
                    // Look up the command mapping for this action type
                    let command_info = command_map.get(&step.action_type);
                    let command = command_info
                        .map(|ci| ci.command.to_string())
                        .unwrap_or_else(|| match &step.action_type {
                            ActionType::Custom(cmd) => cmd.clone(),
                            _ => "unknown_command".to_string(),
                        });

                    // Build args from parameters using semantic mapping
                    let mut args: Vec<String> = Vec::new();

                    // Map parameters based on action type and command expectations
                    match (&step.action_type, command.as_str()) {
                        (ActionType::AddFigure, "add_figure") => {
                            args.push(
                                params
                                    .get("figure_type")
                                    .cloned()
                                    .unwrap_or_else(|| "genesis9".to_string()),
                            );
                        },
                        (ActionType::LoadAsset, "load_asset") => {
                            if let Some(asset_type) = step.parameters.get("asset_type") {
                                args.push(asset_type.clone());
                            } else {
                                args.push("asset".to_string());
                            }
                        },
                        (ActionType::CreateLight, "add_node") => {
                            if let Some(purpose) = step.parameters.get("light_purpose") {
                                args.push(purpose.clone());
                            }
                            if let Some(light_type) = step.parameters.get("light_type") {
                                args.push(light_type.clone());
                            }
                            // Also add name if present
                            if let Some(name) = step.parameters.get("name") {
                                args.push(name.clone());
                            }
                        },
                        (ActionType::SetMorph, "set_morph") => {
                            if let Some(morph) = step.parameters.get("morph") {
                                args.push(morph.clone());
                            }
                            if let Some(value) = step.parameters.get("value") {
                                args.push(value.clone());
                            }
                        },
                        (ActionType::ApplyPose, "apply_pose") => {
                            if let Some(pose_type) = step.parameters.get("pose_type") {
                                args.push(pose_type.clone());
                            }
                        },
                        (ActionType::SetCamera, "add_node") => {
                            if let Some(camera) = step.parameters.get("camera") {
                                args.push(camera.clone());
                            }
                            if let Some(focal_length) = step.parameters.get("focal_length") {
                                args.push(focal_length.clone());
                            }
                            if let Some(focal_distance) = step.parameters.get("focal_distance") {
                                args.push(focal_distance.clone());
                            }
                        },
                        (ActionType::SetRenderOptions, "set_render_options") => {
                            if let Some(width) = step.parameters.get("width") {
                                args.push(width.clone());
                            }
                            if let Some(height) = step.parameters.get("height") {
                                args.push(height.clone());
                            }
                            if let Some(samples) = step.parameters.get("pixel_samples") {
                                args.push(samples.clone());
                            }
                        },
                        (ActionType::AdjustProperty, "set_property") => {
                            if let Some(adj_type) = step.parameters.get("adjustment_type") {
                                args.push(adj_type.clone());
                            }
                            if let Some(value) = step.parameters.get("value") {
                                args.push(value.clone());
                            }
                        },
                        (ActionType::UndoBatchBegin, "begin_undo_batch") => {
                            args.push("AI Scene Composition".to_string());
                        },
                        (ActionType::UndoBatchEnd, "accept_undo_batch") => {
                            args.push("AI Created Scene".to_string());
                        },
                        (ActionType::SetMaterialTexture, "set_material_texture") => {
                            if let Some(channel) = step.parameters.get("channel") {
                                args.push(channel.clone());
                            }
                            if let Some(file_path) = step.parameters.get("file_path") {
                                args.push(file_path.clone());
                            }
                        },
                        (ActionType::SetBodyOpacity, "set_body_opacity") => {
                            if let Some(opacity) = step.parameters.get("opacity") {
                                args.push(opacity.clone());
                            }
                        },
                        (ActionType::SetSurfaceOpacity, "set_surface_opacity") => {
                            if let Some(opacity) = step.parameters.get("opacity") {
                                args.push(opacity.clone());
                            }
                        },
                        // For Custom actions, pass through parameters as needed
                        (ActionType::Custom(ref cmd_name), _) => {
                            // Handle custom commands that might need special treatment
                            match cmd_name.as_str() {
                                "clear_scene" => {
                                    // No args needed for clear_scene
                                },
                                "analyze_scene" => {
                                    // No args needed
                                },
                                "setup_timeline" => {
                                    if let Some(start_frame) = step.parameters.get("start_frame") {
                                        args.push(start_frame.clone());
                                    }
                                    if let Some(end_frame) = step.parameters.get("end_frame") {
                                        args.push(end_frame.clone());
                                    }
                                },
                                "pose_keyframes" => {
                                    // Pose keyframes would need more complex handling
                                },
                                "add_secondary_motion" => {
                                    // Secondary motion would need complex handling
                                },
                                "frame_shot" => {
                                    // No args needed
                                },
                                "final_checks" => {
                                    // No args needed
                                },
                                "identify_issue" => {
                                    // No args needed
                                },
                                "gather_info" => {
                                    // No args needed
                                },
                                "apply_fix" => {
                                    // No args needed
                                },
                                "verify_fix" => {
                                    // No args needed
                                },
                                "select_pose" => {
                                    // No args needed
                                },
                                _ => {
                                    // For unknown custom commands, pass all parameter values
                                    for value in step.parameters.values() {
                                        args.push(value.clone());
                                    }
                                },
                            }
                        },
                        // Fallback: pass all parameter values as args
                        _ => {
                            for value in step.parameters.values() {
                                args.push(value.clone());
                            }
                        },
                    }

                    Some(CompositionStep {
                        description: step.description.clone(),
                        action: AgentAction {
                            action_type: format!("{:?}", step.action_type),
                            command,
                            args,
                        },
                    })
                })
                .collect()
        },
        None => vec![],
    };

    let figure_count: u32 = params
        .get("figure_count")
        .and_then(|c| c.parse().ok())
        .unwrap_or(1);
    let steps = expand_multi_figure_steps(steps, figure_count);

    if steps.is_empty() {
        return AgentResponse {
            success: false,
            result: None,
            error: Some(format!("Could not compose scene from '{}'. Try describing a scene more specifically (e.g., 'fantasy wizard tower', 'modern interior', 'sci-fi spaceship').", request.input)),
            actions: vec![],
            sub_results: vec![],
        };
    }

    // Build informative result message
    let scene_type_info = scene_type
        .map(|st| format!("{:?}", st))
        .unwrap_or_else(|| "generic".to_string());
    let understanding_info = scene_understanding
        .as_ref()
        .map(|u| {
            format!(
                " ({} complexity, {} lights)",
                match u.complexity {
                    crate::knowledge::scene_knowledge::ComplexityLevel::Beginner => "beginner",
                    crate::knowledge::scene_knowledge::ComplexityLevel::Intermediate =>
                        "intermediate",
                    crate::knowledge::scene_knowledge::ComplexityLevel::Advanced => "advanced",
                },
                u.lighting_setup.len(),
            )
        })
        .unwrap_or_default();

    AgentResponse {
        success: true,
        result: Some(format!(
            "Composed {} scene with {} steps{}",
            scene_type_info,
            steps.len(),
            understanding_info,
        )),
        error: None,
        actions: steps.into_iter().map(|s| s.action).collect(),
        sub_results: vec![],
    }
}

/// Duplicate figure-loading steps when the user requests multiple characters.
fn expand_multi_figure_steps(
    steps: Vec<CompositionStep>,
    figure_count: u32,
) -> Vec<CompositionStep> {
    if figure_count <= 1 {
        return steps;
    }
    let mut expanded = Vec::new();
    for step in steps {
        if step.action.command == "add_figure" {
            for i in 1..=figure_count {
                let mut dup = step.clone();
                if i > 1 {
                    dup.description = format!("{} (figure {i})", step.description);
                }
                expanded.push(dup);
            }
        } else {
            expanded.push(step);
        }
    }
    expanded
}
