use super::{database::LocalDatabase, models::*};

#[tauri::command]
#[specta::specta]
pub fn initialize_local_data(app: tauri::AppHandle) -> Result<AppConfig, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    let config = read_app_config(app.clone())?;
    reconcile_initialized_local_data(&database, config, |config| {
        update_app_config(app.clone(), config.clone()).map(|_| ())
    })
}

fn reconcile_initialized_local_data(
    database: &LocalDatabase,
    mut config: AppConfig,
    persist_config: impl FnOnce(&AppConfig) -> Result<(), String>,
) -> Result<AppConfig, String> {
    let default_persona = database
        .ensure_default_persona(&config.default_persona_id)
        .map_err(|error| error.to_string())?;
    if config.default_persona_id != default_persona.id {
        config.default_persona_id = default_persona.id;
        persist_config(&config)?;
    }
    Ok(config)
}

#[tauri::command]
#[specta::specta]
pub fn list_personas(app: tauri::AppHandle) -> Result<Vec<Persona>, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database.list_personas().map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn set_default_persona(
    app: tauri::AppHandle,
    persona_id: String,
) -> Result<DefaultPersonaUpdate, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    let config = read_app_config(app.clone())?;
    set_default_persona_with_persistence(&database, config, &persona_id, |config| {
        update_app_config(app.clone(), config).map(|_| ())
    })
}

fn set_default_persona_with_persistence(
    database: &LocalDatabase,
    previous_config: AppConfig,
    persona_id: &str,
    persist_config: impl FnOnce(AppConfig) -> Result<(), String>,
) -> Result<DefaultPersonaUpdate, String> {
    let previous_persona_id = database
        .list_personas()
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|persona| persona.is_default)
        .map(|persona| persona.id)
        .ok_or_else(|| "默认人格不存在".to_string())?;
    database
        .set_default_persona(persona_id)
        .map_err(|error| error.to_string())?;
    let mut config = previous_config;
    config.default_persona_id = persona_id.to_string();
    if let Err(error) = persist_config(config.clone()) {
        return match database.set_default_persona(&previous_persona_id) {
            Ok(()) => Err(format!("默认人格配置保存失败，已回滚：{error}")),
            Err(rollback_error) => Err(format!(
                "默认人格配置保存失败且数据库回滚失败，需要重新初始化修复：{error}; {rollback_error}"
            )),
        };
    }
    Ok(DefaultPersonaUpdate {
        personas: database
            .list_personas()
            .map_err(|error| error.to_string())?,
        config,
    })
}

#[tauri::command]
#[specta::specta]
pub fn create_persona(app: tauri::AppHandle, draft: PersonaDraft) -> Result<Persona, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .create_persona(draft)
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
pub fn delete_persona(app: tauri::AppHandle, id: String) -> Result<Vec<Persona>, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .delete_persona(&id)
        .map_err(|error| error.to_string())?;
    database.list_personas().map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn create_hotword(app: tauri::AppHandle, draft: HotwordDraft) -> Result<Hotword, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .create_hotword(draft)
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn list_hotwords(app: tauri::AppHandle) -> Result<Vec<Hotword>, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database.list_hotwords().map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
pub fn delete_hotword(app: tauri::AppHandle, id: String) -> Result<Vec<Hotword>, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .delete_hotword(&id)
        .map_err(|error| error.to_string())?;
    database.list_hotwords().map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn enabled_hotword_context(app: tauri::AppHandle) -> Result<String, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .enabled_hotword_context()
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
pub fn list_history_records(
    app: tauri::AppHandle,
    limit: Option<i32>,
) -> Result<Vec<HistoryRecord>, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .list_history_records(i64::from(limit.unwrap_or(20)))
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn history_statistics(app: tauri::AppHandle) -> Result<HistoryStatistics, String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .history_statistics()
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn delete_history_record(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let database = database_for_app(&app)?;
    database.initialize().map_err(|error| error.to_string())?;
    let audio_path = database
        .history_audio_path(&id)
        .map_err(|error| error.to_string())?;
    if let Some(audio_path) = audio_path {
        crate::recording_storage::remove_managed_recording(&app, &audio_path)?;
    }
    database
        .delete_history_record(&id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn read_app_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    use crate::credentials::{load_system_credentials, sanitized_config, AppCredentials};
    use tauri_plugin_store::StoreExt;

    let store = app
        .store(APP_CONFIG_STORE)
        .map_err(|error| error.to_string())?;
    let mut config = match store.get(APP_CONFIG_KEY) {
        Some(value) => serde_json::from_value(value.clone()).map_err(|error| error.to_string())?,
        None => default_app_config(),
    };

    let legacy_credentials = AppCredentials::from_config(&config);
    let credentials = load_system_credentials(&legacy_credentials)?;
    let sanitized = sanitized_config(&config);

    if sanitized != config {
        let value = serde_json::to_value(&sanitized).map_err(|error| error.to_string())?;
        store.set(APP_CONFIG_KEY.to_string(), value);
        store.save().map_err(|error| error.to_string())?;
    }

    config = sanitized;
    credentials.apply_to(&mut config);
    Ok(config)
}

#[tauri::command]
#[specta::specta]
pub fn update_app_config(app: tauri::AppHandle, config: AppConfig) -> Result<AppConfig, String> {
    use crate::credentials::{sanitized_config, save_system_credentials, AppCredentials};
    use tauri_plugin_store::StoreExt;

    let credentials = AppCredentials::from_config(&config);
    save_system_credentials(&credentials)?;

    let store = app
        .store(APP_CONFIG_STORE)
        .map_err(|error| error.to_string())?;
    let persisted_config = sanitized_config(&config);
    let value = serde_json::to_value(&persisted_config).map_err(|error| error.to_string())?;
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

pub fn update_history_delivery_for_app(
    app: &tauri::AppHandle,
    history_id: &str,
    delivery_method: &str,
) -> Result<(), String> {
    let database = database_for_app(app)?;
    database.initialize().map_err(|error| error.to_string())?;
    database
        .update_history_delivery_method(history_id, delivery_method)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_database(label: &str) -> LocalDatabase {
        let path = std::env::temp_dir().join(format!(
            "xiluolin-command-{label}-{}.sqlite",
            uuid::Uuid::new_v4()
        ));
        let database = LocalDatabase::open(path).unwrap();
        database.initialize().unwrap();
        database
    }

    #[test]
    fn failed_default_persona_persistence_restores_previous_database_default() {
        let database = test_database("set-default-rollback");
        let config = default_app_config();

        let error = set_default_persona_with_persistence(&database, config, "verbatim", |_| {
            Err("injected config write failure".to_string())
        })
        .expect_err("a configuration persistence failure should fail the command");

        assert!(error.contains("已回滚"));
        assert!(database
            .list_personas()
            .unwrap()
            .iter()
            .any(|persona| persona.id == GENERAL_PERSONA_ID && persona.is_default));
        assert!(!database
            .list_personas()
            .unwrap()
            .iter()
            .any(|persona| persona.id == VERBATIM_PERSONA_ID && persona.is_default));
    }

    #[test]
    fn reconciliation_persists_repaired_default_without_changing_other_config() {
        let database = test_database("initialize-reconciliation");
        let connection = rusqlite::Connection::open(database.path()).unwrap();
        connection
            .execute("UPDATE personas SET is_default = 0", [])
            .unwrap();
        let mut config = default_app_config();
        config.default_persona_id = "missing-persona".to_string();
        config.asr_model = "keep-asr-model".to_string();
        config.openai_model = "keep-text-model".to_string();
        config.retain_recordings = true;
        let mut persisted = None;

        let reconciled = reconcile_initialized_local_data(&database, config.clone(), |next| {
            persisted = Some(next.clone());
            Ok(())
        })
        .expect("reconciliation should repair an orphaned config persona id");

        assert_eq!(reconciled.default_persona_id, GENERAL_PERSONA_ID);
        assert_eq!(persisted, Some(reconciled.clone()));
        assert_eq!(reconciled.asr_model, config.asr_model);
        assert_eq!(reconciled.openai_model, config.openai_model);
        assert_eq!(reconciled.retain_recordings, config.retain_recordings);
        assert_eq!(
            database
                .list_personas()
                .unwrap()
                .iter()
                .filter(|persona| persona.is_default)
                .count(),
            1
        );

        connection
            .execute(
                "UPDATE personas SET is_default = 1 WHERE id = 'verbatim'",
                [],
            )
            .unwrap();
        let stable = reconcile_initialized_local_data(&database, reconciled, |_| Ok(()))
            .expect("reconciliation should collapse multiple defaults");
        assert_eq!(stable.default_persona_id, GENERAL_PERSONA_ID);
        assert_eq!(
            database
                .list_personas()
                .unwrap()
                .iter()
                .filter(|persona| persona.is_default)
                .count(),
            1
        );
    }
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
