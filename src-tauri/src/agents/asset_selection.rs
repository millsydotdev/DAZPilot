use crate::agents::{AgentRequest, AgentResponse, AgentAction};
use crate::ai_system::vector_store;
use crate::asset_matcher::MultiStrategyMatcher;

pub fn execute(request: AgentRequest) -> AgentResponse {
    let input = request.input.to_lowercase();
    
    let search_query = input
        .replace("load", "")
        .replace("the", "")
        .replace("find", "")
        .replace("apply", "")
        .replace("search", "")
        .replace("add", "")
        .trim()
        .to_string();

    if search_query.is_empty() {
        return AgentResponse {
            success: true,
            result: Some("Empty query after cleaning.".to_string()),
            error: None,
            actions: vec![],
        };
    }

    let mut actions = vec![];

    // Strategy 1: Semantic (embedding) match — async, best for vibe queries
    let semantic_results = vector_store::get_semantic_matches(&search_query);
    if !semantic_results.is_empty() {
        for (path, score) in &semantic_results {
            actions.push(AgentAction {
                action_type: "suggest_load".to_string(),
                command: "load_asset".to_string(),
                args: vec![path.clone(), format!("semantic_score:{:.2}", score)],
            });
        }
    }

    // Strategy 2: Multi-strategy matcher (FTS, fuzzy, synonym, keyword)
    if actions.is_empty() {
        let matcher = MultiStrategyMatcher::new();
        let matches = matcher.search_all_assets(&search_query);
        for m in &matches {
            actions.push(AgentAction {
                action_type: "suggest_load".to_string(),
                command: "load_asset".to_string(),
                args: vec![m.path.clone(), format!("{}_score:{:.2}", m.strategy, m.score)],
            });
        }
    }

    let result_msg = if actions.is_empty() {
        format!("No assets found for '{}' via any strategy.", search_query)
    } else {
        format!("Found {} asset(s) for '{}' (strategies: {})",
            actions.len(),
            search_query,
            if !semantic_results.is_empty() { "semantic" } else { "fts/fuzzy/synonym/keyword" })
    };

    AgentResponse {
        success: true,
        result: Some(result_msg),
        error: None,
        actions,
    }
}
