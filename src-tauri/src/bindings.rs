use tauri_specta::{collect_commands, collect_events, Builder, ErrorHandlingMode};

use crate::{
    asr, capture_session, data, events, history_reprocessing, hotkey, indicator, local_asr_model,
    macos_permissions, output, pipeline, readiness, recording, recording_storage, text_polish,
};

pub fn builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .error_handling(ErrorHandlingMode::Throw)
        .commands(collect_commands![
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
            history_reprocessing::read_retained_recording,
            history_reprocessing::reprocess_history_audio,
            history_reprocessing::refine_history_text,
            data::read_app_config,
            data::update_app_config,
            recording::start_recording,
            recording::stop_recording,
            recording::list_audio_devices,
            readiness::read_input_readiness,
            local_asr_model::local_asr_model_info,
            local_asr_model::download_local_asr_model,
            local_asr_model::delete_local_asr_model,
            local_asr_model::verify_local_asr_model,
            macos_permissions::request_macos_permission,
            macos_permissions::open_macos_privacy_settings,
            recording_storage::recording_storage_info,
            recording_storage::open_recordings_directory,
            recording_storage::clear_retained_recordings,
            hotkey::register_hotkey,
            hotkey::register_both_hotkeys,
            hotkey::unregister_hotkey,
            indicator::update_indicator_status,
            output::deliver_text,
        ])
        .events(collect_events![
            events::RecordingCompletedEvent,
            events::RecordingErrorEvent,
            events::LocalAsrDownloadProgressEvent,
        ])
}
