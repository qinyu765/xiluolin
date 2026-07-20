mod common;

use std::fs;

use common::{open_test_database, temp_db_path};
#[allow(unused_imports)]
use xiluolin_lib::data::{
    default_app_config, AppConfig, HistoryRecordDraft, HotwordDraft, PersonaDraft,
    GENERAL_PERSONA_ID,
};

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
            source: "recording".to_string(),
            asr_provider: "zhipu".to_string(),
            asr_model: "glm-asr-2512".to_string(),
            text_provider: "openai".to_string(),
            text_model: "gpt-4o-mini".to_string(),
            used_asr_fallback: false,
            used_fallback: false,
            delivery_method: "pending".to_string(),
            audio_path: None,
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
            source: "recording".to_string(),
            asr_provider: "zhipu".to_string(),
            asr_model: "glm-asr-2512".to_string(),
            text_provider: "openai".to_string(),
            text_model: "gpt-4o-mini".to_string(),
            used_asr_fallback: false,
            used_fallback: false,
            delivery_method: "pending".to_string(),
            audio_path: None,
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
            source: "recording".to_string(),
            asr_provider: "zhipu".to_string(),
            asr_model: "glm-asr-2512".to_string(),
            text_provider: "openai".to_string(),
            text_model: "gpt-4o-mini".to_string(),
            used_asr_fallback: false,
            used_fallback: false,
            delivery_method: "pending".to_string(),
            audio_path: None,
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
            source: "recording".to_string(),
            asr_provider: "zhipu".to_string(),
            asr_model: "glm-asr-2512".to_string(),
            text_provider: "openai".to_string(),
            text_model: "gpt-4o-mini".to_string(),
            used_asr_fallback: false,
            used_fallback: false,
            delivery_method: "pending".to_string(),
            audio_path: None,
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
            source: "recording".to_string(),
            asr_provider: "zhipu".to_string(),
            asr_model: "glm-asr-2512".to_string(),
            text_provider: "openai".to_string(),
            text_model: "gpt-4o-mini".to_string(),
            used_asr_fallback: false,
            used_fallback: false,
            delivery_method: "pending".to_string(),
            audio_path: None,
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
fn legacy_history_schema_is_migrated_without_data_loss() {
    let path = temp_db_path("legacy-history-migration");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch(
            r#"
            CREATE TABLE history_records (
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
            INSERT INTO history_records (
                id, raw_text, final_text, persona_id, persona_name,
                duration_ms, output_chars, output_mode
            ) VALUES (
                'legacy', '旧原文', '旧结果', 'prompt-engineer',
                'Prompt 工程师', 1000, 3, 'copy'
            );
            "#,
        )
        .unwrap();
    drop(connection);

    let database = open_test_database(&path);
    let records = database.list_history_records(10).unwrap();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id, "legacy");
    assert_eq!(records[0].source, "unknown");
    assert_eq!(records[0].delivery_method, "pending");
    assert!(records[0].audio_path.is_none());
}

#[test]
fn history_metadata_and_delivery_method_roundtrip() {
    let database = open_test_database(&temp_db_path("history-metadata"));
    let created = database
        .create_history_record(HistoryRecordDraft {
            raw_text: "原始文本".to_string(),
            final_text: "整理结果".to_string(),
            persona_id: "prompt-engineer".to_string(),
            persona_name: "Prompt 工程师".to_string(),
            duration_ms: 1800,
            output_mode: "pending".to_string(),
            source: "recording".to_string(),
            asr_provider: "zhipu".to_string(),
            asr_model: "glm-asr-2512".to_string(),
            text_provider: "openai".to_string(),
            text_model: "gpt-4o-mini".to_string(),
            used_asr_fallback: false,
            used_fallback: true,
            delivery_method: "pending".to_string(),
            audio_path: Some("/managed/recording.wav".to_string()),
        })
        .unwrap();

    database
        .update_history_delivery_method(&created.id, "paste")
        .unwrap();
    let record = database.list_history_records(1).unwrap().remove(0);

    assert_eq!(record.source, "recording");
    assert_eq!(record.asr_provider, "zhipu");
    assert_eq!(record.text_provider, "openai");
    assert!(record.used_fallback);
    assert_eq!(record.delivery_method, "paste");
    assert_eq!(record.output_mode, "paste");
    assert_eq!(record.audio_path.as_deref(), Some("/managed/recording.wav"));
}

#[test]
fn history_can_be_reprocessed_without_losing_audio_link() {
    let database = open_test_database(&temp_db_path("history-reprocessing"));
    let created = database
        .create_history_record(HistoryRecordDraft {
            raw_text: "旧原文".to_string(),
            final_text: "旧结果".to_string(),
            persona_id: "prompt-engineer".to_string(),
            persona_name: "Prompt 工程师".to_string(),
            duration_ms: 1600,
            output_mode: "paste".to_string(),
            source: "recording".to_string(),
            asr_provider: "zhipu".to_string(),
            asr_model: "old-asr".to_string(),
            text_provider: "openai".to_string(),
            text_model: "old-text".to_string(),
            used_asr_fallback: false,
            used_fallback: false,
            delivery_method: "paste".to_string(),
            audio_path: Some("/managed/original.wav".to_string()),
        })
        .unwrap();

    let retranscribed = database
        .update_history_after_transcription(
            &created.id,
            "新原文",
            "新结果",
            "task-collaborator",
            "任务协作者",
            "openai",
            "whisper-1",
            "zhipu",
            "glm-4.7-flash",
            false,
            true,
        )
        .unwrap();
    assert_eq!(retranscribed.raw_text, "新原文");
    assert_eq!(retranscribed.asr_provider, "openai");
    assert_eq!(
        retranscribed.audio_path.as_deref(),
        Some("/managed/original.wav")
    );
    assert_eq!(retranscribed.delivery_method, "paste");

    let refined = database
        .update_history_after_refinement(
            &created.id,
            "再次整理",
            "prompt-engineer",
            "Prompt 工程师",
            "openai",
            "gpt-4o-mini",
            false,
        )
        .unwrap();
    assert_eq!(refined.raw_text, "新原文");
    assert_eq!(refined.final_text, "再次整理");
    assert_eq!(refined.asr_provider, "openai");
    assert_eq!(refined.audio_path.as_deref(), Some("/managed/original.wav"));
}
