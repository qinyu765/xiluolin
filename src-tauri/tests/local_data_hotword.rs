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
fn adding_hotwords_keeps_existing_dictionary_entries_and_metadata() {
    let database = open_test_database(&temp_db_path("hotword-add"));
    let existing = database
        .create_hotword(HotwordDraft {
            text: "XiLuoLin".to_string(),
            category: "产品名".to_string(),
            enabled: false,
        })
        .expect("existing hotword should be created");

    let hotwords = database
        .add_hotwords(vec![
            " XiLuoLin ".to_string(),
            "新词".to_string(),
            "新词".to_string(),
        ])
        .expect("hotwords should be added");

    assert_eq!(hotwords.len(), 2);
    let preserved = hotwords
        .iter()
        .find(|hotword| hotword.text == "XiLuoLin")
        .expect("existing hotword should remain");
    assert_eq!(preserved.id, existing.id);
    assert_eq!(preserved.category, "产品名");
    assert!(!preserved.enabled);

    let added = hotwords
        .iter()
        .find(|hotword| hotword.text == "新词")
        .expect("new hotword should be added");
    assert_eq!(added.category, "");
    assert!(added.enabled);
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
fn enabled_hotword_texts_keep_dictionary_order_for_asr() {
    let database = open_test_database(&temp_db_path("hotword-asr-order"));
    for (text, enabled) in [
        (" 第一 ", true),
        ("第二", false),
        ("第三", true),
        ("第一", true),
    ] {
        database
            .create_hotword(HotwordDraft {
                text: text.to_string(),
                category: "".to_string(),
                enabled,
            })
            .expect("hotword should be created");
    }

    let mut expected = Vec::new();
    for hotword in database
        .list_hotwords()
        .expect("dictionary should load")
        .into_iter()
        .filter(|hotword| hotword.enabled)
    {
        let text = hotword.text.trim().to_string();
        if !text.is_empty() && !expected.contains(&text) {
            expected.push(text);
        }
    }

    assert_eq!(
        database
            .enabled_hotword_texts()
            .expect("enabled texts should load"),
        expected
    );
}

#[test]
fn enabled_hotword_snapshot_stably_deduplicates_once_for_asr_and_context() {
    let database = open_test_database(&temp_db_path("hotword-snapshot"));
    for (text, category) in [
        ("  XiLuoLin ", "产品名"),
        ("XiLuoLin", "重复项"),
        ("\t", "空白"),
        ("智谱", "模型"),
    ] {
        database
            .create_hotword(HotwordDraft {
                text: text.to_string(),
                category: category.to_string(),
                enabled: true,
            })
            .expect("hotword should be created");
    }

    let mut expected = Vec::new();
    for mut hotword in database
        .list_hotwords()
        .expect("dictionary should load")
        .into_iter()
        .filter(|hotword| hotword.enabled)
    {
        hotword.text = hotword.text.trim().to_string();
        if !hotword.text.is_empty()
            && !expected
                .iter()
                .any(|existing: &xiluolin_lib::data::Hotword| existing.text == hotword.text)
        {
            expected.push(hotword);
        }
    }
    let expected_asr_hotwords = expected
        .iter()
        .map(|hotword| hotword.text.clone())
        .collect::<Vec<_>>();
    let expected_context = expected
        .iter()
        .map(|hotword| format!("- {}（{}）", hotword.text, hotword.category))
        .collect::<Vec<_>>()
        .join("\n");
    let snapshot = database
        .enabled_hotword_snapshot()
        .expect("enabled hotword snapshot should load");

    assert_eq!(snapshot.asr_hotwords, expected_asr_hotwords);
    assert_eq!(snapshot.hotword_context, expected_context);
}

#[test]
fn replacing_hotwords_trims_deduplicates_and_preserves_existing_metadata() {
    let database = open_test_database(&temp_db_path("hotword-replace"));
    let existing = database
        .create_hotword(HotwordDraft {
            text: "  XiLuoLin  ".to_string(),
            category: "产品名".to_string(),
            enabled: false,
        })
        .expect("existing hotword should be created");
    database
        .create_hotword(HotwordDraft {
            text: "旧词".to_string(),
            category: "旧分类".to_string(),
            enabled: true,
        })
        .expect("obsolete hotword should be created");

    let replaced = database
        .replace_hotwords(vec![
            " XiLuoLin ".to_string(),
            "".to_string(),
            "XiLuoLin".to_string(),
            "新词".to_string(),
            "  ".to_string(),
        ])
        .expect("dictionary replacement should succeed");

    assert_eq!(replaced.len(), 2);
    assert_eq!(replaced[0].id, existing.id);
    assert_eq!(replaced[0].text, "XiLuoLin");
    assert_eq!(replaced[0].category, "产品名");
    assert!(!replaced[0].enabled);
    assert_eq!(replaced[1].text, "新词");
    assert_eq!(replaced[1].category, "");
    assert!(replaced[1].enabled);
    assert!(!replaced.iter().any(|hotword| hotword.text == "旧词"));

    let empty = database
        .replace_hotwords(Vec::new())
        .expect("an empty dictionary should be saved");
    assert!(empty.is_empty());
}
