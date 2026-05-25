use std::{
    fmt,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::{
    asr::{transcribe_audio_file, AsrConfig},
    data::{HistoryRecord, HistoryRecordDraft, LocalDatabase, Persona},
    text_polish::{polish_text_with_openai, OpenAiTextConfig, TextPolishRequest},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceInputRequest {
    pub audio_bytes: Vec<u8>,
    pub audio_extension: String,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceInputResult {
    pub raw_text: String,
    pub final_text: String,
    pub used_text_fallback: bool,
    pub history_record: Option<HistoryRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceInputError {
    EmptyAudio,
    UnsupportedAudioExtension(String),
    MissingDefaultPersona,
    RequestFailed(String),
}

impl fmt::Display for VoiceInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyAudio => write!(formatter, "音频文件不能为空"),
            Self::UnsupportedAudioExtension(extension) => {
                write!(
                    formatter,
                    "仅支持 wav 或 mp3 音频文件，当前格式：{extension}"
                )
            }
            Self::MissingDefaultPersona => write!(formatter, "默认人格不存在"),
            Self::RequestFailed(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for VoiceInputError {}

pub fn prepare_uploaded_audio_file(
    audio_bytes: Vec<u8>,
    audio_extension: &str,
) -> Result<PathBuf, VoiceInputError> {
    if audio_bytes.is_empty() {
        return Err(VoiceInputError::EmptyAudio);
    }

    let extension = audio_extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    if extension != "wav" && extension != "mp3" {
        return Err(VoiceInputError::UnsupportedAudioExtension(extension));
    }

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| VoiceInputError::RequestFailed(error.to_string()))?
        .as_nanos();
    let path = std::env::temp_dir()
        .join("xiluolin-audio")
        .join(format!("voice-input-{nanos}.{extension}"));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| VoiceInputError::RequestFailed(error.to_string()))?;
    }
    std::fs::write(&path, audio_bytes)
        .map_err(|error| VoiceInputError::RequestFailed(error.to_string()))?;

    Ok(path)
}

pub fn process_voice_input(
    request: VoiceInputRequest,
    asr_config: AsrConfig,
    openai_config: OpenAiTextConfig,
    database: &LocalDatabase,
    auto_save_history: bool,
    output_mode: &str,
) -> Result<VoiceInputResult, VoiceInputError> {
    let audio_path = prepare_uploaded_audio_file(request.audio_bytes, &request.audio_extension)?;
    let transcription = transcribe_audio_file(&audio_path, &asr_config)
        .map_err(|error| VoiceInputError::RequestFailed(error.to_string()));
    let _ = std::fs::remove_file(&audio_path);
    let transcription = transcription?;

    let persona = default_persona(database)?;
    let hotword_context = database
        .enabled_hotword_context()
        .map_err(|error| VoiceInputError::RequestFailed(error.to_string()))?;
    let polish_result = polish_text_with_openai(
        &TextPolishRequest {
            raw_text: transcription.text.clone(),
            persona_description: persona.description.clone(),
            hotword_context,
        },
        &openai_config,
    )
    .map_err(|error| VoiceInputError::RequestFailed(error.to_string()))?;

    let history_record = if auto_save_history {
        Some(
            database
                .create_history_record(HistoryRecordDraft {
                    raw_text: transcription.text.clone(),
                    final_text: polish_result.final_text.clone(),
                    persona_id: persona.id,
                    persona_name: persona.name,
                    duration_ms: request.duration_ms.max(0),
                    output_mode: output_mode.to_string(),
                })
                .map_err(|error| VoiceInputError::RequestFailed(error.to_string()))?,
        )
    } else {
        None
    };

    Ok(VoiceInputResult {
        raw_text: transcription.text,
        final_text: polish_result.final_text,
        used_text_fallback: polish_result.used_fallback,
        history_record,
    })
}

fn default_persona(database: &LocalDatabase) -> Result<Persona, VoiceInputError> {
    database
        .list_personas()
        .map_err(|error| VoiceInputError::RequestFailed(error.to_string()))?
        .into_iter()
        .find(|persona| persona.is_default)
        .ok_or(VoiceInputError::MissingDefaultPersona)
}

#[tauri::command]
pub fn process_uploaded_audio(
    app: tauri::AppHandle,
    request: VoiceInputRequest,
) -> Result<VoiceInputResult, String> {
    use crate::data::{read_app_config, LocalDatabase};
    use tauri::Manager;

    let config = read_app_config(app.clone())?;
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&app_data_dir).map_err(|error| error.to_string())?;
    let database = LocalDatabase::open(app_data_dir.join("xiluolin.sqlite"))
        .map_err(|error| error.to_string())?;
    database.initialize().map_err(|error| error.to_string())?;

    process_voice_input(
        request,
        AsrConfig {
            provider: config.asr_provider.clone(),
            api_key: config.asr_api_key,
            base_url: config.asr_base_url,
            model: config.asr_model,
        },
        OpenAiTextConfig {
            api_key: config.openai_api_key,
            base_url: config.openai_base_url,
            model: config.openai_model,
        },
        &database,
        config.auto_save_history,
        &config.output_mode,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn process_recording_file(
    app: tauri::AppHandle,
    file_path: String,
    duration_ms: i64,
) -> Result<VoiceInputResult, String> {
    use crate::data::{read_app_config, LocalDatabase};
    use tauri::Manager;

    // 读取录音文件
    let audio_bytes = std::fs::read(&file_path).map_err(|error| error.to_string())?;

    // 获取文件扩展名
    let audio_extension = std::path::Path::new(&file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("wav")
        .to_string();

    let config = read_app_config(app.clone())?;

    // 调试：打印配置中的 API Key
    eprintln!("=== 配置调试信息 ===");
    eprintln!("ASR API Key 长度: {}", config.asr_api_key.len());
    eprintln!("ASR API Key: {}", config.asr_api_key);
    eprintln!("ASR Base URL: {}", config.asr_base_url);
    eprintln!("ASR Model: {}", config.asr_model);
    eprintln!("===================");

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&app_data_dir).map_err(|error| error.to_string())?;
    let database = LocalDatabase::open(app_data_dir.join("xiluolin.sqlite"))
        .map_err(|error| error.to_string())?;
    database.initialize().map_err(|error| error.to_string())?;

    process_voice_input(
        VoiceInputRequest {
            audio_bytes,
            audio_extension,
            duration_ms,
        },
        AsrConfig {
            provider: config.asr_provider.clone(),
            api_key: config.asr_api_key,
            base_url: config.asr_base_url,
            model: config.asr_model,
        },
        OpenAiTextConfig {
            api_key: config.openai_api_key,
            base_url: config.openai_base_url,
            model: config.openai_model,
        },
        &database,
        config.auto_save_history,
        &config.output_mode,
    )
    .map_err(|error| error.to_string())
}
