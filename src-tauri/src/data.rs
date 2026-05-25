use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const DEFAULT_PERSONA_ID: &str = "prompt-engineer";
const APP_CONFIG_STORE: &str = "settings.json";
const APP_CONFIG_KEY: &str = "app_config";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub default_persona_id: String,
    #[serde(default = "default_asr_provider")]
    pub asr_provider: String,
    #[serde(default)]
    pub asr_api_key: String,
    pub asr_base_url: String,
    pub asr_model: String,
    #[serde(default = "default_openai_asr_model")]
    pub openai_asr_model: String,
    #[serde(default)]
    pub openai_api_key: String,
    pub openai_base_url: String,
    pub openai_model: String,
    #[serde(default = "default_text_provider")]
    pub text_provider: String,
    #[serde(default)]
    pub zhipu_api_key: String,
    #[serde(default = "default_zhipu_base_url")]
    pub zhipu_base_url: String,
    #[serde(default = "default_zhipu_model")]
    pub zhipu_model: String,
    #[serde(default)]
    pub longpress_shortcut: String,
    #[serde(default)]
    pub toggle_shortcut: String,
    pub auto_save_history: bool,
    #[serde(default)]
    pub mute_system_audio: bool,
    #[serde(default)]
    pub selected_microphone: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Persona {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonaDraft {
    pub name: String,
    pub description: String,
    pub icon: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hotword {
    pub id: String,
    pub text: String,
    pub category: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotwordDraft {
    pub text: String,
    pub category: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub id: String,
    pub raw_text: String,
    pub final_text: String,
    pub persona_id: String,
    pub persona_name: String,
    pub duration_ms: i64,
    pub output_chars: i64,
    pub output_mode: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryRecordDraft {
    pub raw_text: String,
    pub final_text: String,
    pub persona_id: String,
    pub persona_name: String,
    pub duration_ms: i64,
    pub output_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryStatistics {
    pub total_count: i64,
    pub total_duration_ms: i64,
    pub total_output_chars: i64,
    pub estimated_saved_ms: i64,
    pub top_persona_name: Option<String>,
    pub top_persona_count: i64,
}

fn default_asr_provider() -> String {
    "zhipu".to_string()
}

fn default_openai_asr_model() -> String {
    "whisper-1".to_string()
}

fn default_text_provider() -> String {
    "zhipu".to_string()
}

fn default_zhipu_base_url() -> String {
    "https://open.bigmodel.cn/api/paas/v4".to_string()
}

fn default_zhipu_model() -> String {
    "glm-4.7-flash".to_string()
}

pub fn default_app_config() -> AppConfig {
    AppConfig {
        default_persona_id: DEFAULT_PERSONA_ID.to_string(),
        asr_provider: default_asr_provider(),
        asr_api_key: "".to_string(),
        asr_base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
        asr_model: "glm-asr-2512".to_string(),
        openai_asr_model: default_openai_asr_model(),
        openai_api_key: "".to_string(),
        openai_base_url: "https://api.openai.com/v1".to_string(),
        openai_model: "gpt-4o-mini".to_string(),
        text_provider: default_text_provider(),
        zhipu_api_key: "".to_string(),
        zhipu_base_url: default_zhipu_base_url(),
        zhipu_model: default_zhipu_model(),
        longpress_shortcut: "CommandOrControl+Shift+R".to_string(),
        toggle_shortcut: "Alt+Space".to_string(),
        auto_save_history: true,
        mute_system_audio: false,
        selected_microphone: "".to_string(),
    }
}

pub struct LocalDatabase {
    connection: Connection,
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
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )?;
        self.seed_builtin_personas()?;
        Ok(())
    }

    pub fn table_names(&self) -> rusqlite::Result<Vec<String>> {
        let mut statement = self.connection.prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect()
    }

    pub fn list_personas(&self) -> rusqlite::Result<Vec<Persona>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT id, name, description, icon, is_default, created_at, updated_at
            FROM personas
            ORDER BY is_default DESC, created_at ASC
            "#,
        )?;

        let rows = statement.query_map([], persona_from_row)?;
        rows.collect()
    }

    pub fn set_default_persona(&self, persona_id: &str) -> rusqlite::Result<()> {
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute("UPDATE personas SET is_default = 0", [])?;
        let updated = transaction.execute(
            "UPDATE personas SET is_default = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            [persona_id],
        )?;
        if updated == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn create_persona(&self, draft: PersonaDraft) -> rusqlite::Result<Persona> {
        let id = Uuid::new_v4().to_string();
        self.connection.execute(
            r#"
            INSERT INTO personas (
                id, name, description, icon, is_default
            )
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                id,
                draft.name,
                draft.description,
                draft.icon,
                0  // is_default = false
            ],
        )?;

        self.get_persona(&id)
    }

    pub fn update_persona(&self, id: &str, draft: PersonaDraft) -> rusqlite::Result<Persona> {
        let updated = self.connection.execute(
            r#"
            UPDATE personas
            SET name = ?2,
                description = ?3,
                icon = ?4,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?1
            "#,
            params![
                id,
                draft.name,
                draft.description,
                draft.icon
            ],
        )?;

        if updated == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        self.get_persona(id)
    }

    pub fn delete_persona(&self, id: &str) -> rusqlite::Result<()> {
        let deleted = self.connection.execute(
            "DELETE FROM personas WHERE id = ?1",
            [id],
        )?;

        if deleted == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        Ok(())
    }

    fn get_persona(&self, id: &str) -> rusqlite::Result<Persona> {
        self.connection.query_row(
            r#"
            SELECT id, name, description, icon, is_default, created_at, updated_at
            FROM personas
            WHERE id = ?1
            "#,
            [id],
            persona_from_row,
        )
    }

    pub fn create_hotword(&self, draft: HotwordDraft) -> rusqlite::Result<Hotword> {
        let id = Uuid::new_v4().to_string();
        self.connection.execute(
            r#"
            INSERT INTO hotwords (id, text, category, enabled)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![
                id,
                draft.text,
                draft.category,
                bool_to_int(draft.enabled)
            ],
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
            params![
                id,
                draft.text,
                draft.category,
                bool_to_int(draft.enabled)
            ],
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
                duration_ms, output_chars, output_mode
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                id,
                draft.raw_text,
                draft.final_text,
                draft.persona_id,
                draft.persona_name,
                draft.duration_ms,
                output_chars,
                draft.output_mode
            ],
        )?;

        self.get_history_record(&id)
    }

    pub fn list_history_records(&self, limit: i64) -> rusqlite::Result<Vec<HistoryRecord>> {
        let safe_limit = limit.clamp(1, 100);
        let mut statement = self.connection.prepare(
            r#"
            SELECT id, raw_text, final_text, persona_id, persona_name,
                   duration_ms, output_chars, output_mode, created_at
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
        self.connection.execute(
            "DELETE FROM history_records WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    fn seed_builtin_personas(&self) -> rusqlite::Result<()> {
        let persona_count: i64 =
            self.connection
                .query_row("SELECT COUNT(*) FROM personas", [], |row| row.get(0))?;
        if persona_count > 0 {
            return Ok(());
        }

        for persona in builtin_personas() {
            self.connection.execute(
                r#"
                INSERT INTO personas (
                    id, name, description, icon, is_default
                )
                VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT(id) DO NOTHING
                "#,
                params![
                    persona.id,
                    persona.name,
                    persona.description,
                    persona.icon,
                    bool_to_int(persona.is_default)
                ],
            )?;
        }

        Ok(())
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

    fn get_history_record(&self, id: &str) -> rusqlite::Result<HistoryRecord> {
        self.connection.query_row(
            r#"
            SELECT id, raw_text, final_text, persona_id, persona_name,
                   duration_ms, output_chars, output_mode, created_at
            FROM history_records
            WHERE id = ?1
            "#,
            [id],
            history_record_from_row,
        )
    }
}

#[tauri::command]
pub fn initialize_local_data(app: tauri::AppHandle) -> Result<AppConfig, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    read_app_config(app)
}

#[tauri::command]
pub fn list_personas(app: tauri::AppHandle) -> Result<Vec<Persona>, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database.list_personas().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_default_persona(
    app: tauri::AppHandle,
    persona_id: String,
) -> Result<Vec<Persona>, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .set_default_persona(&persona_id)
        .map_err(|error| error.to_string())?;
    let mut config = read_app_config(app.clone())?;
    config.default_persona_id = persona_id;
    update_app_config(app, config)?;
    database.list_personas().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_persona(app: tauri::AppHandle, draft: PersonaDraft) -> Result<Persona, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .create_persona(draft)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_persona(
    app: tauri::AppHandle,
    id: String,
    draft: PersonaDraft,
) -> Result<Persona, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .update_persona(&id, draft)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_persona(app: tauri::AppHandle, id: String) -> Result<Vec<Persona>, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .delete_persona(&id)
        .map_err(|error| error.to_string())?;
    database.list_personas().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_hotword(app: tauri::AppHandle, draft: HotwordDraft) -> Result<Hotword, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .create_hotword(draft)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_hotwords(app: tauri::AppHandle) -> Result<Vec<Hotword>, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database.list_hotwords().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_hotword(
    app: tauri::AppHandle,
    id: String,
    draft: HotwordDraft,
) -> Result<Hotword, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .update_hotword(&id, draft)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_hotword(app: tauri::AppHandle, id: String) -> Result<Vec<Hotword>, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .delete_hotword(&id)
        .map_err(|error| error.to_string())?;
    database.list_hotwords().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn enabled_hotword_context(app: tauri::AppHandle) -> Result<String, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .enabled_hotword_context()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_history_record(
    app: tauri::AppHandle,
    draft: HistoryRecordDraft,
) -> Result<HistoryRecord, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .create_history_record(draft)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_history_records(
    app: tauri::AppHandle,
    limit: Option<i64>,
) -> Result<Vec<HistoryRecord>, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .list_history_records(limit.unwrap_or(20))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn history_statistics(app: tauri::AppHandle) -> Result<HistoryStatistics, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .history_statistics()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_history_record(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .delete_history_record(&id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn read_app_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    use tauri_plugin_store::StoreExt;

    let store = app
        .store(APP_CONFIG_STORE)
        .map_err(|error| error.to_string())?;
    let Some(value) = store.get(APP_CONFIG_KEY) else {
        return Ok(default_app_config());
    };

    serde_json::from_value(value.clone()).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_app_config(app: tauri::AppHandle, config: AppConfig) -> Result<AppConfig, String> {
    use tauri_plugin_store::StoreExt;

    let store = app
        .store(APP_CONFIG_STORE)
        .map_err(|error| error.to_string())?;
    let value = serde_json::to_value(&config).map_err(|error| error.to_string())?;
    store.set(APP_CONFIG_KEY.to_string(), value);
    store.save().map_err(|error| error.to_string())?;

    // 热更新快捷键
    let app_clone = app.clone();
    let config_clone = config.clone();
    tauri::async_runtime::spawn(async move {
        let longpress = if config_clone.longpress_shortcut.is_empty() {
            None
        } else {
            Some(config_clone.longpress_shortcut)
        };
        let toggle = if config_clone.toggle_shortcut.is_empty() {
            None
        } else {
            Some(config_clone.toggle_shortcut)
        };
        let _ = crate::hotkey::register_both_hotkeys(app_clone, longpress, toggle).await;
    });

    Ok(config)
}

fn database_for_app(app: &tauri::AppHandle) -> Result<LocalDatabase, String> {
    use tauri::Manager;

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&app_data_dir).map_err(|error| error.to_string())?;
    LocalDatabase::open(app_data_dir.join("xiluolin.sqlite")).map_err(|error| error.to_string())
}

fn builtin_personas() -> Vec<PersonaSeed> {
    vec![
        PersonaSeed {
            id: "prompt-engineer",
            name: "Prompt 工程师",
            description: "将语音转换为清晰、可执行的 AI Prompt。输出结构：目标、上下文、约束、期望结果。适合与 Agent 工具协作。",
            icon: "Bot",
            is_default: true,
        },
        PersonaSeed {
            id: "task-collaborator",
            name: "任务协作者",
            description: "将口述任务整理为结构化的工作指令。包含：背景、要求、交付物、时间节点。语气温和明确。",
            icon: "ClipboardList",
            is_default: false,
        },
        PersonaSeed {
            id: "idea-organizer",
            name: "灵感整理师",
            description: "将碎片化想法整理为可展开的创作素材。输出：标题候选、关键要点、后续待办。适合写作和创作场景。",
            icon: "Lightbulb",
            is_default: false,
        },
        PersonaSeed {
            id: "formal-message",
            name: "正式消息助手",
            description: "将口语化表达转换为正式的办公消息或邮件。语气礼貌准确，可直接发送。",
            icon: "Mail",
            is_default: false,
        },
        PersonaSeed {
            id: "translator",
            name: "翻译官",
            description: "如果文本为中文，翻译成自然流畅的英文；如已是英文则仅做清理润色，不改变语言。专有名词保持原样。",
            icon: "Languages",
            is_default: false,
        },
    ]
}

struct PersonaSeed {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    icon: &'static str,
    is_default: bool,
}

fn persona_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Persona> {
    Ok(Persona {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        icon: row.get(3)?,
        is_default: int_to_bool(row.get(4)?),
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
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
        created_at: row.get(8)?,
    })
}

fn format_hotword_context(hotwords: &[Hotword]) -> String {
    hotwords
        .iter()
        .map(|hotword| {
            if hotword.category.trim().is_empty() {
                format!("- {}", hotword.text)
            } else {
                format!("- {}（{}）", hotword.text, hotword.category)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn bool_to_int(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn int_to_bool(value: i64) -> bool {
    value != 0
}
