use crate::database;
use crate::library_scanner::AssetInfo;
use serde_json;

/// Query user_assets WHERE compatibility JSON contains the given figure_id.
/// Uses SQLite LIKE to match JSON array elements, e.g. '%"Genesis 9"%'
pub fn resolve_compatible_assets(figure_id: &str, category: Option<&str>) -> Vec<AssetInfo> {
    let guard = match database::get_db() {
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

    let pattern = format!("%\"{}\"%", figure_id);
    let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match category {
        Some(cat) => (
            "SELECT asset_path, asset_name, file_type, file_size, category, subcategory, thumbnail_path, compatibility, dforce_enabled, asset_type_detail, tags, vendor FROM user_assets WHERE user_id='default' AND compatibility LIKE ?1 AND category=?2 ORDER BY asset_name LIMIT 200".to_string(),
            vec![Box::new(pattern) as Box<dyn rusqlite::types::ToSql>, Box::new(cat.to_string())],
        ),
        None => (
            "SELECT asset_path, asset_name, file_type, file_size, category, subcategory, thumbnail_path, compatibility, dforce_enabled, asset_type_detail, tags, vendor FROM user_assets WHERE user_id='default' AND compatibility LIKE ?1 ORDER BY asset_name LIMIT 200".to_string(),
            vec![Box::new(pattern) as Box<dyn rusqlite::types::ToSql>],
        ),
    };

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = match stmt.query_map(params_refs.as_slice(), map_asset_row) {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    rows.filter_map(|r| r.ok()).collect()
}

pub fn get_figure_morphs(figure_id: &str) -> Vec<AssetInfo> {
    resolve_compatible_assets(figure_id, Some("morphs"))
}

pub fn get_figure_expressions(figure_id: &str) -> Vec<AssetInfo> {
    // Expressions are morphs with "expression" in name or subcategory
    let all_morphs = resolve_compatible_assets(figure_id, Some("morphs"));
    all_morphs
        .into_iter()
        .filter(|a| {
            let lower = a.name.to_lowercase();
            lower.contains("expression") || lower.contains("facial") || lower.contains("emotion")
        })
        .collect()
}

pub fn get_figure_outfits(figure_id: &str) -> Vec<AssetInfo> {
    resolve_compatible_assets(figure_id, Some("clothing"))
}

pub fn list_known_figures() -> Vec<String> {
    let guard = match database::get_db() {
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

    // Get unique figure names from user_assets where category='figures'
    let mut stmt = match conn.prepare("SELECT DISTINCT asset_name FROM user_assets WHERE user_id='default' AND category='figures' ORDER BY asset_name") {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let rows = match stmt.query_map([], |row| row.get::<_, String>(0)) {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    rows.filter_map(|r| r.ok()).collect()
}

fn map_asset_row(row: &rusqlite::Row) -> rusqlite::Result<AssetInfo> {
    let compatibility_str: Option<String> = row.get(7).ok().flatten();
    let compatibility_base: Vec<String> = compatibility_str
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let tags_str: Option<String> = row.get(10).ok().flatten();
    let tags: Vec<String> = tags_str
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    Ok(AssetInfo {
        path: row.get(0)?,
        name: row.get(1)?,
        file_type: row.get::<_, String>(2).unwrap_or_default(),
        size: row.get::<_, i64>(3).unwrap_or(0) as u64,
        category: row.get::<_, String>(4).unwrap_or_default(),
        subcategory: row.get(5)?,
        metadata: std::collections::HashMap::new(),
        thumbnail_path: row.get(6).ok().flatten(),
        compatibility_base,
        dforce_enabled: row.get::<_, i32>(8).unwrap_or(0) != 0,
        asset_type_detail: row.get(9).ok().flatten(),
        tags,
        vendor: row.get(11).ok().flatten(),
        visual_properties: None,
        visual_description: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_known_figures_no_panic() {
        // Just verify no crash when DB is not initialized
        let figures = list_known_figures();
        // DB might not be initialized, so it's ok if empty
        assert!(figures.is_empty());
    }

    #[test]
    fn test_resolve_compatible_assets_no_db() {
        let assets = resolve_compatible_assets("Genesis 9", None);
        assert!(assets.is_empty());
    }
}
