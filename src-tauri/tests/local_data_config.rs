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

    assert_eq!(
        config,
        AppConfig {
            default_persona_id: GENERAL_PERSONA_ID.to_string(),
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
            retain_recordings: false,
            local_asr_model: "ggml-base-q5_1.bin".to_string(),
            allow_cloud_fallback: false,
            fallback_asr_provider: "zhipu".to_string(),
        }
    );
}
