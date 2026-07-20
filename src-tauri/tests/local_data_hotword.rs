mod common;

use common::{open_test_database, temp_db_path};
#[allow(unused_imports)]
use xiluolin_lib::data::{
    default_app_config, AppConfig, HistoryRecordDraft, HotwordDraft, PersonaDraft,
    GENERAL_PERSONA_ID,
};

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
