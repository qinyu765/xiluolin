use std::path::{Path, PathBuf};

use rusqlite::Connection;

pub struct LocalDatabase {
    pub(super) connection: Connection,
    path: PathBuf,
}

impl LocalDatabase {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let connection = Connection::open(&path_buf)?;
        Ok(Self {
            connection,
            path: path_buf,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn initialize(&self) -> rusqlite::Result<()> {
        self.connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS personas (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                icon TEXT NOT NULL DEFAULT 'Sparkles',
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS hotwords (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                category TEXT NOT NULL,
                enabled INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS history_records (
                id TEXT PRIMARY KEY,
                raw_text TEXT NOT NULL,
                final_text TEXT NOT NULL,
                persona_id TEXT NOT NULL,
                persona_name TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                output_chars INTEGER NOT NULL,
                output_mode TEXT NOT NULL,
                source TEXT NOT NULL DEFAULT 'unknown',
                asr_provider TEXT NOT NULL DEFAULT '',
                asr_model TEXT NOT NULL DEFAULT '',
                text_provider TEXT NOT NULL DEFAULT '',
                text_model TEXT NOT NULL DEFAULT '',
                used_asr_fallback INTEGER NOT NULL DEFAULT 0,
                used_fallback INTEGER NOT NULL DEFAULT 0,
                delivery_method TEXT NOT NULL DEFAULT 'pending',
                audio_path TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )?;
        self.ensure_history_record_columns()?;
        self.seed_builtin_personas()?;
        Ok(())
    }

    fn ensure_history_record_columns(&self) -> rusqlite::Result<()> {
        let mut statement = self
            .connection
            .prepare("PRAGMA table_info(history_records)")?;
        let existing = statement
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<rusqlite::Result<std::collections::HashSet<_>>>()?;
        let columns = [
            ("source", "TEXT NOT NULL DEFAULT 'unknown'"),
            ("asr_provider", "TEXT NOT NULL DEFAULT ''"),
            ("asr_model", "TEXT NOT NULL DEFAULT ''"),
            ("text_provider", "TEXT NOT NULL DEFAULT ''"),
            ("text_model", "TEXT NOT NULL DEFAULT ''"),
            ("used_asr_fallback", "INTEGER NOT NULL DEFAULT 0"),
            ("used_fallback", "INTEGER NOT NULL DEFAULT 0"),
            ("delivery_method", "TEXT NOT NULL DEFAULT 'pending'"),
            ("audio_path", "TEXT"),
        ];
        for (name, definition) in columns {
            if !existing.contains(name) {
                self.connection.execute(
                    &format!("ALTER TABLE history_records ADD COLUMN {name} {definition}"),
                    [],
                )?;
            }
        }
        Ok(())
    }

    pub fn table_names(&self) -> rusqlite::Result<Vec<String>> {
        let mut statement = self.connection.prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect()
    }
}
