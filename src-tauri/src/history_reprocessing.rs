use tauri::Manager;

use crate::{
    asr::AsrConfig,
    data::{read_app_config, HistoryRecord, LocalDatabase},
    pipeline::{process_voice_input, HistoryContext, VoiceInputRequest},
    recording_storage::read_managed_recording,
    text_polish::{polish_text_with_openai, TextPolishConfig, TextPolishRequest},
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

fn selected_models(
    config: &crate::data::AppConfig,
) -> (String, String, String, String, String, String) {
    let (asr_api_key, asr_base_url, asr_model) = config.selected_asr_config();
    let (text_api_key, text_base_url, text_model) = config.selected_text_config();
    (
        asr_api_key.to_string(),
        asr_base_url.to_string(),
        asr_model.to_string(),
        text_api_key.to_string(),
        text_base_url.to_string(),
        text_model.to_string(),
    )
}

fn default_persona(database: &LocalDatabase) -> Result<crate::data::Persona, String> {
    database
        .list_personas()
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|persona| persona.is_default)
        .ok_or_else(|| "默认人格不存在".to_string())
}

#[tauri::command]
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
    let persona = default_persona(&database)?;
    let (asr_api_key, asr_base_url, asr_model, text_api_key, text_base_url, text_model) =
        selected_models(&config);
    let text_provider = config.text_provider.clone();

    let result = process_voice_input(
        VoiceInputRequest {
            audio_bytes,
            audio_extension: "wav".to_string(),
            duration_ms: existing.duration_ms,
        },
        AsrConfig {
            provider: config.asr_provider.clone(),
            api_key: asr_api_key,
            base_url: asr_base_url,
            model: asr_model.clone(),
        },
        TextPolishConfig {
            provider: text_provider.clone(),
            api_key: text_api_key,
            base_url: text_base_url,
            model: text_model.clone(),
        },
        &database,
        false,
        HistoryContext {
            source: "reprocess".to_string(),
            asr_provider: config.asr_provider.clone(),
            asr_model: asr_model.clone(),
            text_provider: text_provider.clone(),
            text_model: text_model.clone(),
            audio_path: None,
        },
    )
    .map_err(|error| error.to_string())?;

    database
        .update_history_after_transcription(
            &history_id,
            &result.raw_text,
            &result.final_text,
            &persona.id,
            &persona.name,
            &config.asr_provider,
            &asr_model,
            &text_provider,
            &text_model,
            result.used_text_fallback,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
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
    let hotword_context = database
        .enabled_hotword_context()
        .map_err(|error| error.to_string())?;
    let (_, _, _, text_api_key, text_base_url, text_model) = selected_models(&config);
    let text_provider = config.text_provider.clone();
    let result = polish_text_with_openai(
        &TextPolishRequest {
            raw_text: existing.raw_text,
            persona_description: persona.description.clone(),
            hotword_context,
        },
        &TextPolishConfig {
            provider: text_provider.clone(),
            api_key: text_api_key,
            base_url: text_base_url,
            model: text_model.clone(),
        },
    )
    .map_err(|error| error.to_string())?;

    database
        .update_history_after_refinement(
            &history_id,
            &result.final_text,
            &persona.id,
            &persona.name,
            &text_provider,
            &text_model,
            result.used_fallback,
        )
        .map_err(|error| error.to_string())
}
