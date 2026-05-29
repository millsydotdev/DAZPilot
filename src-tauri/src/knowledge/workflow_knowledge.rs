use crate::knowledge::daz_concepts::DazKnowledgeBase;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maps workflow ActionTypes to the actual bridge command names and parameter templates
#[derive(Debug, Clone)]
pub struct CommandMapping {
    /// Primary bridge command to execute for this action
    pub command: &'static str,
    /// Alternative commands if primary doesn't apply
    pub alternatives: &'static [&'static str],
    /// How to map workflow parameters to command parameters
    pub param_mapping: &'static [(&'static str, &'static str)],
}

/// Returns the bridge command map for all ActionTypes.
/// This bridges workflow planning with command execution.
pub fn action_to_command_map() -> HashMap<ActionType, CommandMapping> {
    let mut m = HashMap::new();
    m.insert(
        ActionType::LoadAsset,
        CommandMapping {
            command: "load_asset",
            alternatives: &["add_figure", "add_node"],
            param_mapping: &[("asset_type", "path"), ("item_type", "path")],
        },
    );
    m.insert(
        ActionType::ApplyPose,
        CommandMapping {
            command: "apply_pose",
            alternatives: &["set_morph", "set_bone_transform"],
            param_mapping: &[("pose_application", "path")],
        },
    );
    m.insert(
        ActionType::AdjustProperty,
        CommandMapping {
            command: "set_property",
            alternatives: &[
                "set_morph",
                "set_light",
                "set_render_settings",
                "set_camera",
                "set_material_property",
            ],
            param_mapping: &[
                ("property_type", "property_name"),
                ("adjustment_type", "property_name"),
            ],
        },
    );
    m.insert(
        ActionType::CreateLight,
        CommandMapping {
            command: "add_node",
            alternatives: &["set_light"],
            param_mapping: &[("light_purpose", "type")],
        },
    );
    m.insert(
        ActionType::CreateCamera,
        CommandMapping {
            command: "add_node",
            alternatives: &["set_camera"],
            param_mapping: &[("camera_type", "type")],
        },
    );
    m.insert(
        ActionType::Render,
        CommandMapping {
            command: "render_preview",
            alternatives: &["set_render_options", "set_render_settings"],
            param_mapping: &[],
        },
    );
    m.insert(
        ActionType::ChangeMaterial,
        CommandMapping {
            command: "set_material_property",
            alternatives: &["set_material_texture", "get_material_channels"],
            param_mapping: &[("material_type", "material_name")],
        },
    );
    m.insert(
        ActionType::ExportScene,
        CommandMapping {
            command: "export_scene",
            alternatives: &["save_scene"],
            param_mapping: &[],
        },
    );
    m.insert(
        ActionType::RunSimulation,
        CommandMapping {
            command: "run_dforce_simulation",
            alternatives: &[],
            param_mapping: &[],
        },
    );
    m.insert(
        ActionType::UndoBatchBegin,
        CommandMapping {
            command: "begin_undo_batch",
            alternatives: &[],
            param_mapping: &[],
        },
    );
    m.insert(
        ActionType::UndoBatchEnd,
        CommandMapping {
            command: "accept_undo_batch",
            alternatives: &["cancel_undo_batch"],
            param_mapping: &[("caption", "caption")],
        },
    );
    m.insert(
        ActionType::AddFigure,
        CommandMapping {
            command: "add_figure",
            alternatives: &["load_asset"],
            param_mapping: &[("figure_type", "figure_type")],
        },
    );
    m.insert(
        ActionType::SetMorph,
        CommandMapping {
            command: "set_morph",
            alternatives: &["apply_morph", "adjust_property"],
            param_mapping: &[("morph", "morph"), ("value", "value")],
        },
    );
    m.insert(
        ActionType::SetExpression,
        CommandMapping {
            command: "apply_expression",
            alternatives: &["set_morph"],
            param_mapping: &[("expression", "expression_id"), ("value", "value")],
        },
    );
    m.insert(
        ActionType::SetCamera,
        CommandMapping {
            command: "set_camera",
            alternatives: &["add_node"],
            param_mapping: &[
                ("camera", "camera"),
                ("focal_length", "focal_length"),
                ("focal_distance", "focal_distance"),
            ],
        },
    );
    m.insert(
        ActionType::SetRenderOptions,
        CommandMapping {
            command: "set_render_options",
            alternatives: &["set_render_settings", "render_preview"],
            param_mapping: &[
                ("width", "width"),
                ("height", "height"),
                ("pixel_samples", "pixel_samples"),
            ],
        },
    );
    m.insert(
        ActionType::SetMaterialTexture,
        CommandMapping {
            command: "set_material_texture",
            alternatives: &["set_material_property"],
            param_mapping: &[("channel", "channel"), ("file_path", "file_path")],
        },
    );
    m.insert(
        ActionType::SetBodyOpacity,
        CommandMapping {
            command: "set_body_opacity",
            alternatives: &["set_material_property"],
            param_mapping: &[("figure", "node_id"), ("opacity", "value")],
        },
    );
    m.insert(
        ActionType::SetSurfaceOpacity,
        CommandMapping {
            command: "set_surface_opacity",
            alternatives: &["set_material_property"],
            param_mapping: &[
                ("figure", "node_id"),
                ("surface", "surface_pattern"),
                ("opacity", "value"),
            ],
        },
    );
    m.insert(
        ActionType::GetInternalSurfaces,
        CommandMapping {
            command: "get_internal_surfaces",
            alternatives: &["get_material_zones"],
            param_mapping: &[("figure", "node_id")],
        },
    );
    m.insert(
        ActionType::ShowAnatomy,
        CommandMapping {
            command: "show_anatomy",
            alternatives: &["set_surface_opacity"],
            param_mapping: &[("figure", "node_id")],
        },
    );
    m.insert(
        ActionType::PlaceAssetInside,
        CommandMapping {
            command: "place_asset_inside",
            alternatives: &["load_asset", "set_node_transform"],
            param_mapping: &[("figure", "figure_id"), ("asset", "asset_path")],
        },
    );
    m.insert(
        ActionType::Animate,
        CommandMapping {
            command: "play_timeline",
            alternatives: &["set_keyframe", "seek_to_frame"],
            param_mapping: &[],
        },
    );
    m.insert(
        ActionType::RunScript,
        CommandMapping {
            command: "run_script",
            alternatives: &[],
            param_mapping: &[("script", "script")],
        },
    );
    m.insert(
        ActionType::SearchContent,
        CommandMapping {
            command: "search_content",
            alternatives: &["load_asset"],
            param_mapping: &[
                ("query", "query"),
                ("type", "type"),
                ("max_results", "max_results"),
            ],
        },
    );
    m.insert(
        ActionType::ListBones,
        CommandMapping {
            command: "list_bones",
            alternatives: &["get_node_properties"],
            param_mapping: &[("figure_id", "figure_id")],
        },
    );
    m.insert(
        ActionType::SetBoneTransform,
        CommandMapping {
            command: "set_bone_transform",
            alternatives: &["set_property", "set_morph"],
            param_mapping: &[
                ("bone_name", "bone_name"),
                ("position", "position"),
                ("rotation", "rotation"),
            ],
        },
    );
    m
}

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// Add a Genesis figure to the scene
    AddFigure,
    /// Set a morph dial value on a figure (0.0-1.0)
    SetMorph,
    /// Set an expression dial value on a figure
    SetExpression,
    /// Switch active camera or adjust its properties
    SetCamera,
    /// Configure detailed render options (samples, depth, gamma)
    SetRenderOptions,
    /// Assign a texture map to a material surface channel
    SetMaterialTexture,
    /// Set uniform opacity across a figure/body
    SetBodyOpacity,
    /// Set opacity for one or more matching material surfaces
    SetSurfaceOpacity,
    /// Discover likely internal anatomy material surfaces
    GetInternalSurfaces,
    /// Make likely internal anatomy material surfaces fully opaque
    ShowAnatomy,
    /// Load and position an asset inside a figure
    PlaceAssetInside,
    /// Trigger timeline playback or animation
    Animate,
    /// Run arbitrary DazScript
    RunScript,
    /// Search the Daz content library
    SearchContent,
    /// List bones in a figure's skeleton
    ListBones,
    /// Set a bone's world-space transform
    SetBoneTransform,
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
        self.workflow_templates.insert(
            WorkflowType::CreateCharacter,
            WorkflowTemplate {
                workflow_type: WorkflowType::CreateCharacter,
                name: "Create Character".to_string(),
                description: "Load a character and prepare it for use in a scene".to_string(),
                steps: vec![
                    WorkflowStepTemplate {
                        description: "Load base figure".to_string(),
                        action_type: ActionType::LoadAsset,
                        parameters_template: HashMap::from([(
                            "asset_type".to_string(),
                            "figure".to_string(),
                        )]),
                        prerequisites: vec![],
                        estimated_time_seconds: 5,
                        difficulty: DifficultyLevel::Trivial,
                    },
                    WorkflowStepTemplate {
                        description: "Apply desired morphs".to_string(),
                        action_type: ActionType::AdjustProperty,
                        parameters_template: HashMap::from([(
                            "property_type".to_string(),
                            "morph".to_string(),
                        )]),
                        prerequisites: vec!["load_base_figure".to_string()],
                        estimated_time_seconds: 10,
                        difficulty: DifficultyLevel::Easy,
                    },
                    WorkflowStepTemplate {
                        description: "Apply base pose".to_string(),
                        action_type: ActionType::ApplyPose,
                        parameters_template: HashMap::from([(
                            "pose_type".to_string(),
                            "basic".to_string(),
                        )]),
                        prerequisites: vec![
                            "load_base_figure".to_string(),
                            "apply_desired_morphs".to_string(),
                        ],
                        estimated_time_seconds: 5,
                        difficulty: DifficultyLevel::Trivial,
                    },
                ],
                tags: vec![
                    "character".to_string(),
                    "setup".to_string(),
                    "basics".to_string(),
                ],
            },
        );

        // Create Outfit workflow
        self.workflow_templates.insert(
            WorkflowType::CreateOutfit,
            WorkflowTemplate {
                workflow_type: WorkflowType::CreateOutfit,
                name: "Create Outfit".to_string(),
                description: "Dress a figure with clothing and accessories".to_string(),
                steps: vec![
                    WorkflowStepTemplate {
                        description: "Ensure figure is loaded and posed".to_string(),
                        action_type: ActionType::LoadAsset,
                        parameters_template: HashMap::from([(
                            "asset_type".to_string(),
                            "figure".to_string(),
                        )]),
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
                        parameters_template: HashMap::from([(
                            "adjustment_type".to_string(),
                            "fit_control".to_string(),
                        )]),
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
                        parameters_template: HashMap::from([(
                            "asset_type".to_string(),
                            "accessories".to_string(),
                        )]),
                        prerequisites: vec!["add_secondary_clothing".to_string()],
                        estimated_time_seconds: 8,
                        difficulty: DifficultyLevel::Easy,
                    },
                ],
                tags: vec![
                    "clothing".to_string(),
                    "outfit".to_string(),
                    "character".to_string(),
                ],
            },
        );

        // Setup Lighting workflow
        self.workflow_templates.insert(
            WorkflowType::SetupLighting,
            WorkflowTemplate {
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
                        parameters_template: HashMap::from([(
                            "light_purpose".to_string(),
                            "key".to_string(),
                        )]),
                        prerequisites: vec!["analyze_scene".to_string()],
                        estimated_time_seconds: 5,
                        difficulty: DifficultyLevel::Easy,
                    },
                    WorkflowStepTemplate {
                        description: "Add fill light".to_string(),
                        action_type: ActionType::CreateLight,
                        parameters_template: HashMap::from([(
                            "light_purpose".to_string(),
                            "fill".to_string(),
                        )]),
                        prerequisites: vec!["add_key_light".to_string()],
                        estimated_time_seconds: 5,
                        difficulty: DifficultyLevel::Easy,
                    },
                    WorkflowStepTemplate {
                        description: "Add rim/back light".to_string(),
                        action_type: ActionType::CreateLight,
                        parameters_template: HashMap::from([(
                            "light_purpose".to_string(),
                            "rim".to_string(),
                        )]),
                        prerequisites: vec!["add_fill_light".to_string()],
                        estimated_time_seconds: 5,
                        difficulty: DifficultyLevel::Easy,
                    },
                    WorkflowStepTemplate {
                        description: "Adjust lighting ratios and colors".to_string(),
                        action_type: ActionType::AdjustProperty,
                        parameters_template: HashMap::from([(
                            "adjustment_type".to_string(),
                            "lighting_balance".to_string(),
                        )]),
                        prerequisites: vec!["add_rim_light".to_string()],
                        estimated_time_seconds: 10,
                        difficulty: DifficultyLevel::Moderate,
                    },
                ],
                tags: vec![
                    "lighting".to_string(),
                    "setup".to_string(),
                    "scene".to_string(),
                ],
            },
        );

        // Create Scene workflow (full scene from scratch)
        self.workflow_templates.insert(
            WorkflowType::CreateScene,
            WorkflowTemplate {
                workflow_type: WorkflowType::CreateScene,
                name: "Create Scene".to_string(),
                description:
                    "Build a complete Daz3D scene with figures, lighting, camera, and rendering"
                        .to_string(),
                steps: vec![
                    WorkflowStepTemplate {
                        description: "Begin undo batch for scene creation".to_string(),
                        action_type: ActionType::UndoBatchBegin,
                        parameters_template: HashMap::new(),
                        prerequisites: vec![],
                        estimated_time_seconds: 1,
                        difficulty: DifficultyLevel::Trivial,
                    },
                    WorkflowStepTemplate {
                        description: "Clear existing scene if needed".to_string(),
                        action_type: ActionType::Custom("clear_scene".to_string()),
                        parameters_template: HashMap::new(),
                        prerequisites: vec![],
                        estimated_time_seconds: 2,
                        difficulty: DifficultyLevel::Easy,
                    },
                    WorkflowStepTemplate {
                        description: "Add base figure".to_string(),
                        action_type: ActionType::AddFigure,
                        parameters_template: HashMap::from([(
                            "figure_type".to_string(),
                            "genesis9".to_string(),
                        )]),
                        prerequisites: vec![],
                        estimated_time_seconds: 5,
                        difficulty: DifficultyLevel::Trivial,
                    },
                    WorkflowStepTemplate {
                        description: "Load environment if specified".to_string(),
                        action_type: ActionType::LoadAsset,
                        parameters_template: HashMap::from([(
                            "asset_type".to_string(),
                            "environment".to_string(),
                        )]),
                        prerequisites: vec!["add_base_figure".to_string()],
                        estimated_time_seconds: 8,
                        difficulty: DifficultyLevel::Easy,
                    },
                    WorkflowStepTemplate {
                        description: "Apply figure morphs for desired appearance".to_string(),
                        action_type: ActionType::SetMorph,
                        parameters_template: HashMap::from([
                            ("morph".to_string(), "Head_Height".to_string()),
                            ("value".to_string(), "0.3".to_string()),
                        ]),
                        prerequisites: vec![
                            "add_base_figure".to_string(),
                            "load_environment".to_string(),
                        ],
                        estimated_time_seconds: 10,
                        difficulty: DifficultyLevel::Easy,
                    },
                    WorkflowStepTemplate {
                        description: "Load clothing if specified".to_string(),
                        action_type: ActionType::LoadAsset,
                        parameters_template: HashMap::from([(
                            "asset_type".to_string(),
                            "clothing".to_string(),
                        )]),
                        prerequisites: vec!["apply_figure_morphs".to_string()],
                        estimated_time_seconds: 8,
                        difficulty: DifficultyLevel::Easy,
                    },
                    WorkflowStepTemplate {
                        description: "Load props if specified".to_string(),
                        action_type: ActionType::LoadAsset,
                        parameters_template: HashMap::from([(
                            "asset_type".to_string(),
                            "props".to_string(),
                        )]),
                        prerequisites: vec!["load_clothing".to_string()],
                        estimated_time_seconds: 6,
                        difficulty: DifficultyLevel::Easy,
                    },
                    WorkflowStepTemplate {
                        description: "Apply base pose to figure".to_string(),
                        action_type: ActionType::ApplyPose,
                        parameters_template: HashMap::from([(
                            "pose_type".to_string(),
                            "basic".to_string(),
                        )]),
                        prerequisites: vec!["load_clothing".to_string(), "load_props".to_string()],
                        estimated_time_seconds: 5,
                        difficulty: DifficultyLevel::Easy,
                    },
                    WorkflowStepTemplate {
                        description: "Set up key light".to_string(),
                        action_type: ActionType::CreateLight,
                        parameters_template: HashMap::from([
                            ("light_purpose".to_string(), "key".to_string()),
                            ("light_type".to_string(), "distant_light".to_string()),
                        ]),
                        prerequisites: vec!["apply_base_pose".to_string()],
                        estimated_time_seconds: 4,
                        difficulty: DifficultyLevel::Easy,
                    },
                    WorkflowStepTemplate {
                        description: "Set up fill light".to_string(),
                        action_type: ActionType::CreateLight,
                        parameters_template: HashMap::from([
                            ("light_purpose".to_string(), "fill".to_string()),
                            ("light_type".to_string(), "point_light".to_string()),
                        ]),
                        prerequisites: vec!["set_up_key_light".to_string()],
                        estimated_time_seconds: 4,
                        difficulty: DifficultyLevel::Easy,
                    },
                    WorkflowStepTemplate {
                        description: "Set up rim/back light".to_string(),
                        action_type: ActionType::CreateLight,
                        parameters_template: HashMap::from([
                            ("light_purpose".to_string(), "rim".to_string()),
                            ("light_type".to_string(), "spot_light".to_string()),
                        ]),
                        prerequisites: vec!["set_up_fill_light".to_string()],
                        estimated_time_seconds: 4,
                        difficulty: DifficultyLevel::Easy,
                    },
                    WorkflowStepTemplate {
                        description: "Adjust light properties (intensity, color)".to_string(),
                        action_type: ActionType::AdjustProperty,
                        parameters_template: HashMap::from([(
                            "adjustment_type".to_string(),
                            "lighting_balance".to_string(),
                        )]),
                        prerequisites: vec!["set_up_rim_light".to_string()],
                        estimated_time_seconds: 6,
                        difficulty: DifficultyLevel::Moderate,
                    },
                    WorkflowStepTemplate {
                        description: "Configure camera framing".to_string(),
                        action_type: ActionType::SetCamera,
                        parameters_template: HashMap::from([
                            ("camera".to_string(), "Perspective View".to_string()),
                            ("focal_length".to_string(), "85".to_string()),
                            ("focal_distance".to_string(), "200".to_string()),
                        ]),
                        prerequisites: vec!["adjust_light_properties".to_string()],
                        estimated_time_seconds: 4,
                        difficulty: DifficultyLevel::Easy,
                    },
                    WorkflowStepTemplate {
                        description: "Set render options for final output".to_string(),
                        action_type: ActionType::SetRenderOptions,
                        parameters_template: HashMap::from([
                            ("width".to_string(), "1920".to_string()),
                            ("height".to_string(), "1080".to_string()),
                            ("pixel_samples".to_string(), "256".to_string()),
                        ]),
                        prerequisites: vec!["configure_camera".to_string()],
                        estimated_time_seconds: 3,
                        difficulty: DifficultyLevel::Easy,
                    },
                    WorkflowStepTemplate {
                        description: "Render the scene".to_string(),
                        action_type: ActionType::Render,
                        parameters_template: HashMap::new(),
                        prerequisites: vec!["set_render_options".to_string()],
                        estimated_time_seconds: 5,
                        difficulty: DifficultyLevel::Easy,
                    },
                    WorkflowStepTemplate {
                        description: "Accept undo batch".to_string(),
                        action_type: ActionType::UndoBatchEnd,
                        parameters_template: HashMap::from([(
                            "caption".to_string(),
                            "AI Created Scene".to_string(),
                        )]),
                        prerequisites: vec!["render_scene".to_string()],
                        estimated_time_seconds: 1,
                        difficulty: DifficultyLevel::Trivial,
                    },
                ],
                tags: vec![
                    "scene".to_string(),
                    "full".to_string(),
                    "complete".to_string(),
                    "creation".to_string(),
                ],
            },
        );

        // Pose Character workflow
        self.workflow_templates.insert(
            WorkflowType::PoseCharacter,
            WorkflowTemplate {
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
                        parameters_template: HashMap::from([(
                            "pose_application".to_string(),
                            "full_apply".to_string(),
                        )]),
                        prerequisites: vec!["select_or_create_pose".to_string()],
                        estimated_time_seconds: 5,
                        difficulty: DifficultyLevel::Easy,
                    },
                    WorkflowStepTemplate {
                        description: "Adjust pose with morphs if needed".to_string(),
                        action_type: ActionType::AdjustProperty,
                        parameters_template: HashMap::from([(
                            "adjustment_type".to_string(),
                            "pose_morphs".to_string(),
                        )]),
                        prerequisites: vec!["apply_pose_to_figure".to_string()],
                        estimated_time_seconds: 10,
                        difficulty: DifficultyLevel::Moderate,
                    },
                    WorkflowStepTemplate {
                        description: "Fine-tune with joint adjustments".to_string(),
                        action_type: ActionType::AdjustProperty,
                        parameters_template: HashMap::from([(
                            "adjustment_type".to_string(),
                            "joint_adjustment".to_string(),
                        )]),
                        prerequisites: vec!["adjust_pose_with_morphs".to_string()],
                        estimated_time_seconds: 15,
                        difficulty: DifficultyLevel::Hard,
                    },
                ],
                tags: vec![
                    "posing".to_string(),
                    "character".to_string(),
                    "animation".to_string(),
                ],
            },
        );

        // Animate Character workflow
        self.workflow_templates.insert(
            WorkflowType::AnimateCharacter,
            WorkflowTemplate {
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
                        parameters_template: HashMap::from([(
                            "adjustment_type".to_string(),
                            "interpolation".to_string(),
                        )]),
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
                tags: vec![
                    "animation".to_string(),
                    "character".to_string(),
                    "timeline".to_string(),
                ],
            },
        );

        // Render Still workflow
        self.workflow_templates.insert(
            WorkflowType::RenderStill,
            WorkflowTemplate {
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
                        parameters_template: HashMap::from([(
                            "adjustment_type".to_string(),
                            "camera_settings".to_string(),
                        )]),
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
                        parameters_template: HashMap::from([(
                            "caption".to_string(),
                            "AI Generated Render".to_string(),
                        )]),
                        prerequisites: vec!["trigger_render".to_string()],
                        estimated_time_seconds: 1,
                        difficulty: DifficultyLevel::Trivial,
                    },
                ],
                tags: vec![
                    "render".to_string(),
                    "still".to_string(),
                    "output".to_string(),
                ],
            },
        );

        // Fix Common Issue workflow
        self.workflow_templates.insert(
            WorkflowType::FixCommonIssue,
            WorkflowTemplate {
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
                tags: vec![
                    "troubleshooting".to_string(),
                    "fix".to_string(),
                    "support".to_string(),
                ],
            },
        );
    }

    /// Get a workflow by ID
    pub fn get_workflow(&self, id: &str) -> Option<&Workflow> {
        self.workflows.get(id)
    }

    /// Get workflows by tag
    pub fn get_workflows_by_tag(&self, tag: &str) -> Vec<&Workflow> {
        self.workflows
            .values()
            .filter(|w| w.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Get workflows suitable for a goal
    pub fn get_workflows_for_goal(&self, goal: &str) -> Vec<&Workflow> {
        let lower_goal = goal.to_lowercase();
        self.workflows
            .values()
            .filter(|w| {
                w.name.to_lowercase().contains(&lower_goal)
                    || w.description.to_lowercase().contains(&lower_goal)
                    || w.tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&lower_goal))
            })
            .collect()
    }

    /// Generate a workflow from a template
    pub fn generate_workflow_from_template(
        &self,
        workflow_type: WorkflowType,
        parameters: &HashMap<String, String>,
    ) -> Option<Workflow> {
        let template = self.workflow_templates.get(&workflow_type)?;

        // Convert template steps to actual steps with parameter substitution
        let mut steps = Vec::new();

        let template_ids: Vec<String> = template
            .steps
            .iter()
            .map(|step| slugify_workflow_step(&step.description))
            .collect();

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
                if let Some(prev_idx) = template_ids[..step_idx].iter().position(|id| id == prereq)
                {
                    prerequisites.push(format!("step_{}_{}", workflow_type as u32, prev_idx));
                } else {
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
        }

        Some(Workflow {
            id: format!(
                "workflow_{}_{}",
                workflow_type as u32,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ),
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
                    .to_string(),
            );

            // Update success rate using exponential moving average
            let alpha = 0.3; // learning rate
            let new_success = if success { 1.0 } else { 0.0 };
            workflow.success_rate = (1.0 - alpha) * workflow.success_rate + alpha * new_success;
        }
    }
}

fn slugify_workflow_step(description: &str) -> String {
    description
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

impl Default for WorkflowKnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}
