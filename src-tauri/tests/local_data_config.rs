mod common;

use common::{open_test_database, temp_db_path};
#[allow(unused_imports)]
use xiluolin_lib::data::{
    default_app_config, AppConfig, HistoryRecordDraft, HotwordDraft, PersonaDraft,
    GENERAL_PERSONA_ID,
};

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
fn default_config_contains_provider_and_output_defaults() {
    let config = default_app_config();

    assert_eq!(config.config_version, 2);
    assert_eq!(config.default_persona_id, GENERAL_PERSONA_ID);
    assert_eq!(config.asr.primary, "zhipu");
    assert_eq!(config.asr.settings["zhipu"].model, "glm-asr-2512");
    assert_eq!(config.asr.settings["openai"].model, "whisper-1");
    assert_eq!(config.asr.settings["local"].model, "ggml-base-q5_1.bin");
    assert_eq!(config.text.primary, "zhipu");
    assert_eq!(config.text.settings["zhipu"].model, "glm-4.7-flash");
    assert_eq!(config.text.settings["openai"].model, "gpt-4o-mini");
    assert_eq!(config.longpress_shortcut, "CommandOrControl+Shift+R");
    assert_eq!(config.toggle_shortcut, "Alt+Space");
    assert!(!config.fn_hold_enabled);
    assert!(config.auto_save_history);
}

#[test]
fn legacy_config_defaults_fn_hold_to_disabled() {
    let mut value = serde_json::to_value(default_app_config()).unwrap();
    value.as_object_mut().unwrap().remove("fn_hold_enabled");

    let config: AppConfig = serde_json::from_value(value).unwrap();

    assert!(!config.fn_hold_enabled);
}
