use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::SystemTime;

/// Represents a failure case that the AI has learned from
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureCase {
    pub id: String,
    pub context: FailureContext,
    pub attempted_action: String,
    pub error_message: String,
    pub resolution: Option<String>,
    pub timestamp: String,
    pub occurrence_count: u32,
    pub tags: Vec<String>,
}

/// Context in which a failure occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureContext {
    pub scene_state: Option<SceneStateSnapshot>,
    pub user_intent: String,
    pub assets_involved: Vec<String>,
    pub recent_actions: Vec<String>,
}

/// Snapshot of scene state at time of failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneStateSnapshot {
    pub active_figure: Option<String>,
    pub selected_nodes: Vec<String>,
    pub lights_in_scene: Vec<String>,
    pub cameras_in_scene: Vec<String>,
}

/// Knowledge base that learns from failures to improve future suggestions
#[derive(Debug)]
pub struct FailureKnowledgeBase {
    pub failures: Mutex<HashMap<String, FailureCase>>,
    pub failure_patterns: Mutex<HashMap<String, Vec<String>>>, // pattern -> list of failure IDs
    pub success_patterns: Mutex<HashMap<String, Vec<String>>>, // pattern -> list of success IDs
}

impl FailureKnowledgeBase {
    pub fn new() -> Self {
        Self {
            failures: Mutex::new(HashMap::new()),
            failure_patterns: Mutex::new(HashMap::new()),
            success_patterns: Mutex::new(HashMap::new()),
        }
    }

    /// Record a failure for learning
    pub fn record_failure(
        &self,
        context: FailureContext,
        action: &str,
        error: &str,
        resolution: Option<String>,
    ) {
        let mut failures = self.failures.lock().unwrap();
        let mut patterns = self.failure_patterns.lock().unwrap();

        // Generate failure ID
        let id = format!(
            "failure_{}_{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            fastrand::u64(..)
        );

        // Create failure case
        let failure = FailureCase {
            id: id.clone(),
            context,
            attempted_action: action.to_string(),
            error_message: error.to_string(),
            resolution,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
            occurrence_count: 1,
            tags: Vec::new(), // TODO: extract tags from context/error
        };

        // Store the failure
        failures.insert(id.clone(), failure.clone());

        // Update patterns
        let pattern = self.extract_failure_pattern(&failure);
        patterns.entry(pattern).or_default().push(id);
    }

    /// Record a success for contrastive learning
    pub fn record_success(&self, context: FailureContext, action: &str) {
        let mut patterns = self.success_patterns.lock().unwrap();

        // Generate success ID
        let id = format!(
            "success_{}_{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            fastrand::u64(..)
        );

        // For successes, we might not store full details to save space
        // Just track the pattern
        let pattern = self.extract_success_pattern(&context, action);
        patterns.entry(pattern).or_default().push(id);
    }

    /// Get failures related to a given context
    pub fn get_related_failures(&self, context: &FailureContext, limit: usize) -> Vec<FailureCase> {
        let failures = self.failures.lock().unwrap();
        let mut scored: Vec<(f32, FailureCase)> = failures
            .values()
            .map(|f| {
                let relevance = self.calculate_context_relevance(context, &f.context);
                (relevance, f.clone())
            })
            .filter(|(score, _)| *score > 0.1) // Only relevant failures
            .collect();

        // Sort by relevance (highest first)
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // Return top failures
        scored
            .into_iter()
            .take(limit)
            .map(|(_, failure)| failure)
            .collect()
    }

    /// Get warnings based on failure patterns
    pub fn get_warnings_for_action(&self, context: &FailureContext, _action: &str) -> Vec<String> {
        let related_failures = self.get_related_failures(context, 10);
        let mut warnings = Vec::new();

        // Extract common error messages from similar failures
        let mut error_counts = HashMap::new();
        for failure in &related_failures {
            *error_counts.entry(&failure.error_message).or_insert(0) += 1;
        }

        // Convert to warnings
        for (error, count) in error_counts {
            if count >= 2 {
                // Only warn if we've seen this error multiple times
                warnings.push(format!(
                    "Warning: This action has previously caused errors '{}' ({} times). Consider alternatives.",
                    error, count
                ));
            }
        }

        warnings
    }

    /// Get suggested alternatives based on past failures
    pub fn get_suggested_alternatives(
        &self,
        _context: &FailureContext,
        _action: &str,
    ) -> Vec<String> {
        // This would ideally look at what actions succeeded in similar contexts
        // For now, return empty - would need success tracking integrated
        Vec::new()
    }

    /// Extract a pattern from a failure for categorization
    fn extract_failure_pattern(&self, failure: &FailureCase) -> String {
        let mut parts = Vec::new();

        // Add context elements
        if let Some(ref figure) = failure
            .context
            .scene_state
            .as_ref()
            .and_then(|s| s.active_figure.as_ref())
        {
            parts.push(format!("figure:{}", figure));
        }

        // Add action type
        if failure.attempted_action.contains("load") {
            parts.push("action:load".to_string());
        } else if failure.attempted_action.contains("pose") {
            parts.push("action:pose".to_string());
        } else if failure.attempted_action.contains("light") {
            parts.push("action:light".to_string());
        } else {
            parts.push(format!("action:{}", failure.attempted_action));
        }

        // Add error type
        if failure.error_message.contains("not found") {
            parts.push("error:not_found".to_string());
        } else if failure.error_message.contains("incompatible") {
            parts.push("error:incompatible".to_string());
        } else if failure.error_message.contains("failed") {
            parts.push("error:failed".to_string());
        } else {
            parts.push("error:other".to_string());
        }

        parts.join("_")
    }

    /// Extract a pattern from a success context
    fn extract_success_pattern(&self, context: &FailureContext, action: &str) -> String {
        let mut parts = Vec::new();

        // Add context elements
        if let Some(ref figure) = context
            .scene_state
            .as_ref()
            .and_then(|s| s.active_figure.as_ref())
        {
            parts.push(format!("figure:{}", figure));
        }

        // Add action type
        if action.contains("load") {
            parts.push("action:load".to_string());
        } else if action.contains("pose") {
            parts.push("action:pose".to_string());
        } else if action.contains("light") {
            parts.push("action:light".to_string());
        } else {
            parts.push(format!("action:{}", action));
        }

        parts.join("_")
    }

    /// Calculate relevance between two contexts (0.0 to 1.0)
    fn calculate_context_relevance(&self, ctx1: &FailureContext, ctx2: &FailureContext) -> f32 {
        let mut score = 0.0;
        let mut total_weight = 0.0;

        // Compare active figures
        let figure1 = ctx1
            .scene_state
            .as_ref()
            .and_then(|s| s.active_figure.as_ref());
        let figure2 = ctx2
            .scene_state
            .as_ref()
            .and_then(|s| s.active_figure.as_ref());
        if let (Some(f1), Some(f2)) = (figure1, figure2) {
            if f1 == f2 {
                score += 0.3;
            }
            total_weight += 0.3;
        }

        // Compare user intents (simple string similarity)
        let intent_similarity = self.string_similarity(&ctx1.user_intent, &ctx2.user_intent);
        score += intent_similarity * 0.2;
        total_weight += 0.2;

        // Compare assets involved (Jaccard similarity)
        let assets1: std::collections::HashSet<_> = ctx1.assets_involved.iter().collect();
        let assets2: std::collections::HashSet<_> = ctx2.assets_involved.iter().collect();
        if !assets1.is_empty() || !assets2.is_empty() {
            let intersection = assets1.intersection(&assets2).count();
            let union = assets1.union(&assets2).count();
            if union > 0 {
                let jaccard = intersection as f32 / union as f32;
                score += jaccard * 0.3;
            }
            total_weight += 0.3;
        }

        // Compare recent actions (simple overlap)
        let actions1: std::collections::HashSet<_> = ctx1.recent_actions.iter().collect();
        let actions2: std::collections::HashSet<_> = ctx2.recent_actions.iter().collect();
        if !actions1.is_empty() || !actions2.is_empty() {
            let intersection = actions1.intersection(&actions2).count();
            let union = actions1.union(&actions2).count();
            if union > 0 {
                let jaccard = intersection as f32 / union as f32;
                score += jaccard * 0.2;
            }
            total_weight += 0.2;
        }

        if total_weight > 0.0 {
            score / total_weight
        } else {
            0.0
        }
    }

    /// Simple string similarity (0.0 to 1.0)
    fn string_similarity(&self, a: &str, b: &str) -> f32 {
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }

        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();

        if a_lower == b_lower {
            return 1.0;
        }

        // Simple character-level similarity
        let matches = a_lower
            .chars()
            .zip(b_lower.chars())
            .filter(|(ca, cb)| ca == cb)
            .count();

        let max_len = a_lower.len().max(b_lower.len()) as f32;
        if max_len > 0.0 {
            matches as f32 / max_len
        } else {
            0.0
        }
    }
}

impl Default for FailureKnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}
