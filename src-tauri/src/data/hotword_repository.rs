use rusqlite::params;
use uuid::Uuid;

use super::{
    database::LocalDatabase,
    helpers::{bool_to_int, format_hotword_context, int_to_bool},
    models::*,
};

impl LocalDatabase {
    pub fn create_hotword(&self, draft: HotwordDraft) -> rusqlite::Result<Hotword> {
        let id = Uuid::new_v4().to_string();
        self.connection.execute(
            r#"
            INSERT INTO hotwords (id, text, category, enabled)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![id, draft.text, draft.category, bool_to_int(draft.enabled)],
        )?;

        self.get_hotword(&id)
    }

    pub fn list_hotwords(&self) -> rusqlite::Result<Vec<Hotword>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT id, text, category, enabled, created_at, updated_at
            FROM hotwords
            ORDER BY created_at ASC, id ASC
            "#,
        )?;

        let rows = statement.query_map([], hotword_from_row)?;
        rows.collect()
    }

    pub fn update_hotword(&self, id: &str, draft: HotwordDraft) -> rusqlite::Result<Hotword> {
        let updated = self.connection.execute(
            r#"
            UPDATE hotwords
            SET text = ?2,
                category = ?3,
                enabled = ?4,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?1
            "#,
            params![id, draft.text, draft.category, bool_to_int(draft.enabled)],
        )?;
        if updated == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        self.get_hotword(id)
    }

    pub fn delete_hotword(&self, id: &str) -> rusqlite::Result<()> {
        let deleted = self
            .connection
            .execute("DELETE FROM hotwords WHERE id = ?1", [id])?;
        if deleted == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        Ok(())
    }

    pub fn enabled_hotword_context(&self) -> rusqlite::Result<String> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT id, text, category, enabled, created_at, updated_at
            FROM hotwords
            WHERE enabled = 1
            ORDER BY created_at ASC, id ASC
            "#,
        )?;

        let rows = statement.query_map([], hotword_from_row)?;
        let hotwords = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(format_hotword_context(&hotwords))
    }

    fn get_hotword(&self, id: &str) -> rusqlite::Result<Hotword> {
        self.connection.query_row(
            r#"
            SELECT id, text, category, enabled, created_at, updated_at
            FROM hotwords
            WHERE id = ?1
            "#,
            [id],
            hotword_from_row,
        )
    }
}

fn hotword_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Hotword> {
    Ok(Hotword {
        id: row.get(0)?,
        text: row.get(1)?,
        category: row.get(2)?,
        enabled: int_to_bool(row.get(3)?),
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}
