use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core Daz3D concepts that the AI needs to understand for reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Figure {
    pub name: String,
    pub base_meshes: Vec<String>,
    pub compatible_categories: Vec<String>,
    pub popular_morphs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Morph {
    pub name: String,
    pub category: String, // body, face, expression, etc.
    pub figure: String,   // which figure it works with
    pub min_value: f32,
    pub max_value: f32,
    pub default_value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pose {
    pub name: String,
    pub figure: String,
    pub description: String,
    pub keyframes: Vec<PoseKeyframe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoseKeyframe {
    pub node: String,
    pub property: String,
    pub frame: i32,
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
    pub r#type: String, // point, spot, distant, area
    pub color: String,  // RGB or temperature
    pub intensity_range: (f32, f32),
    pub typical_use: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub name: String,
    pub shader_type: String, // uber, skin, hair, cloth, etc.
    pub compatible_with: Vec<String>, // what figures/objects it works on
    pub adjustable_properties: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCategory {
    pub name: String,
    pub description: String,
    pub typical_use_cases: Vec<String>,
    pub compatible_with_figures: Vec<String>,
    pub required_props: Vec<String>, // what properties are typically adjusted
}

/// Knowledge base of Daz3D concepts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DazKnowledgeBase {
    pub figures: HashMap<String, Figure>,
    pub morphs: HashMap<String, Morph>,
    pub poses: HashMap<String, Pose>,
    pub lights: HashMap<String, Light>,
    pub materials: HashMap<String, Material>,
    pub asset_categories: HashMap<String, AssetCategory>,
}

impl DazKnowledgeBase {
    pub fn new() -> Self {
        let mut knowledge = DazKnowledgeBase {
            figures: HashMap::new(),
            morphs: HashMap::new(),
            poses: HashMap::new(),
            lights: HashMap::new(),
            materials: HashMap::new(),
            asset_categories: HashMap::new(),
        };
        
        // Initialize with core Daz3D concepts
        knowledge.init_figures();
        knowledge.init_morphs();
        knowledge.init_poses();
        knowledge.init_lights();
        knowledge.init_materials();
        knowledge.init_asset_categories();
        
        knowledge
    }
    
    fn init_figures(&mut self) {
        self.figures.insert("Genesis 8 Female".to_string(), Figure {
            name: "Genesis 8 Female".to_string(),
            base_meshes: vec![
                "Head".to_string(),
                "Neck".to_string(), 
                "Torso".to_string(),
                "Abdomen".to_string(),
                "Chest".to_string(),
                "Pelvis".to_string(),
                "Left Clavicle".to_string(),
                "Right Clavicle".to_string(),
                "Left UpperArm".to_string(),
                "Right UpperArm".to_string(),
                "Left Forearm".to_string(),
                "Right Forearm".to_string(),
                "Left Hand".to_string(),
                "Right Hand".to_string(),
                "Left UpperLeg".to_string(),
                "Right UpperLeg".to_string(),
                "Left LowerLeg".to_string(),
                "Right LowerLeg".to_string(),
                "Left Foot".to_string(),
                "Right Foot".to_string(),
            ],
            compatible_categories: vec![
                "clothing".to_string(),
                "hair".to_string(),
                "poses".to_string(),
                "morphs".to_string(),
                "materials".to_string(),
                "accessories".to_string(),
            ],
            popular_morphs: vec![
                "Fitness".to_string(),
                "BodyBuilder".to_string(),
                "Pregnant".to_string(),
                "Old".to_string(),
                "Teen".to_string(),
                "Expression_Basic".to_string(),
            ],
        });
        
        self.figures.insert("Genesis 9 Female".to_string(), Figure {
            name: "Genesis 9 Female".to_string(),
            base_meshes: vec![
                "Head".to_string(),
                "Neck".to_string(),
                "Torso".to_string(), 
                "Abdomen".to_string(),
                "Chest".to_string(),
                "Pelvis".to_string(),
                "Left Clavicle".to_string(),
                "Right Clavicle".to_string(),
                "Left UpperArm".to_string(),
                "Right UpperArm".to_string(),
                "Left Forearm".to_string(),
                "Right Forearm".to_string(),
                "Left Hand".to_string(),
                "Right Hand".to_string(),
                "Left UpperLeg".to_string(),
                "Right UpperLeg".to_string(),
                "Left LowerLeg".to_string(),
                "Right LowerLeg".to_string(),
                "Left Foot".to_string(),
                "Right Foot".to_string(),
            ],
            compatible_categories: vec![
                "clothing".to_string(),
                "hair".to_string(),
                "poses".to_string(),
                "morphs".to_string(),
                "materials".to_string(),
                "accessories".to_string(),
            ],
            popular_morphs: vec![
                "Fitness".to_string(),
                "BodyBuilder".to_string(),
                "Pregnant".to_string(),
                "Old".to_string(),
                "Teen".to_string(),
                "Expression_Basic".to_string(),
                "Facial_Expressions".to_string(),
            ],
        });
        
        self.figures.insert("Genesis 8 Male".to_string(), Figure {
            name: "Genesis 8 Male".to_string(),
            base_meshes: vec![
                "Head".to_string(),
                "Neck".to_string(),
                "Torso".to_string(),
                "Abdomen".to_string(),
                "Chest".to_string(),
                "Pelvis".to_string(),
                "Left Clavicle".to_string(),
                "Right Clavicle".to_string(),
                "Left UpperArm".to_string(),
                "Right UpperArm".to_string(),
                "Left Forearm".to_string(),
                "Right Forearm".to_string(),
                "Left Hand".to_string(),
                "Right Hand".to_string(),
                "Left UpperLeg".to_string(),
                "Right UpperLeg".to_string(),
                "Left LowerLeg".to_string(),
                "Right LowerLeg".to_string(),
                "Left Foot".to_string(),
                "Right Foot".to_string(),
            ],
            compatible_categories: vec![
                "clothing".to_string(),
                "hair".to_string(),
                "poses".to_string(),
                "morphs".to_string(),
                "materials".to_string(),
                "accessories".to_string(),
            ],
            popular_morphs: vec![
                "Fitness".to_string(),
                "BodyBuilder".to_string(),
                "Old".to_string(),
                "Teen".to_string(),
                "Expression_Basic".to_string(),
            ],
        });
    }
    
    fn init_morphs(&mut self) {
        // Body morphs
        self.morphs.insert("Fitness".to_string(), Morph {
            name: "Fitness".to_string(),
            category: "body".to_string(),
            figure: "Genesis 8 Female".to_string(),
            min_value: 0.0,
            max_value: 1.0,
            default_value: 0.0,
        });
        
        self.morphs.insert("BodyBuilder".to_string(), Morph {
            name: "BodyBuilder".to_string(),
            category: "body".to_string(),
            figure: "Genesis 8 Female".to_string(),
            min_value: 0.0,
            max_value: 1.0,
            default_value: 0.0,
        });
        
        self.morphs.insert("Pregnant".to_string(), Morph {
            name: "Pregnant".to_string(),
            category: "body".to_string(),
            figure: "Genesis 8 Female".to_string(),
            min_value: 0.0,
            max_value: 1.0,
            default_value: 0.0,
        });
        
        // Face morphs
        self.morphs.insert("Expression_Basic".to_string(), Morph {
            name: "Expression_Basic".to_string(),
            category: "expression".to_string(),
            figure: "Genesis 8 Female".to_string(),
            min_value: 0.0,
            max_value: 1.0,
            default_value: 0.0,
        });
        
        self.morphs.insert("Facial_Expressions".to_string(), Morph {
            name: "Facial_Expressions".to_string(),
            category: "expression".to_string(),
            figure: "Genesis 9 Female".to_string(),
            min_value: 0.0,
            max_value: 1.0,
            default_value: 0.0,
        });
    }
    
    fn init_poses(&mut self) {
        self.poses.insert("Basic Standing".to_string(), Pose {
            name: "Basic Standing".to_string(),
            figure: "Genesis 8 Female".to_string(),
            description: "A neutral standing pose with arms at sides and feet together".to_string(),
            keyframes: vec![
                PoseKeyframe {
                    node: "Left UpperArm".to_string(),
                    property: "xRotate".to_string(),
                    frame: 1,
                    value: 0.0,
                },
                PoseKeyframe {
                    node: "Right UpperArm".to_string(),
                    property: "xRotate".to_string(),
                    frame: 1,
                    value: 0.0,
                },
                PoseKeyframe {
                    node: "Left UpperLeg".to_string(),
                    property: "yRotate".to_string(),
                    frame: 1,
                    value: 0.0,
                },
                PoseKeyframe {
                    node: "Right UpperLeg".to_string(),
                    property: "yRotate".to_string(),
                    frame: 1,
                    value: 0.0,
                },
            ],
        });
        
        self.poses.insert("Casual Lean".to_string(), Pose {
            name: "Casual Lean".to_string(),
            figure: "Genesis 8 Female".to_string(),
            description: "A relaxed pose with weight on one leg and hand on hip".to_string(),
            keyframes: vec![
                PoseKeyframe {
                    node: "Left UpperLeg".to_string(),
                    property: "yRotate".to_string(),
                    frame: 1,
                    value: -15.0,
                },
                PoseKeyframe {
                    node: "Right UpperLeg".to_string(),
                    property: "yRotate".to_string(),
                    frame: 1,
                    value: 10.0,
                },
                PoseKeyframe {
                    node: "Spine".to_string(),
                    property: "xRotate".to_string(),
                    frame: 1,
                    value: 5.0,
                },
                PoseKeyframe {
                    node: "Left UpperArm".to_string(),
                    property: "zRotate".to_string(),
                    frame: 1,
                    value: -20.0,
                },
                PoseKeyframe {
                    node: "Right UpperArm".to_string(),
                    property: "xRotate".to_string(),
                    frame: 1,
                    value: 15.0,
                },
            ],
        });
    }
    
    fn init_lights(&mut self) {
        self.lights.insert("Point Light".to_string(), Light {
            r#type: "point_light".to_string(),
            color: "255,255,255".to_string(),
            intensity_range: (0.0, 10.0),
            typical_use: "General purpose omnidirectional lighting".to_string(),
        });
        
        self.lights.insert("Spot Light".to_string(), Light {
            r#type: "spot_light".to_string(),
            color: "255,255,255".to_string(),
            intensity_range: (0.0, 10.0),
            typical_use: "Focused directional lighting for highlighting".to_string(),
        });
        
        self.lights.insert("Distant Light".to_string(), Light {
            r#type: "distant_light".to_string(),
            color: "255,255,255".to_string(),
            intensity_range: (0.0, 5.0),
            typical_use: "Simulates sunlight with parallel rays".to_string(),
        });
        
        self.lights.insert("Area Light".to_string(), Light {
            r#type: "area_light".to_string(),
            color: "255,255,255".to_string(),
            intensity_range: (0.0, 5.0),
            typical_use: "Soft, diffused lighting from rectangular source".to_string(),
        });
    }
    
    fn init_materials(&mut self) {
        self.materials.insert("UberSurface".to_string(), Material {
            name: "UberSurface".to_string(),
            shader_type: "uber".to_string(),
            compatible_with: [
                "Genesis 8 Female".to_string(),
                "Genesis 9 Female".to_string(),
                "Genesis 8 Male".to_string(),
                "Genesis 9 Male".to_string(),
            ].to_vec(),
            adjustable_properties: vec![
                "Base Color".to_string(),
                "Specular Strength".to_string(),
                "Roughness".to_string(),
                "Metallic".to_string(),
                "Transmission".to_string(),
                "Emission Color".to_string(),
                "Emission Strength".to_string(),
                "Normal Strength".to_string(),
                "Clearcoat".to_string(),
                "Clearcoat Roughness".to_string(),
            ],
        });
        
        self.materials.insert("Skin Shader".to_string(), Material {
            name: "Skin Shader".to_string(),
            shader_type: "skin".to_string(),
            compatible_with: [
                "Genesis 8 Female".to_string(),
                "Genesis 9 Female".to_string(),
                "Genesis 8 Male".to_string(),
                "Genesis 9 Male".to_string(),
            ].to_vec(),
            adjustable_properties: vec![
                "Base Color".to_string(),
                "Specular".to_string(),
                "Roughness".to_string(),
                "Subsurface".to_string(),
                "Subsurface Radius".to_string(),
                "Subsurface Color".to_string(),
                "Sheen".to_string(),
                "Sheen Tint".to_string(),
                "Clearcoat".to_string(),
                "Clearcoat Roughness".to_string(),
                "Anisotropy".to_string(),
                "Anisotropy Rotation".to_string(),
            ],
        });
    }
    
    fn init_asset_categories(&mut self) {
        self.asset_categories.insert("clothing".to_string(), AssetCategory {
            name: "clothing".to_string(),
            description: "Wearable items like shirts, pants, dresses, etc.".to_string(),
            typical_use_cases: vec![
                "Dressing characters".to_string(),
                "Creating outfits for scenes".to_string(),
                "Character customization".to_string(),
            ],
            compatible_with_figures: vec![
                "Genesis 8 Female".to_string(),
                "Genesis 9 Female".to_string(),
                "Genesis 8 Male".to_string(),
                "Genesis 9 Male".to_string(),
            ],
            required_props: vec![
                "Fit Control".to_string(),
                "Pose Control".to_string(),
                "Adjust".to_string(),
            ],
        });
        
        self.asset_categories.insert("hair".to_string(), AssetCategory {
            name: "hair".to_string(),
            description: "Hair accessories and hairstyles".to_string(),
            typical_use_cases: vec![
                "Styling character hair".to_string(),
                "Adding hair accessories".to_string(),
                "Creating different looks".to_string(),
            ],
            compatible_with_figures: vec![
                "Genesis 8 Female".to_string(),
                "Genesis 9 Female".to_string(),
                "Genesis 8 Male".to_string(),
                "Genesis 9 Male".to_string(),
            ],
            required_props: vec![
                "Front/Back Bang".to_string(),
                "Left/Right Side".to_string(),
                "Length".to_string(),
                "Wave/Curl".to_string(),
            ],
        });
        
        self.asset_categories.insert("poses".to_string(), AssetCategory {
            name: "poses".to_string(),
            description: "Character poses and animations".to_string(),
            typical_use_cases: vec![
                "Posing characters in scenes".to_string(),
                "Creating animations".to_string(),
                "Setting up action sequences".to_string(),
            ],
            compatible_with_figures: vec![
                "Genesis 8 Female".to_string(),
                "Genesis 9 Female".to_string(),
                "Genesis 8 Male".to_string(),
                "Genesis 9 Male".to_string(),
            ],
            required_props: vec![
                "Mirror".to_string(),
                "Strength".to_string(),
                "Fade In/Out".to_string(),
            ],
        });
        
        self.asset_categories.insert("environments".to_string(), AssetCategory {
            name: "environments".to_string(),
            description: "Scenes, backgrounds, and props for settings".to_string(),
            typical_use_cases: vec![
                "Setting scene locations".to_string(),
                "Creating indoor/outdoor settings".to_string(),
                "Adding props and set dressing".to_string(),
            ],
            compatible_with_figures: vec!["Scene".to_string()],
            required_props: vec![
                "Position".to_string(),
                "Rotation".to_string(),
                "Scale".to_string(),
            ],
        });
    }
    
    /// Get a figure by name
    pub fn get_figure(&self, name: &str) -> Option<&Figure> {
        self.figures.get(name)
    }
    
    /// Get morphs compatible with a figure
    pub fn get_morphs_for_figure(&self, figure: &str) -> Vec<&Morph> {
        self.morphs.values()
            .filter(|m| m.figure == figure)
            .collect()
    }
    
    /// Get poses for a figure
    pub fn get_poses_for_figure(&self, figure: &str) -> Vec<&Pose> {
        self.poses.values()
            .filter(|p| p.figure == figure)
            .collect()
    }
    
    /// Get lights of a specific type
    pub fn get_lights_by_type(&self, light_type: &str) -> Vec<&Light> {
        self.lights.values()
            .filter(|l| l.r#type == light_type)
            .collect()
    }
    
    /// Get materials compatible with a figure
    pub fn get_materials_for_figure(&self, figure: &str) -> Vec<&Material> {
        self.materials.values()
            .filter(|m| m.compatible_with.contains(&figure.to_string()))
            .collect()
    }
}

impl Default for DazKnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}

