pub mod asr;
pub mod audio_control;
pub mod capture_session;
pub mod credentials;
pub mod data;
pub mod focus_capture;
pub mod hotkey;
pub mod indicator;
pub mod output;
pub mod pipeline;
pub mod readiness;
pub mod recording;
pub mod text_polish;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(capture_session::CaptureSessionState::new())
        .manage(recording::RecordingState::new())
        .setup(|app| {
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
        .invoke_handler(tauri::generate_handler![
            asr::transcribe_audio_path,
            text_polish::polish_text,
            pipeline::process_uploaded_audio,
            pipeline::process_recording_file,
            capture_session::abort_capture_session,
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
            readiness::read_input_readiness,
            hotkey::register_hotkey,
            hotkey::register_both_hotkeys,
            hotkey::unregister_hotkey,
            indicator::update_indicator_status,
            output::deliver_text,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
