use tauri::Manager;

use crate::{
    data::{read_app_config, HistoryRecord, LocalDatabase, VERBATIM_PROCESSING_MODE},
    pipeline::{
        normalize_verbatim_text, process_voice_input_routed, HistoryContext, VoiceInputRequest,
        VoiceInputResult,
    },
    providers::text::{default_text_registry, route_text, TextInput},
    recording_storage::read_managed_recording,
};

fn database_for_app(app: &tauri::AppHandle) -> Result<LocalDatabase, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    let database = LocalDatabase::open(app_data_dir.join("xiluolin.sqlite"))
        .map_err(|error| error.to_string())?;
    database.initialize().map_err(|error| error.to_string())?;
    Ok(database)
}

fn default_persona(database: &LocalDatabase) -> Result<crate::data::Persona, String> {
    database
        .list_personas()
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|persona| persona.is_default)
        .ok_or_else(|| "默认人格不存在".to_string())
}

pub fn persist_reprocessed_history(
    database: &LocalDatabase,
    history_id: &str,
    result: &VoiceInputResult,
) -> Result<HistoryRecord, String> {
    database
        .update_history_after_transcription(
            history_id,
            &result.raw_text,
            &result.final_text,
            &result.actual_persona_id,
            &result.actual_persona_name,
            &result.actual_asr_provider,
            &result.actual_asr_model,
            &result.actual_text_provider,
            &result.actual_text_model,
            &result.text_processing_mode,
            result.used_asr_fallback,
            result.used_text_fallback,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn read_retained_recording(
    app: tauri::AppHandle,
    history_id: String,
) -> Result<Vec<u8>, String> {
    let database = database_for_app(&app)?;
    let audio_path = database
        .history_audio_path(&history_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "该历史记录没有保留录音".to_string())?;
    read_managed_recording(&app, &audio_path)
}

#[tauri::command]
#[specta::specta]
pub fn reprocess_history_audio(
    app: tauri::AppHandle,
    history_id: String,
) -> Result<HistoryRecord, String> {
    let database = database_for_app(&app)?;
    let existing = database
        .get_history_record(&history_id)
        .map_err(|error| error.to_string())?;
    let audio_path = existing
        .audio_path
        .clone()
        .ok_or_else(|| "该历史记录没有保留录音".to_string())?;
    let audio_bytes = read_managed_recording(&app, &audio_path)?;
    let config = read_app_config(app.clone())?;
    let local_model_path = if config
        .asr
        .provider_ids()
        .any(|provider| provider == "local")
    {
        Some(crate::local_asr_model::model_path(&app)?)
    } else {
        None
    };
    let text_provider = config.text.primary.clone();
    let text_model = config
        .selected_text_settings()
        .map(|settings| settings.model.clone())
        .unwrap_or_default();

    let result = process_voice_input_routed(
        VoiceInputRequest {
            audio_bytes,
            audio_extension: "wav".to_string(),
            duration_ms: existing.duration_ms,
        },
        config.asr,
        config.text,
        local_model_path,
        &database,
        false,
        HistoryContext {
            source: "reprocess".to_string(),
            text_provider: text_provider.clone(),
            text_model: text_model.clone(),
            audio_path: None,
        },
    )
    .map_err(|error| error.to_string())?;

    persist_reprocessed_history(&database, &history_id, &result)
}

#[tauri::command]
#[specta::specta]
pub fn refine_history_text(
    app: tauri::AppHandle,
    history_id: String,
) -> Result<HistoryRecord, String> {
    let database = database_for_app(&app)?;
    let existing = database
        .get_history_record(&history_id)
        .map_err(|error| error.to_string())?;
    let config = read_app_config(app)?;
    let persona = default_persona(&database)?;
    let (final_text, used_fallback, record_text_provider, record_text_model) =
        if persona.processing_mode == VERBATIM_PROCESSING_MODE {
            (
                normalize_verbatim_text(&existing.raw_text),
                false,
                String::new(),
                String::new(),
            )
        } else {
            let hotword_context = database
                .enabled_hotword_context()
                .map_err(|error| error.to_string())?;
            let result = route_text(
                &TextInput {
                    raw_text: existing.raw_text,
                    persona_id: persona.id.clone(),
                    persona_description: persona.description.clone(),
                    hotword_context,
                },
                &config.text,
                &default_text_registry(),
            )
            .map_err(|error| error.to_string())?;
            (
                result.output.text,
                result.used_text_fallback,
                result.output.provider,
                result.output.model,
            )
        };

    database
        .update_history_after_refinement(
            &history_id,
            &final_text,
            &persona.id,
            &persona.name,
            &record_text_provider,
            &record_text_model,
            &persona.processing_mode,
            used_fallback,
        )
        .map_err(|error| error.to_string())
}
