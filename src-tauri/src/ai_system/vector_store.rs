use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingQuery {
    pub query: String,
    pub top_k: usize,
}

#[derive(Debug, Deserialize)]
struct OllamaEmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

// ── In-memory caches ──────────────────────────────────────────────────────────
// Cache for query -> embedding results (queries are normalized to lowercase)
static QUERY_EMBEDDING_CACHE: Lazy<Mutex<HashMap<String, Vec<f32>>>> =
    Lazy::new(|| Mutex::new(HashMap::with_capacity(256)));

// Cache for all stored asset embeddings (path -> embedding), invalidated on write
static ASSET_EMBEDDING_CACHE: Lazy<Mutex<Option<Vec<(String, Vec<f32>)>>>> =
    Lazy::new(|| Mutex::new(None));

const EMBEDDING_CACHE_MAX: usize = 256;

fn get_ollama_host() -> String {
    crate::database::get_setting("ollama_host")
        .ok()
        .flatten()
        .filter(|h| !h.is_empty())
        .unwrap_or_else(|| "http://localhost:11434".to_string())
}

fn get_embed_model() -> String {
    crate::database::get_setting("ollama_embed_model")
        .ok()
        .flatten()
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| "nomic-embed-text".to_string())
}

/// Generate embedding with caching: returns cached value if this exact query was
/// seen recently, otherwise calls Ollama and stores the result.
async fn generate_embedding(text: &str) -> Option<Vec<f32>> {
    let key = text.to_lowercase();

    // Fast path: check cache first
    {
        let cache = QUERY_EMBEDDING_CACHE.lock().unwrap();
        if let Some(emb) = cache.get(&key) {
            return Some(emb.clone());
        }
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .ok()?;
    let url = format!("{}/api/embed", get_ollama_host());
    let body = serde_json::json!({
        "model": get_embed_model(),
        "input": [text],
    });
    let resp = client.post(&url).json(&body).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let data: OllamaEmbedResponse = resp.json().await.ok()?;
    let embedding = data.embeddings.into_iter().next()?;

    // Store in cache (evict oldest if full)
    {
        let mut cache = QUERY_EMBEDDING_CACHE.lock().unwrap();
        if cache.len() >= EMBEDDING_CACHE_MAX {
            // Simple eviction: remove a random entry (first one)
            if let Some(key) = cache.keys().next().cloned() {
                cache.remove(&key);
            }
        }
        cache.insert(key, embedding.clone());
    }

    Some(embedding)
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

fn ensure_embeddings_table(conn: &rusqlite::Connection) {
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS asset_embeddings (
            asset_path TEXT PRIMARY KEY,
            embedding BLOB NOT NULL,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        );",
    );
}

fn read_embeddings_from_db() -> Vec<(String, Vec<f32>)> {
    // Check cache first
    {
        let cache = ASSET_EMBEDDING_CACHE.lock().unwrap();
        if let Some(ref cached) = *cache {
            return cached.clone();
        }
    }

    let guard = match crate::database::get_db() {
        Ok(g) => g,
        Err(_) => return vec![],
    };
    let db = match guard.as_ref() {
        Some(d) => d,
        None => return vec![],
    };
    let conn = match rusqlite::Connection::open(db.path()) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let sql = "SELECT asset_path, embedding FROM asset_embeddings";
    let mut stmt = match conn.prepare(sql) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let mut results = Vec::new();
    if let Ok(rows) = stmt.query_map(rusqlite::params![], |row| {
        let path: String = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        let emb: Vec<f32> = blob
            .chunks(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        Ok((path, emb))
    }) {
        for row in rows.flatten() {
            results.push(row);
        }
    }

    // Populate cache
    {
        let mut cache = ASSET_EMBEDDING_CACHE.lock().unwrap();
        *cache = Some(results.clone());
    }

    results
}

/// Invalidate the in-memory asset embedding cache (call when new embeddings are written)
pub fn invalidate_embedding_cache() {
    let mut cache = ASSET_EMBEDDING_CACHE.lock().unwrap();
    *cache = None;
}

pub async fn embed_all_assets() -> usize {
    let rows: Vec<(String, String, String, Option<String>)> = {
        let guard = match crate::database::get_db() {
            Ok(g) => g,
            Err(_) => return 0,
        };
        let db = match guard.as_ref() {
            Some(d) => d,
            None => return 0,
        };
        let conn = match rusqlite::Connection::open(db.path()) {
            Ok(c) => c,
            Err(_) => return 0,
        };
        ensure_embeddings_table(&conn);

        let sql = "SELECT asset_path, asset_name, category, tags, visual_description FROM user_assets WHERE user_id='default'";
        let mut stmt = match conn.prepare(sql) {
            Ok(s) => s,
            Err(_) => return 0,
        };
        stmt.query_map(rusqlite::params![], |row| {
            let path: String = row.get(0)?;
            let name: String = row.get(1)?;
            let tags: String = row
                .get::<_, Option<String>>(3)
                .unwrap_or_default()
                .unwrap_or_default();
            let desc: Option<String> = row.get(4).ok().flatten();
            Ok((path, name, tags, desc))
        })
        .ok()
        .map(|r| r.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
    };

    let mut count = 0;
    let mut new_embeddings: Vec<(String, Vec<u8>)> = Vec::new();
    for (path, name, tags, desc) in &rows {
        let already: bool = {
            let guard = match crate::database::get_db() {
                Ok(g) => g,
                Err(_) => return count,
            };
            let db = match guard.as_ref() {
                Some(d) => d,
                None => return count,
            };
            let conn = match rusqlite::Connection::open(db.path()) {
                Ok(c) => c,
                Err(_) => return 0,
            };
            conn.query_row(
                "SELECT 1 FROM asset_embeddings WHERE asset_path=?1",
                rusqlite::params![path],
                |_| Ok(()),
            )
            .is_ok()
        };
        if already {
            continue;
        }
        let text = desc.as_ref().map(|d| d.as_str()).unwrap_or(name);
        let text = format!("{} {} {}", text, name, tags);
        if let Some(embedding) = generate_embedding(&text).await {
            let blob: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
            new_embeddings.push((path.clone(), blob));
            count += 1;
        }
    }

    // Batch insert all new embeddings in a single transaction
    if !new_embeddings.is_empty() {
        let guard = match crate::database::get_db() {
            Ok(g) => g,
            Err(_) => return count,
        };
        let db = match guard.as_ref() {
            Some(d) => d,
            None => return count,
        };
        if let Ok(conn) = rusqlite::Connection::open(db.path()) {
            let _ = conn.execute_batch("BEGIN TRANSACTION");
            for (path, blob) in &new_embeddings {
                let _ = conn.execute(
                    "INSERT OR REPLACE INTO asset_embeddings (asset_path, embedding) VALUES (?1, ?2)",
                    rusqlite::params![path, blob],
                );
            }
            let _ = conn.execute_batch("COMMIT");
        }
        invalidate_embedding_cache();
    }
    count
}

pub async fn get_semantic_matches_async(query: &str) -> Vec<(String, f32)> {
    let query_embedding = match generate_embedding(query).await {
        Some(e) => e,
        None => return vec![],
    };
    let stored = read_embeddings_from_db();
    if stored.is_empty() {
        return vec![];
    }
    let mut scored: Vec<(String, f32)> = stored
        .into_iter()
        .map(|(path, emb)| {
            let sim = cosine_similarity(&query_embedding, &emb);
            (path, sim)
        })
        .filter(|(_, sim)| *sim > 0.5)
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(10);
    scored
}

pub fn get_semantic_matches(query: &str) -> Vec<(String, f32)> {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            tokio::task::block_in_place(|| handle.block_on(get_semantic_matches_async(query)))
        },
        Err(_) => {
            if let Ok(rt) = tokio::runtime::Runtime::new() {
                rt.block_on(get_semantic_matches_async(query))
            } else {
                vec![]
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_empty() {
        assert_eq!(cosine_similarity(&[], &[]), 0.0);
        assert_eq!(cosine_similarity(&[1.0], &[]), 0.0);
    }

    #[test]
    fn test_cosine_similarity_mismatched_len() {
        assert_eq!(cosine_similarity(&[1.0, 0.0], &[1.0]), 0.0);
    }
}
