use std::{
    fmt,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::{
    asr::{build_asr_config, transcribe_audio_file, AsrConfig},
    capture_session::{CaptureSessionState, CaptureSource, CaptureStatus},
    data::{HistoryRecord, HistoryRecordDraft, LocalDatabase, Persona},
    indicator,
    text_polish::{polish_text_with_openai, TextPolishConfig, TextPolishRequest},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceInputRequest {
    pub audio_bytes: Vec<u8>,
    pub audio_extension: String,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryContext {
    pub source: String,
    pub text_provider: String,
    pub text_model: String,
    pub audio_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceInputResult {
    pub raw_text: String,
    pub final_text: String,
    pub actual_asr_provider: String,
    pub actual_asr_model: String,
    pub used_asr_fallback: bool,
    pub used_text_fallback: bool,
    pub history_record: Option<HistoryRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceInputStage {
    Transcribing,
    Refining,
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
    text_config: TextPolishConfig,
    database: &LocalDatabase,
    auto_save_history: bool,
    history_context: HistoryContext,
) -> Result<VoiceInputResult, VoiceInputError> {
    process_voice_input_with_progress(
        request,
        asr_config,
        text_config,
        database,
        auto_save_history,
        history_context,
        |_| {},
    )
}

pub fn process_voice_input_with_progress(
    request: VoiceInputRequest,
    asr_config: AsrConfig,
    text_config: TextPolishConfig,
    database: &LocalDatabase,
    auto_save_history: bool,
    history_context: HistoryContext,
    mut progress: impl FnMut(VoiceInputStage),
) -> Result<VoiceInputResult, VoiceInputError> {
    let start_time = std::time::Instant::now();
    eprintln!("[⏱️ 性能] process_voice_input 开始");

    // 1. 准备音频文件
    let step1_start = std::time::Instant::now();
    let audio_path = prepare_uploaded_audio_file(request.audio_bytes, &request.audio_extension)?;
    eprintln!(
        "[⏱️ 性能] 步骤1: 准备音频文件 - 耗时 {:?}",
        step1_start.elapsed()
    );

    // 2. ASR 识别
    progress(VoiceInputStage::Transcribing);
    let step2_start = std::time::Instant::now();
    let transcription = transcribe_audio_file(&audio_path, &asr_config)
        .map_err(|error| VoiceInputError::RequestFailed(error.to_string()));
    let _ = std::fs::remove_file(&audio_path);
    let transcription = transcription?;
    eprintln!(
        "[⏱️ 性能] 步骤2: ASR 识别 - 耗时 {:?}",
        step2_start.elapsed()
    );

    // 3. 获取人格和热词
    let step3_start = std::time::Instant::now();
    let persona = default_persona(database)?;
    let hotword_context = database
        .enabled_hotword_context()
        .map_err(|error| VoiceInputError::RequestFailed(error.to_string()))?;
    eprintln!(
        "[⏱️ 性能] 步骤3: 获取人格和热词 - 耗时 {:?}",
        step3_start.elapsed()
    );

    // 4. 文本润色
    progress(VoiceInputStage::Refining);
    let step4_start = std::time::Instant::now();
    let polish_result = polish_text_with_openai(
        &TextPolishRequest {
            raw_text: transcription.text.clone(),
            persona_description: persona.description.clone(),
            hotword_context,
        },
        &text_config,
    )
    .map_err(|error| VoiceInputError::RequestFailed(error.to_string()))?;
    eprintln!(
        "[⏱️ 性能] 步骤4: 文本润色 - 耗时 {:?}",
        step4_start.elapsed()
    );

    // 5. 保存历史记录并返回关联 ID，供后续投递方式和录音留存更新。
    let history_record = if auto_save_history {
        let draft = HistoryRecordDraft {
            raw_text: transcription.text.clone(),
            final_text: polish_result.final_text.clone(),
            persona_id: persona.id,
            persona_name: persona.name.clone(),
            duration_ms: request.duration_ms.max(0),
            output_mode: "pending".to_string(),
            source: history_context.source,
            asr_provider: transcription.provider.clone(),
            asr_model: transcription.model.clone(),
            text_provider: history_context.text_provider,
            text_model: history_context.text_model,
            used_asr_fallback: transcription.used_fallback,
            used_fallback: polish_result.used_fallback,
            delivery_method: "pending".to_string(),
            audio_path: history_context.audio_path,
        };
        Some(
            database
                .create_history_record(draft)
                .map_err(|error| VoiceInputError::RequestFailed(error.to_string()))?,
        )
    } else {
        None
    };

    eprintln!(
        "[⏱️ 性能] process_voice_input 总耗时: {:?}",
        start_time.elapsed()
    );

    Ok(VoiceInputResult {
        raw_text: transcription.text,
        final_text: polish_result.final_text,
        actual_asr_provider: transcription.provider,
        actual_asr_model: transcription.model,
        used_asr_fallback: transcription.used_fallback,
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

    let asr_config = build_asr_config(&app, &config)?;
    let (text_api_key, text_base_url, text_model) = config.selected_text_config();
    let text_api_key = text_api_key.to_string();
    let text_base_url = text_base_url.to_string();
    let text_model = text_model.to_string();

    let history_context = HistoryContext {
        source: "upload".to_string(),
        text_provider: config.text_provider.clone(),
        text_model: text_model.clone(),
        audio_path: None,
    };

    process_voice_input(
        request,
        asr_config,
        TextPolishConfig {
            provider: config.text_provider,
            api_key: text_api_key,
            base_url: text_base_url,
            model: text_model,
        },
        &database,
        config.auto_save_history,
        history_context,
    )
    .map_err(|error| error.to_string())
}

struct RecordingCleanupGuard {
    path: Option<std::path::PathBuf>,
}

impl RecordingCleanupGuard {
    fn disarm(&mut self) {
        self.path = None;
    }
}

impl Drop for RecordingCleanupGuard {
    fn drop(&mut self) {
        if let Some(path) = &self.path {
            if std::fs::remove_file(path).is_err() {
                eprintln!("[隐私] 应用录音临时文件清理失败");
            }
        }
    }
}

pub fn consume_app_recording<T>(
    recordings_dir: &std::path::Path,
    file_path: &std::path::Path,
    process: impl FnOnce(Vec<u8>, String, &std::path::Path) -> Result<(T, bool), String>,
) -> Result<T, String> {
    let recordings_dir =
        std::fs::canonicalize(recordings_dir).map_err(|_| "无法访问应用录音目录".to_string())?;
    let recording_path =
        std::fs::canonicalize(file_path).map_err(|_| "无法访问录音文件".to_string())?;

    if !recording_path.starts_with(&recordings_dir) || !recording_path.is_file() {
        return Err("录音文件不在应用录音目录中".to_string());
    }

    let extension = recording_path
        .extension()
        .and_then(|extension| extension.to_str())
        .filter(|extension| extension.eq_ignore_ascii_case("wav"))
        .ok_or_else(|| "应用录音文件必须是 WAV 格式".to_string())?
        .to_ascii_lowercase();
    let mut cleanup = RecordingCleanupGuard {
        path: Some(recording_path.clone()),
    };
    let audio_bytes =
        std::fs::read(&recording_path).map_err(|_| "读取应用录音文件失败".to_string())?;

    let (result, retain_recording) = process(audio_bytes, extension, &recording_path)?;
    if retain_recording {
        cleanup.disarm();
    }
    Ok(result)
}

#[tauri::command]
pub fn process_recording_file(
    app: tauri::AppHandle,
    session_id: String,
    file_path: String,
    duration_ms: i64,
) -> Result<VoiceInputResult, String> {
    use crate::data::{read_app_config, LocalDatabase};
    use tauri::Manager;

    let sessions = app.state::<CaptureSessionState>();
    let context = sessions.delivery_context(&session_id)?;
    let show_indicator = context.source == CaptureSource::Hotkey;
    let result = (|| {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|error| error.to_string())?;
        let recordings_dir = app_data_dir.join("recordings");

        consume_app_recording(
            &recordings_dir,
            std::path::Path::new(&file_path),
            |audio_bytes, audio_extension, recording_path| {
                let config = read_app_config(app.clone())?;
                eprintln!("录音处理配置已加载，ASR Provider：{}", config.asr_provider);

                std::fs::create_dir_all(&app_data_dir).map_err(|error| error.to_string())?;
                let database = LocalDatabase::open(app_data_dir.join("xiluolin.sqlite"))
                    .map_err(|error| error.to_string())?;
                database.initialize().map_err(|error| error.to_string())?;

                let asr_config = build_asr_config(&app, &config)?;
                let (text_api_key, text_base_url, text_model) = config.selected_text_config();
                let text_api_key = text_api_key.to_string();
                let text_base_url = text_base_url.to_string();
                let text_model = text_model.to_string();

                let history_context = HistoryContext {
                    source: "recording".to_string(),
                    text_provider: config.text_provider.clone(),
                    text_model: text_model.clone(),
                    audio_path: if config.retain_recordings && config.auto_save_history {
                        Some(recording_path.to_string_lossy().to_string())
                    } else {
                        None
                    },
                };

                let retain_requested = config.retain_recordings && config.auto_save_history;
                let processed = process_voice_input_with_progress(
                    VoiceInputRequest {
                        audio_bytes,
                        audio_extension,
                        duration_ms,
                    },
                    asr_config,
                    TextPolishConfig {
                        provider: config.text_provider,
                        api_key: text_api_key,
                        base_url: text_base_url,
                        model: text_model,
                    },
                    &database,
                    config.auto_save_history,
                    history_context,
                    |stage| {
                        let (status, indicator_status) = match stage {
                            VoiceInputStage::Transcribing => {
                                (CaptureStatus::Transcribing, "transcribing")
                            }
                            VoiceInputStage::Refining => (CaptureStatus::Refining, "refining"),
                        };
                        let _ = sessions.update_status(&session_id, status);
                        if show_indicator {
                            let _ = indicator::update_indicator(&app, indicator_status);
                        }
                    },
                )
                .map_err(|error| error.to_string())?;
                let retain_recording = retain_requested && processed.history_record.is_some();
                Ok((processed, retain_recording))
            },
        )
    })();

    if let Ok(processed) = &result {
        if let Some(history) = &processed.history_record {
            let _ = sessions.attach_history(&session_id, history.id.clone());
        }
    }

    if let Err(error) = &result {
        // 错误路径必须无条件释放会话。finish 受状态机约束且此前忽略了返回值，
        // 一旦状态更新异常就会永久触发“上一条语音输入仍在处理中”。
        sessions.cancel(&session_id);
        if show_indicator {
            let _ = indicator::finish_indicator(&app, "failed");
        }
        eprintln!("语音输入处理失败：{error}");
    }

    result
}
