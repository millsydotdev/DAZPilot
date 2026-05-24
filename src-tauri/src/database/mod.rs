

pub mod schema;

use std::sync::Mutex;
use once_cell::sync::Lazy;

pub static DATABASE: Lazy<Mutex<Option<SqliteDatabase>>> = Lazy::new(|| Mutex::new(None));

pub fn init_database(app_data_dir: &std::path::Path) -> Result<(), String> {
    let db_path = app_data_dir.join("dazpilot.db");
    
    let db = SqliteDatabase::new(&db_path)
        .map_err(|e| format!("Failed to create database: {}", e))?;
    
    db.initialize()
        .map_err(|e| format!("Failed to initialize database: {}", e))?;
    
    let mut guard = DATABASE.lock().map_err(|e| e.to_string())?;
    *guard = Some(db);
    
    Ok(())
}

pub fn get_db() -> Result<std::sync::MutexGuard<'static, Option<SqliteDatabase>>, String> {
    DATABASE.lock().map_err(|e| e.to_string())
}

pub struct SqliteDatabase {
    path: std::path::PathBuf,
}

impl SqliteDatabase {
    pub fn new(path: &std::path::Path) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            path: path.to_path_buf(),
        })
    }
    
    pub fn initialize(&self) -> Result<(), rusqlite::Error> {
        let conn = rusqlite::Connection::open(&self.path)?;
        
        if let Err(e) = conn.execute_batch(r#"
            -- Core tables
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                last_login TEXT
            );
            
            CREATE TABLE IF NOT EXISTS content_sources (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                source_path TEXT NOT NULL,
                source_type TEXT NOT NULL,
                source_name TEXT,
                last_scanned TEXT,
                asset_count INTEGER DEFAULT 0
            );
            
            CREATE TABLE IF NOT EXISTS user_assets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                source_id INTEGER,
                asset_path TEXT NOT NULL UNIQUE,
                asset_name TEXT NOT NULL,
                original_name TEXT,
                category TEXT,
                subcategory TEXT,
                vendor TEXT,
                compatible_figures TEXT,
                body_zones TEXT,
                materials TEXT,
                file_type TEXT,
                file_size INTEGER,
                indexed_at TEXT DEFAULT CURRENT_TIMESTAMP,
                thumbnail_path TEXT,
                compatibility TEXT,
                dforce_enabled INTEGER DEFAULT 0,
                asset_type_detail TEXT,
                tags TEXT,
                FOREIGN KEY (source_id) REFERENCES content_sources(id)
            );
        "#) {
            log::warn!("Core schema creation non-fatal: {}", e);
        }

        // Separate migration batch for columns added after initial schema.
        // We run ALTER TABLE separately and ignore "duplicate column" errors because
        // SQLite's bundled version may not support ALTER TABLE ADD COLUMN IF NOT EXISTS.
        let migration_sql = r#"
            ALTER TABLE user_assets ADD COLUMN thumbnail_path TEXT;
        "#;
        if let Err(e) = conn.execute_batch(migration_sql) {
            let err_str = e.to_string();
            // Ignore "duplicate column" errors — column already exists
            if !err_str.contains("duplicate column") {
                log::warn!("Migration (thumbnail_path) non-fatal: {}", err_str);
            }
        }
        let migration_sql = r#"
            ALTER TABLE user_assets ADD COLUMN compatibility TEXT;
        "#;
        if let Err(e) = conn.execute_batch(migration_sql) {
            let err_str = e.to_string();
            if !err_str.contains("duplicate column") {
                log::warn!("Migration (compatibility) non-fatal: {}", err_str);
            }
        }
        let migration_sql = r#"
            ALTER TABLE user_assets ADD COLUMN dforce_enabled INTEGER DEFAULT 0;
        "#;
        if let Err(e) = conn.execute_batch(migration_sql) {
            let err_str = e.to_string();
            if !err_str.contains("duplicate column") {
                log::warn!("Migration (dforce_enabled) non-fatal: {}", err_str);
            }
        }
        let migration_sql = r#"
            ALTER TABLE user_assets ADD COLUMN asset_type_detail TEXT;
        "#;
        if let Err(e) = conn.execute_batch(migration_sql) {
            let err_str = e.to_string();
            if !err_str.contains("duplicate column") {
                log::warn!("Migration (asset_type_detail) non-fatal: {}", err_str);
            }
        }
        let migration_sql = r#"
            ALTER TABLE user_assets ADD COLUMN tags TEXT;
        "#;
        if let Err(e) = conn.execute_batch(migration_sql) {
            let err_str = e.to_string();
            if !err_str.contains("duplicate column") {
                log::warn!("Migration (tags) non-fatal: {}", err_str);
            }
        }

        let migration_sql = r#"
            ALTER TABLE user_assets ADD COLUMN visual_properties TEXT;
        "#;
        if let Err(e) = conn.execute_batch(migration_sql) {
            let err_str = e.to_string();
            if !err_str.contains("duplicate column") {
                log::warn!("Migration (visual_properties) non-fatal: {}", err_str);
            }
        }
        let migration_sql = r#"
            ALTER TABLE user_assets ADD COLUMN visual_description TEXT;
        "#;
        if let Err(e) = conn.execute_batch(migration_sql) {
            let err_str = e.to_string();
            if !err_str.contains("duplicate column") {
                log::warn!("Migration (visual_description) non-fatal: {}", err_str);
            }
        }

        // Continue with the rest of the schema
        conn.execute_batch(r#"

            -- FTS5 Virtual table for ultra-fast, smart text searching of assets
            CREATE VIRTUAL TABLE IF NOT EXISTS user_assets_fts USING fts5(
                asset_name,
                original_name,
                category,
                subcategory,
                vendor,
                asset_path,
                tags,
                asset_type_detail,
                visual_properties,
                visual_description,
                content='user_assets',
                content_rowid='id'
            );

            -- Trigger to automatically index new assets on insert
            CREATE TRIGGER IF NOT EXISTS user_assets_ai AFTER INSERT ON user_assets BEGIN
                INSERT INTO user_assets_fts(rowid, asset_name, original_name, category, subcategory, vendor, asset_path, tags, asset_type_detail, visual_properties, visual_description)
                VALUES (new.id, new.asset_name, new.original_name, new.category, new.subcategory, new.vendor, new.asset_path, new.tags, new.asset_type_detail, new.visual_properties, new.visual_description);
            END;

            -- Trigger to automatically clean FTS index on deletion
            CREATE TRIGGER IF NOT EXISTS user_assets_ad AFTER DELETE ON user_assets BEGIN
                INSERT INTO user_assets_fts(user_assets_fts, rowid, asset_name, original_name, category, subcategory, vendor, asset_path, tags, asset_type_detail, visual_properties, visual_description)
                VALUES('delete', old.id, old.asset_name, old.original_name, old.category, old.subcategory, old.vendor, old.asset_path, old.tags, old.asset_type_detail, old.visual_properties, old.visual_description);
            END;

            -- Trigger to automatically update FTS index on field updates
            CREATE TRIGGER IF NOT EXISTS user_assets_au AFTER UPDATE ON user_assets BEGIN
                INSERT INTO user_assets_fts(user_assets_fts, rowid, asset_name, original_name, category, subcategory, vendor, asset_path, tags, asset_type_detail, visual_properties, visual_description)
                VALUES('delete', old.id, old.asset_name, old.original_name, old.category, old.subcategory, old.vendor, old.asset_path, old.tags, old.asset_type_detail, old.visual_properties, old.visual_description);
                INSERT INTO user_assets_fts(rowid, asset_name, original_name, category, subcategory, vendor, asset_path, tags, asset_type_detail, visual_properties, visual_description)
                VALUES (new.id, new.asset_name, new.original_name, new.category, new.subcategory, new.vendor, new.asset_path, new.tags, new.asset_type_detail, new.visual_properties, new.visual_description);
            END;

            -- One-time sync migration to populate index for pre-existing assets
            INSERT OR REPLACE INTO user_assets_fts(rowid, asset_name, original_name, category, subcategory, vendor, asset_path, tags, asset_type_detail, visual_properties, visual_description)
            SELECT id, asset_name, original_name, category, subcategory, vendor, asset_path, tags, asset_type_detail, visual_properties, visual_description FROM user_assets;

            CREATE TABLE IF NOT EXISTS sdk_classes (
                name TEXT PRIMARY KEY,
                file TEXT NOT NULL,
                line INTEGER NOT NULL,
                description TEXT,
                parents TEXT,
                related_classes TEXT,
                indexed_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS sdk_methods (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                class_name TEXT NOT NULL,
                name TEXT NOT NULL,
                return_type TEXT,
                parameters TEXT,
                description TEXT,
                access TEXT,
                line INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sdk_enums (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                class_name TEXT NOT NULL,
                name TEXT NOT NULL,
                values_json TEXT,
                line INTEGER NOT NULL
            );
            
            -- Permission tables
            CREATE TABLE IF NOT EXISTS permissions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                category TEXT NOT NULL,
                default_state TEXT DEFAULT 'prompt',
                requires_prompt INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS user_roles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                is_admin INTEGER DEFAULT 0,
                inherit_from TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS role_permissions (
                role_id TEXT NOT NULL,
                permission_id TEXT NOT NULL,
                state TEXT NOT NULL,
                conditions TEXT,
                PRIMARY KEY (role_id, permission_id)
            );
            
            CREATE TABLE IF NOT EXISTS user_role_assignments (
                user_id TEXT NOT NULL,
                role_id TEXT NOT NULL,
                assigned_at TEXT DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (user_id, role_id)
            );
            
            -- AI decision tracking
            CREATE TABLE IF NOT EXISTS ai_decisions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                session_id TEXT,
                category TEXT,
                decision_type TEXT,
                decision_data TEXT,
                user_response TEXT,
                confidence REAL,
                timestamp TEXT DEFAULT CURRENT_TIMESTAMP
            );
            
            -- Workflow chains
            CREATE TABLE IF NOT EXISTS workflow_chains (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                chain_hash TEXT NOT NULL,
                steps TEXT NOT NULL,
                success_count INTEGER DEFAULT 0,
                total_count INTEGER DEFAULT 0,
                last_used TEXT DEFAULT CURRENT_TIMESTAMP
            );
            
            -- Accuracy metrics
            CREATE TABLE IF NOT EXISTS accuracy_metrics (
                user_id TEXT NOT NULL,
                category TEXT NOT NULL,
                total INTEGER DEFAULT 0,
                accepted INTEGER DEFAULT 0,
                rejected INTEGER DEFAULT 0,
                modified INTEGER DEFAULT 0,
                success_rate REAL DEFAULT 0.0,
                last_updated TEXT DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (user_id, category)
            );
            
            -- User preferences
            CREATE TABLE IF NOT EXISTS user_preferences (
                user_id TEXT PRIMARY KEY,
                preferred_figure TEXT,
                preferred_category TEXT,
                auto_apply_shells INTEGER DEFAULT 0,
                auto_apply_materials INTEGER DEFAULT 0,
                physics_enabled INTEGER DEFAULT 1,
                default_quality TEXT DEFAULT 'preview',
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            
            -- Initialize default data
            INSERT OR IGNORE INTO permissions (id, name, description, category, default_state, requires_prompt)
            VALUES 
                ('feature.scene.create', 'Create Scene', 'Create new scenes', 'feature', 'granted', 0),
                ('feature.scene.save', 'Save Scene', 'Save scenes to disk', 'feature', 'granted', 0),
                ('feature.library.scan', 'Scan Library', 'Scan content directories', 'feature', 'granted', 0),
                ('feature.library.browse', 'Browse Library', 'Browse and search assets', 'feature', 'granted', 0),
                ('feature.animation.create', 'Create Animation', 'Create animations', 'feature', 'granted', 0),
                ('feature.render.preview', 'Preview Render', 'Preview render', 'feature', 'granted', 0),
                ('feature.render.full', 'Full Render', 'Full quality render', 'feature', 'granted', 0),
                ('ai.auto_apply', 'AI Auto-Apply', 'Allow AI to auto-apply preferences', 'ai', 'prompt', 1),
                ('ai.learn_patterns', 'AI Learning', 'Allow AI to learn patterns', 'ai', 'granted', 0),
                ('ai.view_analytics', 'View Analytics', 'View AI performance analytics', 'ai', 'denied', 0),
                ('daz3d.load_assets', 'Load Assets', 'Load assets in Daz3D', 'daz3d', 'granted', 0),
                ('daz3d.modify_scene', 'Modify Scene', 'Modify Daz3D scenes', 'daz3d', 'granted', 0),
                ('daz3d.execute_scripts', 'Execute Scripts', 'Execute scripts in Daz3D', 'daz3d', 'denied', 0),
                ('system.settings', 'System Settings', 'Access system settings', 'system', 'denied', 0),
                ('system.manage_users', 'Manage Users', 'Manage user accounts', 'system', 'denied', 0),
                ('network.cloud_sync', 'Cloud Sync', 'Sync to cloud', 'network', 'denied', 0),
                ('network.download', 'Download Assets', 'Download from network', 'network', 'denied', 0);
            
            -- Initialize default role
            INSERT OR IGNORE INTO user_roles (id, name, description, is_admin)
            VALUES ('basic', 'Basic User', 'Default user role', 0),
                   ('admin', 'Administrator', 'Full admin access', 1);
            
            -- Assign basic permissions to basic role
            INSERT OR IGNORE INTO role_permissions (role_id, permission_id, state)
            SELECT 'basic', id, default_state FROM permissions 
            WHERE category IN ('feature', 'ai') AND default_state = 'granted';
            
            -- Admin gets all
            INSERT OR IGNORE INTO role_permissions (role_id, permission_id, state)
            SELECT 'admin', id, 'granted' FROM permissions;
            
            -- Create default user if not exists
            INSERT OR IGNORE INTO users (id, username) VALUES ('default', 'Default User');
            
            -- Assign default role to user
            INSERT OR IGNORE INTO user_role_assignments (user_id, role_id)
            VALUES ('default', 'basic');
            
            -- Default preferences
            INSERT OR IGNORE INTO user_preferences (user_id) VALUES ('default');

            -- App Settings Table
            CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Scratchpad persistence
            CREATE TABLE IF NOT EXISTS scratchpad_notes (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL DEFAULT 'default',
                content TEXT NOT NULL,
                tags TEXT DEFAULT '[]',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS scratchpad_todos (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL DEFAULT 'default',
                content TEXT NOT NULL,
                completed INTEGER NOT NULL DEFAULT 0,
                priority TEXT NOT NULL DEFAULT 'medium',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            "#
        )?;
        
        Ok(())
    }
    
    pub fn execute(&self, sql: &str) -> Result<(), rusqlite::Error> {
        let conn = rusqlite::Connection::open(&self.path)?;
        conn.execute_batch(sql)
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
    
    pub fn query<T, F>(&self, sql: &str, mapper: F) -> Result<Vec<T>, rusqlite::Error>
    where
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        let conn = rusqlite::Connection::open(&self.path)?;
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map([], mapper)?;
        rows.collect()
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, rusqlite::Error> {
        let conn = rusqlite::Connection::open(&self.path)?;
        let mut stmt = conn.prepare("SELECT value FROM app_settings WHERE key = ?1")?;
        let mut rows = stmt.query([key])?;
        if let Some(row) = rows.next()? {
            let val: String = row.get(0)?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    pub fn save_setting(&self, key: &str, value: &str) -> Result<(), rusqlite::Error> {
        let conn = rusqlite::Connection::open(&self.path)?;
        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value],
        )?;
        Ok(())
    }
}

pub fn get_setting(key: &str) -> Result<Option<String>, String> {
    let db_guard = get_db()?;
    if let Some(ref db) = *db_guard {
        db.get_setting(key).map_err(|e| e.to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

pub fn save_setting(key: &str, value: &str) -> Result<(), String> {
    let db_guard = get_db()?;
    if let Some(ref db) = *db_guard {
        db.save_setting(key, value).map_err(|e| e.to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

// ── Scratchpad persistence helpers ──────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DbNote {
    pub id: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DbTodo {
    pub id: String,
    pub content: String,
    pub completed: bool,
    pub priority: String,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn load_notes() -> Result<Vec<DbNote>, String> {
    let db_guard = get_db()?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    let conn = rusqlite::Connection::open(db.path()).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, content, tags, created_at, updated_at FROM scratchpad_notes WHERE user_id='default' ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            let tags_json: String = row.get(2)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            Ok(DbNote {
                id: row.get(0)?,
                content: row.get(1)?,
                tags,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn save_note(note: &DbNote) -> Result<(), String> {
    let db_guard = get_db()?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    let conn = rusqlite::Connection::open(db.path()).map_err(|e| e.to_string())?;
    let tags_json = serde_json::to_string(&note.tags).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT OR REPLACE INTO scratchpad_notes (id, user_id, content, tags, created_at, updated_at) VALUES (?1, 'default', ?2, ?3, ?4, ?5)",
        rusqlite::params![note.id, note.content, tags_json, note.created_at, note.updated_at],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_note_db(note_id: &str) -> Result<(), String> {
    let db_guard = get_db()?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    let conn = rusqlite::Connection::open(db.path()).map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM scratchpad_notes WHERE id=?1 AND user_id='default'",
        rusqlite::params![note_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_todos() -> Result<Vec<DbTodo>, String> {
    let db_guard = get_db()?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    let conn = rusqlite::Connection::open(db.path()).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, content, completed, priority, created_at, updated_at FROM scratchpad_todos WHERE user_id='default' ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(DbTodo {
                id: row.get(0)?,
                content: row.get(1)?,
                completed: row.get::<_, i32>(2)? != 0,
                priority: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn save_todo(todo: &DbTodo) -> Result<(), String> {
    let db_guard = get_db()?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    let conn = rusqlite::Connection::open(db.path()).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO scratchpad_todos (id, user_id, content, completed, priority, created_at, updated_at) VALUES (?1, 'default', ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![todo.id, todo.content, todo.completed as i32, todo.priority, todo.created_at, todo.updated_at],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_todo_db(todo_id: &str) -> Result<(), String> {
    let db_guard = get_db()?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    let conn = rusqlite::Connection::open(db.path()).map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM scratchpad_todos WHERE id=?1 AND user_id='default'",
        rusqlite::params![todo_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn clear_completed_todos() -> Result<(), String> {
    let db_guard = get_db()?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    let conn = rusqlite::Connection::open(db.path()).map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM scratchpad_todos WHERE completed=1 AND user_id='default'",
        [],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

