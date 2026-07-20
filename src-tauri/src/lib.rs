pub mod app_migration;
pub mod asr;
pub mod audio_control;
pub mod bindings;
pub mod capture_session;
pub mod credentials;
pub mod data;
pub mod events;
pub mod focus_capture;
pub mod history_reprocessing;
pub mod hotkey;
pub mod indicator;
pub mod local_asr;
pub mod local_asr_model;
pub mod macos_permissions;
pub mod output;
pub mod pipeline;
pub mod readiness;
pub mod recording;
pub mod recording_storage;
pub mod text_polish;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let bindings = bindings::builder();

    let event_bindings = bindings.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(capture_session::CaptureSessionState::new())
        .manage(recording::RecordingState::new())
        .setup(move |app| {
            event_bindings.mount_events(app);
            app_migration::migrate_legacy_app_data(app.handle())?;
            indicator::ensure_indicator(app.handle())?;
            // 初始化快捷键状态
            app.manage(Arc::new(Mutex::new(hotkey::HotkeyState::default())));

            // 从配置读取并注册默认快捷键
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match data::read_app_config(app_handle.clone()) {
                    Ok(config) => {
                        println!(
                            "读取到配置: longpress={}, toggle={}",
                            config.longpress_shortcut, config.toggle_shortcut
                        );

                        let longpress = if config.longpress_shortcut.is_empty() {
                            None
                        } else {
                            Some(config.longpress_shortcut.clone())
                        };
                        let toggle = if config.toggle_shortcut.is_empty() {
                            None
                        } else {
                            Some(config.toggle_shortcut.clone())
                        };

                        match hotkey::register_both_hotkeys(app_handle, longpress, toggle).await {
                            Ok(_) => println!("快捷键注册成功"),
                            Err(e) => eprintln!("快捷键注册失败: {}", e),
                        }
                    }
                    Err(e) => {
                        eprintln!("读取配置失败: {}", e);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(bindings.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
