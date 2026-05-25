pub mod asr;
pub mod data;
pub mod hotkey;
pub mod output;
pub mod pipeline;
pub mod recording;
pub mod text_polish;

use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // 初始化快捷键状态
            app.manage(Arc::new(Mutex::new(hotkey::HotkeyState::default())));

            // 从配置读取并注册默认快捷键
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(config) = data::read_app_config(app_handle.clone()) {
                    // 根据录音模式注册对应的快捷键
                    let shortcut = if config.recording_mode == "long_press" {
                        config.longpress_shortcut
                    } else {
                        config.toggle_shortcut
                    };
                    let _ = hotkey::register_hotkey(
                        app_handle,
                        shortcut,
                        config.recording_mode,
                    ).await;
                }
            });

            Ok(())
        })
        .manage(recording::RecordingState::new())
        .invoke_handler(tauri::generate_handler![
            asr::transcribe_audio_path,
            text_polish::polish_text,
            pipeline::process_uploaded_audio,
            pipeline::process_recording_file,
            data::initialize_local_data,
            data::list_personas,
            data::set_default_persona,
            data::create_persona,
            data::update_persona,
            data::delete_persona,
            data::create_hotword,
            data::list_hotwords,
            data::update_hotword,
            data::delete_hotword,
            data::enabled_hotword_context,
            data::create_history_record,
            data::list_history_records,
            data::history_statistics,
            data::delete_history_record,
            data::read_app_config,
            data::update_app_config,
            recording::start_recording,
            recording::stop_recording,
            recording::list_audio_devices,
            hotkey::register_hotkey,
            hotkey::unregister_hotkey,
            output::output_text,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
