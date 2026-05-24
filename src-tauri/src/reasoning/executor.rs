use crate::ai_action::execute_structured_action;
use crate::mcp_client::is_connected;
use crate::reasoning::planner::{Plan, PlanStep, PlanningContext};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Executes plans by carrying out their steps in order
pub struct Executor {
    pub validator: super::validator::Validator,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            validator: super::validator::Validator::new(),
        }
    }

    /// Execute a plan step by step
    pub async fn execute_plan(&self, plan: &Plan, context: &PlanningContext) -> ExecutionResult {
        // 1. Validate the plan first
        let validation = self.validator.validate_plan(plan, context);
        if !validation.is_valid {
            return ExecutionResult::Failed {
                reason: "Plan validation failed".to_string(),
                details: validation.errors.join("\n"),
                step_executed: 0,
            };
        }

        // 2. Check if we're connected to Daz3D (if needed)
        if self.requires_daz3d_connection(plan) && !is_connected() {
            return ExecutionResult::Failed {
                reason: "Not connected to Daz3D".to_string(),
                details: "Please connect to Daz3D Studio before executing this plan".to_string(),
                step_executed: 0,
            };
        }

        // 3. Execute steps in order, respecting prerequisites
        let mut completed_steps = std::collections::HashSet::new();
        let mut step_results = Vec::new();
        let start_time = Instant::now();

        // Continue until all steps are done or we encounter a failure
        loop {
            // Find next executable step (all prerequisites met)
            let mut next_step = None;
            let mut next_step_index = None;

            for (idx, step) in plan.steps.iter().enumerate() {
                if !completed_steps.contains(&step.id) {
                    // Check if all prerequisites are met
                    let prereqs_met = step
                        .prerequisites
                        .iter()
                        .all(|prereq| completed_steps.contains(prereq));

                    if prereqs_met {
                        next_step = Some(step);
                        next_step_index = Some(idx);
                        break;
                    }
                }
            }

            // If no executable step found, we're either done or deadlocked
            if next_step.is_none() {
                // Check if all steps are completed
                if completed_steps.len() == plan.steps.len() {
                    break; // All done successfully
                } else {
                    // Deadlock - circular dependency or missing prerequisite
                    return ExecutionResult::Failed {
                        reason: "Deadlock detected - cannot proceed with remaining steps"
                            .to_string(),
                        details: format!(
                            "Completed steps: {}, Total steps: {}",
                            completed_steps.len(),
                            plan.steps.len()
                        ),
                        step_executed: completed_steps.len(),
                    };
                }
            }

            let step = next_step.unwrap();
            let _step_index = next_step_index.unwrap();

            // 4. Execute the step
            let step_result = self.execute_step(step, context).await;
            step_results.push((step.id.clone(), step_result.clone()));

            match &step_result {
                ExecutionStepResult::Success { .. } => {
                    completed_steps.insert(step.id.clone());
                },
                ExecutionStepResult::Failed { .. } => {
                    // Step failed - determine if we can continue or need to abort
                    if self.is_critical_step(step, &plan.steps) {
                        return ExecutionResult::Failed {
                            reason: format!("Critical step failed: {}", step.description),
                            details: match &step_result {
                                ExecutionStepResult::Failed { reason, details } => {
                                    format!("{}: {}", reason, details)
                                },
                                _ => "Unknown error".to_string(),
                            },
                            step_executed: completed_steps.len(),
                        };
                    }
                    // For non-critical steps, we might continue but mark as problematic
                    completed_steps.insert(step.id.clone()); // Still count as attempted
                },
                ExecutionStepResult::Skipped { reason: _ } => {
                    completed_steps.insert(step.id.clone());
                },
            }

            // Small delay between steps to prevent overwhelming the bridge
            sleep(Duration::from_millis(100)).await;
        }

        // 5. Calculate final result
        let total_time = start_time.elapsed();
        let successful_steps = step_results
            .iter()
            .filter(|(_, res)| matches!(res, ExecutionStepResult::Success { .. }))
            .count();

        if successful_steps == plan.steps.len() {
            // All steps succeeded
            ExecutionResult::Success {
                total_time,
                steps_executed: plan.steps.len(),
                step_results: step_results
                    .into_iter()
                    .map(|(id, res)| {
                        (
                            id,
                            match res {
                                ExecutionStepResult::Success { output } => Output::Success(output),
                                _ => Output::Failure("Unexpected".to_string()),
                            },
                        )
                    })
                    .collect(),
            }
        } else if successful_steps > 0 {
            // Partial success
            ExecutionResult::PartialSuccess {
                successful_steps,
                total_steps: plan.steps.len(),
                total_time,
                step_results: step_results
                    .into_iter()
                    .map(|(id, res)| {
                        (
                            id,
                            match res {
                                ExecutionStepResult::Success { output } => Output::Success(output),
                                ExecutionStepResult::Failed { reason, details } => {
                                    Output::Failure(format!("{}: {}", reason, details))
                                },
                                ExecutionStepResult::Skipped { reason } => Output::Skipped(reason),
                            },
                        )
                    })
                    .collect(),
            }
        } else {
            // All steps failed
            ExecutionResult::Failed {
                reason: "All steps failed".to_string(),
                details: format!("Executed {} steps, all failed", step_results.len()),
                step_executed: 0,
            }
        }
    }

    /// Execute a single step
    async fn execute_step(
        &self,
        step: &PlanStep,
        _context: &PlanningContext,
    ) -> ExecutionStepResult {
        if Self::has_unresolved_optional_args(step) {
            return ExecutionStepResult::Skipped {
                reason: format!(
                    "Optional step '{}' skipped because required content was not resolved",
                    step.description
                ),
            };
        }

        match execute_structured_action(step.action.clone()) {
            Ok(output) => ExecutionStepResult::Success { output },
            Err(e) => ExecutionStepResult::Failed {
                reason: e,
                details: format!("Failed to execute command: {}", step.action.command),
            },
        }
    }

    fn has_unresolved_optional_args(step: &PlanStep) -> bool {
        let optional_content_step = matches!(
            step.action.command.as_str(),
            "apply_pose" | "set_material_texture"
        );
        if !optional_content_step {
            return false;
        }

        ["pose_path", "file_path"].iter().any(|key| {
            step.action
                .args
                .get(*key)
                .and_then(|value| value.as_str())
                .map(|value| {
                    value.trim().is_empty() || value.contains("TODO") || value.contains("unknown")
                })
                .unwrap_or(false)
        })
    }

    /// Determine if a step is critical (failure should abort the plan)
    fn is_critical_step(&self, step: &PlanStep, all_steps: &[PlanStep]) -> bool {
        // Steps that create essential resources are usually critical
        let critical_commands = [
            "add_figure", // Without a figure, most other steps don't make sense
            "load_asset", // If we're trying to load a specific requested asset
            "add_node",   // If we're creating essential lights/cameras
        ];

        if critical_commands.contains(&step.action.command.as_str()) {
            return true;
        }

        // If many other steps depend on this one, it's likely critical
        let dependents = all_steps
            .iter()
            .filter(|s| s.prerequisites.contains(&step.id))
            .count();

        dependents >= 2
    }

    /// Determine if a plan requires Daz3D connection
    fn requires_daz3d_connection(&self, plan: &Plan) -> bool {
        plan.steps.iter().any(|step| {
            // Most commands require connection except pure planning ones
            !matches!(
                step.action.command.as_str(),
                "begin_undo_batch" | "accept_undo_batch" | "cancel_undo_batch" | "chat"
            )
        })
    }
}

/// Result of executing a plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionResult {
    /// Plan executed successfully completely
    Success {
        total_time: Duration,
        steps_executed: usize,
        step_results: Vec<(String, Output)>,
    },

    /// Plan executed partially (some steps succeeded, some failed/skipped)
    PartialSuccess {
        successful_steps: usize,
        total_steps: usize,
        total_time: Duration,
        step_results: Vec<(String, Output)>,
    },

    /// Plan failed to execute
    Failed {
        reason: String,
        details: String,
        step_executed: usize, // number of steps attempted before failure
    },
}

impl ExecutionResult {
    pub fn is_success(&self) -> bool {
        matches!(self, ExecutionResult::Success { .. })
    }
}

/// Result of executing a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStepResult {
    /// Step succeeded
    Success { output: String },

    /// Step failed
    Failed { reason: String, details: String },

    /// Step was skipped (prerequisites not met, or intentionally skipped)
    Skipped { reason: String },
}

/// Output from a step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Output {
    Success(String),
    Failure(String),
    Skipped(String),
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}
