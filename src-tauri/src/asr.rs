use std::{
    fmt,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const MAX_AUDIO_BYTES: u64 = 25 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsrConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AsrTranscription {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsrError {
    MissingApiKey,
    MissingAudioFile(PathBuf),
    UnsupportedAudioFormat(String),
    AudioTooLarge { max_bytes: u64, actual_bytes: u64 },
    RequestFailed(String),
    InvalidResponse(String),
}

impl fmt::Display for AsrError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingApiKey => write!(formatter, "智谱 ASR API Key 不能为空"),
            Self::MissingAudioFile(path) => write!(formatter, "音频文件不存在：{}", path.display()),
            Self::UnsupportedAudioFormat(extension) => {
                write!(formatter, "仅支持 wav 或 mp3 音频文件，当前格式：{extension}")
            }
            Self::AudioTooLarge {
                max_bytes,
                actual_bytes,
            } => write!(
                formatter,
                "音频文件过大，最大支持 {max_bytes} 字节，当前为 {actual_bytes} 字节"
            ),
            Self::RequestFailed(message) => write!(formatter, "智谱 ASR 请求失败：{message}"),
            Self::InvalidResponse(message) => write!(formatter, "智谱 ASR 响应解析失败：{message}"),
        }
    }
}

impl std::error::Error for AsrError {}

#[derive(Debug, Deserialize)]
struct ZhipuTranscriptionResponse {
    text: String,
}

pub fn transcribe_audio_file(
    audio_path: &Path,
    config: &AsrConfig,
) -> Result<AsrTranscription, AsrError> {
    validate_audio_file(audio_path, config)?;

    let url = transcriptions_url(&config.base_url);
    let form = ureq::unversioned::multipart::Form::new()
        .text("model", config.model.trim())
        .text("stream", "false")
        .file("file", audio_path)
        .map_err(|error| AsrError::RequestFailed(error.to_string()))?;

    let response = ureq::post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key.trim()))
        .send(form)
        .map_err(|error| AsrError::RequestFailed(error.to_string()))?;
    let response = response
        .into_body()
        .read_json::<ZhipuTranscriptionResponse>()
        .map_err(|error| AsrError::InvalidResponse(error.to_string()))?;

    Ok(AsrTranscription {
        text: response.text.trim().to_string(),
    })
}

fn validate_audio_file(audio_path: &Path, config: &AsrConfig) -> Result<(), AsrError> {
    if config.api_key.trim().is_empty() {
        return Err(AsrError::MissingApiKey);
    }

    let metadata = audio_path
        .metadata()
        .map_err(|_| AsrError::MissingAudioFile(audio_path.to_path_buf()))?;
    if metadata.len() > MAX_AUDIO_BYTES {
        return Err(AsrError::AudioTooLarge {
            max_bytes: MAX_AUDIO_BYTES,
            actual_bytes: metadata.len(),
        });
    }

    let extension = audio_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if extension != "wav" && extension != "mp3" {
        return Err(AsrError::UnsupportedAudioFormat(extension));
    }

    Ok(())
}

fn transcriptions_url(base_url: &str) -> String {
    format!(
        "{}/audio/transcriptions",
        base_url.trim_end_matches('/')
    )
}

#[tauri::command]
pub fn transcribe_audio_path(
    audio_path: String,
    api_key: String,
    base_url: String,
    model: String,
) -> Result<AsrTranscription, String> {
    let config = AsrConfig {
        api_key,
        base_url,
        model,
    };

    transcribe_audio_file(Path::new(&audio_path), &config).map_err(|error| error.to_string())
}
