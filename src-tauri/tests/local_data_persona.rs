mod common;

use std::fs;

use common::{open_test_database, temp_db_path};
#[allow(unused_imports)]
use xiluolin_lib::data::{
    default_app_config, AppConfig, HistoryRecordDraft, HotwordDraft, PersonaDraft,
    GENERAL_PERSONA_ID,
};

#[test]
fn fresh_database_uses_read_only_general_persona_by_default() {
    let database = open_test_database(&temp_db_path("general-persona-default"));
    let personas = database.list_personas().expect("personas should load");
    let general = personas
        .first()
        .expect("general persona should be listed first");

    assert_eq!(general.id, GENERAL_PERSONA_ID);
    assert_eq!(general.name, "通用人格");
    assert_eq!(
        general.description,
        "让文本保持自然、清晰、口语化的语气，同时更精炼易读，要把句尾的句号去掉。"
    );
    assert!(general.is_default);
    assert_eq!(general.processing_mode, "polish");

    let verbatim = personas
        .iter()
        .find(|persona| persona.id == "verbatim")
        .expect("verbatim persona should be seeded");
    assert_eq!(verbatim.name, "原文听写");
    assert_eq!(verbatim.processing_mode, "verbatim");
    assert!(!verbatim.is_default);

    let update_error = database
        .update_persona(
            GENERAL_PERSONA_ID,
            PersonaDraft {
                name: "已修改".to_string(),
                description: "已修改".to_string(),
                icon: "Bot".to_string(),
                processing_mode: "polish".to_string(),
            },
        )
        .expect_err("general persona should not be editable");
    assert!(update_error.contains("不可修改"));

    let delete_error = database
        .delete_persona(GENERAL_PERSONA_ID)
        .expect_err("general persona should not be deletable");
    assert!(delete_error.contains("不可删除"));
}

#[test]
fn existing_database_adds_general_persona_without_changing_default() {
    let path = temp_db_path("general-persona-existing-default");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch(
            r#"
            CREATE TABLE personas (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                icon TEXT NOT NULL DEFAULT 'Sparkles',
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            INSERT INTO personas (id, name, description, icon, is_default)
            VALUES ('prompt-engineer', 'Prompt 工程师', '旧描述', 'Bot', 1);
            "#,
        )
        .unwrap();
    drop(connection);

    let database = open_test_database(&path);
    let personas = database.list_personas().expect("personas should load");
    let general = personas
        .iter()
        .find(|persona| persona.id == GENERAL_PERSONA_ID)
        .expect("general persona should be added");
    let prompt_engineer = personas
        .iter()
        .find(|persona| persona.id == "prompt-engineer")
        .expect("existing persona should remain");

    assert!(!general.is_default);
    assert_eq!(general.processing_mode, "polish");
    assert!(prompt_engineer.is_default);
    assert_eq!(prompt_engineer.processing_mode, "polish");
    assert!(personas
        .iter()
        .any(|persona| persona.id == "verbatim" && persona.processing_mode == "verbatim"));

    database
        .initialize()
        .expect("migration should be idempotent");
    let verbatim_count = database
        .list_personas()
        .expect("personas should reload")
        .iter()
        .filter(|persona| persona.id == "verbatim")
        .count();
    assert_eq!(verbatim_count, 1);
}

#[test]
fn custom_personas_roundtrip_the_selected_processing_mode() {
    let database = open_test_database(&temp_db_path("persona-processing-mode"));

    let created = database
        .create_persona(PersonaDraft {
            name: "不润色".to_string(),
            description: "保留识别结果".to_string(),
            icon: "FileText".to_string(),
            processing_mode: "verbatim".to_string(),
        })
        .expect("persona should be created");

    assert_eq!(created.processing_mode, "verbatim");
}

#[test]
fn default_persona_can_be_changed_and_persisted() {
    let database = open_test_database(&temp_db_path("default-persona-change"));

    let initial_personas = database.list_personas().expect("personas should load");
    let initial_order = initial_personas
        .iter()
        .map(|persona| persona.id.clone())
        .collect::<Vec<_>>();
    let general = initial_personas
        .iter()
        .find(|persona| persona.id == GENERAL_PERSONA_ID)
        .expect("general persona should exist");
    let task_collaborator = initial_personas
        .iter()
        .find(|persona| persona.id == "task-collaborator")
        .expect("task collaborator should exist");

    assert!(general.is_default);
    assert!(!task_collaborator.is_default);

    database
        .set_default_persona("task-collaborator")
        .expect("default persona should update");

    let personas = database.list_personas().expect("personas should reload");
    let updated_order = personas
        .iter()
        .map(|persona| persona.id.clone())
        .collect::<Vec<_>>();
    let general = personas
        .iter()
        .find(|persona| persona.id == GENERAL_PERSONA_ID)
        .expect("general persona should still exist");
    let task_collaborator = personas
        .iter()
        .find(|persona| persona.id == "task-collaborator")
        .expect("task collaborator should still exist");

    assert!(!general.is_default);
    assert!(task_collaborator.is_default);
    assert_eq!(updated_order, initial_order);
}

#[test]
fn ensure_default_persona_repairs_missing_and_multiple_defaults() {
    let database = open_test_database(&temp_db_path("repair-default-persona"));
    let connection = rusqlite::Connection::open(database.path()).unwrap();
    connection
        .execute("UPDATE personas SET is_default = 0", [])
        .unwrap();

    let repaired = database
        .ensure_default_persona("missing-persona")
        .expect("missing config persona should fall back to general");
    assert_eq!(repaired.id, GENERAL_PERSONA_ID);

    connection
        .execute(
            "UPDATE personas SET is_default = 1 WHERE id IN ('general', 'verbatim')",
            [],
        )
        .unwrap();
    let repaired = database
        .ensure_default_persona("verbatim")
        .expect("existing config persona should win");
    assert_eq!(repaired.id, "verbatim");
    assert_eq!(
        database
            .list_personas()
            .unwrap()
            .into_iter()
            .filter(|persona| persona.is_default)
            .count(),
        1
    );
}

#[test]
fn conditional_delete_cannot_remove_a_persona_made_default_by_another_connection() {
    let database = open_test_database(&temp_db_path("conditional-default-delete"));
    let connection = rusqlite::Connection::open(database.path()).unwrap();
    connection
        .execute("UPDATE personas SET is_default = 0", [])
        .unwrap();
    connection
        .execute(
            "UPDATE personas SET is_default = 1 WHERE id = 'verbatim'",
            [],
        )
        .unwrap();

    let error = database
        .delete_persona("verbatim")
        .expect_err("production deletion must reject a persona made default by another connection");
    assert!(error.contains("默认人格不可删除"));
    assert!(database
        .list_personas()
        .unwrap()
        .iter()
        .any(|persona| persona.id == "verbatim" && persona.is_default));
}
