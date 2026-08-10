use rusqlite::params;
use uuid::Uuid;

use super::{
    database::LocalDatabase,
    helpers::{bool_to_int, int_to_bool},
    models::*,
};

impl LocalDatabase {
    pub fn list_personas(&self) -> rusqlite::Result<Vec<Persona>> {
        list_personas_from_connection(&self.connection)
    }

    pub fn set_default_persona(&self, persona_id: &str) -> rusqlite::Result<()> {
        self.set_default_persona_and_list(persona_id).map(|_| ())
    }

    pub fn set_default_persona_and_list(&self, persona_id: &str) -> rusqlite::Result<Vec<Persona>> {
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute("UPDATE personas SET is_default = 0", [])?;
        let updated = transaction.execute(
            "UPDATE personas SET is_default = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            [persona_id],
        )?;
        if updated == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        let personas = list_personas_from_connection(&transaction)?;
        transaction.commit()?;
        Ok(personas)
    }

    pub fn create_persona(&self, draft: PersonaDraft) -> rusqlite::Result<Persona> {
        let id = Uuid::new_v4().to_string();
        self.connection.execute(
            r#"
            INSERT INTO personas (
                id, name, description, icon, is_default, processing_mode
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                id,
                draft.name,
                draft.description,
                draft.icon,
                0, // is_default = false
                normalized_processing_mode(&draft.processing_mode)
            ],
        )?;

        self.get_persona(&id)
    }

    pub fn update_persona(&self, id: &str, draft: PersonaDraft) -> Result<Persona, String> {
        if id == GENERAL_PERSONA_ID {
            return Err("通用人格是系统内置人格，不可修改".to_string());
        }

        let updated = self
            .connection
            .execute(
                r#"
            UPDATE personas
            SET name = ?2,
                description = ?3,
                icon = ?4,
                processing_mode = ?5,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?1
            "#,
                params![
                    id,
                    draft.name,
                    draft.description,
                    draft.icon,
                    normalized_processing_mode(&draft.processing_mode)
                ],
            )
            .map_err(|error| error.to_string())?;

        if updated == 0 {
            return Err("人格不存在".to_string());
        }

        self.get_persona(id).map_err(|error| error.to_string())
    }

    pub fn delete_persona(&self, id: &str) -> Result<(), String> {
        if id == GENERAL_PERSONA_ID {
            return Err("通用人格是系统内置人格，不可删除".to_string());
        }

        let deleted = self
            .connection
            .execute(
                "DELETE FROM personas WHERE id = ?1 AND is_default = 0",
                [id],
            )
            .map_err(|error| error.to_string())?;

        if deleted == 0 {
            if self
                .connection
                .query_row(
                    "SELECT is_default FROM personas WHERE id = ?1",
                    [id],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|error| error.to_string())?
                != 0
            {
                return Err("默认人格不可删除，请先设置其他人格为默认人格".to_string());
            }
            return Err("人格不存在".to_string());
        }

        Ok(())
    }

    pub fn ensure_default_persona(&self, preferred_id: &str) -> rusqlite::Result<Persona> {
        let transaction = self.connection.unchecked_transaction()?;
        let selected_id = transaction
            .query_row(
                "SELECT id FROM personas WHERE id = ?1",
                [preferred_id],
                |row| row.get::<_, String>(0),
            )
            .ok()
            .unwrap_or_else(|| GENERAL_PERSONA_ID.to_string());
        transaction.execute("UPDATE personas SET is_default = 0", [])?;
        transaction.execute(
            "UPDATE personas SET is_default = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            [&selected_id],
        )?;
        transaction.commit()?;
        self.get_persona(&selected_id)
    }

    fn get_persona(&self, id: &str) -> rusqlite::Result<Persona> {
        self.connection.query_row(
            r#"
            SELECT id, name, description, icon, is_default, processing_mode, created_at, updated_at
            FROM personas
            WHERE id = ?1
            "#,
            [id],
            persona_from_row,
        )
    }

    pub(super) fn seed_builtin_personas(&self) -> rusqlite::Result<()> {
        let persona_count: i64 =
            self.connection
                .query_row("SELECT COUNT(*) FROM personas", [], |row| row.get(0))?;
        let is_fresh_database = persona_count == 0;

        for persona in builtin_personas() {
            if !is_fresh_database
                && persona.id != GENERAL_PERSONA_ID
                && persona.id != VERBATIM_PERSONA_ID
            {
                continue;
            }
            let is_default = is_fresh_database && persona.id == GENERAL_PERSONA_ID;
            self.connection.execute(
                r#"
                INSERT INTO personas (
                    id, name, description, icon, is_default, processing_mode
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(id) DO NOTHING
                "#,
                params![
                    persona.id,
                    persona.name,
                    persona.description,
                    persona.icon,
                    bool_to_int(is_default),
                    persona.processing_mode
                ],
            )?;
        }

        Ok(())
    }
}

fn builtin_personas() -> Vec<PersonaSeed> {
    vec![
        PersonaSeed {
            id: GENERAL_PERSONA_ID,
            name: "通用人格",
            description: "让文本保持自然、清晰、口语化的语气，同时更精炼易读，要把句尾的句号去掉。",
            icon: "Sparkles",
            processing_mode: POLISH_PROCESSING_MODE,
        },
        PersonaSeed {
            id: VERBATIM_PERSONA_ID,
            name: "原文听写",
            description: "保留语音识别原文，仅清理首尾和连续空白，不进行文本润色。",
            icon: "BookOpen",
            processing_mode: VERBATIM_PROCESSING_MODE,
        },
        PersonaSeed {
            id: "prompt-engineer",
            name: "Prompt 工程师",
            description: "将语音转换为清晰、可执行的 AI Prompt。输出结构：目标、上下文、约束、期望结果。适合与 Agent 工具协作。",
            icon: "Bot",
            processing_mode: POLISH_PROCESSING_MODE,
        },
        PersonaSeed {
            id: "task-collaborator",
            name: "任务协作者",
            description: "将口述任务整理为结构化的工作指令。包含：背景、要求、交付物、时间节点。语气温和明确。",
            icon: "ClipboardList",
            processing_mode: POLISH_PROCESSING_MODE,
        },
        PersonaSeed {
            id: "idea-organizer",
            name: "灵感整理师",
            description: "将碎片化想法整理为可展开的创作素材。输出：标题候选、关键要点、后续待办。适合写作和创作场景。",
            icon: "Lightbulb",
            processing_mode: POLISH_PROCESSING_MODE,
        },
        PersonaSeed {
            id: "formal-message",
            name: "正式消息助手",
            description: "将口语化表达转换为正式的办公消息或邮件。语气礼貌准确，可直接发送。",
            icon: "Mail",
            processing_mode: POLISH_PROCESSING_MODE,
        },
        PersonaSeed {
            id: "translator",
            name: "翻译官",
            description: "如果文本为中文，翻译成自然流畅的英文；如已是英文则仅做清理润色，不改变语言。专有名词保持原样。",
            icon: "Languages",
            processing_mode: POLISH_PROCESSING_MODE,
        },
    ]
}

struct PersonaSeed {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    icon: &'static str,
    processing_mode: &'static str,
}

fn list_personas_from_connection(
    connection: &rusqlite::Connection,
) -> rusqlite::Result<Vec<Persona>> {
    let mut statement = connection.prepare(
        r#"
        SELECT id, name, description, icon, is_default, processing_mode, created_at, updated_at
        FROM personas
        ORDER BY CASE WHEN id = 'general' THEN 0 ELSE 1 END,
                 created_at ASC,
                 rowid ASC
        "#,
    )?;
    let rows = statement.query_map([], persona_from_row)?;
    rows.collect()
}

fn persona_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Persona> {
    Ok(Persona {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        icon: row.get(3)?,
        is_default: int_to_bool(row.get(4)?),
        processing_mode: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}
