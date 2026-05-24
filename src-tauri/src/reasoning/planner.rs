use crate::knowledge::asset_knowledge::AssetKnowledgeBase;
use crate::knowledge::scene_knowledge::{SceneKnowledgeBase, SceneType};
use crate::knowledge::workflow_knowledge::{WorkflowKnowledgeBase, WorkflowType, WorkflowStep, ActionType};
use crate::knowledge::failure_knowledge::{FailureKnowledgeBase, SceneStateSnapshot};
use crate::knowledge::daz_concepts::DazKnowledgeBase;
use crate::knowledge::command_knowledge::CommandKnowledgeBase;
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
    pub command_knowledge: CommandKnowledgeBase,
}

impl Planner {
    pub fn new() -> Self {
        Self {
            asset_knowledge: Arc::new(AssetKnowledgeBase::new()),
            scene_knowledge: SceneKnowledgeBase::new(),
            workflow_knowledge: WorkflowKnowledgeBase::new(),
            failure_knowledge: Arc::new(FailureKnowledgeBase::new()),
            daz_knowledge: DazKnowledgeBase::new(),
            command_knowledge: CommandKnowledgeBase::new(),
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
    fn build_asset_plan(&self, assets: Vec<String>, figures: Vec<String>, goal: &Goal, context: &PlanningContext) -> Option<Plan> {
        let mut steps = Vec::new();
        let mut previous: Option<String> = None;

        for figure in figures {
            let step_id = format!("asset_figure_{}", steps.len());
            let figure_type = if figure.to_lowercase().contains('9') { "genesis9" } else { "genesis8" };
            steps.push(PlanStep {
                id: step_id.clone(),
                description: format!("Add {}", figure),
                action: StructuredAiAction {
                    command: "add_figure".to_string(),
                    args: serde_json::json!({ "figure_type": figure_type }),
                    confidence: 0.9,
                    sdk_refs: vec!["DzFigure".to_string(), "DzContentMgr".to_string()],
                    requires_confirmation: false,
                },
                prerequisites: previous.iter().cloned().collect(),
                estimated_time_seconds: 5,
                confidence: 0.9,
                alternatives: Vec::new(),
            });
            previous = Some(step_id);
        }

        for asset in assets {
            let step_id = format!("asset_load_{}", steps.len());
            let (command, args, confidence, sdk_refs) = if let Some(path) = resolve_asset_path(&asset, None, context) {
                (
                    "load_asset".to_string(),
                    serde_json::json!({ "path": path }),
                    0.85,
                    vec!["DzContentMgr".to_string(), "DzAsset".to_string()],
                )
            } else {
                (
                    "search_content".to_string(),
                    serde_json::json!({ "query": asset, "type": "asset", "max_results": "10" }),
                    0.65,
                    vec!["DzContentMgr".to_string()],
                )
            };
            steps.push(PlanStep {
                id: step_id.clone(),
                description: format!("Resolve {}", asset),
                action: StructuredAiAction {
                    command,
                    args,
                    confidence,
                    sdk_refs,
                    requires_confirmation: false,
                },
                prerequisites: previous.iter().cloned().collect(),
                estimated_time_seconds: 5,
                confidence,
                alternatives: Vec::new(),
            });
            previous = Some(step_id);
        }

        if steps.is_empty() {
            None
        } else {
            Some(self.plan_from_steps(goal, "Asset-focused composition plan", steps, RiskLevel::Low))
        }
    }
    
    /// Build a plan based directly on intent (fallback to current behavior)
    fn build_intent_plan(&self, intent: Intent, goal: &Goal, _context: &PlanningContext) -> Option<Plan> {
        let action = match intent {
            Intent::CreateScene => StructuredAiAction {
                command: "add_figure".to_string(),
                args: serde_json::json!({ "figure_type": "genesis9" }),
                confidence: 0.75,
                sdk_refs: vec!["DzFigure".to_string()],
                requires_confirmation: false,
            },
            Intent::CreateLight => StructuredAiAction {
                command: "add_node".to_string(),
                args: serde_json::json!({ "type": "point_light", "name": "AI_Key_Light" }),
                confidence: 0.75,
                sdk_refs: vec!["DzLight".to_string()],
                requires_confirmation: false,
            },
            Intent::Render => StructuredAiAction {
                command: "render_preview".to_string(),
                args: serde_json::json!({}),
                confidence: 0.75,
                sdk_refs: vec!["DzRenderMgr".to_string()],
                requires_confirmation: false,
            },
            _ => return None,
        };

        Some(self.plan_from_steps(goal, "Intent fallback plan", vec![PlanStep {
            id: "intent_step_0".to_string(),
            description: format!("Execute {:?}", intent),
            action,
            prerequisites: Vec::new(),
            estimated_time_seconds: 5,
            confidence: 0.75,
            alternatives: Vec::new(),
        }], RiskLevel::Low))
    }
    
    /// Convert a workflow step to a StructuredAiAction
    fn workflow_step_to_action(&self, step: &WorkflowStep, context: &PlanningContext) -> Option<StructuredAiAction> {
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
                       "clothing" | "hair" | "material" => {
                           let query = step.parameters.get("query")
                               .or_else(|| step.parameters.get("asset_query"))
                               .cloned()
                               .unwrap_or_else(|| asset_type.clone());
                           if let Some(path) = resolve_asset_path(&query, Some(&asset_type), context) {
                               command_str = "load_asset".to_string();
                               args_json = serde_json::json!({ "path": path });
                               conf = 0.82;
                           } else {
                               command_str = "search_content".to_string();
                               args_json = serde_json::json!({
                                   "query": query,
                                   "type": asset_type,
                                   "max_results": "10",
                               });
                               conf = 0.65;
                           }
                       }
                       "pose" => {
                           let query = step.parameters.get("query")
                               .or_else(|| step.parameters.get("pose_type"))
                               .cloned()
                               .unwrap_or_else(|| "pose".to_string());
                           if let Some(path) = resolve_asset_path(&query, Some("poses"), context)
                               .or_else(|| resolve_asset_path(&query, Some("pose"), context)) {
                               command_str = "apply_pose".to_string();
                               args_json = serde_json::json!({ "pose_path": path, "figure_id": "selected" });
                               conf = 0.82;
                           } else {
                               command_str = "search_content".to_string();
                               args_json = serde_json::json!({
                                   "query": query,
                                   "type": "pose",
                                   "max_results": "10",
                               });
                               conf = 0.65;
                           }
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
                if let Some(path) = resolve_asset_path(pose_type, Some("poses"), context)
                    .or_else(|| resolve_asset_path(pose_type, Some("pose"), context)) {
                    Some(StructuredAiAction {
                        command: "apply_pose".to_string(),
                        args: serde_json::json!({ "pose_path": path, "figure_id": "selected" }),
                        confidence: 0.82,
                        sdk_refs: vec!["DzPose".to_string()],
                        requires_confirmation: false,
                    })
                } else {
                    Some(StructuredAiAction {
                        command: "apply_pose".to_string(),
                        args: serde_json::json!({ "pose_path": "", "figure_id": "selected" }),
                        confidence: 0.45,
                        sdk_refs: vec!["DzPose".to_string()],
                        requires_confirmation: false,
                    })
                }
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
            ActionType::AddFigure => {
                let figure_type_default = "genesis9".to_string();
                let figure_type = step.parameters.get("figure_type")
                    .unwrap_or(&figure_type_default);
                Some(StructuredAiAction {
                    command: "add_figure".to_string(),
                    args: serde_json::json!({ "figure_type": figure_type }),
                    confidence: 0.9,
                    sdk_refs: vec!["DzFigure".to_string()],
                    requires_confirmation: false,
                })
            }
            ActionType::SetMorph => {
                let morph_default = "Head_Height".to_string();
                let value_default = "0.3".to_string();
                let morph = step.parameters.get("morph")
                    .unwrap_or(&morph_default);
                let value = step.parameters.get("value")
                    .unwrap_or(&value_default);
                Some(StructuredAiAction {
                    command: "set_morph".to_string(),
                    args: serde_json::json!({
                        "node_id": "selected",
                        "morph": morph,
                        "value": value,
                    }),
                    confidence: 0.8,
                    sdk_refs: vec!["DzModifier".to_string(), "DzFigure".to_string()],
                    requires_confirmation: false,
                })
            }
            ActionType::SetExpression => {
                let expr_default = "Smile".to_string();
                let value_default = "0.5".to_string();
                let expr = step.parameters.get("expression")
                    .unwrap_or(&expr_default);
                let value = step.parameters.get("value")
                    .unwrap_or(&value_default);
                Some(StructuredAiAction {
                    command: "apply_expression".to_string(),
                    args: serde_json::json!({
                        "figure_id": "selected",
                        "expression_id": expr,
                        "value": value,
                    }),
                    confidence: 0.75,
                    sdk_refs: vec!["DzModifier".to_string()],
                    requires_confirmation: false,
                })
            }
            ActionType::SetCamera => {
                let camera_default = "Perspective View".to_string();
                let fl_default = "".to_string();
                let fd_default = "".to_string();
                let camera = step.parameters.get("camera")
                    .unwrap_or(&camera_default);
                let focal_length = step.parameters.get("focal_length")
                    .unwrap_or(&fl_default);
                let focal_distance = step.parameters.get("focal_distance")
                    .unwrap_or(&fd_default);
                Some(StructuredAiAction {
                    command: "set_camera".to_string(),
                    args: serde_json::json!({
                        "camera": camera,
                        "focal_length": focal_length,
                        "focal_distance": focal_distance,
                    }),
                    confidence: 0.85,
                    sdk_refs: vec!["DzCamera".to_string()],
                    requires_confirmation: false,
                })
            }
            ActionType::SetRenderOptions => {
                let width_default = "1920".to_string();
                let height_default = "1080".to_string();
                let samples_default = "256".to_string();
                let depth_default = "4".to_string();
                let rate_default = "1.0".to_string();
                let gamma_default = "2.2".to_string();
                let width = step.parameters.get("width").unwrap_or(&width_default);
                let height = step.parameters.get("height").unwrap_or(&height_default);
                let samples = step.parameters.get("pixel_samples").unwrap_or(&samples_default);
                let depth = step.parameters.get("ray_trace_depth").unwrap_or(&depth_default);
                let rate = step.parameters.get("shading_rate").unwrap_or(&rate_default);
                let gamma = step.parameters.get("gamma").unwrap_or(&gamma_default);
                Some(StructuredAiAction {
                    command: "set_render_options".to_string(),
                    args: serde_json::json!({
                        "width": width,
                        "height": height,
                        "pixel_samples": samples,
                        "ray_trace_depth": depth,
                        "shading_rate": rate,
                        "gamma": gamma,
                    }),
                    confidence: 0.85,
                    sdk_refs: vec!["DzRenderOptions".to_string(), "DzRenderMgr".to_string()],
                    requires_confirmation: false,
                })
            }
            ActionType::SetMaterialTexture => {
                let channel_default = "Base Color".to_string();
                let file_default = "".to_string();
                let channel = step.parameters.get("channel").unwrap_or(&channel_default);
                let file_path = step.parameters.get("file_path").unwrap_or(&file_default);
                Some(StructuredAiAction {
                    command: "set_material_texture".to_string(),
                    args: serde_json::json!({
                        "node_id": "selected",
                        "channel": channel,
                        "file_path": file_path,
                    }),
                    confidence: 0.7,
                    sdk_refs: vec!["DzImageProperty".to_string(), "DzDefaultMaterial".to_string()],
                    requires_confirmation: false,
                })
            }
            ActionType::Animate => {
                Some(StructuredAiAction {
                    command: "play_timeline".to_string(),
                    args: serde_json::json!({}),
                    confidence: 0.8,
                    sdk_refs: vec!["DzTime".to_string()],
                    requires_confirmation: false,
                })
            }
            ActionType::RunScript => {
                let script_default = "".to_string();
                let script = step.parameters.get("script").unwrap_or(&script_default);
                Some(StructuredAiAction {
                    command: "run_script".to_string(),
                    args: serde_json::json!({ "script": script }),
                    confidence: 0.6,
                    sdk_refs: vec!["DazScript".to_string()],
                    requires_confirmation: true,
                })
            }
            ActionType::SearchContent => {
                let query_default = "".to_string();
                let type_default = "figure".to_string();
                let max_default = "10".to_string();
                let query = step.parameters.get("query").unwrap_or(&query_default);
                let asset_type = step.parameters.get("type").unwrap_or(&type_default);
                let max_results = step.parameters.get("max_results").unwrap_or(&max_default);
                Some(StructuredAiAction {
                    command: "search_content".to_string(),
                    args: serde_json::json!({
                        "query": query,
                        "type": asset_type,
                        "max_results": max_results,
                    }),
                    confidence: 0.7,
                    sdk_refs: vec!["DzContentMgr".to_string()],
                    requires_confirmation: false,
                })
            }
            ActionType::ListBones => {
                let figure_id_default = "selected".to_string();
                let figure_id = step.parameters.get("figure_id").unwrap_or(&figure_id_default);
                Some(StructuredAiAction {
                    command: "list_bones".to_string(),
                    args: serde_json::json!({ "figure_id": figure_id }),
                    confidence: 0.9,
                    sdk_refs: vec!["DzBone".to_string(), "DzSkeleton".to_string()],
                    requires_confirmation: false,
                })
            }
            ActionType::SetBoneTransform => {
                let bone_default = "hip".to_string();
                let rot_default = "[0,0,0]".to_string();
                let bone = step.parameters.get("bone_name").unwrap_or(&bone_default);
                let rotation = step.parameters.get("rotation").unwrap_or(&rot_default);
                Some(StructuredAiAction {
                    command: "set_bone_transform".to_string(),
                    args: serde_json::json!({
                        "figure_id": "selected",
                        "bone_name": bone,
                        "rotation": rotation,
                    }),
                    confidence: 0.7,
                    sdk_refs: vec!["DzBone".to_string(), "DzSkeleton".to_string()],
                    requires_confirmation: true,
                })
            }
            ActionType::Custom(action_name) => {
                // Try to map known custom action names to bridge commands
                match action_name.as_str() {
                    "clear_scene" => Some(StructuredAiAction {
                        command: "clear_scene".to_string(),
                        args: serde_json::json!({}),
                        confidence: 0.9,
                        sdk_refs: vec!["DzScene".to_string()],
                        requires_confirmation: true,
                    }),
                    "frame_shot" => Some(StructuredAiAction {
                        command: "set_camera".to_string(),
                        args: serde_json::json!({
                            "camera": "Perspective View",
                            "focal_length": "85",
                            "focal_distance": "200",
                        }),
                        confidence: 0.7,
                        sdk_refs: vec!["DzCamera".to_string()],
                        requires_confirmation: false,
                    }),
                    "final_checks" => Some(StructuredAiAction {
                        command: "list_nodes".to_string(),
                        args: serde_json::json!({}),
                        confidence: 0.9,
                        sdk_refs: vec!["DzScene".to_string()],
                        requires_confirmation: false,
                    }),
                    "identify_issue" | "gather_info" => Some(StructuredAiAction {
                        command: "get_scene_info".to_string(),
                        args: serde_json::json!({}),
                        confidence: 0.9,
                        sdk_refs: vec!["DzScene".to_string()],
                        requires_confirmation: false,
                    }),
                    "analyze_scene" => Some(StructuredAiAction {
                        command: "get_scene_info".to_string(),
                        args: serde_json::json!({}),
                        confidence: 0.9,
                        sdk_refs: vec!["DzScene".to_string()],
                        requires_confirmation: false,
                    }),
                    "setup_timeline" => Some(StructuredAiAction {
                        command: "set_timeline_range".to_string(),
                        args: serde_json::json!({
                            "start_frame": "1",
                            "end_frame": "30",
                        }),
                        confidence: 0.85,
                        sdk_refs: vec!["DzTime".to_string()],
                        requires_confirmation: false,
                    }),
                    "pose_keyframes" => Some(StructuredAiAction {
                        command: "apply_pose".to_string(),
                        args: serde_json::json!({
                            "pose_path": "",
                            "figure_id": "selected",
                        }),
                        confidence: 0.6,
                        sdk_refs: vec!["DzPose".to_string(), "DzKeyframe".to_string()],
                        requires_confirmation: false,
                    }),
                    "select_pose" => Some(StructuredAiAction {
                        command: "search_content".to_string(),
                        args: serde_json::json!({
                            "query": "pose",
                            "type": "pose",
                            "max_results": "5",
                        }),
                        confidence: 0.7,
                        sdk_refs: vec!["DzContentMgr".to_string()],
                        requires_confirmation: false,
                    }),
                    "add_secondary_motion" => Some(StructuredAiAction {
                        command: "run_dforce_simulation".to_string(),
                        args: serde_json::json!({
                            "node_id": "selected",
                            "start_frame": 0,
                            "end_frame": 30,
                        }),
                        confidence: 0.6,
                        sdk_refs: vec!["DzSimulator".to_string()],
                        requires_confirmation: true,
                    }),
                    "apply_fix" => Some(StructuredAiAction {
                        command: "set_property".to_string(),
                        args: serde_json::json!({
                            "node_id": "selected",
                            "property": "unknown",
                            "value": "0",
                        }),
                        confidence: 0.5,
                        sdk_refs: vec![],
                        requires_confirmation: true,
                    }),
                    "verify_fix" => Some(StructuredAiAction {
                        command: "get_scene_info".to_string(),
                        args: serde_json::json!({}),
                        confidence: 0.9,
                        sdk_refs: vec!["DzScene".to_string()],
                        requires_confirmation: false,
                    }),
                    _ => {
                        // Unknown custom actions return None (will be skipped)
                        None
                    }
                }
            }
        }
    }
    
    /// Validate and refine a plan
    fn validate_and_refine(&self, plan: Plan, _context: &PlanningContext) -> Option<Plan> {
        let wants_clear = plan.goal.description.to_lowercase().contains("clear")
            || plan.goal.description.to_lowercase().contains("empty")
            || plan.goal.description.to_lowercase().contains("from scratch");

        let mut steps: Vec<PlanStep> = plan.steps.into_iter()
            .filter(|step| {
                if step.action.command == "clear_scene" && !wants_clear {
                    return false;
                }

                !step.action.args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .map(|path| path.is_empty() || path.contains("TODO") || path.contains("unknown"))
                    .unwrap_or(false)
            })
            .collect();

        let valid_ids: std::collections::HashSet<String> = steps.iter().map(|step| step.id.clone()).collect();
        for step in &mut steps {
            step.prerequisites.retain(|prereq| valid_ids.contains(prereq));
        }

        if steps.is_empty() {
            return None;
        }

        let estimated_total_time_seconds = steps.iter().map(|step| step.estimated_time_seconds).sum();
        let confidence = steps.iter().map(|step| step.confidence).sum::<f32>() / steps.len() as f32;
        let has_high_risk = steps.iter().any(|step| step.action.requires_confirmation);

        Some(Plan {
            steps,
            estimated_total_time_seconds,
            confidence,
            risk_level: if has_high_risk { RiskLevel::High } else { plan.risk_level },
            ..plan
        })
    }
    
    /// Infer workflow type from goal
    fn infer_workflow_type(&self, goal: &Goal) -> Option<WorkflowType> {
        let lower_desc = goal.description.to_lowercase();
        let _lower_intent = format!("{:?}", goal.intent).to_lowercase();
        
        // Check for workflow keywords in description.
        // Scene/full/complete have highest priority (full scene creation).
        if lower_desc.contains("scene") || lower_desc.contains("full") || lower_desc.contains("complete") {
            return Some(WorkflowType::CreateScene);
        }
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

    fn plan_from_steps(&self, goal: &Goal, description: &str, steps: Vec<PlanStep>, risk_level: RiskLevel) -> Plan {
        let estimated_total_time_seconds = steps.iter().map(|s| s.estimated_time_seconds).sum();
        let confidence = if steps.is_empty() {
            0.0
        } else {
            steps.iter().map(|s| s.confidence).sum::<f32>() / steps.len() as f32
        };
        Plan {
            id: format!("plan_{}_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(), fastrand::u32(..)),
            goal_id: goal.id.clone(),
            goal: goal.clone(),
            description: description.to_string(),
            steps,
            estimated_total_time_seconds,
            confidence,
            risk_level,
            fallback_plan: None,
        }
    }
}

fn resolve_asset_path(query: &str, category: Option<&str>, context: &PlanningContext) -> Option<String> {
    let query_lower = query.to_lowercase();
    let category_lower = category.map(|c| c.to_lowercase());

    context.available_assets.iter()
        .find(|asset| {
            let category_matches = category_lower
                .as_ref()
                .map(|category| asset.category.eq_ignore_ascii_case(category)
                    || asset.category.eq_ignore_ascii_case(category.trim_end_matches('s')))
                .unwrap_or(true);
            category_matches && (
                asset.name.to_lowercase().contains(&query_lower)
                    || asset.tags.iter().any(|tag| tag.eq_ignore_ascii_case(query))
                    || asset.visual_description
                        .as_ref()
                        .map(|desc| desc.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            )
        })
        .map(|asset| asset.path.clone())
        .or_else(|| crate::ai_action::search_best_matching_asset(query))
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
