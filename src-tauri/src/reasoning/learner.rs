use crate::knowledge::asset_knowledge::AssetKnowledgeBase;
use crate::knowledge::daz_concepts::DazKnowledgeBase;
use crate::knowledge::failure_knowledge::FailureKnowledgeBase;
use crate::knowledge::scene_knowledge::SceneKnowledgeBase;
use crate::knowledge::workflow_knowledge::WorkflowKnowledgeBase;
use crate::reasoning::executor::ExecutionResult;
use crate::reasoning::planner::{Plan, PlanStep, PlanningContext};
use crate::reasoning::validator::ValidationResult;
use std::collections::HashMap;
use std::sync::Mutex;

/// Learns from plan execution to improve future planning
pub struct Learner {
    pub failure_knowledge: FailureKnowledgeBase,
    pub workflow_knowledge: WorkflowKnowledgeBase,
    pub asset_knowledge: AssetKnowledgeBase,
    pub scene_knowledge: SceneKnowledgeBase,
    pub daz_knowledge: DazKnowledgeBase,
    pub plan_success_rates: Mutex<HashMap<String, f32>>, // plan pattern -> success rate
    pub plan_usage_counts: Mutex<HashMap<String, u32>>,  // plan pattern -> usage count
}

impl Learner {
    pub fn new() -> Self {
        Self {
            failure_knowledge: FailureKnowledgeBase::new(),
            workflow_knowledge: WorkflowKnowledgeBase::new(),
            asset_knowledge: AssetKnowledgeBase::new(),
            scene_knowledge: SceneKnowledgeBase::new(),
            daz_knowledge: DazKnowledgeBase::new(),
            plan_success_rates: Mutex::new(HashMap::new()),
            plan_usage_counts: Mutex::new(HashMap::new()),
        }
    }

    /// Learn from a plan execution
    pub fn learn_from_execution(
        &self,
        plan: &Plan,
        context: &PlanningContext,
        result: &ExecutionResult,
        validation: &ValidationResult,
    ) {
        // 1. Record the plan usage pattern
        self.record_plan_usage(plan, result.is_success());

        // 2. Learn from failures if any
        if let ExecutionResult::Failed {
            reason,
            details,
            step_executed,
        } = result
        {
            self.learn_from_failure(plan, context, reason, details, *step_executed);
        } else if let ExecutionResult::PartialSuccess {
            successful_steps,
            total_steps,
            ..
        } = result
        {
            // Learn from the steps that failed
            if *successful_steps < *total_steps {
                // We need to know which steps failed - this would require more detailed tracking
                // For now, we'll treat partial success as a learning opportunity
                self.learn_from_partial_success(plan, context, *successful_steps, *total_steps);
            }
        }

        // 3. Learn from validation issues (warnings, errors)
        self.learn_from_validation(plan, context, validation);

        // 4. Update workflow knowledge based on what worked
        self.update_workflow_knowledge(plan, context, result);

        // 5. Update asset knowledge if we discovered new things
        self.update_asset_knowledge(plan, context, result);
    }

    /// Record that a plan was used and whether it succeeded
    fn record_plan_usage(&self, plan: &Plan, success: bool) {
        let pattern = self.plan_pattern_to_string(plan);

        // Update usage count
        let mut usage_counts = self.plan_usage_counts.lock().unwrap();
        *usage_counts.entry(pattern.clone()).or_insert(0) += 1;

        // Update success rate using exponential moving average
        let mut success_rates = self.plan_success_rates.lock().unwrap();
        let current_rate = *success_rates.get(&pattern).unwrap_or(&0.5); // Start with 50% assumption
        let alpha = 0.3; // Learning rate
        let new_success = if success { 1.0 } else { 0.0 };
        let updated_rate = (1.0 - alpha) * current_rate + alpha * new_success;
        success_rates.insert(pattern, updated_rate);
    }

    /// Learn from a failed plan execution
    fn learn_from_failure(
        &self,
        plan: &Plan,
        context: &PlanningContext,
        reason: &str,
        details: &str,
        step_executed: usize,
    ) {
        // Create failure context from planning context
        let failure_context = self.create_failure_context(plan, context);

        // Record the failure
        self.failure_knowledge.record_failure(
            failure_context,
            &format!("Plan execution failed: {} steps executed", step_executed),
            &format!("{}: {}", reason, details),
            None, // No automatic resolution yet
        );

        // Learn which parts of the plan were problematic
        if let Some(failed_step) = plan.steps.get(step_executed.saturating_sub(1)) {
            // Record failure for this specific step
            let step_context = self.create_step_failure_context(failed_step, plan, context);
            self.failure_knowledge.record_failure(
                step_context,
                &failed_step.action.command,
                &format!("{}: {}", reason, details),
                None,
            );
        }
    }

    /// Learn from partial success
    fn learn_from_partial_success(
        &self,
        plan: &Plan,
        context: &PlanningContext,
        successful_steps: usize,
        total_steps: usize,
    ) {
        // This indicates that some steps worked and some didn't
        // We could analyze which types of steps tend to fail in this context
        // For now, we'll just note that the plan needs improvement

        let _pattern = self.plan_pattern_to_string(plan);
        let success_rate = successful_steps as f32 / total_steps as f32;

        // If success rate is low, we might want to avoid this pattern in similar contexts
        if success_rate < 0.5 {
            // This is a weak signal - in a full system we'd weigh this against other evidence
            // For now, just record it in failure knowledge as a pattern to avoid
            let failure_context = self.create_failure_context(plan, context);
            self.failure_knowledge.record_failure(
                failure_context,
                &format!(
                    "Partial success plan ({}/{}) steps",
                    successful_steps, total_steps
                ),
                "Plan had low success rate in this context",
                None,
            );
        }
    }

    /// Learn from validation results (warnings, errors)
    fn learn_from_validation(
        &self,
        plan: &Plan,
        context: &PlanningContext,
        validation: &ValidationResult,
    ) {
        // Learn from validation errors
        for error in &validation.errors {
            let failure_context = self.create_failure_context(plan, context);
            self.failure_knowledge.record_failure(
                failure_context,
                "Plan validation",
                &format!("Validation error: {}", error),
                None,
            );
        }

        // Learn from validation warnings (these are opportunities for improvement)
        for _warning in &validation.warnings {
            // Warnings are less severe than errors but still indicate areas for improvement
            // We might want to adjust the plan to avoid these warnings in future
            // For now, we'll track them as potential issues
        }
    }

    /// Update workflow knowledge based on what worked in a plan
    fn update_workflow_knowledge(
        &self,
        _plan: &Plan,
        _context: &PlanningContext,
        _result: &ExecutionResult,
    ) {
        // If the plan was based on a workflow template, update that template's success rate
        // This would require tracking which workflow template was used
        // For now, we'll skip this sophisticated tracking
    }

    /// Update asset knowledge if we discovered new things during execution
    fn update_asset_knowledge(
        &self,
        _plan: &Plan,
        _context: &PlanningContext,
        _result: &ExecutionResult,
    ) {
        // If we successfully loaded assets, we might have learned about their usability
        // If we failed to load assets, we might have learned about compatibility issues
        // This would require more detailed tracking of what assets were involved
    }

    /// Create a failure context from planning context
    fn create_failure_context(
        &self,
        plan: &Plan,
        context: &PlanningContext,
    ) -> crate::knowledge::failure_knowledge::FailureContext {
        crate::knowledge::failure_knowledge::FailureContext {
            scene_state: context.scene_state.clone(),
            user_intent: format!("Executing plan: {}", plan.description),
            assets_involved: self.extract_assets_from_plan(plan),
            recent_actions: context.recent_actions.clone(),
        }
    }

    /// Create a failure context for a specific step
    fn create_step_failure_context(
        &self,
        step: &PlanStep,
        _plan: &Plan,
        context: &PlanningContext,
    ) -> crate::knowledge::failure_knowledge::FailureContext {
        crate::knowledge::failure_knowledge::FailureContext {
            scene_state: context.scene_state.clone(),
            user_intent: format!("Executing step: {}", step.description),
            assets_involved: self.extract_assets_from_step(step),
            recent_actions: {
                let mut actions = context.recent_actions.clone();
                actions.push(format!("Executing step: {}", step.description));
                actions
            },
        }
    }

    /// Extract assets referenced in a plan
    fn extract_assets_from_plan(&self, plan: &Plan) -> Vec<String> {
        let mut assets = Vec::new();
        for step in &plan.steps {
            assets.extend(self.extract_assets_from_step(step));
        }
        assets
    }

    /// Extract assets referenced in a plan step
    fn extract_assets_from_step(&self, step: &PlanStep) -> Vec<String> {
        let mut assets = Vec::new();

        // Extract from load_asset commands
        if step.action.command == "load_asset" {
            if let Some(path) = step.action.args.get("path").and_then(|p| p.as_str()) {
                if !path.is_empty() && !path.contains("TODO") && !path.contains("unknown") {
                    assets.push(path.to_string());
                }
            }
        }

        // Extract from add_figure commands (these reference figures, not assets per se)
        if step.action.command == "add_figure" {
            if let Some(figure_type) = step.action.args.get("figure_type").and_then(|p| p.as_str())
            {
                let figure_name = if figure_type == "genesis9" {
                    "Genesis 9 Female".to_string()
                } else {
                    "Genesis 8 Female".to_string()
                };
                assets.push(figure_name);
            }
        }

        assets
    }

    /// Convert a plan to a string pattern for tracking
    fn plan_pattern_to_string(&self, plan: &Plan) -> String {
        let mut parts = Vec::new();

        // Add goal intent
        parts.push(format!("intent:{:?}", plan.goal.intent));

        // Add goal entities (simplified)
        let entity_desc: Vec<String> = plan
            .goal
            .entities
            .iter()
            .map(|e| format!("{}:{}", e.entity_type.clone() as u32, e.value.clone()))
            .collect();
        parts.push(format!("entities:[{}]", entity_desc.join(",")));

        // Add step types (simplified)
        let step_types: Vec<String> = plan
            .steps
            .iter()
            .map(|s| s.action.command.clone())
            .collect();
        parts.push(format!("steps:[{}]", step_types.join(",")));

        parts.join("|")
    }

    /// Get the historical success rate for a plan pattern
    pub fn get_plan_success_rate(&self, plan: &Plan) -> f32 {
        let pattern = self.plan_pattern_to_string(plan);
        let success_rates = self.plan_success_rates.lock().unwrap();
        *success_rates.get(&pattern).unwrap_or(&0.5) // Default to 50% if no history
    }

    /// Get usage count for a plan pattern
    pub fn get_plan_usage_count(&self, plan: &Plan) -> u32 {
        let pattern = self.plan_pattern_to_string(plan);
        let usage_counts = self.plan_usage_counts.lock().unwrap();
        *usage_counts.get(&pattern).unwrap_or(&0)
    }

    /// Get similar successful plans based on context
    pub fn get_similar_successful_plans(
        &self,
        _context: &PlanningContext,
        _limit: usize,
    ) -> Vec<(String, f32)> {
        // This would involve comparing the current context to past contexts
        // and finding plans that succeeded in similar situations
        // For now, return empty - would need more sophisticated context matching
        Vec::new()
    }
}

impl Default for Learner {
    fn default() -> Self {
        Self::new()
    }
}
