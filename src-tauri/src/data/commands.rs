use super::{database::LocalDatabase, models::*};
use tauri::Manager;
use tauri_specta::Event;

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
    let personas = database
        .list_personas()
        .map_err(|error| error.to_string())?;
    let previous_persona_id = personas
        .iter()
        .find(|persona| persona.is_default)
        .map(|persona| persona.id.clone())
        .ok_or_else(|| "默认人格不存在".to_string())?;
    let personas = database
        .set_default_persona_and_list(persona_id)
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
    Ok(DefaultPersonaUpdate { personas, config })
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
    let stored_config = match store.get(APP_CONFIG_KEY) {
        Some(value) => serde_json::from_value(value.clone()).map_err(|error| error.to_string())?,
        None => default_app_config(),
    };
    let mut config = migrate_config_to_v2(stored_config.clone())?;

    let legacy_credentials = AppCredentials::from_config(&config);
    let credentials = load_system_credentials(&legacy_credentials)?;
    let sanitized = sanitized_config(&config);

    if sanitized != stored_config {
        let value = serde_json::to_value(&sanitized).map_err(|error| error.to_string())?;
        save_store_value_transactionally(
            store.get(APP_CONFIG_KEY),
            value,
            |value| store.set(APP_CONFIG_KEY.to_string(), value),
            || {
                store.delete(APP_CONFIG_KEY);
            },
            || store.save().map_err(|error| error.to_string()),
        )?;
    }

    config = sanitized;
    credentials.apply_to(&mut config);
    Ok(config)
}

#[tauri::command]
#[specta::specta]
pub fn update_app_config(app: tauri::AppHandle, config: AppConfig) -> Result<AppConfig, String> {
    use crate::credentials::{
        load_system_credentials, sanitized_config, save_system_credentials, AppCredentials,
    };
    use tauri_plugin_store::StoreExt;

    let credentials = AppCredentials::from_config(&config);
    let previous_credentials = load_system_credentials(&AppCredentials::default())?;
    save_system_credentials(&credentials)?;

    let store = app
        .store(APP_CONFIG_STORE)
        .map_err(|error| error.to_string())?;
    let persisted_config = sanitized_config(&config);
    let value = serde_json::to_value(&persisted_config).map_err(|error| error.to_string())?;
    if let Err(config_error) = save_store_value_transactionally(
        store.get(APP_CONFIG_KEY),
        value,
        |value| store.set(APP_CONFIG_KEY.to_string(), value),
        || {
            store.delete(APP_CONFIG_KEY);
        },
        || store.save().map_err(|error| error.to_string()),
    ) {
        return match save_system_credentials(&previous_credentials) {
            Ok(()) => Err(config_error),
            Err(restore_error) => Err(format!(
                "配置保存失败且凭据恢复失败：{config_error}; {restore_error}"
            )),
        };
    }

    // Fn 监听按同步配置顺序更新，避免连续保存时较早的异步任务覆盖最新开关状态。
    let fn_manager = app.state::<crate::macos_fn::FnHoldManager>();
    if let Err(error) =
        crate::macos_fn::configure_fn_hold(&app, &fn_manager, config.fn_hold_enabled)
    {
        eprintln!("独立 Fn 录音热更新失败：{error}");
        let _ = crate::events::RecordingErrorEvent(error).emit(&app);
    }

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

fn save_store_value_transactionally<T: Clone>(
    previous_value: Option<T>,
    new_value: T,
    mut set: impl FnMut(T),
    mut remove: impl FnMut(),
    mut save: impl FnMut() -> Result<(), String>,
) -> Result<(), String> {
    set(new_value);
    if let Err(primary_error) = save() {
        match previous_value {
            Some(previous_value) => set(previous_value),
            None => remove(),
        }
        if let Err(restore_error) = save() {
            return Err(format!(
                "配置保存失败且磁盘恢复失败；内存配置已恢复：{primary_error}; {restore_error}"
            ));
        }
        return Err(primary_error);
    }
    Ok(())
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
    use std::cell::{Cell, RefCell};

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
        let cache = RefCell::new(config.clone());
        let fail_next_save = Cell::new(true);

        let error = set_default_persona_with_persistence(&database, config, "verbatim", |next| {
            let previous = cache.borrow().clone();
            save_store_value_transactionally(
                Some(previous),
                next,
                |value| *cache.borrow_mut() = value,
                || unreachable!("the pre-existing config must be restored"),
                || {
                    if fail_next_save.replace(false) {
                        Err("injected config write failure".to_string())
                    } else {
                        Ok(())
                    }
                },
            )
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
        assert_eq!(cache.borrow().default_persona_id, GENERAL_PERSONA_ID);
        let reconciled =
            reconcile_initialized_local_data(&database, cache.into_inner(), |_| Ok(()))
                .expect("the restored cache must not reapply the failed default persona");
        assert_eq!(reconciled.default_persona_id, GENERAL_PERSONA_ID);
    }

    #[test]
    fn failed_store_save_restores_the_mutated_cache_before_returning_error() {
        let value = RefCell::new(Some("old-default".to_string()));
        let fail_next_save = Cell::new(true);
        let previous = value.borrow().clone();

        let error = save_store_value_transactionally(
            previous,
            "new-default".to_string(),
            |next| *value.borrow_mut() = Some(next),
            || *value.borrow_mut() = None,
            || {
                if fail_next_save.replace(false) {
                    Err("injected disk save failure".to_string())
                } else {
                    Ok(())
                }
            },
        )
        .expect_err("a failed save should be returned after the cache is restored");

        assert_eq!(error, "injected disk save failure");
        assert_eq!(value.borrow().as_deref(), Some("old-default"));
    }

    #[test]
    fn failed_first_store_save_removes_a_new_cache_entry() {
        let value = RefCell::new(None::<String>);
        let save_count = Cell::new(0);

        let error = save_store_value_transactionally(
            None,
            "new-default".to_string(),
            |next| *value.borrow_mut() = Some(next),
            || *value.borrow_mut() = None,
            || {
                let attempt = save_count.get();
                save_count.set(attempt + 1);
                if attempt == 0 {
                    Err("injected disk save failure".to_string())
                } else {
                    Ok(())
                }
            },
        )
        .expect_err("a failed first save should remove a newly inserted cache entry");

        assert_eq!(error, "injected disk save failure");
        assert_eq!(*value.borrow(), None);
        assert_eq!(save_count.get(), 2);
    }

    #[test]
    fn failed_restore_save_reports_both_errors_after_restoring_cache() {
        let value = RefCell::new(Some("old-default".to_string()));
        let previous = value.borrow().clone();

        let error = save_store_value_transactionally(
            previous,
            "new-default".to_string(),
            |next| *value.borrow_mut() = Some(next),
            || *value.borrow_mut() = None,
            || Err("disk unavailable".to_string()),
        )
        .expect_err("both the primary and restore save failures should be reported");

        assert!(error.contains("配置保存失败且磁盘恢复失败"));
        assert!(error.matches("disk unavailable").count() >= 2);
        assert_eq!(value.borrow().as_deref(), Some("old-default"));
    }

    #[test]
    fn successful_default_persona_update_returns_prepared_personas_and_config() {
        let database = test_database("set-default-success");
        let connection = rusqlite::Connection::open(database.path()).unwrap();
        connection
            .execute(
                "UPDATE personas SET updated_at = '2000-01-01 00:00:00' WHERE id = ?1",
                [VERBATIM_PERSONA_ID],
            )
            .unwrap();
        let update = set_default_persona_with_persistence(
            &database,
            default_app_config(),
            VERBATIM_PERSONA_ID,
            |_| Ok(()),
        )
        .expect("successful persistence should return the authoritative update");

        assert_eq!(update.config.default_persona_id, VERBATIM_PERSONA_ID);
        assert_eq!(
            update
                .personas
                .iter()
                .find(|persona| persona.is_default)
                .unwrap()
                .id,
            VERBATIM_PERSONA_ID
        );
        let authoritative_personas = database.list_personas().unwrap();
        assert_eq!(update.personas, authoritative_personas);
        assert_ne!(
            update
                .personas
                .iter()
                .find(|persona| persona.id == VERBATIM_PERSONA_ID)
                .unwrap()
                .updated_at,
            "2000-01-01 00:00:00"
        );
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
