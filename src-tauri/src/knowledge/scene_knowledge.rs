use crate::knowledge::daz_concepts::DazKnowledgeBase;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents understanding of what makes a good scene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneUnderstanding {
    pub scene_type: SceneType,
    pub lighting_setup: Vec<LightingRecommendation>,
    pub composition_rules: Vec<String>,
    pub recommended_assets: Vec<String>,
    pub common_mistakes: Vec<String>,
    pub complexity: ComplexityLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum SceneType {
    Portrait,
    FullBody,
    Action,
    Fantasy,
    SciFi,
    Casual,
    Formal,
    ProductRender,
    AnimationTest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingRecommendation {
    pub light_type: String, // point, spot, distant, area
    pub purpose: String,    // key, fill, rim, background
    pub color: String,      // RGB or temperature
    pub intensity_range: (f32, f32),
    pub position_hint: String, // e.g., "45 degrees left, slightly above eye level"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Beginner,
    Intermediate,
    Advanced,
}

/// Knowledge base for scene composition and lighting
#[derive(Debug, Clone)]
pub struct SceneKnowledgeBase {
    pub daz_knowledge: DazKnowledgeBase,
    pub scene_templates: HashMap<SceneType, SceneTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneTemplate {
    pub scene_type: SceneType,
    pub description: String,
    pub lighting_setup: Vec<LightingRecommendation>,
    pub composition_guidelines: Vec<String>,
    pub recommended_figures: Vec<String>,
    pub recommended_assets: Vec<String>, // asset categories or specific assets
    pub common_poses: Vec<String>,
    pub common_mistakes: Vec<String>,
}

impl SceneKnowledgeBase {
    pub fn new() -> Self {
        let mut knowledge = SceneKnowledgeBase {
            daz_knowledge: DazKnowledgeBase::new(),
            scene_templates: HashMap::new(),
        };

        knowledge.init_scene_templates();
        knowledge
    }

    fn init_scene_templates(&mut self) {
        // Portrait scene
        self.scene_templates.insert(
            SceneType::Portrait,
            SceneTemplate {
                scene_type: SceneType::Portrait,
                description: "A portrait focusing on the subject's face and upper body".to_string(),
                lighting_setup: vec![
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "key".to_string(),
                        color: "255,255,255".to_string(),
                        intensity_range: (0.8, 1.2),
                        position_hint: "45 degrees left, slightly above eye level".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "fill".to_string(),
                        color: "255,255,255".to_string(),
                        intensity_range: (0.3, 0.5),
                        position_hint: "45 degrees right, at eye level".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "rim".to_string(),
                        color: "200,200,255".to_string(),
                        intensity_range: (0.4, 0.6),
                        position_hint: "directly behind subject, opposite key light".to_string(),
                    },
                ],
                composition_guidelines: vec![
                    "Frame from mid-chest up to just above head".to_string(),
                    "Subject's eyes should be at upper third of frame".to_string(),
                    "Leave slight space in direction of gaze".to_string(),
                    "Use shallow depth of field if possible".to_string(),
                ],
                recommended_figures: vec![
                    "Genesis 8 Female".to_string(),
                    "Genesis 9 Female".to_string(),
                    "Genesis 8 Male".to_string(),
                    "Genesis 9 Male".to_string(),
                ],
                recommended_assets: vec![
                    "materials".to_string(),
                    "hair".to_string(),
                    "clothing".to_string(),
                ],
                common_poses: vec![
                    "Basic Standing".to_string(),
                    "Relaxed Pose".to_string(),
                    "Look Over Shoulder".to_string(),
                ],
                common_mistakes: vec![
                    "Key light too strong causing harsh shadows".to_string(),
                    "Fill light too strong eliminating all depth".to_string(),
                    "Background too distracting from subject".to_string(),
                    "Subject looking directly at camera with no emotion".to_string(),
                ],
            },
        );

        // Full body scene
        self.scene_templates.insert(
            SceneType::FullBody,
            SceneTemplate {
                scene_type: SceneType::FullBody,
                description: "A full body shot showing the complete figure".to_string(),
                lighting_setup: vec![
                    LightingRecommendation {
                        light_type: "distant_light".to_string(),
                        purpose: "key".to_string(),
                        color: "255,255,255".to_string(),
                        intensity_range: (0.6, 0.9),
                        position_hint: "45 degrees left, 30 degrees above".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "fill".to_string(),
                        color: "255,255,255".to_string(),
                        intensity_range: (0.2, 0.4),
                        position_hint: "45 degrees right, at waist height".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "rim".to_string(),
                        color: "200,200,255".to_string(),
                        intensity_range: (0.3, 0.5),
                        position_hint: "directly behind subject".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "background".to_string(),
                        color: "100,100,150".to_string(),
                        intensity_range: (0.2, 0.3),
                        position_hint: "illuminating background/seam".to_string(),
                    },
                ],
                composition_guidelines: vec![
                    "Show full body from head to toes".to_string(),
                    "Leave small space above head and below feet".to_string(),
                    "Position subject slightly off-center for dynamic composition".to_string(),
                    "Ensure feet are fully visible if standing pose".to_string(),
                ],
                recommended_figures: vec![
                    "Genesis 8 Female".to_string(),
                    "Genesis 9 Female".to_string(),
                    "Genesis 8 Male".to_string(),
                    "Genesis 9 Male".to_string(),
                ],
                recommended_assets: vec![
                    "clothing".to_string(),
                    "shoes".to_string(),
                    "hair".to_string(),
                    "environments".to_string(),
                ],
                common_poses: vec![
                    "Basic Standing".to_string(),
                    "Contrapposto".to_string(),
                    "Weight Shift".to_string(),
                    "Casual Lean".to_string(),
                ],
                common_mistakes: vec![
                    "Feet cut off at bottom of frame".to_string(),
                    "Subject too close to background causing shadows".to_string(),
                    "Lighting inconsistent between upper and lower body".to_string(),
                    "Clothes not conforming properly causing poke-through".to_string(),
                ],
            },
        );

        // Action scene
        self.scene_templates.insert(
            SceneType::Action,
            SceneTemplate {
                scene_type: SceneType::Action,
                description: "A dynamic action pose showing movement or power".to_string(),
                lighting_setup: vec![
                    LightingRecommendation {
                        light_type: "spot_light".to_string(),
                        purpose: "key".to_string(),
                        color: "255,250,200".to_string(),
                        intensity_range: (0.8, 1.2),
                        position_hint: "Low angle, 30 degrees left, highlighting muscles"
                            .to_string(),
                    },
                    LightingRecommendation {
                        light_type: "spot_light".to_string(),
                        purpose: "rim".to_string(),
                        color: "255,100,100".to_string(),
                        intensity_range: (0.6, 0.9),
                        position_hint: "Opposite key, creating dramatic silhouette".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "fill".to_string(),
                        color: "200,200,200".to_string(),
                        intensity_range: (0.1, 0.2),
                        position_hint: "Very low, just to lift shadows slightly".to_string(),
                    },
                ],
                composition_guidelines: vec![
                    "Use low angle to make subject look powerful".to_string(),
                    "Capture mid-motion for dynamism".to_string(),
                    "Use motion blur if animating".to_string(),
                    "Consider dramatic skies or urban backgrounds".to_string(),
                ],
                recommended_figures: vec![
                    "Genesis 8 Female".to_string(),
                    "Genesis 9 Female".to_string(),
                    "Genesis 8 Male".to_string(),
                    "Genesis 9 Male".to_string(),
                ],
                recommended_assets: vec![
                    "poses".to_string(),
                    "expressions".to_string(),
                    "environments".to_string(),
                    "props".to_string(),
                ],
                common_poses: vec![
                    "Martial Arts Stance".to_string(),
                    "Running Pose".to_string(),
                    "Jumping Pose".to_string(),
                    "Power Pose".to_string(),
                ],
                common_mistakes: vec![
                    "Pose looks stiff or unnatural".to_string(),
                    "Lighting doesn't emphasize muscle definition".to_string(),
                    "Background competes with subject for attention".to_string(),
                    "Clothing doesn't follow body movement causing stretches".to_string(),
                ],
            },
        );

        // Fantasy scene
        self.scene_templates.insert(
            SceneType::Fantasy,
            SceneTemplate {
                scene_type: SceneType::Fantasy,
                description: "A fantastical scene with magical or otherworldly elements"
                    .to_string(),
                lighting_setup: vec![
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "key".to_string(),
                        color: "255,200,100".to_string(), // warm golden
                        intensity_range: (0.7, 1.0),
                        position_hint: "45 degrees left, slightly above".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "fill".to_string(),
                        color: "150,200,255".to_string(), // cool blue
                        intensity_range: (0.2, 0.4),
                        position_hint: "45 degrees right, filling shadows".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "area_light".to_string(),
                        purpose: "background".to_string(),
                        color: "100,50,200".to_string(), // purple mystical
                        intensity_range: (0.3, 0.6),
                        position_hint: "Large soft light behind for ambient glow".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "accent".to_string(),
                        color: "255,255,100".to_string(), // yellow sparkles
                        intensity_range: (0.4, 0.8),
                        position_hint: "Small lights for magical effects".to_string(),
                    },
                ],
                composition_guidelines: vec![
                    "Use rule of thirds for placing magical elements".to_string(),
                    "Create depth with foreground, midground, background".to_string(),
                    "Consider using fog or mist for atmosphere".to_string(),
                    "Use color contrast (warm subject, cool background or vice versa)".to_string(),
                ],
                recommended_figures: vec![
                    "Genesis 8 Female".to_string(),
                    "Genesis 9 Female".to_string(),
                    "Genesis 8 Male".to_string(),
                    "Genesis 9 Male".to_string(),
                ],
                recommended_assets: vec![
                    "hair".to_string(),
                    "clothing".to_string(),
                    "environments".to_string(),
                    "props".to_string(),
                    "effects".to_string(),
                ],
                common_poses: vec![
                    "Magic Casting Pose".to_string(),
                    "Ready Stance".to_string(),
                    "Looking Into Distance".to_string(),
                ],
                common_mistakes: vec![
                    "Overdoing effects making scene look cheap".to_string(),
                    "Inconsistent lighting breaking fantasy illusion".to_string(),
                    "Poor color harmony making scene garish".to_string(),
                    "Lack of focal point - eye doesn't know where to look".to_string(),
                ],
            },
        );

        // Sci-Fi scene
        self.scene_templates.insert(
            SceneType::SciFi,
            SceneTemplate {
                scene_type: SceneType::SciFi,
                description: "A futuristic science fiction setting".to_string(),
                lighting_setup: vec![
                    LightingRecommendation {
                        light_type: "spot_light".to_string(),
                        purpose: "key".to_string(),
                        color: "150,150,255".to_string(), // cool blue-white
                        intensity_range: (0.6, 0.9),
                        position_hint: "High angle, 30 degrees left, creating sharp shadows"
                            .to_string(),
                    },
                    LightingRecommendation {
                        light_type: "area_light".to_string(),
                        purpose: "fill".to_string(),
                        color: "100,100,200".to_string(),
                        intensity_range: (0.2, 0.4),
                        position_hint: "Large soft fill from opposite side".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "accent".to_string(),
                        color: "0,255,255".to_string(), // cyan
                        intensity_range: (0.3, 0.6),
                        position_hint: "Small lights for tech details and displays".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "rim".to_string(),
                        color: "255,100,255".to_string(), // magenta
                        intensity_range: (0.4, 0.7),
                        position_hint: "Strong rim for futuristic edge lighting".to_string(),
                    },
                ],
                composition_guidelines: vec![
                    "Use strong geometric shapes and lines".to_string(),
                    "Incorporate technology elements as props".to_string(),
                    "Consider using reflections and refractions".to_string(),
                    "Use color palette of blues, cyans, purples with accent colors".to_string(),
                ],
                recommended_figures: vec![
                    "Genesis 8 Female".to_string(),
                    "Genesis 9 Female".to_string(),
                    "Genesis 8 Male".to_string(),
                    "Genesis 9 Male".to_string(),
                ],
                recommended_assets: vec![
                    "hair".to_string(),
                    "clothing".to_string(),
                    "environments".to_string(),
                    "props".to_string(),
                    "materials".to_string(),
                ],
                common_poses: vec![
                    "Combat Stance".to_string(),
                    "Looking At Device".to_string(),
                    "Walking Forward".to_string(),
                ],
                common_mistakes: vec![
                    "Making it too dark - sci-fi should have visible technology".to_string(),
                    "Mixing too many time periods inconsistently".to_string(),
                    "Lack of scale - making environments feel cramped".to_string(),
                    "Ignoring material properties (everything same shiny plastic)".to_string(),
                ],
            },
        );

        // Casual scene
        self.scene_templates.insert(
            SceneType::Casual,
            SceneTemplate {
                scene_type: SceneType::Casual,
                description: "A relaxed, everyday casual scene".to_string(),
                lighting_setup: vec![
                    LightingRecommendation {
                        light_type: "distant_light".to_string(),
                        purpose: "key".to_string(),
                        color: "255,255,200".to_string(), // warm sunlight
                        intensity_range: (0.5, 0.8),
                        position_hint: "50 degrees above, simulating afternoon sun".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "fill".to_string(),
                        color: "255,255,255".to_string(),
                        intensity_range: (0.1, 0.2),
                        position_hint: "Very weak fill just to prevent pure black shadows"
                            .to_string(),
                    },
                ],
                composition_guidelines: vec![
                    "Use rule of thirds or golden ratio".to_string(),
                    "Capture natural, unposed moments".to_string(),
                    "Consider environmental storytelling".to_string(),
                    "Keep depth of field moderate to show some background".to_string(),
                ],
                recommended_figures: vec![
                    "Genesis 8 Female".to_string(),
                    "Genesis 9 Female".to_string(),
                    "Genesis 8 Male".to_string(),
                    "Genesis 9 Male".to_string(),
                ],
                recommended_assets: vec![
                    "clothing".to_string(),
                    "hair".to_string(),
                    "environments".to_string(),
                    "props".to_string(),
                ],
                common_poses: vec![
                    "Casual Lean".to_string(),
                    "Sitting Relaxed".to_string(),
                    "Walking Naturally".to_string(),
                    "Looking At Phone".to_string(),
                ],
                common_mistakes: vec![
                    "Over-posing making it look unnatural".to_string(),
                    "Lighting too dramatic for casual setting".to_string(),
                    "Clothing too formal or costume-like".to_string(),
                    "Background too busy or distracting".to_string(),
                ],
            },
        );

        // Formal scene
        self.scene_templates.insert(
            SceneType::Formal,
            SceneTemplate {
                scene_type: SceneType::Formal,
                description: "A formal, elegant scene like a gala or red carpet".to_string(),
                lighting_setup: vec![
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "key".to_string(),
                        color: "255,255,255".to_string(),
                        intensity_range: (0.7, 1.0),
                        position_hint: "45 degrees left, slightly above eye level".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "fill".to_string(),
                        color: "255,255,255".to_string(),
                        intensity_range: (0.3, 0.5),
                        position_hint: "45 degrees right, at eye level".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "rim".to_string(),
                        color: "200,200,255".to_string(),
                        intensity_range: (0.5, 0.7),
                        position_hint: "directly behind subject".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "background".to_string(),
                        color: "50,50,50".to_string(),
                        intensity_range: (0.1, 0.2),
                        position_hint: "dark background to make subject pop".to_string(),
                    },
                ],
                composition_guidelines: vec![
                    "Use classic portrait framing".to_string(),
                    "Subject should look confident and poised".to_string(),
                    "Pay attention to clothing details and accessories".to_string(),
                    "Consider using a shallow depth of field".to_string(),
                ],
                recommended_figures: vec![
                    "Genesis 8 Female".to_string(),
                    "Genesis 9 Female".to_string(),
                    "Genesis 8 Male".to_string(),
                    "Genesis 9 Male".to_string(),
                ],
                recommended_assets: vec![
                    "clothing".to_string(),
                    "hair".to_string(),
                    "materials".to_string(),
                    "jewelry".to_string(),
                ],
                common_poses: vec![
                    "Red Carpet Pose".to_string(),
                    "Hand On Hip".to_string(),
                    "Looking Over Shoulder".to_string(),
                    "Elegant Sitting".to_string(),
                ],
                common_mistakes: vec![
                    "Pose looking stiff or uncomfortable".to_string(),
                    "Clothes not fitting properly or wrinkled".to_string(),
                    "Hair looking helmet-like or unnatural".to_string(),
                    "Lighting flat and uninteresting".to_string(),
                ],
            },
        );

        // Product render
        self.scene_templates.insert(
            SceneType::ProductRender,
            SceneTemplate {
                scene_type: SceneType::ProductRender,
                description: "A clean product render showing off an asset".to_string(),
                lighting_setup: vec![
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "key".to_string(),
                        color: "255,255,255".to_string(),
                        intensity_range: (0.6, 0.8),
                        position_hint: "45 degrees left, 30 degrees above".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "fill".to_string(),
                        color: "255,255,255".to_string(),
                        intensity_range: (0.3, 0.4),
                        position_hint: "45 degrees right, 30 degrees above".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "rim".to_string(),
                        color: "200,200,200".to_string(),
                        intensity_range: (0.2, 0.3),
                        position_hint: "directly behind".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "point_light".to_string(),
                        purpose: "background".to_string(),
                        color: "200,200,200".to_string(),
                        intensity_range: (0.1, 0.2),
                        position_hint: "background illumination".to_string(),
                    },
                ],
                composition_guidelines: vec![
                    "Use clean, seamless background".to_string(),
                    "Show product from multiple angles if needed".to_string(),
                    "Highlight key features and details".to_string(),
                    "Use consistent lighting across all shots".to_string(),
                ],
                recommended_figures: vec![],
                recommended_assets: vec!["the asset being rendered".to_string()],
                common_poses: vec![],
                common_mistakes: vec![
                    "Background not seamless causing distractions".to_string(),
                    "Lighting inconsistent making hard to compare".to_string(),
                    "Not showing important details or angles".to_string(),
                    "Using inappropriate materials or shaders".to_string(),
                ],
            },
        );

        // Animation test
        self.scene_templates.insert(
            SceneType::AnimationTest,
            SceneTemplate {
                scene_type: SceneType::AnimationTest,
                description: "A simple scene for testing animation loops".to_string(),
                lighting_setup: vec![
                    LightingRecommendation {
                        light_type: "distant_light".to_string(),
                        purpose: "key".to_string(),
                        color: "255,255,255".to_string(),
                        intensity_range: (0.5, 0.7),
                        position_hint: "45 degrees left, 30 degrees above".to_string(),
                    },
                    LightingRecommendation {
                        light_type: "distant_light".to_string(),
                        purpose: "fill".to_string(),
                        color: "255,255,255".to_string(),
                        intensity_range: (0.1, 0.2),
                        position_hint: "45 degrees right, 30 degrees above".to_string(),
                    },
                ],
                composition_guidelines: vec![
                    "Keep background simple and static".to_string(),
                    "Ensure figure has plenty of room to move".to_string(),
                    "Use neutral lighting that won't change with pose".to_string(),
                    "Consider adding a reference grid or markers".to_string(),
                ],
                recommended_figures: vec![
                    "Genesis 8 Female".to_string(),
                    "Genesis 9 Female".to_string(),
                    "Genesis 8 Male".to_string(),
                    "Genesis 9 Male".to_string(),
                ],
                recommended_assets: vec!["poses".to_string(), "expressions".to_string()],
                common_poses: vec![
                    "Walking Cycle".to_string(),
                    "Running Cycle".to_string(),
                    "Jumping Jacks".to_string(),
                    "Dancing".to_string(),
                ],
                common_mistakes: vec![
                    "Background elements moving unintentionally".to_string(),
                    "Lighting changing with character position".to_string(),
                    "Clothes or hair intersecting with body".to_string(),
                    "Animation not looping smoothly".to_string(),
                ],
            },
        );
    }

    /// Get understanding for a specific scene type
    pub fn get_scene_understanding(
        &self,
        scene_type: SceneType,
        skill_level: Option<crate::reasoning::planner::SkillLevel>,
    ) -> Option<SceneUnderstanding> {
        self.scene_templates
            .get(&scene_type)
            .map(|template| SceneUnderstanding {
                scene_type: template.scene_type,
                lighting_setup: template.lighting_setup.clone(),
                composition_rules: template.composition_guidelines.clone(),
                recommended_assets: template.recommended_assets.clone(),
                common_mistakes: template.common_mistakes.clone(),
                complexity: complexity_for_scene(scene_type, skill_level),
            })
    }

    /// Infer scene type from user input
    pub fn infer_scene_type(&self, input: &str) -> Option<SceneType> {
        let lower = input.to_lowercase();

        if lower.contains("portrait") || lower.contains("headshot") || lower.contains("close up") {
            return Some(SceneType::Portrait);
        }
        if lower.contains("full body")
            || lower.contains("fullbody")
            || lower.contains("head to toe")
        {
            return Some(SceneType::FullBody);
        }
        if lower.contains("action")
            || lower.contains("fight")
            || lower.contains("combat")
            || lower.contains("running")
            || lower.contains("jumping")
            || lower.contains("dynamic")
        {
            return Some(SceneType::Action);
        }
        if lower.contains("fantasy")
            || lower.contains("magical")
            || lower.contains("elf")
            || lower.contains("wizard")
            || lower.contains("fairy")
        {
            return Some(SceneType::Fantasy);
        }
        if lower.contains("sci-fi")
            || lower.contains("scifi")
            || lower.contains("futuristic")
            || lower.contains("cyberpunk")
            || lower.contains("space")
            || lower.contains("robot")
        {
            return Some(SceneType::SciFi);
        }
        if lower.contains("casual")
            || lower.contains("everyday")
            || lower.contains("relaxed")
            || lower.contains("lounge")
            || lower.contains("hanging out")
        {
            return Some(SceneType::Casual);
        }
        if lower.contains("formal")
            || lower.contains("gala")
            || lower.contains("red carpet")
            || lower.contains("evening gown")
            || lower.contains("tuxedo")
        {
            return Some(SceneType::Formal);
        }
        if lower.contains("product") || lower.contains("render") || lower.contains("showcase") {
            return Some(SceneType::ProductRender);
        }
        if lower.contains("animation") || lower.contains("walk cycle") || lower.contains("loop") {
            return Some(SceneType::AnimationTest);
        }

        None
    }
}

fn complexity_for_scene(
    scene_type: SceneType,
    skill: Option<crate::reasoning::planner::SkillLevel>,
) -> ComplexityLevel {
    use crate::reasoning::planner::SkillLevel;

    let base = match scene_type {
        SceneType::Portrait | SceneType::Casual | SceneType::ProductRender => {
            ComplexityLevel::Beginner
        },
        SceneType::FullBody | SceneType::Formal => ComplexityLevel::Intermediate,
        SceneType::Action | SceneType::Fantasy | SceneType::SciFi | SceneType::AnimationTest => {
            ComplexityLevel::Advanced
        },
    };

    match skill {
        Some(SkillLevel::Beginner) => ComplexityLevel::Beginner,
        Some(SkillLevel::Intermediate) => match base {
            ComplexityLevel::Advanced => ComplexityLevel::Intermediate,
            other => other,
        },
        Some(SkillLevel::Advanced) | Some(SkillLevel::Expert) | None => base,
    }
}

impl Default for SceneKnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}
