use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MatchedAsset {
    pub path: String,
    pub name: String,
    pub score: f32,
    pub strategy: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
}

pub struct MultiStrategyMatcher {
    synonym_map: HashMap<&'static str, Vec<&'static str>>,
}

impl Default for MultiStrategyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiStrategyMatcher {
    pub fn new() -> Self {
        let mut map: HashMap<&'static str, Vec<&'static str>> = HashMap::new();
        map.insert("genesis", vec!["figure", "character", "model", "avatar", "base"]);
        map.insert("figure", vec!["genesis", "character", "model", "avatar", "humanoid"]);
        map.insert("character", vec!["figure", "genesis", "model", "person", "humanoid"]);
        map.insert("model", vec!["figure", "genesis", "character", "mesh"]);
        map.insert("outfit", vec!["clothing", "attire", "dress", "garment", "costume", "wear"]);
        map.insert("clothing", vec!["outfit", "attire", "garment", "costume", "wear"]);
        map.insert("dress", vec!["outfit", "clothing", "gown", "frock"]);
        map.insert("hair", vec!["hairstyle", "coiffure", "hairdo", "locks"]);
        map.insert("pose", vec!["posture", "stance", "position", "attitude"]);
        map.insert("morph", vec!["shape", "form", "deformation", "adjust", "modify"]);
        map.insert("light", vec!["illumination", "lighting", "lamp", "luminary"]);
        map.insert("texture", vec!["material", "shader", "surface", "skin"]);
        map.insert("material", vec!["texture", "shader", "surface"]);
        map.insert("environment", vec!["env", "background", "set", "world", "scene", "location"]);
        map.insert("prop", vec!["accessory", "object", "item", "decoration"]);
        map.insert("weapon", vec!["sword", "gun", "blade", "bow", "staff", "axe"]);
        map.insert("fantasy", vec!["magical", "medieval", "mythical", "enchanted", "elven"]);
        map.insert("sci_fi", vec!["futuristic", "scifi", "cyberpunk", "robot", "tech", "space"]);
        map.insert("casual", vec!["everyday", "relaxed", "informal", "lounge", "comfortable"]);
        map.insert("formal", vec!["elegant", "fancy", "dressy", "suit", "tuxedo", "sophisticated"]);
        map.insert("female", vec!["woman", "girl", "feminine", "lady"]);
        map.insert("male", vec!["man", "guy", "masculine", "gentleman"]);
        map.insert("baby", vec!["infant", "toddler", "child", "kid", "young"]);
        map.insert("sword", vec!["blade", "weapon", "scimitar", "longsword"]);
        map.insert("gun", vec!["pistol", "rifle", "weapon", "blaster", "firearm"]);
        map.insert("chair", vec!["seat", "throne", "bench", "stool"]);
        map.insert("table", vec!["desk", "counter", "stand"]);
        map.insert("bed", vec!["cot", "bunk", "mattress"]);
        map.insert("animal", vec!["creature", "beast", "pet", "monster"]);
        map.insert("car", vec!["vehicle", "auto", "automobile", "truck"]);
        map.insert("building", vec!["house", "structure", "architecture", "castle", "tower"]);
        map.insert("interior", vec!["indoor", "room", "inside", "house"]);
        map.insert("exterior", vec!["outdoor", "outside", "landscape", "nature"]);
        map.insert("water", vec!["ocean", "sea", "lake", "river", "pool", "liquid"]);
        map.insert("fire", vec!["flame", "burning", "blaze", "inferno"]);
        map.insert("jewelry", vec!["jewellery", "necklace", "ring", "bracelet", "gem", "crown"]);
        map.insert("shoe", vec!["shoes", "boot", "sandal", "footwear", "heel"]);
        Self { synonym_map: map }
    }

    fn normalize_word(word: &str) -> String {
        word.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect()
    }

    pub fn expand_with_synonyms(&self, query: &str) -> Vec<String> {
        let words: Vec<String> = query.split_whitespace().map(|w| Self::normalize_word(w)).filter(|w| !w.is_empty()).collect();
        if words.is_empty() {
            return vec![];
        }
        let expansions: Vec<Vec<String>> = words.iter().map(|word| {
            if let Some(syns) = self.synonym_map.get(word.as_str()) {
                let mut options = vec![word.clone()];
                options.extend(syns.iter().map(|s| s.to_string()));
                options
            } else {
                vec![word.clone()]
            }
        }).collect();

        let mut results = Vec::new();
        let mut stack: Vec<Vec<String>> = vec![vec![]];
        while let Some(prefix) = stack.pop() {
            let depth = prefix.len();
            if depth == expansions.len() {
                results.push(prefix.join(" "));
            } else {
                for option in &expansions[depth] {
                    let mut next = prefix.clone();
                    next.push(option.clone());
                    stack.push(next);
                }
            }
        }
        results
    }

    fn score_fuzzy(asset_name: &str, query_words: &[String]) -> f32 {
        let name_lower = asset_name.to_lowercase();
        let mut matched = 0;
        let mut partial = 0;
        for qw in query_words {
            if name_lower.contains(qw) {
                matched += 1;
            } else {
                if name_lower.contains(&qw[..qw.len().min(3)]) {
                    partial += 1;
                }
            }
        }
        let total = query_words.len() as f32;
        if total == 0.0 { return 0.0; }
        let exact_ratio = matched as f32 / total;
        let partial_ratio = partial as f32 / total;
        exact_ratio + partial_ratio * 0.4
    }

    pub fn search_all_assets(&self, query: &str) -> Vec<MatchedAsset> {
        // Vision descriptions are the richest signal — try them first
        let mut results = self.description_search(query);
        if results.is_empty() {
            results = self.fts_search(query);
        }
        if results.is_empty() {
            results = self.fuzzy_search(query);
        }
        if results.is_empty() {
            results = self.synonym_search(query);
        }
        if results.is_empty() {
            results = self.keyword_search(query);
        }
        results
    }

    fn description_search(&self, query: &str) -> Vec<MatchedAsset> {
        let conn = match Self::open_db_conn() {
            Some(c) => c,
            None => return vec![],
        };
        let keywords: Vec<String> = query.split_whitespace()
            .map(|w| w.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect())
            .filter(|w: &String| !w.is_empty())
            .collect();
        if keywords.is_empty() {
            return vec![];
        }
        let mut all_results: Vec<MatchedAsset> = Vec::new();
        for kw in &keywords {
            let sql = "SELECT asset_path, asset_name, category, tags FROM user_assets WHERE user_id='default' AND LOWER(visual_description) LIKE ?1 LIMIT 20";
            let pattern = format!("%{}%", kw);
            if let Ok(mut stmt) = conn.prepare(sql) {
                if let Ok(mut rows) = stmt.query(rusqlite::params![pattern]) {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                if let (Ok(path), Ok(name)) = (row.get::<_, String>(0), row.get::<_, String>(1)) {
                                    let category: Option<String> = row.get(2).ok().flatten();
                                    let tags_str: Option<String> = row.get(3).ok().flatten();
                                    let tags: Vec<String> = tags_str.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
                                    all_results.push(MatchedAsset { path, name, score: 0.85, strategy: "vision_desc".into(), category, tags });
                                }
                            }
                            Ok(None) => break,
                            Err(_) => break,
                        }
                    }
                }
            }
        }
        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        all_results.dedup_by_key(|a| a.path.clone());
        all_results.truncate(10);
        all_results
    }

    pub fn search_best(&self, query: &str) -> Option<MatchedAsset> {
        self.search_all_assets(query).into_iter().next()
    }

    fn open_db_conn() -> Option<rusqlite::Connection> {
        let guard = crate::database::get_db().ok()?;
        let db = guard.as_ref()?;
        rusqlite::Connection::open(db.path()).ok()
    }

    fn fts_search(&self, query: &str) -> Vec<MatchedAsset> {
        let conn = match Self::open_db_conn() {
            Some(c) => c,
            None => return vec![],
        };
        let fts = crate::format_fts_query(query);
        if fts.is_empty() {
            return vec![];
        }
        let sql = "SELECT user_assets.asset_path, user_assets.asset_name, user_assets.category, user_assets.tags, bm25(user_assets_fts) as score FROM user_assets JOIN user_assets_fts ON user_assets.id = user_assets_fts.rowid WHERE user_assets.user_id='default' AND user_assets_fts MATCH ? ORDER BY bm25(user_assets_fts) LIMIT 10";
        let mut stmt = match conn.prepare(sql) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        let mut rows = match stmt.query(rusqlite::params![fts]) {
            Ok(r) => r,
            Err(_) => return vec![],
        };
        let mut results: Vec<MatchedAsset> = Vec::new();
        loop {
            match rows.next() {
                Ok(Some(row)) => {
                    let path: String = match row.get(0) { Ok(p) => p, Err(_) => continue };
                    let name: String = match row.get(1) { Ok(n) => n, Err(_) => continue };
                    let category: Option<String> = row.get(2).ok().flatten();
                    let tags_str: Option<String> = row.get(3).ok().flatten();
                    let bm25_score: f64 = match row.get(4) { Ok(s) => s, Err(_) => continue };
                    let tags: Vec<String> = tags_str.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
                    results.push(MatchedAsset {
                        path, name,
                        score: (1.0 / (1.0 + bm25_score as f32)).clamp(0.0, 1.0),
                        strategy: "fts".into(),
                        category, tags,
                    });
                }
                Ok(None) => break,
                Err(_) => break,
            }
        }
        results
    }

    fn fuzzy_search(&self, query: &str) -> Vec<MatchedAsset> {
        let conn = match Self::open_db_conn() {
            Some(c) => c,
            None => return vec![],
        };
        let all_sql = "SELECT asset_path, asset_name, category, tags FROM user_assets WHERE user_id='default' LIMIT 500";
        let mut stmt = match conn.prepare(all_sql) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        let mut rows = match stmt.query(rusqlite::params![]) {
            Ok(r) => r,
            Err(_) => return vec![],
        };
        let query_words: Vec<String> = query.split_whitespace()
            .map(|w| w.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect())
            .filter(|w: &String| !w.is_empty())
            .collect();
        if query_words.is_empty() {
            return vec![];
        }
        let mut results: Vec<MatchedAsset> = Vec::new();
        loop {
            match rows.next() {
                Ok(Some(row)) => {
                    let path: String = match row.get::<_, String>(0) { Ok(p) => p, Err(_) => continue };
                    let name: String = match row.get::<_, String>(1) { Ok(n) => n, Err(_) => continue };
                    let category: Option<String> = row.get(2).ok().flatten();
                    let tags_str: Option<String> = row.get(3).ok().flatten();
                    let tags: Vec<String> = tags_str.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
                    let score = Self::score_fuzzy(&name, &query_words);
                    if score > 0.0 {
                        results.push(MatchedAsset { path, name, score, strategy: "fuzzy".into(), category, tags });
                    }
                }
                Ok(None) => break,
                Err(_) => break,
            }
        }
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(10);
        results
    }

    fn synonym_search(&self, query: &str) -> Vec<MatchedAsset> {
        let expansions = self.expand_with_synonyms(query);
        if expansions.is_empty() {
            return vec![];
        }
        let conn = match Self::open_db_conn() {
            Some(c) => c,
            None => return vec![],
        };
        let mut all_results: Vec<MatchedAsset> = Vec::new();
        for expanded in &expansions {
            let fts = crate::format_fts_query(expanded);
            if fts.is_empty() { continue; }
            let sql = "SELECT user_assets.asset_path, user_assets.asset_name, user_assets.category, user_assets.tags FROM user_assets JOIN user_assets_fts ON user_assets.id = user_assets_fts.rowid WHERE user_assets.user_id='default' AND user_assets_fts MATCH ? ORDER BY bm25(user_assets_fts) LIMIT 5";
            if let Ok(mut stmt) = conn.prepare(sql) {
                if let Ok(mut rows) = stmt.query(rusqlite::params![fts]) {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                if let (Ok(path), Ok(name)) = (row.get::<_, String>(0), row.get::<_, String>(1)) {
                                    let category: Option<String> = row.get(2).ok().flatten();
                                    let tags_str: Option<String> = row.get(3).ok().flatten();
                                    let tags: Vec<String> = tags_str.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
                                    all_results.push(MatchedAsset { path, name, score: 0.7, strategy: "synonym".into(), category, tags });
                                }
                            }
                            Ok(None) => break,
                            Err(_) => break,
                        }
                    }
                }
            }
        }
        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        all_results.dedup_by_key(|a| a.path.clone());
        all_results.truncate(5);
        all_results
    }

    fn keyword_search(&self, query: &str) -> Vec<MatchedAsset> {
        let conn = match Self::open_db_conn() {
            Some(c) => c,
            None => return vec![],
        };
        let query_lower = query.to_lowercase();
        let keywords: Vec<String> = query_lower.split_whitespace()
            .map(|w| w.chars().filter(|c| c.is_alphanumeric()).collect())
            .filter(|w: &String| !w.is_empty())
            .collect();
        if keywords.is_empty() {
            return vec![];
        }
        let mut results: Vec<MatchedAsset> = Vec::new();
        for kw in &keywords {
            let sql = "SELECT asset_path, asset_name, category, tags FROM user_assets WHERE user_id='default' AND (LOWER(asset_name) LIKE ?1 OR LOWER(tags) LIKE ?1) LIMIT 10";
            let pattern = format!("%{}%", kw);
            if let Ok(mut stmt) = conn.prepare(sql) {
                if let Ok(mut rows) = stmt.query(rusqlite::params![pattern]) {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                if let (Ok(path), Ok(name)) = (row.get::<_, String>(0), row.get::<_, String>(1)) {
                                    let category: Option<String> = row.get(2).ok().flatten();
                                    let tags_str: Option<String> = row.get(3).ok().flatten();
                                    let tags: Vec<String> = tags_str.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
                                    results.push(MatchedAsset { path, name, score: 0.5, strategy: "keyword".into(), category, tags });
                                }
                            }
                            Ok(None) => break,
                            Err(_) => break,
                        }
                    }
                }
            }
        }
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.dedup_by_key(|a| a.path.clone());
        results.truncate(10);
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synonym_map_contains_common_terms() {
        let matcher = MultiStrategyMatcher::new();
        assert!(matcher.synonym_map.contains_key("outfit"));
        assert!(matcher.synonym_map.contains_key("female"));
        assert!(matcher.synonym_map.contains_key("fantasy"));
        assert!(matcher.synonym_map.contains_key("pose"));
    }

    #[test]
    fn test_expand_with_synonyms_single_word() {
        let matcher = MultiStrategyMatcher::new();
        let results = matcher.expand_with_synonyms("outfit");
        assert!(results.len() > 1);
        assert!(results.iter().any(|r| r.contains("outfit")));
        assert!(results.iter().any(|r| r.contains("clothing")));
    }

    #[test]
    fn test_expand_with_synonyms_multi_word() {
        let matcher = MultiStrategyMatcher::new();
        let results = matcher.expand_with_synonyms("female outfit");
        assert!(results.len() > 1);
        assert!(results.iter().any(|r| r.contains("female") && r.contains("outfit")));
    }

    #[test]
    fn test_expand_with_synonyms_unknown_word() {
        let matcher = MultiStrategyMatcher::new();
        let results = matcher.expand_with_synonyms("zyxwv");
        assert_eq!(results, vec!["zyxwv"]);
    }

    #[test]
    fn test_score_fuzzy_exact_match() {
        let score = MultiStrategyMatcher::score_fuzzy("Summer Dress", &["summer".to_string(), "dress".to_string()]);
        assert!(score > 0.9);
    }

    #[test]
    fn test_score_fuzzy_partial_match() {
        let score = MultiStrategyMatcher::score_fuzzy("Sunlight Maxi Dress", &["summer".to_string(), "dress".to_string()]);
        assert!(score > 0.0);
        assert!(score < 1.0);
    }

    #[test]
    fn test_score_fuzzy_no_match() {
        let score = MultiStrategyMatcher::score_fuzzy("Rustic Armor", &["flower".to_string(), "garden".to_string()]);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_normalize_word() {
        assert_eq!(MultiStrategyMatcher::normalize_word("Hello!"), "hello");
        assert_eq!(MultiStrategyMatcher::normalize_word("  foo-bar  "), "foobar");
    }
}
