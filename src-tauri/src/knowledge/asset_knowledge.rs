use crate::figure_resolver::resolve_compatible_assets;
use crate::knowledge::daz_concepts::DazKnowledgeBase;
use crate::library_scanner::AssetInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

/// Represents semantic understanding of an asset beyond basic metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAsset {
    pub basic_info: AssetInfo,
    pub semantic_tags: Vec<String>,
    pub use_cases: Vec<String>,
    pub compatibility_notes: String,
    pub typical_setup: Vec<String>, // steps to properly use this asset
    pub alternatives: Vec<String>,  // similar assets that could work
    pub complexity: ComplexityLevel,
    pub popularity: PopularityLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Beginner,     // easy to use, minimal setup
    Intermediate, // some knowledge required
    Advanced,     // expert knowledge needed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PopularityLevel {
    Rare,
    Uncommon,
    Common,
    Popular,
    VeryPopular,
}

/// Knowledge base that understands asset semantics and relationships
#[derive(Debug)]
pub struct AssetKnowledgeBase {
    pub daz_knowledge: DazKnowledgeBase,
    pub semantic_assets: Mutex<HashMap<String, SemanticAsset>>,
}

impl AssetKnowledgeBase {
    pub fn new() -> Self {
        Self {
            daz_knowledge: DazKnowledgeBase::new(),
            semantic_assets: Mutex::new(HashMap::new()),
        }
    }

    /// Get or create semantic understanding for an asset
    pub fn get_semantic_asset(&self, asset_info: &AssetInfo) -> SemanticAsset {
        // Check if we already have semantic understanding cached
        if let Some(cached) = self.semantic_assets.lock().unwrap().get(&asset_info.path) {
            return cached.clone();
        }

        // Generate semantic understanding from basic info + Daz knowledge
        let semantic = self.generate_semantic_understanding(asset_info);

        // Cache it
        self.semantic_assets
            .lock()
            .unwrap()
            .insert(asset_info.path.clone(), semantic.clone());

        semantic
    }

    /// Generate semantic understanding from asset info and Daz concepts
    fn generate_semantic_understanding(&self, asset_info: &AssetInfo) -> SemanticAsset {
        let mut semantic_tags = Vec::new();
        let mut use_cases = Vec::new();
        let mut compatibility_notes = String::new();
        let mut typical_setup = Vec::new();
        let mut alternatives: Vec<String> = self
            .find_compatible_assets(
                asset_info
                    .compatibility_base
                    .first()
                    .map(|s| s.as_str())
                    .unwrap_or("Genesis 8 Female"),
                Some(&asset_info.category),
            )
            .into_iter()
            .filter(|a| a.path != asset_info.path)
            .take(5)
            .map(|a| a.name)
            .collect();
        if alternatives.is_empty() && asset_info.category == "poses" {
            alternatives = self
                .daz_knowledge
                .get_poses_for_figure("Genesis 8 Female")
                .iter()
                .take(3)
                .map(|p| p.name.clone())
                .collect();
        }
        let complexity;
        let popularity;

        // Infer semantics from category and metadata
        match asset_info.category.as_str() {
            "figures" => {
                semantic_tags.push("character".to_string());
                semantic_tags.push("figure".to_string());
                use_cases.push("Primary character in scene".to_string());
                use_cases.push("Posing and animating".to_string());

                if let Some(figure) = self.daz_knowledge.get_figure(&asset_info.name) {
                    compatibility_notes = format!(
                        "Compatible with: {}",
                        figure.compatible_categories.join(", ")
                    );
                    typical_setup = vec![
                        "Load figure into scene".to_string(),
                        "Apply desired morphs".to_string(),
                        "Apply pose".to_string(),
                        "Add clothing and hair".to_string(),
                        "Set up lighting".to_string(),
                    ];
                }

                complexity = ComplexityLevel::Beginner;
                popularity = PopularityLevel::VeryPopular;
            },

            "clothing" => {
                semantic_tags.push("wearable".to_string());
                semantic_tags.push("clothing".to_string());
                use_cases.push("Dressing characters".to_string());
                use_cases.push("Creating outfits".to_string());

                // Check compatibility with figures
                let compatible_figures: Vec<String> = asset_info
                    .compatibility_base
                    .iter()
                    .filter(|f| f.contains("Genesis"))
                    .cloned()
                    .collect();

                if !compatible_figures.is_empty() {
                    compatibility_notes =
                        format!("Works with figures: {}", compatible_figures.join(", "));
                } else {
                    compatibility_notes = "Check compatibility manually".to_string();
                }

                typical_setup = vec![
                    "Load base figure first".to_string(),
                    "Load clothing asset".to_string(),
                    "Use 'Fit To' to conform to figure".to_string(),
                    "Adjust using control parameters".to_string(),
                ];

                complexity = ComplexityLevel::Beginner;
                popularity = PopularityLevel::Common;
            },

            "hair" => {
                semantic_tags.push("hair".to_string());
                semantic_tags.push("hairstyle".to_string());
                use_cases.push("Styling character hair".to_string());
                use_cases.push("Adding hair accessories".to_string());

                typical_setup = vec![
                    "Load base figure".to_string(),
                    "Load hair asset".to_string(),
                    "Conform hair to figure head".to_string(),
                    "Adjust using styling parameters".to_string(),
                ];

                complexity = ComplexityLevel::Beginner;
                popularity = PopularityLevel::Common;
            },

            "poses" => {
                semantic_tags.push("pose".to_string());
                semantic_tags.push("animation".to_string());
                use_cases.push("Posing characters".to_string());
                use_cases.push("Creating animation sequences".to_string());

                typical_setup = vec![
                    "Load figure".to_string(),
                    "Load pose asset".to_string(),
                    "Apply pose to figure".to_string(),
                    "Adjust strength if needed".to_string(),
                ];

                complexity = ComplexityLevel::Beginner;
                popularity = PopularityLevel::Common;
            },

            "morphs" => {
                semantic_tags.push("morph".to_string());
                semantic_tags.push("shape".to_string());
                use_cases.push("Customizing character shape".to_string());
                use_cases.push("Creating character variations".to_string());

                typical_setup = vec![
                    "Load figure".to_string(),
                    "Load morph asset".to_string(),
                    "Adjust morph dial to desired value".to_string(),
                ];

                complexity = ComplexityLevel::Beginner;
                popularity = PopularityLevel::VeryPopular;
            },

            "materials" => {
                semantic_tags.push("material".to_string());
                semantic_tags.push("texture".to_string());
                use_cases.push("Applying surface appearance".to_string());
                use_cases.push("Creating realistic surfaces".to_string());

                typical_setup = vec![
                    "Select target surface".to_string(),
                    "Load material asset".to_string(),
                    "Apply to selected surface".to_string(),
                    "Adjust material properties as needed".to_string(),
                ];

                complexity = ComplexityLevel::Beginner;
                popularity = PopularityLevel::Common;
            },

            "lights" => {
                semantic_tags.push("light".to_string());
                semantic_tags.push("lighting".to_string());
                use_cases.push("Illuminating scenes".to_string());
                use_cases.push("Creating mood and atmosphere".to_string());

                // Determine light type from metadata or filename
                let light_type = if asset_info.name.to_lowercase().contains("spot") {
                    "spot"
                } else if asset_info.name.to_lowercase().contains("distant")
                    || asset_info.name.to_lowercase().contains("sun")
                {
                    "distant"
                } else if asset_info.name.to_lowercase().contains("area") {
                    "area"
                } else {
                    "point"
                };

                if let Some(light_info) = self
                    .daz_knowledge
                    .get_lights_by_type(&format!("{}_light", light_type))
                    .first()
                {
                    compatibility_notes =
                        format!("{} light: {}", light_info.r#type, light_info.typical_use);
                }

                typical_setup = vec![
                    "Load light asset into scene".to_string(),
                    "Position light as needed".to_string(),
                    "Adjust intensity and color".to_string(),
                    "Consider adding light modifiers or gels".to_string(),
                ];

                complexity = ComplexityLevel::Beginner;
                popularity = PopularityLevel::Common;
            },

            "environments" => {
                semantic_tags.push("environment".to_string());
                semantic_tags.push("scene".to_string());
                use_cases.push("Setting scene location".to_string());
                use_cases.push("Creating backgrounds".to_string());
                use_cases.push("Adding set dressing and props".to_string());

                typical_setup = vec![
                    "Load environment asset".to_string(),
                    "Position and scale as needed".to_string(),
                    "Add lighting appropriate for setting".to_string(),
                    "Add characters and props".to_string(),
                ];

                complexity = ComplexityLevel::Beginner;
                popularity = PopularityLevel::Common;
            },

            _ => {
                semantic_tags.push("asset".to_string());
                use_cases.push("General purpose asset".to_string());
                compatibility_notes = "Standard asset - check documentation".to_string();
                typical_setup = vec![
                    "Load asset into scene".to_string(),
                    "Consult documentation for specific usage".to_string(),
                ];

                complexity = ComplexityLevel::Beginner;
                popularity = PopularityLevel::Uncommon;
            },
        }

        // Add tags from filename analysis
        let lower_name = asset_info.name.to_lowercase();
        if lower_name.contains("casual") || lower_name.contains("everyday") {
            semantic_tags.push("casual".to_string());
            use_cases.push("Everyday wear".to_string());
        }
        if lower_name.contains("formal")
            || lower_name.contains("evening")
            || lower_name.contains("gown")
        {
            semantic_tags.push("formal".to_string());
            use_cases.push("Formal occasions".to_string());
        }
        if lower_name.contains("sport")
            || lower_name.contains("athletic")
            || lower_name.contains("workout")
        {
            semantic_tags.push("sport".to_string());
            use_cases.push("Athletic activities".to_string());
        }
        if lower_name.contains("fantasy")
            || lower_name.contains("medieval")
            || lower_name.contains("elven")
        {
            semantic_tags.push("fantasy".to_string());
            use_cases.push("Fantasy settings".to_string());
        }
        if lower_name.contains("sci-fi")
            || lower_name.contains("futuristic")
            || lower_name.contains("cyber")
        {
            semantic_tags.push("sci_fi".to_string());
            use_cases.push("Science fiction settings".to_string());
        }
        if lower_name.contains("hair") && !asset_info.category.eq("hair") {
            semantic_tags.push("hair_accessory".to_string());
            use_cases.push("Hair styling".to_string());
        }

        SemanticAsset {
            basic_info: asset_info.clone(),
            semantic_tags,
            use_cases,
            compatibility_notes,
            typical_setup,
            alternatives, // TODO: populate with similar assets
            complexity,
            popularity,
        }
    }

    /// Find assets compatible with a given figure
    pub fn find_compatible_assets(&self, figure: &str, category: Option<&str>) -> Vec<AssetInfo> {
        resolve_compatible_assets(figure, category)
    }

    /// Get use cases for an asset category
    pub fn get_use_cases_for_category(&self, category: &str) -> Vec<String> {
        self.daz_knowledge
            .asset_categories
            .get(category)
            .map(|cat| cat.typical_use_cases.clone())
            .unwrap_or_default()
    }

    /// Get complexity level for asset usage
    pub fn get_complexity(&self, asset_info: &AssetInfo) -> ComplexityLevel {
        self.get_semantic_asset(asset_info).complexity
    }

    /// Get popularity level
    pub fn get_popularity(&self, asset_info: &AssetInfo) -> PopularityLevel {
        self.get_semantic_asset(asset_info).popularity
    }
}

impl Default for AssetKnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}
