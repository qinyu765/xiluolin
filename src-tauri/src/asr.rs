use std::{
    fmt,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const MAX_AUDIO_BYTES: u64 = 25 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsrConfig {
    pub provider: String,
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
                write!(
                    formatter,
                    "仅支持 wav 或 mp3 音频文件，当前格式：{extension}"
                )
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

#[derive(Debug, Deserialize)]
struct OpenAITranscriptionResponse {
    text: String,
}

pub fn transcribe_audio_file(
    audio_path: &Path,
    config: &AsrConfig,
) -> Result<AsrTranscription, AsrError> {
    let start_time = std::time::Instant::now();
    eprintln!("[⏱️ ASR] 开始音频转写");

    let step1_start = std::time::Instant::now();
    validate_audio_file(audio_path, config)?;
    eprintln!("[⏱️ ASR] 验证音频文件 - 耗时 {:?}", step1_start.elapsed());

    let result = match config.provider.as_str() {
        "openai" => transcribe_with_openai(audio_path, config),
        "zhipu" | _ => transcribe_with_zhipu(audio_path, config),
    };

    eprintln!("[⏱️ ASR] 总耗时: {:?}", start_time.elapsed());
    result
}

fn transcribe_with_openai(
    audio_path: &Path,
    config: &AsrConfig,
) -> Result<AsrTranscription, AsrError> {
    let start_time = std::time::Instant::now();
    let url = format!(
        "{}/audio/transcriptions",
        config.base_url.trim_end_matches('/')
    );

    eprintln!("[⏱️ ASR OpenAI] Request URL: {}", url);
    eprintln!("[⏱️ ASR OpenAI] Model: {}", config.model.trim());

    // 构建 multipart form
    let step1_start = std::time::Instant::now();
    let form = ureq::unversioned::multipart::Form::new()
        .text("model", config.model.trim())
        .file("file", audio_path)
        .map_err(|error| AsrError::RequestFailed(error.to_string()))?;
    eprintln!(
        "[⏱️ ASR OpenAI] 构建 multipart form - 耗时 {:?}",
        step1_start.elapsed()
    );

    // 创建禁用自动状态码错误的 agent
    let step2_start = std::time::Instant::now();
    let agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .new_agent();
    eprintln!(
        "[⏱️ ASR OpenAI] 创建 HTTP agent - 耗时 {:?}",
        step2_start.elapsed()
    );

    let step3_start = std::time::Instant::now();
    let response = agent
        .post(&url)
        .header(
            "Authorization",
            &format!("Bearer {}", config.api_key.trim()),
        )
        .send(form)
        .map_err(|error| AsrError::RequestFailed(error.to_string()))?;
    eprintln!(
        "[⏱️ ASR OpenAI] 发送 HTTP 请求并等待响应 - 耗时 {:?}",
        step3_start.elapsed()
    );

    // 检查状态码
    let step4_start = std::time::Instant::now();
    let status_code = response.status().as_u16();
    if status_code >= 400 && status_code < 600 {
        let body = response.into_body().read_to_string().unwrap_or_default();
        eprintln!(
            "[⏱️ ASR OpenAI] Error: status={}, body={}",
            status_code, body
        );
        return Err(AsrError::RequestFailed(format!(
            "http status: {}, body: {}",
            status_code, body
        )));
    }

    let transcription: OpenAITranscriptionResponse = response
        .into_body()
        .read_json()
        .map_err(|error| AsrError::InvalidResponse(error.to_string()))?;
    eprintln!(
        "[⏱️ ASR OpenAI] 解析响应 - 耗时 {:?}",
        step4_start.elapsed()
    );

    eprintln!("[⏱️ ASR OpenAI] 总耗时: {:?}", start_time.elapsed());

    Ok(AsrTranscription {
        text: transcription.text.trim().to_string(),
    })
}

fn transcribe_with_zhipu(
    audio_path: &Path,
    config: &AsrConfig,
) -> Result<AsrTranscription, AsrError> {
    let url = transcriptions_url(&config.base_url);

    // 打印调试信息
    eprintln!("ASR Request URL: {}", url);
    eprintln!("ASR Model: {}", config.model.trim());
    eprintln!("ASR API Key length: {}", config.api_key.trim().len());
    eprintln!(
        "ASR API Key first 10 chars: {}",
        &config.api_key.trim().chars().take(10).collect::<String>()
    );
    eprintln!("Audio Path: {}", audio_path.display());

    // 检查音频文件的声道信息
    if let Ok(reader) = hound::WavReader::open(audio_path) {
        let spec = reader.spec();
        eprintln!(
            "Audio Spec: channels={}, sample_rate={}, bits_per_sample={}",
            spec.channels, spec.sample_rate, spec.bits_per_sample
        );
    } else {
        eprintln!("无法读取音频文件的 WAV 规格信息");
    }

    // 构建 multipart form
    let form = ureq::unversioned::multipart::Form::new()
        .text("model", config.model.trim())
        .text("stream", "false")
        .file("file", audio_path)
        .map_err(|error| AsrError::RequestFailed(error.to_string()))?;

    // 创建禁用自动状态码错误的 agent,以便获取错误响应体
    let agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .new_agent();

    let response = agent
        .post(&url)
        .header(
            "Authorization",
            &format!("Bearer {}", config.api_key.trim()),
        )
        .send(form)
        .map_err(|error| AsrError::RequestFailed(error.to_string()))?;

    // 手动检查状态码
    let status = response.status();
    let status_code = status.as_u16();
    if status_code >= 400 && status_code < 600 {
        let body = response.into_body().read_to_string().unwrap_or_default();
        eprintln!("ASR Error Response: status={}, body={}", status_code, body);
        return Err(AsrError::RequestFailed(format!(
            "http status: {}, body: {}",
            status_code, body
        )));
    }

    let transcription: ZhipuTranscriptionResponse = response
        .into_body()
        .read_json()
        .map_err(|error| AsrError::InvalidResponse(error.to_string()))?;

    Ok(AsrTranscription {
        text: transcription.text.trim().to_string(),
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
    format!("{}/audio/transcriptions", base_url.trim_end_matches('/'))
}

#[tauri::command]
pub fn transcribe_audio_path(
    audio_path: String,
    provider: String,
    api_key: String,
    base_url: String,
    model: String,
) -> Result<AsrTranscription, String> {
    let config = AsrConfig {
        provider,
        api_key,
        base_url,
        model,
    };

    transcribe_audio_file(Path::new(&audio_path), &config).map_err(|error| error.to_string())
}
