use dazpilot_lib::database;
use serial_test::serial;

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
    let fts_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM user_assets_fts WHERE user_assets_fts MATCH 'Genesis'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        fts_count, 1,
        "FTS5 synchronized trigger should automatically index the asset."
    );
}

#[test]
fn test_prefix_query_formatter() {
    use dazpilot_lib::format_fts_query;

    let formatted = format_fts_query("Genesis 8 Female");
    assert_eq!(
        formatted, "\"Genesis*\" AND \"8*\" AND \"Female*\"",
        "FTS query formatter should append wildcard to word boundaries."
    );
}

#[test]
#[serial]
fn test_scene_preset_persistence_round_trip() {
    let temp_dir = tempfile::tempdir().unwrap();
    database::init_database(temp_dir.path()).unwrap();

    let preset = database::DbScenePreset {
        id: "portrait".to_string(),
        name: "Portrait Setup".to_string(),
        description: "Camera and light".to_string(),
        category: "scene".to_string(),
        thumbnail: None,
        scene_data: serde_json::json!({
            "figures": [],
            "props": [],
            "lights": [],
            "cameras": [{ "id": "camera-1", "name": "Portrait Camera" }],
            "activeCamera": "camera-1",
            "selectedItem": "camera-1"
        }),
        created_at: 100,
        updated_at: 200,
    };

    database::save_scene_preset(&preset).unwrap();
    let loaded = database::load_scene_presets().unwrap();

    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].name, "Portrait Setup");
    assert_eq!(loaded[0].scene_data["activeCamera"], "camera-1");

    database::delete_scene_preset(&preset.id).unwrap();
    assert!(database::load_scene_presets().unwrap().is_empty());
}
