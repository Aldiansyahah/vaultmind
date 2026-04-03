use rusqlite::{Connection, OptionalExtension};

use crate::error::{Result, StorageError};

const CURRENT_SCHEMA_VERSION: u32 = 1;

const MIGRATION_001: &str = "
CREATE TABLE IF NOT EXISTS notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL DEFAULT '',
    content_hash TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS note_tags (
    note_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (note_id, tag_id),
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_notes_path ON notes(path);
CREATE INDEX IF NOT EXISTS idx_notes_title ON notes(title);
CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);
";

fn run_migration_001(conn: &Connection) -> Result<()> {
    conn.execute_batch(MIGRATION_001)
        .map_err(StorageError::from)?;
    Ok(())
}

pub fn init_database(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            version INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )?;

    let current_version: u32 = conn
        .query_row(
            "SELECT version FROM schema_version WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .optional()?
        .unwrap_or(0);

    if current_version >= CURRENT_SCHEMA_VERSION {
        return Ok(());
    }

    run_migrations(conn, current_version)?;

    conn.execute(
        "INSERT OR REPLACE INTO schema_version (id, version) VALUES (1, ?1)",
        [CURRENT_SCHEMA_VERSION],
    )?;

    Ok(())
}

fn run_migrations(conn: &Connection, from_version: u32) -> Result<()> {
    if from_version < 1 {
        run_migration_001(conn)?;
    }

    if from_version >= CURRENT_SCHEMA_VERSION {
        return Err(StorageError::Migration(
            "Migration version mismatch".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_connection() -> Connection {
        Connection::open_in_memory().expect("Failed to create in-memory database")
    }

    #[test]
    fn test_init_database_creates_tables() {
        let conn = create_test_connection();
        let result = init_database(&conn);
        assert!(result.is_ok());

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .expect("Failed to prepare query")
            .query_map([], |row| row.get(0))
            .expect("Failed to query tables")
            .map(|r| r.expect("Failed to get table name"))
            .collect();

        assert!(tables.contains(&"notes".to_string()));
        assert!(tables.contains(&"tags".to_string()));
        assert!(tables.contains(&"note_tags".to_string()));
        assert!(tables.contains(&"schema_version".to_string()));
    }

    #[test]
    fn test_init_database_is_idempotent() {
        let conn = create_test_connection();
        assert!(init_database(&conn).is_ok());
        assert!(init_database(&conn).is_ok());
    }

    #[test]
    fn test_schema_version_is_set() {
        let conn = create_test_connection();
        init_database(&conn).expect("Failed to init database");

        let version: u32 = conn
            .query_row(
                "SELECT version FROM schema_version WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .expect("Failed to get schema version");

        assert_eq!(version, CURRENT_SCHEMA_VERSION);
    }
}
