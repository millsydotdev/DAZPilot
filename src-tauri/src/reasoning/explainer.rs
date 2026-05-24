use crate::reasoning::planner::PlanningContext;
use crate::reasoning::planner::{Plan, PlanStep};

/// Explains plans and decisions to the user in understandable terms
pub struct Explainer {
    pub knowledge_base: crate::knowledge::daz_concepts::DazKnowledgeBase,
}

impl Explainer {
    pub fn new() -> Self {
        Self {
            knowledge_base: crate::knowledge::daz_concepts::DazKnowledgeBase::new(),
        }
    }

    /// Explain why a plan was chosen
    pub fn explain_plan_selection(
        &self,
        plan: &Plan,
        alternatives: &[Plan],
        _context: &PlanningContext,
    ) -> String {
        let mut explanation = String::new();

        explanation.push_str("I chose this plan because:\n");
        explanation.push_str(&format!(
            "• Overall confidence: {:.0}%\n",
            plan.confidence * 100.0
        ));
        explanation.push_str(&format!(
            "• Estimated time: {} seconds\n",
            plan.estimated_total_time_seconds
        ));
        explanation.push_str(&format!("• Risk level: {:?}\n", plan.risk_level));

        if let Some(ref fallback) = plan.fallback_plan {
            explanation.push_str(&format!(
                "• Has fallback plan: Yes ({} steps)\n",
                fallback.steps.len()
            ));
        } else {
            explanation.push_str("• Has fallback plan: No\n");
        }

        explanation.push_str("\nPlan details:\n");
        for (i, step) in plan.steps.iter().enumerate() {
            explanation.push_str(&format!(
                "{}. {} ({}s, {:.0}% confidence)\n",
                i + 1,
                step.description,
                step.estimated_time_seconds,
                step.confidence * 100.0
            ));
        }

        if !alternatives.is_empty() {
            explanation.push_str(&format!(
                "\nConsidered {} alternative plans:\n",
                alternatives.len()
            ));
            for (i, alt) in alternatives.iter().enumerate().take(3) {
                // Show top 3
                explanation.push_str(&format!(
                    "{}. {} (confidence: {:.0}%)\n",
                    i + 1,
                    alt.description,
                    alt.confidence * 100.0
                ));
            }
            if alternatives.len() > 3 {
                explanation.push_str(&format!("  ... and {} more\n", alternatives.len() - 3));
            }
        }

        explanation
    }

    /// Explain what a plan step does
    pub fn explain_step(&self, step: &PlanStep, _context: &PlanningContext) -> String {
        let mut explanation = String::new();

        explanation.push_str(&format!("Step: {}\n", step.description));
        explanation.push_str(&format!("Command: {}\n", step.action.command));
        explanation.push_str(&format!(
            "Estimated time: {} seconds\n",
            step.estimated_time_seconds
        ));
        explanation.push_str(&format!("Confidence: {:.0}%\n", step.confidence * 100.0));

        // Explain what the command does
        explanation.push_str(&format!(
            "Effect: {}\n",
            self.describe_command_effect(&step.action.command, &step.action.args)
        ));

        // Explain prerequisites
        if !step.prerequisites.is_empty() {
            explanation.push_str(&format!(
                "Prerequisites: {}\n",
                step.prerequisites.join(", ")
            ));
        } else {
            explanation.push_str("Prerequisites: None (can start immediately)\n");
        }

        // Explain alternatives if any
        if !step.alternatives.is_empty() {
            explanation.push_str(&format!(
                "Alternatives available: {} options\n",
                step.alternatives.len()
            ));
        }

        explanation
    }

    /// Explain why a plan failed
    pub fn explain_failure(
        &self,
        _plan: &Plan,
        failed_step: &PlanStep,
        error: &str,
        context: &PlanningContext,
    ) -> String {
        let mut explanation = String::new();

        explanation.push_str(&format!(
            "The plan failed at step: {}\n",
            failed_step.description
        ));
        explanation.push_str(&format!("Error: {}\n", error));

        // Provide context
        explanation.push_str("Context when failure occurred:\n");
        explanation.push_str(&format!(
            "• Attempted command: {}\n",
            failed_step.action.command
        ));
        explanation.push_str(&format!("• Command args: {}\n", failed_step.action.args));

        // Provide suggestions based on failure type
        explanation.push_str("Suggestions:\n");
        let suggestions = self.generate_failure_suggestions(failed_step, error, context);
        for suggestion in suggestions {
            explanation.push_str(&format!("• {}\n", suggestion));
        }

        explanation
    }

    /// Explain why an alternative approach might be better
    pub fn explain_alternative(
        &self,
        original: &Plan,
        alternative: &Plan,
        _context: &PlanningContext,
    ) -> String {
        let mut explanation = String::new();

        explanation.push_str(&format!(
            "Alternative plan '{}' might be better because:\n",
            alternative.description
        ));
        explanation.push_str(&format!(
            "• Higher confidence: {:.0}% vs {:.0}%\n",
            alternative.confidence * 100.0,
            original.confidence * 100.0
        ));
        explanation.push_str(&format!(
            "• Lower estimated time: {}s vs {}s\n",
            alternative.estimated_total_time_seconds, original.estimated_total_time_seconds
        ));
        explanation.push_str(&format!(
            "• Lower risk: {:?} vs {:?}\n",
            alternative.risk_level, original.risk_level
        ));

        explanation
    }

    /// Describe what a command does in plain language
    fn describe_command_effect(&self, command: &str, args: &serde_json::Value) -> String {
        match command {
            "add_figure" => {
                let figure_type = args
                    .get("figure_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let figure_name = if figure_type == "genesis9" {
                    "Genesis 9 Female"
                } else {
                    "Genesis 8 Female"
                };
                format!("Add a {} to the scene", figure_name)
            },
            "load_asset" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                format!("Load asset from {}", path)
            },
            "apply_pose" => {
                let pose = args
                    .get("pose")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                format!("Apply the {} pose", pose)
            },
            "set_keyframe" => {
                let node_id = args
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let property = args
                    .get("property")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let frame = args.get("frame").and_then(|v| v.as_i64()).unwrap_or(0);
                let value = args.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                format!(
                    "Set {} on {} to {} at frame {}",
                    property, node_id, value, frame
                )
            },
            "add_node" => {
                let node_type = args
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unnamed");
                format!("Add a {} named {}", node_type, name)
            },
            "set_property" => {
                let node_id = args
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let property = args
                    .get("property")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let value = args
                    .get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                format!("Set {} of {} to {}", property, node_id, value)
            },
            "set_light" => {
                let node_id = args
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let property = args
                    .get("property")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let value = args
                    .get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                format!("Set light {}'s {} to {}", node_id, property, value)
            },
            "render_preview" => "Generate a preview render of the current scene".to_string(),
            "export_scene" => {
                let format = args
                    .get("settings")
                    .and_then(|s| s.get("format").and_then(|v| v.as_str()))
                    .unwrap_or("unknown");
                format!("Export the scene as a {} file", format)
            },
            "run_dforce_simulation" => {
                let node_id = args
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let start = args
                    .get("start_frame")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let end = args.get("end_frame").and_then(|v| v.as_i64()).unwrap_or(30);
                format!(
                    "Run a dForce physics simulation on {} from frame {} to {}",
                    node_id, start, end
                )
            },
            "begin_undo_batch" => "Start recording changes for undo/redo".to_string(),
            "accept_undo_batch" => "Accept and name the recorded changes".to_string(),
            "cancel_undo_batch" => "Discard the recorded changes".to_string(),
            _ => {
                format!("Execute the {} command", command)
            },
        }
    }

    /// Generate suggestions for fixing a failed step
    fn generate_failure_suggestions(
        &self,
        step: &PlanStep,
        error: &str,
        _context: &PlanningContext,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        if error.contains("not found") || error.contains("failed to find") {
            suggestions.push("Check that the asset name or path is correct".to_string());
            suggestions.push("Try searching for the asset using different keywords".to_string());
            suggestions
                .push("Make sure the asset is installed and indexed in your library".to_string());
        }

        if error.contains("invalid") || error.contains("unsupported") {
            suggestions.push("Verify that the command parameters are correct".to_string());
            suggestions
                .push("Check if the asset type is compatible with the operation".to_string());
        }

        if error.contains("permission") || error.contains("denied") {
            suggestions.push("This operation may require explicit permission".to_string());
            suggestions.push("Try running a similar, lower-risk action first".to_string());
        }

        if step.action.command == "load_asset" && error.contains("not found") {
            suggestions.push("Try using the asset's exact filename from your library".to_string());
            suggestions.push("Search for similar assets that might work instead".to_string());
        }

        if step.action.command == "add_figure" {
            suggestions.push("Make sure the figure is installed in your Daz3D library".to_string());
            suggestions
                .push("Try checking the exact figure name in your content library".to_string());
        }

        if step.action.command.starts_with("set_") && error.contains("property") {
            suggestions.push(
                "Verify that the property name is correct for the selected node type".to_string(),
            );
            suggestions
                .push("Try checking what properties are available for this node type".to_string());
        }

        // If we have no specific suggestions, give general ones
        if suggestions.is_empty() {
            suggestions.push("Double-check the command parameters and try again".to_string());
            suggestions.push("Consider breaking this down into smaller steps".to_string());
            suggestions.push("Look at similar successful actions in your history".to_string());
        }

        suggestions
    }

    /// Explain what a plan achieves
    pub fn explain_plan_outcome(&self, plan: &Plan) -> String {
        let mut explanation = String::new();

        explanation.push_str("If successful, this plan will:\n");

        // Group steps by type to give a higher-level explanation
        let mut figures_added = 0;
        let mut assets_loaded = 0;
        let mut lights_added = 0;
        let mut poses_applied = 0;
        let mut renders_done = 0;

        for step in &plan.steps {
            match step.action.command.as_str() {
                "add_figure" => figures_added += 1,
                "load_asset" => assets_loaded += 1,
                "add_node" => {
                    if let Some(node_type) = step.action.args.get("type").and_then(|v| v.as_str()) {
                        if node_type.contains("_light") {
                            lights_added += 1;
                        }
                    }
                },
                "apply_pose" => poses_applied += 1,
                "render_preview" | "render" => renders_done += 1,
                _ => {},
            }
        }

        if figures_added > 0 {
            explanation.push_str(&format!(
                "• Add {} figure{}\n",
                figures_added,
                if figures_added == 1 { "" } else { "s" }
            ));
        }
        if assets_loaded > 0 {
            explanation.push_str(&format!(
                "• Load {} asset{}\n",
                assets_loaded,
                if assets_loaded == 1 { "" } else { "s" }
            ));
        }
        if lights_added > 0 {
            explanation.push_str(&format!(
                "• Add {} light{}\n",
                lights_added,
                if lights_added == 1 { "" } else { "s" }
            ));
        }
        if poses_applied > 0 {
            explanation.push_str(&format!(
                "• Apply {} pose{}\n",
                poses_applied,
                if poses_applied == 1 { "" } else { "s" }
            ));
        }
        if renders_done > 0 {
            explanation.push_str(&format!(
                "• Generate {} render{}\n",
                renders_done,
                if renders_done == 1 { "" } else { "s" }
            ));
        }

        if explanation.ends_with(":\n") {
            explanation.push_str("• No significant changes to the scene\n");
        }

        explanation
    }
}

impl Default for Explainer {
    fn default() -> Self {
        Self::new()
    }
}
