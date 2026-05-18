use dazpilot_lib::database;

#[test]
fn test_database_fts_integration() {
    // 1. Create a clean temp directory for testing
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_dazpilot.db");
    
    // 2. Initialize SqliteDatabase
    let db = database::SqliteDatabase::new(&db_path).unwrap();
    db.initialize().unwrap();
    
    // 3. Open standard connection for assert checks
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    
    // Test direct trigger insertion and prefix synchronization
    conn.execute(
        "INSERT INTO user_assets (user_id, asset_path, asset_name, file_type, file_size, category, subcategory)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            "default",
            "C:/DazLibrary/People/Genesis 8 Female/Genesis 8 Female.duf",
            "Genesis 8 Female",
            "figure",
            1024,
            "figures",
            "Genesis 8"
        ],
    ).unwrap();
    
    // 4. Verify FTS entry was created automatically by trigger
    let fts_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM user_assets_fts WHERE user_assets_fts MATCH 'Genesis'",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(fts_count, 1, "FTS5 synchronized trigger should automatically index the asset.");
}

#[test]
fn test_prefix_query_formatter() {
    use dazpilot_lib::format_fts_query;
    
    let formatted = format_fts_query("Genesis 8 Female");
    assert_eq!(formatted, "\"Genesis*\" AND \"8*\" AND \"Female*\"", "FTS query formatter should append wildcard to word boundaries.");
}
