use crate::agents::{AgentRequest, AgentResponse, AgentAction};
use crate::ai_system::vector_store;

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    
    // 1. Clean query
    let search_query = input
        .replace("load", "")
        .replace("the", "")
        .replace("find", "")
        .replace("apply", "")
        .replace("search", "")
        .trim()
        .to_string();

    // 2. Proactively try semantic matching for 'vibe' queries
    let semantic_results = vector_store::get_semantic_matches(&search_query);
    
    let mut actions = vec![];
    let result_msg = if semantic_results.is_empty() {
        "No semantic matches found. Defaulting to keyword search...".to_string()
    } else {
        format!("Found {} semantic match(es).", semantic_results.len())
    };

    if !semantic_results.is_empty() {
        for (path, score) in semantic_results {
            actions.push(AgentAction {
                action_type: "suggest_load".to_string(),
                command: "load_asset".to_string(),
                args: vec![path, format!("Confidence: {:.2}", score)],
            });
        }
    } else {
        // Fallback to existing FTS5 logic...
        // [Existing logic here]
    }
    
    AgentResponse {
        success: true,
        result: Some(result_msg),
        error: None,
        actions,
    }
}
