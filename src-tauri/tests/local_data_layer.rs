use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use xiluolin_lib::data::{
    default_app_config, AppConfig, HistoryRecordDraft, HotwordDraft, LocalDatabase,
};

fn temp_db_path(test_name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("xiluolin-tests")
        .join(format!("{test_name}-{nanos}.sqlite"))
}

fn open_test_database(path: &Path) -> LocalDatabase {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("test database parent should be created");
    }

    let database = LocalDatabase::open(path).expect("test database should open");
    database
        .initialize()
        .expect("test database should initialize");
    database
}

#[test]
fn initialize_creates_required_tables() {
    let database = open_test_database(&temp_db_path("creates-required-tables"));

    let table_names = database
        .table_names()
        .expect("table names should be readable");

    assert!(table_names.contains(&"personas".to_string()));
    assert!(table_names.contains(&"hotwords".to_string()));
    assert!(table_names.contains(&"history_records".to_string()));
}

#[test]
fn initialize_is_idempotent_and_keeps_existing_data() {
    let database = open_test_database(&temp_db_path("idempotent-initialize"));
    let created = database
        .create_hotword(HotwordDraft {
            text: "Next.js".to_string(),
            category: "技术词".to_string(),
            enabled: true,
        })
        .expect("hotword should be created");

    database.initialize().expect("second init should pass");

    let hotwords = database
        .list_hotwords()
        .expect("hotwords should remain readable");

    assert_eq!(hotwords.len(), 1);
    assert_eq!(hotwords[0].id, created.id);
    assert_eq!(hotwords[0].text, "Next.js");
}

#[test]
fn hotword_roundtrip_keeps_enabled_state() {
    let database = open_test_database(&temp_db_path("hotword-roundtrip"));

    let created = database
        .create_hotword(HotwordDraft {
            text: "七牛云".to_string(),
            category: "产品名".to_string(),
            enabled: false,
        })
        .expect("hotword should be created");

    let hotwords = database
        .list_hotwords()
        .expect("hotwords should be readable");

    assert_eq!(hotwords.len(), 1);
    assert_eq!(hotwords[0].id, created.id);
    assert_eq!(hotwords[0].text, "七牛云");
    assert_eq!(hotwords[0].category, "产品名");
    assert!(!hotwords[0].enabled);
}

#[test]
fn hotword_can_be_updated_deleted_and_formatted_as_context() {
    let database = open_test_database(&temp_db_path("hotword-crud-context"));

    let first = database
        .create_hotword(HotwordDraft {
            text: "Next.js".to_string(),
            category: "技术词".to_string(),
            enabled: true,
        })
        .expect("first hotword should be created");
    let second = database
        .create_hotword(HotwordDraft {
            text: "七牛云".to_string(),
            category: "产品名".to_string(),
            enabled: false,
        })
        .expect("second hotword should be created");

    let updated = database
        .update_hotword(
            &second.id,
            HotwordDraft {
                text: "七牛云存储".to_string(),
                category: "云服务".to_string(),
                enabled: true,
            },
        )
        .expect("hotword should be updated");
    database
        .delete_hotword(&first.id)
        .expect("hotword should be deleted");

    let hotwords = database
        .list_hotwords()
        .expect("hotwords should be readable");
    let context = database
        .enabled_hotword_context()
        .expect("context should be built");

    assert_eq!(hotwords.len(), 1);
    assert_eq!(hotwords[0].id, updated.id);
    assert_eq!(hotwords[0].text, "七牛云存储");
    assert_eq!(hotwords[0].category, "云服务");
    assert!(hotwords[0].enabled);
    assert_eq!(context, "- 七牛云存储（云服务）");
}

#[test]
fn disabled_hotwords_are_excluded_from_context() {
    let database = open_test_database(&temp_db_path("hotword-context-enabled"));

    database
        .create_hotword(HotwordDraft {
            text: "Codex".to_string(),
            category: "工具名".to_string(),
            enabled: false,
        })
        .expect("hotword should be created");

    let context = database
        .enabled_hotword_context()
        .expect("context should be built");

    assert_eq!(context, "");
}

#[test]
fn history_records_are_returned_newest_first() {
    let database = open_test_database(&temp_db_path("history-order"));

    database
        .create_history_record(HistoryRecordDraft {
            raw_text: "第一条原文".to_string(),
            final_text: "第一条结果".to_string(),
            persona_id: "prompt-engineer".to_string(),
            persona_name: "Prompt 工程师".to_string(),
            duration_ms: 1200,
            output_mode: "copy".to_string(),
        })
        .expect("first history record should be created");

    database
        .create_history_record(HistoryRecordDraft {
            raw_text: "第二条原文".to_string(),
            final_text: "第二条结果".to_string(),
            persona_id: "task-collaborator".to_string(),
            persona_name: "任务协作者".to_string(),
            duration_ms: 2400,
            output_mode: "paste".to_string(),
        })
        .expect("second history record should be created");

    let records = database
        .list_history_records(10)
        .expect("history records should be readable");

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].final_text, "第二条结果");
    assert_eq!(records[0].output_chars, 5);
    assert_eq!(records[1].final_text, "第一条结果");
}

#[test]
fn history_statistics_are_calculated_from_saved_records() {
    let database = open_test_database(&temp_db_path("history-statistics"));

    database
        .create_history_record(HistoryRecordDraft {
            raw_text: "第一段原文".to_string(),
            final_text: "字".repeat(80),
            persona_id: "prompt-engineer".to_string(),
            persona_name: "Prompt 工程师".to_string(),
            duration_ms: 30_000,
            output_mode: "copy".to_string(),
        })
        .expect("first history record should be created");

    database
        .create_history_record(HistoryRecordDraft {
            raw_text: "第二段原文".to_string(),
            final_text: "词".repeat(40),
            persona_id: "prompt-engineer".to_string(),
            persona_name: "Prompt 工程师".to_string(),
            duration_ms: 10_000,
            output_mode: "copy".to_string(),
        })
        .expect("second history record should be created");

    database
        .create_history_record(HistoryRecordDraft {
            raw_text: "第三段原文".to_string(),
            final_text: "项".repeat(10),
            persona_id: "task-collaborator".to_string(),
            persona_name: "任务协作者".to_string(),
            duration_ms: 5_000,
            output_mode: "paste".to_string(),
        })
        .expect("third history record should be created");

    let statistics = database
        .history_statistics()
        .expect("history statistics should be readable");

    assert_eq!(statistics.total_count, 3);
    assert_eq!(statistics.total_duration_ms, 45_000);
    assert_eq!(statistics.total_output_chars, 130);
    assert_eq!(statistics.estimated_saved_ms, 52_500);
    assert_eq!(
        statistics.top_persona_name,
        Some("Prompt 工程师".to_string())
    );
    assert_eq!(statistics.top_persona_count, 2);
}

#[test]
fn empty_history_statistics_return_zero_values() {
    let database = open_test_database(&temp_db_path("empty-history-statistics"));

    let statistics = database
        .history_statistics()
        .expect("empty history statistics should be readable");

    assert_eq!(statistics.total_count, 0);
    assert_eq!(statistics.total_duration_ms, 0);
    assert_eq!(statistics.total_output_chars, 0);
    assert_eq!(statistics.estimated_saved_ms, 0);
    assert_eq!(statistics.top_persona_name, None);
    assert_eq!(statistics.top_persona_count, 0);
}

#[test]
fn default_config_contains_provider_and_output_defaults() {
    let config = default_app_config();

    assert_eq!(
        config,
        AppConfig {
            default_persona_id: "prompt-engineer".to_string(),
            asr_provider: "zhipu".to_string(),
            asr_api_key: "".to_string(),
            asr_base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            asr_model: "glm-asr-2512".to_string(),
            openai_asr_model: "whisper-1".to_string(),
            openai_api_key: "".to_string(),
            openai_base_url: "https://api.openai.com/v1".to_string(),
            openai_model: "gpt-4o-mini".to_string(),
            text_provider: "zhipu".to_string(),
            zhipu_api_key: "".to_string(),
            zhipu_base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            zhipu_model: "glm-4.7-flash".to_string(),
            longpress_shortcut: "CommandOrControl+Shift+R".to_string(),
            toggle_shortcut: "Alt+Space".to_string(),
            auto_save_history: true,
            mute_system_audio: false,
            selected_microphone: "".to_string(),
        }
    );
}

#[test]
fn default_persona_can_be_changed_and_persisted() {
    let database = open_test_database(&temp_db_path("default-persona-change"));

    let initial_personas = database.list_personas().expect("personas should load");
    let prompt_engineer = initial_personas
        .iter()
        .find(|persona| persona.id == "prompt-engineer")
        .expect("prompt engineer should exist");
    let task_collaborator = initial_personas
        .iter()
        .find(|persona| persona.id == "task-collaborator")
        .expect("task collaborator should exist");

    assert!(prompt_engineer.is_default);
    assert!(!task_collaborator.is_default);

    database
        .set_default_persona("task-collaborator")
        .expect("default persona should update");

    let personas = database.list_personas().expect("personas should reload");
    let prompt_engineer = personas
        .iter()
        .find(|persona| persona.id == "prompt-engineer")
        .expect("prompt engineer should still exist");
    let task_collaborator = personas
        .iter()
        .find(|persona| persona.id == "task-collaborator")
        .expect("task collaborator should still exist");

    assert!(!prompt_engineer.is_default);
    assert!(task_collaborator.is_default);
}
