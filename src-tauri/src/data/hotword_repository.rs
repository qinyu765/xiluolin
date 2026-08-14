use rusqlite::params;
use uuid::Uuid;

use super::{
    database::LocalDatabase,
    helpers::{bool_to_int, format_hotword_context, int_to_bool},
    models::*,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnabledHotwordSnapshot {
    pub asr_hotwords: Vec<String>,
    pub hotword_context: String,
}

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

    pub fn add_hotwords(&self, texts: Vec<String>) -> rusqlite::Result<Vec<Hotword>> {
        let normalized = normalize_hotword_texts(texts);
        let existing = self.list_hotwords()?;
        let transaction = self.connection.unchecked_transaction()?;

        for text in normalized {
            if existing.iter().any(|hotword| hotword.text.trim() == text) {
                continue;
            }
            transaction.execute(
                r#"
                INSERT INTO hotwords (id, text, category, enabled)
                VALUES (?1, ?2, '', 1)
                "#,
                params![Uuid::new_v4().to_string(), text],
            )?;
        }

        transaction.commit()?;
        self.list_hotwords()
    }

    pub fn replace_hotwords(&self, texts: Vec<String>) -> rusqlite::Result<Vec<Hotword>> {
        let normalized = normalize_hotword_texts(texts);
        let existing = self.list_hotwords()?;
        let transaction = self.connection.unchecked_transaction()?;
        let mut kept_ids = Vec::new();

        for text in &normalized {
            if let Some(existing_hotword) = existing
                .iter()
                .find(|hotword| hotword.text.trim() == text && !kept_ids.contains(&hotword.id))
            {
                if existing_hotword.text != *text {
                    transaction.execute(
                        "UPDATE hotwords SET text = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
                        params![&existing_hotword.id, text],
                    )?;
                }
                kept_ids.push(existing_hotword.id.clone());
            } else {
                transaction.execute(
                    r#"
                    INSERT INTO hotwords (id, text, category, enabled)
                    VALUES (?1, ?2, '', 1)
                    "#,
                    params![Uuid::new_v4().to_string(), text],
                )?;
            }
        }

        for hotword in existing {
            if !kept_ids.contains(&hotword.id) {
                transaction.execute("DELETE FROM hotwords WHERE id = ?1", [hotword.id])?;
            }
        }

        transaction.commit()?;
        let hotwords = self.list_hotwords()?;
        Ok(normalized
            .into_iter()
            .filter_map(|text| {
                hotwords
                    .iter()
                    .find(|hotword| hotword.text == text)
                    .cloned()
            })
            .collect())
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
        Ok(self.enabled_hotword_snapshot()?.hotword_context)
    }

    pub fn enabled_hotword_texts(&self) -> rusqlite::Result<Vec<String>> {
        Ok(self.enabled_hotword_snapshot()?.asr_hotwords)
    }

    pub fn enabled_hotword_snapshot(&self) -> rusqlite::Result<EnabledHotwordSnapshot> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT id, text, category, enabled, created_at, updated_at
            FROM hotwords
            WHERE enabled = 1
            ORDER BY created_at ASC, id ASC
            "#,
        )?;

        let rows = statement.query_map([], hotword_from_row)?;
        let hotwords = normalize_enabled_hotwords(rows.collect::<rusqlite::Result<Vec<_>>>()?);
        Ok(EnabledHotwordSnapshot {
            asr_hotwords: hotwords
                .iter()
                .map(|hotword| hotword.text.clone())
                .collect(),
            hotword_context: format_hotword_context(&hotwords),
        })
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

fn normalize_enabled_hotwords(hotwords: Vec<Hotword>) -> Vec<Hotword> {
    let mut normalized = Vec::new();
    for mut hotword in hotwords {
        hotword.text = hotword.text.trim().to_string();
        if !hotword.text.is_empty()
            && !normalized
                .iter()
                .any(|existing: &Hotword| existing.text == hotword.text)
        {
            normalized.push(hotword);
        }
    }
    normalized
}

fn normalize_hotword_texts(texts: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for text in texts {
        let text = text.trim().to_string();
        if !text.is_empty() && !normalized.contains(&text) {
            normalized.push(text);
        }
    }
    normalized
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
