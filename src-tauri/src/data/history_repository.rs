use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use super::{
    database::LocalDatabase,
    helpers::{bool_to_int, int_to_bool},
    models::*,
};

impl LocalDatabase {
    pub fn create_history_record(
        &self,
        draft: HistoryRecordDraft,
    ) -> rusqlite::Result<HistoryRecord> {
        let id = Uuid::new_v4().to_string();
        let output_chars = draft.final_text.chars().count() as i64;
        self.connection.execute(
            r#"
            INSERT INTO history_records (
                id, raw_text, final_text, persona_id, persona_name,
                duration_ms, output_chars, output_mode, source,
                asr_provider, asr_model, text_provider, text_model,
                used_asr_fallback, used_fallback, delivery_method, audio_path
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            "#,
            params![
                id,
                draft.raw_text,
                draft.final_text,
                draft.persona_id,
                draft.persona_name,
                draft.duration_ms,
                output_chars,
                draft.output_mode,
                draft.source,
                draft.asr_provider,
                draft.asr_model,
                draft.text_provider,
                draft.text_model,
                bool_to_int(draft.used_asr_fallback),
                bool_to_int(draft.used_fallback),
                draft.delivery_method,
                draft.audio_path
            ],
        )?;

        self.get_history_record(&id)
    }

    pub fn list_history_records(&self, limit: i64) -> rusqlite::Result<Vec<HistoryRecord>> {
        let safe_limit = limit.clamp(1, 100);
        let mut statement = self.connection.prepare(
            r#"
            SELECT id, raw_text, final_text, persona_id, persona_name,
                   duration_ms, output_chars, output_mode, source,
                   asr_provider, asr_model, text_provider, text_model,
                   used_asr_fallback, used_fallback, delivery_method, audio_path, created_at
            FROM history_records
            ORDER BY datetime(created_at) DESC, rowid DESC
            LIMIT ?1
            "#,
        )?;

        let rows = statement.query_map([safe_limit], history_record_from_row)?;
        rows.collect()
    }

    pub fn history_statistics(&self) -> rusqlite::Result<HistoryStatistics> {
        let (total_count, total_duration_ms, total_output_chars): (i64, i64, i64) =
            self.connection.query_row(
                r#"
                SELECT COUNT(*),
                       COALESCE(SUM(duration_ms), 0),
                       COALESCE(SUM(output_chars), 0)
                FROM history_records
                "#,
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?;
        let top_persona = self
            .connection
            .query_row(
                r#"
                SELECT persona_name, COUNT(*) AS usage_count
                FROM history_records
                GROUP BY persona_id, persona_name
                ORDER BY usage_count DESC, persona_name ASC
                LIMIT 1
                "#,
                [],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()?;
        let manual_input_ms = total_output_chars * 60_000 / 80;
        let estimated_saved_ms = (manual_input_ms - total_duration_ms).max(0);
        let (top_persona_name, top_persona_count) = match top_persona {
            Some((name, count)) => (Some(name), count),
            None => (None, 0),
        };

        Ok(HistoryStatistics {
            total_count,
            total_duration_ms,
            total_output_chars,
            estimated_saved_ms,
            top_persona_name,
            top_persona_count,
        })
    }

    pub fn delete_history_record(&self, id: &str) -> rusqlite::Result<()> {
        self.connection
            .execute("DELETE FROM history_records WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_history_after_transcription(
        &self,
        id: &str,
        raw_text: &str,
        final_text: &str,
        persona_id: &str,
        persona_name: &str,
        asr_provider: &str,
        asr_model: &str,
        text_provider: &str,
        text_model: &str,
        used_asr_fallback: bool,
        used_fallback: bool,
    ) -> rusqlite::Result<HistoryRecord> {
        let output_chars = final_text.chars().count() as i64;
        self.connection.execute(
            r#"
            UPDATE history_records
            SET raw_text = ?2, final_text = ?3, persona_id = ?4, persona_name = ?5,
                output_chars = ?6, asr_provider = ?7, asr_model = ?8,
                text_provider = ?9, text_model = ?10,
                used_asr_fallback = ?11, used_fallback = ?12
            WHERE id = ?1
            "#,
            params![
                id,
                raw_text,
                final_text,
                persona_id,
                persona_name,
                output_chars,
                asr_provider,
                asr_model,
                text_provider,
                text_model,
                bool_to_int(used_asr_fallback),
                bool_to_int(used_fallback)
            ],
        )?;
        self.get_history_record(id)
    }

    pub fn update_history_after_refinement(
        &self,
        id: &str,
        final_text: &str,
        persona_id: &str,
        persona_name: &str,
        text_provider: &str,
        text_model: &str,
        used_fallback: bool,
    ) -> rusqlite::Result<HistoryRecord> {
        let output_chars = final_text.chars().count() as i64;
        self.connection.execute(
            r#"
            UPDATE history_records
            SET final_text = ?2, persona_id = ?3, persona_name = ?4,
                output_chars = ?5, text_provider = ?6, text_model = ?7,
                used_fallback = ?8
            WHERE id = ?1
            "#,
            params![
                id,
                final_text,
                persona_id,
                persona_name,
                output_chars,
                text_provider,
                text_model,
                bool_to_int(used_fallback)
            ],
        )?;
        self.get_history_record(id)
    }

    pub fn update_history_delivery_method(
        &self,
        id: &str,
        delivery_method: &str,
    ) -> rusqlite::Result<()> {
        self.connection.execute(
            "UPDATE history_records SET delivery_method = ?2, output_mode = ?2 WHERE id = ?1",
            params![id, delivery_method],
        )?;
        Ok(())
    }

    pub fn history_audio_path(&self, id: &str) -> rusqlite::Result<Option<String>> {
        self.connection
            .query_row(
                "SELECT audio_path FROM history_records WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .optional()
            .map(|value| value.flatten())
    }

    pub fn list_history_audio_paths(&self) -> rusqlite::Result<Vec<String>> {
        let mut statement = self
            .connection
            .prepare("SELECT audio_path FROM history_records WHERE audio_path IS NOT NULL")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect()
    }

    pub fn clear_history_audio_paths(&self) -> rusqlite::Result<()> {
        self.connection
            .execute("UPDATE history_records SET audio_path = NULL", [])?;
        Ok(())
    }

    pub fn get_history_record(&self, id: &str) -> rusqlite::Result<HistoryRecord> {
        self.connection.query_row(
            r#"
            SELECT id, raw_text, final_text, persona_id, persona_name,
                   duration_ms, output_chars, output_mode, source,
                   asr_provider, asr_model, text_provider, text_model,
                   used_asr_fallback, used_fallback, delivery_method, audio_path, created_at
            FROM history_records
            WHERE id = ?1
            "#,
            [id],
            history_record_from_row,
        )
    }
}

fn history_record_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<HistoryRecord> {
    Ok(HistoryRecord {
        id: row.get(0)?,
        raw_text: row.get(1)?,
        final_text: row.get(2)?,
        persona_id: row.get(3)?,
        persona_name: row.get(4)?,
        duration_ms: row.get(5)?,
        output_chars: row.get(6)?,
        output_mode: row.get(7)?,
        source: row.get(8)?,
        asr_provider: row.get(9)?,
        asr_model: row.get(10)?,
        text_provider: row.get(11)?,
        text_model: row.get(12)?,
        used_asr_fallback: int_to_bool(row.get(13)?),
        used_fallback: int_to_bool(row.get(14)?),
        delivery_method: row.get(15)?,
        audio_path: row.get(16)?,
        created_at: row.get(17)?,
    })
}
