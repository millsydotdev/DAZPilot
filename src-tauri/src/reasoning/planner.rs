use crate::knowledge::asset_knowledge::AssetKnowledgeBase;
use crate::knowledge::scene_knowledge::{SceneKnowledgeBase, SceneType};
use crate::knowledge::workflow_knowledge::{WorkflowKnowledgeBase, WorkflowType, WorkflowStep, ActionType};
use crate::knowledge::failure_knowledge::{FailureKnowledgeBase, SceneStateSnapshot};
use crate::knowledge::daz_concepts::DazKnowledgeBase;
use crate::library_scanner::AssetInfo;
use crate::ai_system::{Intent, Entity, EntityType};
use crate::ai_action::StructuredAiAction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use fastrand;

/// Represents a goal that the planner is trying to achieve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub intent: Intent,
    pub entities: Vec<Entity>,
    pub priority: GoalPriority,
    pub constraints: Vec<String>,
}

/// Priority levels for goals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Represents a step in a plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub description: String,
    pub action: StructuredAiAction,
    pub prerequisites: Vec<String>, // IDs of steps that must complete first
    pub estimated_time_seconds: u32,
    pub confidence: f32, // 0.0 to 1.0
    pub alternatives: Vec<StructuredAiAction>, // alternative actions if this fails
}

/// A complete plan to achieve a goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub goal_id: String,
    pub goal: Goal,
    pub description: String,
    pub steps: Vec<PlanStep>,
    pub estimated_total_time_seconds: u32,
    pub confidence: f32, // overall confidence in plan
    pub risk_level: RiskLevel,
    pub fallback_plan: Option<Box<Plan>>, // plan to use if this fails
}

/// Risk levels for plans
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// The main planner that generates multi-step plans from goals
#[derive(Clone)]
pub struct Planner {
    pub asset_knowledge: Arc<AssetKnowledgeBase>,
    pub scene_knowledge: SceneKnowledgeBase,
    pub workflow_knowledge: WorkflowKnowledgeBase,
    pub failure_knowledge: Arc<FailureKnowledgeBase>,
    pub daz_knowledge: DazKnowledgeBase,
}

impl Planner {
    pub fn new() -> Self {
        Self {
            asset_knowledge: Arc::new(AssetKnowledgeBase::new()),
            scene_knowledge: SceneKnowledgeBase::new(),
            workflow_knowledge: WorkflowKnowledgeBase::new(),
            failure_knowledge: Arc::new(FailureKnowledgeBase::new()),
            daz_knowledge: DazKnowledgeBase::new(),
        }
    }
    
    /// Generate a plan to achieve a goal
    pub fn plan_for_goal(&self, goal: &Goal, context: &PlanningContext) -> Option<Plan> {
        // 1. Analyze the goal and context
        let analysis = self.analyze_goal(goal, context);
        
        // 2. Generate candidate approaches
        let candidates = self.generate_candidates(&analysis, context);
        
        // 3. Select the best approach
        let best = self.select_best_approach(candidates, context)?;
        
        // 4. Develop into detailed plan
        let plan = self.develop_plan(best, goal, context)?;
        
        // 5. Validate and refine
        let validated = self.validate_and_refine(plan, context)?;
        
        Some(validated)
    }
    
    /// Analyze a goal to understand what needs to be achieved
    fn analyze_goal(&self, goal: &Goal, _context: &PlanningContext) -> GoalAnalysis {
        let mut analysis = GoalAnalysis {
            goal: goal.clone(),
            required_assets: Vec::new(),
            required_figures: Vec::new(),
            suggested_scene_type: None,
            suggested_workflow_type: None,
            constraints: Vec::new(),
            opportunities: Vec::new(),
        };
        
        // Extract required assets from entities
        for entity in &goal.entities {
            match entity.entity_type {
                crate::ai_system::EntityType::Figure => {
                    if let Some(figure) = self.daz_knowledge.get_figure(&entity.value) {
                        analysis.required_figures.push(figure.name.clone());
                    }
                }
                crate::ai_system::EntityType::Asset => {
                    // TODO: look up asset by name in knowledge base
                    analysis.required_assets.push(entity.value.clone());
                }
                crate::ai_system::EntityType::Pose => {
                    // TODO: handle pose entities
                }
                crate::ai_system::EntityType::Material => {
                    // TODO: handle material entities
                }
                _ => {}
            }
        }
        
        // Infer scene type from goal description
        analysis.suggested_scene_type = self.scene_knowledge.infer_scene_type(&goal.description);
        
        // Look for workflow matches
        analysis.suggested_workflow_type = self.infer_workflow_type(goal);
        
        analysis
    }
    
    /// Generate candidate approaches to achieve the goal
    fn generate_candidates(&self, analysis: &GoalAnalysis, _context: &PlanningContext) -> Vec<PlanApproach> {
        let mut candidates = Vec::new();
        
        // Candidate 1: Use existing workflow if available
        if let Some(workflow_type) = analysis.suggested_workflow_type {
            candidates.push(PlanApproach::WorkflowBased {
                workflow_type,
                confidence: 0.8,
                 description: format!("Using {:?} workflow", workflow_type),
            });
        }
        
        // Candidate 2: Build from scene knowledge
        if let Some(scene_type) = analysis.suggested_scene_type {
            candidates.push(PlanApproach::SceneBased {
                scene_type,
                confidence: 0.7,
                 description: format!("Building plan for {:?} scene", scene_type),
            });
        }
        
        // Candidate 3: Asset-focused approach
        if !analysis.required_assets.is_empty() || !analysis.required_figures.is_empty() {
            candidates.push(PlanApproach::AssetFocused {
                assets: analysis.required_assets.clone(),
                figures: analysis.required_figures.clone(),
                confidence: 0.6,
                description: "Asset-focused approach".to_string(),
            });
        }
        
        // Candidate 4: Intent-based approach (fallback to current heuristic)
        candidates.push(PlanApproach::IntentBased {
            intent: analysis.goal.intent.clone(),
            confidence: 0.5,
            description: "Direct intent mapping".to_string(),
        });
        
        candidates
    }
    
    /// Select the best approach from candidates
    fn select_best_approach(&self, candidates: Vec<PlanApproach>, _context: &PlanningContext) -> Option<PlanApproach> {
        if candidates.is_empty() {
            return None;
        }
        
        // For now, just pick the highest confidence
        // In a more advanced version, we would weigh against context, constraints, etc.
        let mut best = &candidates[0];
        for candidate in &candidates[1..] {
            if candidate.confidence() > best.confidence() {
                best = candidate;
            }
        }
        
        Some(best.clone())
    }
    
    /// Develop a detailed plan from an approach
    fn develop_plan(&self, approach: PlanApproach, goal: &Goal, context: &PlanningContext) -> Option<Plan> {
        match approach {
            PlanApproach::WorkflowBased { workflow_type, .. } => {
                self.build_workflow_plan(workflow_type, goal, context)
            }
            PlanApproach::SceneBased { scene_type, .. } => {
                self.build_scene_plan(scene_type, goal, context)
            }
            PlanApproach::AssetFocused { assets, figures, .. } => {
                self.build_asset_plan(assets, figures, goal, context)
            }
            PlanApproach::IntentBased { intent, .. } => {
                self.build_intent_plan(intent, goal, context)
            }
        }
    }
    
    /// Build a plan based on a workflow template
    fn build_workflow_plan(&self, workflow_type: WorkflowType, goal: &Goal, context: &PlanningContext) -> Option<Plan> {
        // Generate workflow from template
        let workflow_kb = self.workflow_knowledge.clone();
        let workflow = workflow_kb.generate_workflow_from_template(workflow_type, &HashMap::new())?;
        
        // Convert workflow steps to plan steps
        let mut plan_steps = Vec::new();
        for workflow_step in workflow.steps {
            // Convert WorkflowStep to StructuredAiAction
            let action = self.workflow_step_to_action(&workflow_step, context)?;
            
            plan_steps.push(PlanStep {
                id: workflow_step.id.clone(),
                description: workflow_step.description.clone(),
                action,
                prerequisites: workflow_step.prerequisites.clone(),
                estimated_time_seconds: workflow_step.estimated_time_seconds,
                confidence: 0.8, // TODO: calculate based on knowledge
                alternatives: Vec::new(), // TODO: generate alternatives
            });
        }
        
        // Calculate totals
        let estimated_time = plan_steps.iter()
            .map(|s| s.estimated_time_seconds)
            .sum();
        
        let confidence = if plan_steps.is_empty() { 0.0 } else {
            plan_steps.iter().map(|s| s.confidence).sum::<f32>() / plan_steps.len() as f32
        };
        
        Some(Plan {
            id: format!("plan_{}_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(), fastrand::u32(..)),
            goal_id: goal.id.clone(),
            goal: goal.clone(),
            description: format!("Workflow-based plan: {}", workflow.name),
            steps: plan_steps,
            estimated_total_time_seconds: estimated_time,
            confidence,
            risk_level: RiskLevel::Medium, // TODO: calculate
            fallback_plan: None, // TODO: generate fallback
        })
    }
    
    /// Build a plan based on scene knowledge
    fn build_scene_plan(&self, scene_type: SceneType, goal: &Goal, context: &PlanningContext) -> Option<Plan> {
        // Analyze the goal to understand what's needed
        let analysis = self.analyze_goal(goal, context);
        let scene_understanding = self.scene_knowledge.get_scene_understanding(scene_type)?;
        
        // Convert scene understanding to plan steps
        let mut plan_steps = Vec::new();
         let mut step_counter: u32 = 0;
        
        // Step 1: Load required figures
        for figure in &analysis.required_figures {
            plan_steps.push(PlanStep {
                id: format!("load_figure_{}", step_counter),
                description: format!("Load figure {}", figure),
                action: StructuredAiAction {
                    command: "add_figure".to_string(),
                    args: serde_json::json!({ "figure_type": if figure.contains("9") { "genesis9" } else { "genesis8" } }),
                    confidence: 0.9,
                    sdk_refs: vec!["DzFigure".to_string()],
                    requires_confirmation: false,
                },
                prerequisites: Vec::new(),
                estimated_time_seconds: 5,
                confidence: 0.9,
                alternatives: Vec::new(),
            });
            step_counter += 1;
        }
        
        // Step 2: Apply lighting setup from scene knowledge
        for (_i, lighting_rec) in scene_understanding.lighting_setup.iter().enumerate() {
            plan_steps.push(PlanStep {
                id: format!("add_light_{}", step_counter),
                description: format!("Add {} light for {} purpose", lighting_rec.light_type, lighting_rec.purpose),
                action: StructuredAiAction {
                    command: "add_node".to_string(),
                    args: serde_json::json!({
                        "type": lighting_rec.light_type,
                        "name": format!("AI_{}_{}", lighting_rec.light_type, lighting_rec.purpose)
                    }),
                    confidence: 0.85,
                    sdk_refs: vec!["DzLight".to_string()],
                    requires_confirmation: false,
                },
                prerequisites: if step_counter > 0 { vec![format!("load_figure_{}", step_counter - 1)] } else { Vec::new() },
                estimated_time_seconds: 4,
                confidence: 0.85,
                alternatives: Vec::new(),
            });
            step_counter += 1;
        }
        
        // Step 3: Load required assets
        for asset in &analysis.required_assets {
            plan_steps.push(PlanStep {
                id: format!("load_asset_{}", step_counter),
                description: format!("Load asset {}", asset),
                action: StructuredAiAction {
                    command: "load_asset".to_string(),
                    args: serde_json::json!({ "path": asset }), // TODO: resolve asset name to path
                    confidence: 0.7,
                    sdk_refs: vec!["DzContentMgr".to_string(), "DzAsset".to_string()],
                    requires_confirmation: false,
                },
                prerequisites: if step_counter > 0 { vec![format!("load_figure_{}", step_counter - 1)] } else { Vec::new() },
                estimated_time_seconds: 6,
                confidence: 0.7,
                alternatives: Vec::new(),
            });
            step_counter += 1;
        }
        
        // Calculate totals and return plan
        let estimated_time = plan_steps.iter()
            .map(|s| s.estimated_time_seconds)
            .sum();
        
        let confidence = if plan_steps.is_empty() { 0.0 } else {
            plan_steps.iter().map(|s| s.confidence).sum::<f32>() / plan_steps.len() as f32
        };
        
        Some(Plan {
            id: format!("plan_{}_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(), fastrand::u32(..)),
            goal_id: goal.id.clone(),
            goal: goal.clone(),
             description: format!("Scene-based plan for {:?} scene", scene_type),
            steps: plan_steps,
            estimated_total_time_seconds: estimated_time,
            confidence,
            risk_level: RiskLevel::Low,
            fallback_plan: None,
        })
    }
    
    /// Build a plan focused on specific assets
    fn build_asset_plan(&self, _assets: Vec<String>, _figures: Vec<String>, _goal: &Goal, _context: &PlanningContext) -> Option<Plan> {
        // TODO: Implement asset-focused planning
        None
    }
    
    /// Build a plan based directly on intent (fallback to current behavior)
    fn build_intent_plan(&self, _intent: Intent, _goal: &Goal, _context: &PlanningContext) -> Option<Plan> {
        // TODO: Implement intent-based planning as fallback
        None
    }
    
    /// Convert a workflow step to a StructuredAiAction
    fn workflow_step_to_action(&self, step: &WorkflowStep, _context: &PlanningContext) -> Option<StructuredAiAction> {
        match &step.action_type {
            ActionType::LoadAsset => {
                // Extract asset type from parameters
                let asset_type_default = "asset".to_string();
                let asset_type = step.parameters.get("asset_type")
                    .unwrap_or(&asset_type_default)
                    .to_lowercase();
                
                   // Determine what to load based on asset_type
                    let figure_type_default = "genesis8".to_string();
                    let figure_type = step.parameters.get("figure_type")
                        .unwrap_or(&figure_type_default);
                    let command_str;
                    let mut args_json = serde_json::json!({});
                    let conf;
                   
                   match asset_type.as_str() {
                       "figure" => {
                           command_str = "add_figure".to_string();
                           args_json = serde_json::json!({ "figure_type": figure_type.clone() });
                           conf = 0.9;
                       }
                       "clothing" | "hair" | "pose" | "material" => {
                           // For these, we'd need to search the asset library
                           // For now, use a placeholder
                           command_str = "load_asset".to_string();
                           args_json = serde_json::json!({ "path": "TODO: resolve asset" });
                           conf = 0.7;
                       }
                       "light" => {
                            let light_type_default = "point_light".to_string();
                            let light_type = step.parameters.get("light_type")
                                .unwrap_or(&light_type_default);
                           command_str = "add_node".to_string();
                           args_json = serde_json::json!({
                               "type": light_type,
                               "name": format!("AI_{}", light_type)
                           });
                           conf = 0.85;
                       }
                       _ => {
                           command_str = "load_asset".to_string();
                           args_json = serde_json::json!({ "path": "unknown" });
                           conf = 0.5;
                       }
                   };
                
                 Some(StructuredAiAction {
                     command: command_str,
                     args: args_json,
                     confidence: conf,
                     sdk_refs: vec![], // TODO: populate based on command
                     requires_confirmation: false,
                 })
            }
            ActionType::ApplyPose => {
                let pose_type_default = "basic".to_string();
                let pose_type = step.parameters.get("pose_type")
                    .unwrap_or(&pose_type_default);
                Some(StructuredAiAction {
                    command: "apply_pose".to_string(),
                    args: serde_json::json!({ "pose": pose_type }),
                    confidence: 0.8,
                    sdk_refs: vec!["DzPose".to_string()],
                    requires_confirmation: false,
                })
            }
            ActionType::AdjustProperty => {
                let prop_default = "unknown".to_string();
                let prop = step.parameters.get("property")
                    .unwrap_or(&prop_default);
                Some(StructuredAiAction {
                    command: "set_property".to_string(),
                    args: serde_json::json!({
                        "node_id": "selected",
                        "property": prop,
                        "value": "1.0" // TODO: get actual value
                    }),
                    confidence: 0.7,
                    sdk_refs: vec!["DzProperty".to_string()],
                    requires_confirmation: false,
                })
            }
            ActionType::CreateLight => {
                let light_type_default = "point_light".to_string();
                let light_type = step.parameters.get("type")
                    .unwrap_or(&light_type_default);
                Some(StructuredAiAction {
                    command: "add_node".to_string(),
                    args: serde_json::json!({
                        "type": light_type,
                        "name": format!("AI_{}", light_type)
                    }),
                    confidence: 0.85,
                    sdk_refs: vec!["DzLight".to_string()],
                    requires_confirmation: false,
                })
            }
            ActionType::CreateCamera => {
                Some(StructuredAiAction {
                    command: "add_node".to_string(),
                    args: serde_json::json!({
                        "type": "camera",
                        "name": "AI_Camera"
                    }),
                    confidence: 0.85,
                    sdk_refs: vec!["DzCamera".to_string()],
                    requires_confirmation: false,
                })
            }
            ActionType::Render => {
                Some(StructuredAiAction {
                    command: "render_preview".to_string(),
                    args: serde_json::json!({}),
                    confidence: 0.8,
                    sdk_refs: vec!["DzRenderMgr".to_string()],
                    requires_confirmation: false,
                })
            }
            ActionType::ChangeMaterial => {
                Some(StructuredAiAction {
                    command: "set_material_property".to_string(),
                    args: serde_json::json!({
                        "node_id": "selected",
                        "property": "Base Color",
                        "value": "255,255,255"
                    }),
                    confidence: 0.7,
                    sdk_refs: vec!["DzMaterial".to_string()],
                    requires_confirmation: false,
                })
            }
            ActionType::ExportScene => {
                Some(StructuredAiAction {
                    command: "export_scene".to_string(),
                    args: serde_json::json!({
                        "node_id": "",
                        "path": "scene_output.duf",
                        "settings": serde_json::json!({})
                    }),
                    confidence: 0.7,
                    sdk_refs: vec!["DzExportMgr".to_string(), "DzExporter".to_string()],
                    requires_confirmation: true,
                })
            }
            ActionType::RunSimulation => {
                Some(StructuredAiAction {
                    command: "run_dforce_simulation".to_string(),
                    args: serde_json::json!({
                        "node_id": "selected",
                        "start_frame": 0,
                        "end_frame": 30
                    }),
                    confidence: 0.7,
                    sdk_refs: vec!["DzSimulator".to_string()],
                    requires_confirmation: true,
                })
            }
            ActionType::UndoBatchBegin => {
                Some(StructuredAiAction {
                    command: "begin_undo_batch".to_string(),
                    args: serde_json::json!({}),
                    confidence: 0.95,
                    sdk_refs: vec![],
                    requires_confirmation: false,
                })
            }
            ActionType::UndoBatchEnd => {
                Some(StructuredAiAction {
                    command: "accept_undo_batch".to_string(),
                    args: serde_json::json!({
                        "caption": "AI Generated Action"
                    }),
                    confidence: 0.95,
                    sdk_refs: vec![],
                    requires_confirmation: false,
                })
            }
            ActionType::Custom(_) => {
                // Custom actions need special handling
                // For now, return None to skip
                None
            }
        }
    }
    
    /// Validate and refine a plan
    fn validate_and_refine(&self, plan: Plan, _context: &PlanningContext) -> Option<Plan> {
        // TODO: Implement validation against constraints, permissions, asset availability
        // For now, just return the plan as-is
        Some(plan)
    }
    
    /// Infer workflow type from goal
    fn infer_workflow_type(&self, goal: &Goal) -> Option<WorkflowType> {
        let lower_desc = goal.description.to_lowercase();
        let _lower_intent = format!("{:?}", goal.intent).to_lowercase();
        
        // Check for workflow keywords in description
        if lower_desc.contains("create") && (lower_desc.contains("character") || lower_desc.contains("figure")) {
            return Some(WorkflowType::CreateCharacter);
        }
        if lower_desc.contains("outfit") || lower_desc.contains("dress") || lower_desc.contains("clothing") {
            return Some(WorkflowType::CreateOutfit);
        }
        if lower_desc.contains("light") || lower_desc.contains("lighting") {
            return Some(WorkflowType::SetupLighting);
        }
        if lower_desc.contains("pose") {
            return Some(WorkflowType::PoseCharacter);
        }
        if lower_desc.contains("animate") || lower_desc.contains("animation") {
            return Some(WorkflowType::AnimateCharacter);
        }
        if lower_desc.contains("render") {
            return Some(WorkflowType::RenderStill);
        }
        if lower_desc.contains("fix") || lower_desc.contains("problem") || lower_desc.contains("issue") {
            return Some(WorkflowType::FixCommonIssue);
        }
        
        // Check intent
        if matches!(goal.intent, Intent::LoadAsset) && 
           goal.entities.iter().any(|e| matches!(e.entity_type, EntityType::Figure)) {
            return Some(WorkflowType::CreateCharacter);
        }
        if matches!(goal.intent, Intent::ApplyPose) {
            return Some(WorkflowType::PoseCharacter);
        }
        if matches!(goal.intent, Intent::Render) {
            return Some(WorkflowType::RenderStill);
        }
        if matches!(goal.intent, Intent::CreateLight) {
            return Some(WorkflowType::SetupLighting);
        }
        
        None
    }
}

/// Analysis of a goal to inform planning
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GoalAnalysis {
    pub goal: Goal,
    pub required_assets: Vec<String>,
    pub required_figures: Vec<String>,
    pub suggested_scene_type: Option<SceneType>,
    pub suggested_workflow_type: Option<WorkflowType>,
    pub constraints: Vec<String>,
    pub opportunities: Vec<String>,
}

/// Different approaches to planning a goal
#[derive(Debug, Clone, Serialize, Deserialize)]
enum PlanApproach {
    WorkflowBased {
        workflow_type: WorkflowType,
        confidence: f32,
        description: String,
    },
    SceneBased {
        scene_type: SceneType,
        confidence: f32,
        description: String,
    },
    AssetFocused {
        assets: Vec<String>,
        figures: Vec<String>,
        confidence: f32,
        description: String,
    },
    IntentBased {
        intent: Intent,
        confidence: f32,
        description: String,
    },
}

impl PlanApproach {
    fn confidence(&self) -> f32 {
        match self {
            PlanApproach::WorkflowBased { confidence, .. } => *confidence,
            PlanApproach::SceneBased { confidence, .. } => *confidence,
            PlanApproach::AssetFocused { confidence, .. } => *confidence,
            PlanApproach::IntentBased { confidence, .. } => *confidence,
        }
    }
}

/// Context information for planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningContext {
    pub scene_state: Option<SceneStateSnapshot>,
    pub recent_actions: Vec<String>,
    pub user_preferences: Option<UserPreferences>,
    pub available_assets: Vec<AssetInfo>,
    pub constraints: Vec<String>,
}

/// User preferences for planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub preferred_figures: Vec<String>,
    pub preferred_lighting_styles: Vec<String>,
    pub skill_level: SkillLevel,
    pub favorite_workflows: Vec<String>,
}

/// Skill level of the user
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}
