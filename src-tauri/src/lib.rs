pub mod asr;
pub mod data;
pub mod text_polish;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            asr::transcribe_audio_path,
            text_polish::polish_text,
            data::initialize_local_data,
            data::list_personas,
            data::set_default_persona,
            data::create_hotword,
            data::list_hotwords,
            data::update_hotword,
            data::delete_hotword,
            data::enabled_hotword_context,
            data::create_history_record,
            data::list_history_records,
            data::read_app_config,
            data::update_app_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
