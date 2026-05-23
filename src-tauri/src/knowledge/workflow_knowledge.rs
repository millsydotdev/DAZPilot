use crate::knowledge::daz_concepts::DazKnowledgeBase;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a step in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub description: String,
    pub action_type: ActionType,
    pub parameters: HashMap<String, String>,
    pub prerequisites: Vec<String>, // IDs of steps that must complete first
    pub estimated_time_seconds: u32,
    pub difficulty: DifficultyLevel,
}

/// Types of actions that can be in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    LoadAsset,
    ApplyPose,
    AdjustProperty,
    CreateLight,
    CreateCamera,
    Render,
    ChangeMaterial,
    ExportScene,
    RunSimulation,
    UndoBatchBegin,
    UndoBatchEnd,
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Trivial,
    Easy,
    Moderate,
    Hard,
    Expert,
}

/// Represents a complete workflow for achieving a goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub tags: Vec<String>,
    pub success_rate: f32, // 0.0 to 1.0 based on historical usage
    pub usage_count: u32,
    pub last_used: Option<String>, // timestamp
}

/// Knowledge base of common workflows and how to generate them
#[derive(Debug, Clone)]
pub struct WorkflowKnowledgeBase {
    pub daz_knowledge: DazKnowledgeBase,
    pub workflows: HashMap<String, Workflow>,
    pub workflow_templates: HashMap<WorkflowType, WorkflowTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub workflow_type: WorkflowType,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStepTemplate>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStepTemplate {
    pub description: String,
    pub action_type: ActionType,
    pub parameters_template: HashMap<String, String>,
    pub prerequisites: Vec<String>,
    pub estimated_time_seconds: u32,
    pub difficulty: DifficultyLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum WorkflowType {
    CreateCharacter,
    CreateOutfit,
    SetupLighting,
    CreateScene,
    PoseCharacter,
    AnimateCharacter,
    RenderStill,
    RenderAnimation,
    FixCommonIssue,
}

impl WorkflowKnowledgeBase {
    pub fn new() -> Self {
        let mut knowledge = WorkflowKnowledgeBase {
            daz_knowledge: DazKnowledgeBase::new(),
            workflows: HashMap::new(),
            workflow_templates: HashMap::new(),
        };
        
        knowledge.init_workflow_templates();
        knowledge
    }
    
    fn init_workflow_templates(&mut self) {
        // Create Character workflow
        self.workflow_templates.insert(WorkflowType::CreateCharacter, WorkflowTemplate {
            workflow_type: WorkflowType::CreateCharacter,
            name: "Create Character".to_string(),
            description: "Load a character and prepare it for use in a scene".to_string(),
            steps: vec![
                WorkflowStepTemplate {
                    description: "Load base figure".to_string(),
                    action_type: ActionType::LoadAsset,
                    parameters_template: HashMap::from([
                        ("asset_type".to_string(), "figure".to_string()),
                    ]),
                    prerequisites: vec![],
                    estimated_time_seconds: 5,
                    difficulty: DifficultyLevel::Trivial,
                },
                WorkflowStepTemplate {
                    description: "Apply desired morphs".to_string(),
                    action_type: ActionType::AdjustProperty,
                    parameters_template: HashMap::from([
                        ("property_type".to_string(), "morph".to_string()),
                    ]),
                    prerequisites: vec!["load_base_figure".to_string()],
                    estimated_time_seconds: 10,
                    difficulty: DifficultyLevel::Easy,
                },
                WorkflowStepTemplate {
                    description: "Apply base pose".to_string(),
                    action_type: ActionType::ApplyPose,
                    parameters_template: HashMap::from([
                        ("pose_type".to_string(), "basic".to_string()),
                    ]),
                    prerequisites: vec!["load_base_figure".to_string(), "apply_desired_morphs".to_string()],
                    estimated_time_seconds: 5,
                    difficulty: DifficultyLevel::Trivial,
                },
            ],
            tags: vec!["character".to_string(), "setup".to_string(), "basics".to_string()],
        });
        
        // Create Outfit workflow
        self.workflow_templates.insert(WorkflowType::CreateOutfit, WorkflowTemplate {
            workflow_type: WorkflowType::CreateOutfit,
            name: "Create Outfit".to_string(),
            description: "Dress a figure with clothing and accessories".to_string(),
            steps: vec![
                WorkflowStepTemplate {
                    description: "Ensure figure is loaded and posed".to_string(),
                    action_type: ActionType::LoadAsset,
                    parameters_template: HashMap::from([
                        ("asset_type".to_string(), "figure".to_string()),
                    ]),
                    prerequisites: vec![],
                    estimated_time_seconds: 5,
                    difficulty: DifficultyLevel::Trivial,
                },
                WorkflowStepTemplate {
                    description: "Load primary clothing item".to_string(),
                    action_type: ActionType::LoadAsset,
                    parameters_template: HashMap::from([
                        ("asset_type".to_string(), "clothing".to_string()),
                        ("item_type".to_string(), "primary".to_string()),
                    ]),
                    prerequisites: vec!["ensure_figure_ready".to_string()],
                    estimated_time_seconds: 8,
                    difficulty: DifficultyLevel::Easy,
                },
                WorkflowStepTemplate {
                    description: "Conform clothing to figure".to_string(),
                    action_type: ActionType::AdjustProperty,
                    parameters_template: HashMap::from([
                        ("adjustment_type".to_string(), "fit_control".to_string()),
                    ]),
                    prerequisites: vec!["load_primary_clothing".to_string()],
                    estimated_time_seconds: 15,
                    difficulty: DifficultyLevel::Moderate,
                },
                WorkflowStepTemplate {
                    description: "Add secondary clothing items".to_string(),
                    action_type: ActionType::LoadAsset,
                    parameters_template: HashMap::from([
                        ("asset_type".to_string(), "clothing".to_string()),
                        ("item_type".to_string(), "secondary".to_string()),
                    ]),
                    prerequisites: vec!["conform_primary_clothing".to_string()],
                    estimated_time_seconds: 10,
                    difficulty: DifficultyLevel::Easy,
                },
                WorkflowStepTemplate {
                    description: "Add accessories".to_string(),
                    action_type: ActionType::LoadAsset,
                    parameters_template: HashMap::from([
                        ("asset_type".to_string(), "accessories".to_string()),
                    ]),
                    prerequisites: vec!["add_secondary_clothing".to_string()],
                    estimated_time_seconds: 8,
                    difficulty: DifficultyLevel::Easy,
                },
            ],
            tags: vec!["clothing".to_string(), "outfit".to_string(), "character".to_string()],
        });
        
        // Setup Lighting workflow
        self.workflow_templates.insert(WorkflowType::SetupLighting, WorkflowTemplate {
            workflow_type: WorkflowType::SetupLighting,
            name: "Setup Lighting".to_string(),
            description: "Create a lighting setup appropriate for the scene".to_string(),
            steps: vec![
                WorkflowStepTemplate {
                    description: "Analyze scene for lighting needs".to_string(),
                    action_type: ActionType::Custom("analyze_scene".to_string()),
                    parameters_template: HashMap::new(),
                    prerequisites: vec![],
                    estimated_time_seconds: 5,
                    difficulty: DifficultyLevel::Moderate,
                },
                WorkflowStepTemplate {
                    description: "Add key light".to_string(),
                    action_type: ActionType::CreateLight,
                    parameters_template: HashMap::from([
                        ("light_purpose".to_string(), "key".to_string()),
                    ]),
                    prerequisites: vec!["analyze_scene".to_string()],
                    estimated_time_seconds: 5,
                    difficulty: DifficultyLevel::Easy,
                },
                WorkflowStepTemplate {
                    description: "Add fill light".to_string(),
                    action_type: ActionType::CreateLight,
                    parameters_template: HashMap::from([
                        ("light_purpose".to_string(), "fill".to_string()),
                    ]),
                    prerequisites: vec!["add_key_light".to_string()],
                    estimated_time_seconds: 5,
                    difficulty: DifficultyLevel::Easy,
                },
                WorkflowStepTemplate {
                    description: "Add rim/back light".to_string(),
                    action_type: ActionType::CreateLight,
                    parameters_template: HashMap::from([
                        ("light_purpose".to_string(), "rim".to_string()),
                    ]),
                    prerequisites: vec!["add_fill_light".to_string()],
                    estimated_time_seconds: 5,
                    difficulty: DifficultyLevel::Easy,
                },
                WorkflowStepTemplate {
                    description: "Adjust lighting ratios and colors".to_string(),
                    action_type: ActionType::AdjustProperty,
                    parameters_template: HashMap::from([
                        ("adjustment_type".to_string(), "lighting_balance".to_string()),
                    ]),
                    prerequisites: vec!["add_rim_light".to_string()],
                    estimated_time_seconds: 10,
                    difficulty: DifficultyLevel::Moderate,
                },
            ],
            tags: vec!["lighting".to_string(), "setup".to_string(), "scene".to_string()],
        });
        
        // Pose Character workflow
        self.workflow_templates.insert(WorkflowType::PoseCharacter, WorkflowTemplate {
            workflow_type: WorkflowType::PoseCharacter,
            name: "Pose Character".to_string(),
            description: "Pose a character using poses or manual adjustments".to_string(),
            steps: vec![
                WorkflowStepTemplate {
                    description: "Select or create pose".to_string(),
                    action_type: ActionType::Custom("select_pose".to_string()),
                    parameters_template: HashMap::new(),
                    prerequisites: vec![],
                    estimated_time_seconds: 5,
                    difficulty: DifficultyLevel::Easy,
                },
                WorkflowStepTemplate {
                    description: "Apply pose to figure".to_string(),
                    action_type: ActionType::ApplyPose,
                    parameters_template: HashMap::from([
                        ("pose_application".to_string(), "full_apply".to_string()),
                    ]),
                    prerequisites: vec!["select_or_create_pose".to_string()],
                    estimated_time_seconds: 5,
                    difficulty: DifficultyLevel::Easy,
                },
                WorkflowStepTemplate {
                    description: "Adjust pose with morphs if needed".to_string(),
                    action_type: ActionType::AdjustProperty,
                    parameters_template: HashMap::from([
                        ("adjustment_type".to_string(), "pose_morphs".to_string()),
                    ]),
                    prerequisites: vec!["apply_pose_to_figure".to_string()],
                    estimated_time_seconds: 10,
                    difficulty: DifficultyLevel::Moderate,
                },
                WorkflowStepTemplate {
                    description: "Fine-tune with joint adjustments".to_string(),
                    action_type: ActionType::AdjustProperty,
                    parameters_template: HashMap::from([
                        ("adjustment_type".to_string(), "joint_adjustment".to_string()),
                    ]),
                    prerequisites: vec!["adjust_pose_with_morphs".to_string()],
                    estimated_time_seconds: 15,
                    difficulty: DifficultyLevel::Hard,
                },
            ],
            tags: vec!["posing".to_string(), "character".to_string(), "animation".to_string()],
        });
        
        // Animate Character workflow
        self.workflow_templates.insert(WorkflowType::AnimateCharacter, WorkflowTemplate {
            workflow_type: WorkflowType::AnimateCharacter,
            name: "Animate Character".to_string(),
            description: "Create an animation sequence for a character".to_string(),
            steps: vec![
                WorkflowStepTemplate {
                    description: "Set up animation timeline".to_string(),
                    action_type: ActionType::Custom("setup_timeline".to_string()),
                    parameters_template: HashMap::from([
                        ("start_frame".to_string(), "1".to_string()),
                        ("end_frame".to_string(), "30".to_string()),
                    ]),
                    prerequisites: vec![],
                    estimated_time_seconds: 5,
                    difficulty: DifficultyLevel::Easy,
                },
                WorkflowStepTemplate {
                    description: "Pose character at keyframes".to_string(),
                    action_type: ActionType::Custom("pose_keyframes".to_string()),
                    parameters_template: HashMap::new(),
                    prerequisites: vec!["setup_animation_timeline".to_string()],
                    estimated_time_seconds: 20,
                    difficulty: DifficultyLevel::Moderate,
                },
                WorkflowStepTemplate {
                    description: "Set interpolation between keyframes".to_string(),
                    action_type: ActionType::AdjustProperty,
                    parameters_template: HashMap::from([
                        ("adjustment_type".to_string(), "interpolation".to_string()),
                    ]),
                    prerequisites: vec!["pose_character_at_keyframes".to_string()],
                    estimated_time_seconds: 10,
                    difficulty: DifficultyLevel::Moderate,
                },
                WorkflowStepTemplate {
                    description: "Add secondary motion (hair, clothing)".to_string(),
                    action_type: ActionType::Custom("add_secondary_motion".to_string()),
                    parameters_template: HashMap::new(),
                    prerequisites: vec!["set_interpolation".to_string()],
                    estimated_time_seconds: 15,
                    difficulty: DifficultyLevel::Hard,
                },
            ],
            tags: vec!["animation".to_string(), "character".to_string(), "timeline".to_string()],
        });
        
        // Render Still workflow
        self.workflow_templates.insert(WorkflowType::RenderStill, WorkflowTemplate {
            workflow_type: WorkflowType::RenderStill,
            name: "Render Still Image".to_string(),
            description: "Render a single still image of the current scene".to_string(),
            steps: vec![
                WorkflowStepTemplate {
                    description: "Frame the shot".to_string(),
                    action_type: ActionType::Custom("frame_shot".to_string()),
                    parameters_template: HashMap::new(),
                    prerequisites: vec![],
                    estimated_time_seconds: 5,
                    difficulty: DifficultyLevel::Easy,
                },
                WorkflowStepTemplate {
                    description: "Adjust camera settings".to_string(),
                    action_type: ActionType::AdjustProperty,
                    parameters_template: HashMap::from([
                        ("adjustment_type".to_string(), "camera_settings".to_string()),
                    ]),
                    prerequisites: vec!["frame_the_shot".to_string()],
                    estimated_time_seconds: 5,
                    difficulty: DifficultyLevel::Easy,
                },
                WorkflowStepTemplate {
                    description: "Do final lighting and material checks".to_string(),
                    action_type: ActionType::Custom("final_checks".to_string()),
                    parameters_template: HashMap::new(),
                    prerequisites: vec!["adjust_camera_settings".to_string()],
                    estimated_time_seconds: 10,
                    difficulty: DifficultyLevel::Moderate,
                },
                WorkflowStepTemplate {
                    description: "Begin undo batch".to_string(),
                    action_type: ActionType::UndoBatchBegin,
                    parameters_template: HashMap::new(),
                    prerequisites: vec!["do_final_checks".to_string()],
                    estimated_time_seconds: 1,
                    difficulty: DifficultyLevel::Trivial,
                },
                WorkflowStepTemplate {
                    description: "Trigger render".to_string(),
                    action_type: ActionType::Render,
                    parameters_template: HashMap::new(),
                    prerequisites: vec!["begin_undo_batch".to_string()],
                    estimated_time_seconds: 5,
                    difficulty: DifficultyLevel::Easy,
                },
                WorkflowStepTemplate {
                    description: "Accept undo batch".to_string(),
                    action_type: ActionType::UndoBatchEnd,
                    parameters_template: HashMap::from([
                        ("caption".to_string(), "AI Generated Render".to_string()),
                    ]),
                    prerequisites: vec!["trigger_render".to_string()],
                    estimated_time_seconds: 1,
                    difficulty: DifficultyLevel::Trivial,
                },
            ],
            tags: vec!["render".to_string(), "still".to_string(), "output".to_string()],
        });
        
        // Fix Common Issue workflow
        self.workflow_templates.insert(WorkflowType::FixCommonIssue, WorkflowTemplate {
            workflow_type: WorkflowType::FixCommonIssue,
            name: "Fix Common Issue".to_string(),
            description: "Diagnose and fix common Daz3D issues".to_string(),
            steps: vec![
                WorkflowStepTemplate {
                    description: "Identify the issue type".to_string(),
                    action_type: ActionType::Custom("identify_issue".to_string()),
                    parameters_template: HashMap::new(),
                    prerequisites: vec![],
                    estimated_time_seconds: 5,
                    difficulty: DifficultyLevel::Moderate,
                },
                WorkflowStepTemplate {
                    description: "Gather information about the issue".to_string(),
                    action_type: ActionType::Custom("gather_info".to_string()),
                    parameters_template: HashMap::new(),
                    prerequisites: vec!["identify_issue_type".to_string()],
                    estimated_time_seconds: 10,
                    difficulty: DifficultyLevel::Moderate,
                },
                WorkflowStepTemplate {
                    description: "Apply known fix for issue".to_string(),
                    action_type: ActionType::Custom("apply_fix".to_string()),
                    parameters_template: HashMap::new(),
                    prerequisites: vec!["gather_information".to_string()],
                    estimated_time_seconds: 15,
                    difficulty: DifficultyLevel::Hard,
                },
                WorkflowStepTemplate {
                    description: "Verify fix resolved issue".to_string(),
                    action_type: ActionType::Custom("verify_fix".to_string()),
                    parameters_template: HashMap::new(),
                    prerequisites: vec!["apply_known_fix".to_string()],
                    estimated_time_seconds: 5,
                    difficulty: DifficultyLevel::Easy,
                },
            ],
            tags: vec!["troubleshooting".to_string(), "fix".to_string(), "support".to_string()],
        });
    }
    
    /// Get a workflow by ID
    pub fn get_workflow(&self, id: &str) -> Option<&Workflow> {
        self.workflows.get(id)
    }
    
    /// Get workflows by tag
    pub fn get_workflows_by_tag(&self, tag: &str) -> Vec<&Workflow> {
        self.workflows.values()
            .filter(|w| w.tags.contains(&tag.to_string()))
            .collect()
    }
    
    /// Get workflows suitable for a goal
    pub fn get_workflows_for_goal(&self, goal: &str) -> Vec<&Workflow> {
        let lower_goal = goal.to_lowercase();
        self.workflows.values()
            .filter(|w| {
                w.name.to_lowercase().contains(&lower_goal) ||
                w.description.to_lowercase().contains(&lower_goal) ||
                w.tags.iter().any(|t| t.to_lowercase().contains(&lower_goal))
            })
            .collect()
    }
    
    /// Generate a workflow from a template
    pub fn generate_workflow_from_template(&self, workflow_type: WorkflowType, parameters: &HashMap<String, String>) -> Option<Workflow> {
        let template = self.workflow_templates.get(&workflow_type)?;
        
        // Convert template steps to actual steps with parameter substitution
        let mut steps = Vec::new();
        let mut _step_id_counter = 0;
        
        for (step_idx, template_step) in template.steps.iter().enumerate() {
            let step_id = format!("step_{}_{}", workflow_type as u32, step_idx);
            
            // Substitute parameters in the step description and parameters
            let mut description = template_step.description.clone();
            let mut parameters_map = HashMap::new();
            
            // Replace placeholders in description
            for (key, value) in parameters {
                let placeholder = format!("{{{}}}", key);
                description = description.replace(&placeholder, value);
            }
            
            // Copy and substitute parameters
            for (key, value) in &template_step.parameters_template {
                let mut substituted_value = value.clone();
                for (param_key, param_value) in parameters {
                    let placeholder = format!("{{{}}}", param_key);
                    substituted_value = substituted_value.replace(&placeholder, param_value);
                }
                parameters_map.insert(key.clone(), substituted_value);
            }
            
            // Convert prerequisites from template IDs to actual IDs
            let mut prerequisites = Vec::new();
            for prereq in &template_step.prerequisites {
                // Find the step ID for this prerequisite
                let mut found = false;
                for (prev_idx, prev_template_step) in template.steps[..step_idx].iter().enumerate() {
                    if prev_template_step.description.contains(prereq) {
                        prerequisites.push(format!("step_{}_{}", workflow_type as u32, prev_idx));
                        found = true;
                        break;
                    }
                }
                if !found {
                    // If not found in previous steps, keep as-is (might be external reference)
                    prerequisites.push(prereq.clone());
                }
            }
            
            steps.push(WorkflowStep {
                id: step_id,
                description,
                action_type: template_step.action_type.clone(),
                parameters: parameters_map,
                prerequisites,
                estimated_time_seconds: template_step.estimated_time_seconds,
                difficulty: template_step.difficulty,
            });
            
            _step_id_counter += 1;
        }
        
        Some(Workflow {
            id: format!("workflow_{}_{}", workflow_type as u32, std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()),
            name: template.name.clone(),
            description: template.description.clone(),
            steps,
            tags: template.tags.clone(),
            success_rate: 0.8, // Default - will be updated with actual usage
            usage_count: 0,
            last_used: None,
        })
    }
    
    /// Record workflow usage and update success rate
    pub fn record_workflow_usage(&mut self, workflow_id: &str, success: bool) {
        if let Some(workflow) = self.workflows.get_mut(workflow_id) {
            workflow.usage_count += 1;
            workflow.last_used = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string()
            );
            
            // Update success rate using exponential moving average
            let alpha = 0.3; // learning rate
            let new_success = if success { 1.0 } else { 0.0 };
            workflow.success_rate = (1.0 - alpha) * workflow.success_rate + alpha * new_success;
        }
    }
}
