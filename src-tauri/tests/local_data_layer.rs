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
            source_text: "next 点 js".to_string(),
            target_text: "Next.js".to_string(),
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
    assert_eq!(hotwords[0].target_text, "Next.js");
}

#[test]
fn hotword_roundtrip_keeps_enabled_state() {
    let database = open_test_database(&temp_db_path("hotword-roundtrip"));

    let created = database
        .create_hotword(HotwordDraft {
            source_text: "七牛".to_string(),
            target_text: "七牛云".to_string(),
            category: "产品名".to_string(),
            enabled: false,
        })
        .expect("hotword should be created");

    let hotwords = database
        .list_hotwords()
        .expect("hotwords should be readable");

    assert_eq!(hotwords.len(), 1);
    assert_eq!(hotwords[0].id, created.id);
    assert_eq!(hotwords[0].source_text, "七牛");
    assert_eq!(hotwords[0].target_text, "七牛云");
    assert_eq!(hotwords[0].category, "产品名");
    assert!(!hotwords[0].enabled);
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
fn default_config_contains_provider_and_output_defaults() {
    let config = default_app_config();

    assert_eq!(
        config,
        AppConfig {
            default_persona_id: "prompt-engineer".to_string(),
            asr_base_url: "https://open.bigmodel.cn/api/paas/v4/".to_string(),
            asr_model: "glm-asr-2512".to_string(),
            openai_model: "gpt-4.1-mini".to_string(),
            recording_mode: "toggle".to_string(),
            shortcut: "CommandOrControl+Shift+Space".to_string(),
            output_mode: "copy".to_string(),
            auto_save_history: true,
        }
    );
}
