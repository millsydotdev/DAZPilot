use crate::mcp_client::validate_command;
use crate::reasoning::planner::Plan;
use crate::reasoning::planner::PlanningContext;
use serde::{Deserialize, Serialize};

/// Validates plans for feasibility and correctness
pub struct Validator {
    pub knowledge_base: crate::knowledge::daz_concepts::DazKnowledgeBase,
}

impl Validator {
    pub fn new() -> Self {
        Self {
            knowledge_base: crate::knowledge::daz_concepts::DazKnowledgeBase::new(),
        }
    }

    /// Validate a plan for execution
    pub fn validate_plan(&self, plan: &Plan, context: &PlanningContext) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 1. Validate each step's command
        for step in &plan.steps {
            let cmd_result = validate_command(&step.action.command, &step.action.args);
            if let Err(e) = cmd_result {
                errors.push(format!(
                    "Step '{}': Invalid command '{}': {}",
                    step.description, step.action.command, e
                ));
            }
        }

        // 2. Check for circular dependencies
        if let Err(e) = self.check_circular_dependencies(plan) {
            errors.push(format!("Circular dependency detected: {}", e));
        }

        // 3. Validate prerequisites exist
        for step in &plan.steps {
            for prereq in &step.prerequisites {
                if !plan.steps.iter().any(|s| s.id == *prereq) {
                    warnings.push(format!(
                        "Step '{}': Prerequisite '{}' not found in plan",
                        step.description, prereq
                    ));
                }
            }
        }

        // 4. Check resource availability (simplified)
        let resource_warnings = self.check_resource_availability(plan, context);
        warnings.extend(resource_warnings);

        // 5. Check against user permissions (placeholder)
        let permission_warnings = self.check_permissions(plan, context);
        warnings.extend(permission_warnings);

        let is_valid = errors.is_empty();

        ValidationResult {
            is_valid,
            errors,
            warnings: warnings.clone(),
            suggestions: if !warnings.is_empty() {
                vec![
                    "Consider addressing warnings before execution".to_string(),
                    "You can proceed but be aware of potential issues".to_string(),
                ]
            } else {
                Vec::new()
            },
        }
    }

    /// Check for circular dependencies in plan steps
    fn check_circular_dependencies(&self, plan: &Plan) -> Result<(), String> {
        // Build adjacency list
        let mut adj: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for step in &plan.steps {
            adj.insert(step.id.clone(), Vec::new());
            for prereq in &step.prerequisites {
                adj.get_mut(&step.id).unwrap().push(prereq.clone());
            }
        }

        // DFS to detect cycles
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();

        for step in &plan.steps {
            if Self::has_cycle_util(self, &step.id, &adj, &mut visited, &mut rec_stack)? {
                return Err(format!("Cycle involving step {}", step.id));
            }
        }

        Ok(())
    }

    fn has_cycle_util(
        &self,
        v: &str,
        adj: &std::collections::HashMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
    ) -> Result<bool, String> {
        if !visited.contains(v) {
            visited.insert(v.to_string());
            rec_stack.insert(v.to_string());

            if let Some(neighbors) = adj.get(v) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        if Self::has_cycle_util(self, neighbor, adj, visited, rec_stack)? {
                            return Ok(true);
                        }
                    } else if rec_stack.contains(neighbor) {
                        return Ok(true);
                    }
                }
            }
        }

        if rec_stack.contains(v) {
            rec_stack.remove(v);
        }

        Ok(false)
    }

    /// Check if required resources are available
    fn check_resource_availability(&self, plan: &Plan, _context: &PlanningContext) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check if referenced assets exist in knowledge base
        for step in &plan.steps {
            // Check load_asset commands for asset availability
            if step.action.command == "load_asset" {
                if let Some(path) = step.action.args.get("path").and_then(|p| p.as_str()) {
                    // In a real implementation, we'd check if this asset exists in the library
                    // For now, just warn if it looks like a placeholder
                    if path.contains("TODO") || path.contains("unknown") || path.is_empty() {
                        warnings.push(format!(
                            "Step '{}': Load asset command has unresolved path: '{}'",
                            step.description, path
                        ));
                    }
                }
            }

            // Check asset creation commands for resource availability
            if step.action.command == "add_figure" {
                if let Some(figure_type) =
                    step.action.args.get("figure_type").and_then(|p| p.as_str())
                {
                    let _needed_figure = if figure_type == "genesis9" {
                        "Genesis 9 Female"
                    } else {
                        "Genesis 8 Female"
                    }
                    .to_string();

                    // Check if we have this figure in our knowledge base (we always do for now)
                    // In future, check against actually installed/content
                }
            }

            if step.action.command == "add_node" {
                if let Some(node_type) = step.action.args.get("type").and_then(|p| p.as_str()) {
                    // Validate that this is a known node type
                    let valid_types = [
                        "point_light",
                        "spot_light",
                        "distant_light",
                        "area_light",
                        "camera",
                        "null",
                    ];
                    if !valid_types.contains(&node_type) {
                        warnings.push(format!(
                            "Step '{}': Unknown node type '{}'",
                            step.description, node_type
                        ));
                    }
                }
            }
        }

        warnings
    }

    /// Check if plan actions comply with user permissions
    fn check_permissions(&self, plan: &Plan, _context: &PlanningContext) -> Vec<String> {
        let mut warnings = Vec::new();

        // High-risk operations that might need permission
        let high_risk_commands = ["export_scene", "delete_node", "run_script"];

        for step in &plan.steps {
            if high_risk_commands.contains(&step.action.command.as_str()) {
                warnings.push(format!(
                    "Step '{}': This is a high-risk operation that may require explicit permission",
                    step.description
                ));
            }
        }

        warnings
    }
}

/// Result of plan validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}
