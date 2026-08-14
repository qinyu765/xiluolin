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
    pub local_model_path: Option<PathBuf>,
    pub allow_cloud_fallback: bool,
    pub fallback_provider: String,
    pub fallback_api_key: String,
    pub fallback_base_url: String,
    pub fallback_model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsrRequest {
    pub audio_path: PathBuf,
    pub hotwords: Vec<String>,
    pub context_prompt: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsrCapabilities {
    pub native_hotwords: bool,
    pub max_hotwords: Option<usize>,
    pub supports_prompt: bool,
    pub max_duration_ms: Option<u64>,
    pub live_audio: bool,
}

impl AsrConfig {
    pub fn capabilities(&self) -> AsrCapabilities {
        match self.provider.as_str() {
            "zhipu" => AsrCapabilities {
                native_hotwords: true,
                max_hotwords: Some(100),
                supports_prompt: true,
                max_duration_ms: Some(30_000),
                live_audio: false,
            },
            "openai" => AsrCapabilities {
                native_hotwords: false,
                max_hotwords: None,
                supports_prompt: true,
                max_duration_ms: None,
                live_audio: false,
            },
            "local" => AsrCapabilities {
                native_hotwords: false,
                max_hotwords: None,
                supports_prompt: true,
                max_duration_ms: None,
                live_audio: false,
            },
            _ => AsrCapabilities {
                native_hotwords: false,
                max_hotwords: None,
                supports_prompt: false,
                max_duration_ms: None,
                live_audio: false,
            },
        }
    }
}

pub fn build_soft_prompt(context_prompt: Option<&str>, hotwords: &[String]) -> Option<String> {
    let normalized_hotwords = normalize_hotwords(hotwords);
    let context_prompt = context_prompt
        .map(str::trim)
        .filter(|value| !value.is_empty());
    match (context_prompt, normalized_hotwords.is_empty()) {
        (None, true) => None,
        (Some(context_prompt), true) => Some(context_prompt.to_string()),
        (None, false) => Some(format!(
            "可能出现的专有词：{}",
            normalized_hotwords.join("，")
        )),
        (Some(context_prompt), false) => Some(format!(
            "{context_prompt}\n可能出现的专有词：{}",
            normalized_hotwords.join("，")
        )),
    }
}

fn normalize_hotwords(hotwords: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for hotword in hotwords {
        let hotword = hotword.trim();
        if !hotword.is_empty() && !normalized.iter().any(|value| value == hotword) {
            normalized.push(hotword.to_string());
        }
    }
    normalized
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct AsrTranscription {
    pub text: String,
    pub provider: String,
    pub model: String,
    pub used_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsrError {
    MissingApiKey,
    MissingLocalModel,
    MissingAudioFile(PathBuf),
    UnsupportedAudioFormat(String),
    AudioTooLarge { max_bytes: u64, actual_bytes: u64 },
    HttpStatus(u16),
    RequestFailed(String),
    InvalidResponse(String),
}

impl fmt::Display for AsrError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingApiKey => write!(formatter, "ASR API Key 不能为空"),
            Self::MissingLocalModel => write!(formatter, "本地 ASR 模型尚未下载"),
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
            Self::HttpStatus(status) => write!(formatter, "ASR 服务返回 HTTP {status}"),
            Self::RequestFailed(message) => write!(formatter, "ASR 请求失败：{message}"),
            Self::InvalidResponse(message) => write!(formatter, "ASR 响应解析失败：{message}"),
        }
    }
}

impl std::error::Error for AsrError {}

#[derive(Debug, Deserialize, specta::Type)]
struct ZhipuTranscriptionResponse {
    text: String,
}

#[derive(Debug, Deserialize, specta::Type)]
struct OpenAITranscriptionResponse {
    text: String,
}

pub fn transcribe_audio_file(
    request: &AsrRequest,
    config: &AsrConfig,
) -> Result<AsrTranscription, AsrError> {
    let audio_path = &request.audio_path;
    let start_time = std::time::Instant::now();
    eprintln!("[⏱️ ASR] 开始音频转写");

    let step1_start = std::time::Instant::now();
    validate_audio_file(audio_path, config)?;
    eprintln!("[⏱️ ASR] 验证音频文件 - 耗时 {:?}", step1_start.elapsed());

    let result = match config.provider.as_str() {
        "local" => match transcribe_with_local(audio_path, request, config) {
            Ok(result) => Ok(result),
            Err(local_error) if config.allow_cloud_fallback => {
                eprintln!("本地 ASR 失败，使用显式配置的云端降级：{local_error}");
                let fallback = AsrConfig {
                    provider: config.fallback_provider.clone(),
                    api_key: config.fallback_api_key.clone(),
                    base_url: config.fallback_base_url.clone(),
                    model: config.fallback_model.clone(),
                    local_model_path: None,
                    allow_cloud_fallback: false,
                    fallback_provider: String::new(),
                    fallback_api_key: String::new(),
                    fallback_base_url: String::new(),
                    fallback_model: String::new(),
                };
                validate_audio_file(audio_path, &fallback)?;
                let mut result = match fallback.provider.as_str() {
                    "openai" => transcribe_with_openai(audio_path, request, &fallback),
                    "zhipu" => transcribe_with_zhipu(audio_path, request, &fallback),
                    _ => Err(AsrError::RequestFailed(
                        "云端降级 Provider 无效".to_string(),
                    )),
                }?;
                result.used_fallback = true;
                Ok(result)
            }
            Err(error) => Err(error),
        },
        "openai" => transcribe_with_openai(audio_path, request, config),
        "zhipu" => transcribe_with_zhipu(audio_path, request, config),
        _ => Err(AsrError::RequestFailed("未知 ASR Provider".to_string())),
    };

    eprintln!("[⏱️ ASR] 总耗时: {:?}", start_time.elapsed());
    result
}

fn transcribe_with_openai(
    audio_path: &Path,
    request: &AsrRequest,
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
    let prompt = build_soft_prompt(request.context_prompt.as_deref(), &request.hotwords);
    let file_name = audio_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("recording.wav")
        .to_string();
    let audio_bytes = std::fs::read(audio_path)
        .map_err(|error| AsrError::RequestFailed(format!("读取音频文件失败：{error}")))?;
    let file_part = reqwest::blocking::multipart::Part::bytes(audio_bytes)
        .file_name(file_name)
        .mime_str(audio_mime_type(audio_path))
        .map_err(|error| AsrError::RequestFailed(error.to_string()))?;
    let mut form =
        reqwest::blocking::multipart::Form::new().text("model", config.model.trim().to_string());
    if let Some(prompt) = prompt.as_deref() {
        form = form.text("prompt", prompt.to_string());
    }
    let form = form.part("file", file_part);
    eprintln!(
        "[⏱️ ASR OpenAI] 构建 multipart form - 耗时 {:?}",
        step1_start.elapsed()
    );

    // 创建统一的 blocking HTTP client。
    let step2_start = std::time::Instant::now();
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|error| AsrError::RequestFailed(error.to_string()))?;
    eprintln!(
        "[⏱️ ASR OpenAI] 创建 HTTP agent - 耗时 {:?}",
        step2_start.elapsed()
    );

    let step3_start = std::time::Instant::now();
    let response = client
        .post(&url)
        .bearer_auth(config.api_key.trim())
        .multipart(form)
        .send()
        .map_err(|error| AsrError::RequestFailed(error.to_string()))?;
    eprintln!(
        "[⏱️ ASR OpenAI] 发送 HTTP 请求并等待响应 - 耗时 {:?}",
        step3_start.elapsed()
    );

    // 检查状态码
    let step4_start = std::time::Instant::now();
    let status = response.status();
    let status_code = status.as_u16();
    if !status.is_success() {
        eprintln!("[⏱️ ASR OpenAI] Error: status={status_code}");
        return Err(AsrError::HttpStatus(status_code));
    }

    let transcription: OpenAITranscriptionResponse = response
        .json()
        .map_err(|error| AsrError::InvalidResponse(error.to_string()))?;
    eprintln!(
        "[⏱️ ASR OpenAI] 解析响应 - 耗时 {:?}",
        step4_start.elapsed()
    );

    eprintln!("[⏱️ ASR OpenAI] 总耗时: {:?}", start_time.elapsed());

    Ok(AsrTranscription {
        text: transcription.text.trim().to_string(),
        provider: "openai".to_string(),
        model: config.model.clone(),
        used_fallback: false,
    })
}

fn transcribe_with_zhipu(
    audio_path: &Path,
    request: &AsrRequest,
    config: &AsrConfig,
) -> Result<AsrTranscription, AsrError> {
    let url = transcriptions_url(&config.base_url);

    eprintln!("ASR 请求已开始，模型：{}", config.model.trim());

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

    // reqwest 对服务端提前拒绝 multipart 上传的场景响应处理稳定，并支持 HTTP/2。
    let file_name = audio_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("recording.wav")
        .to_string();
    let audio_bytes = std::fs::read(audio_path)
        .map_err(|error| AsrError::RequestFailed(format!("读取音频文件失败：{error}")))?;
    let file_part = reqwest::blocking::multipart::Part::bytes(audio_bytes)
        .file_name(file_name)
        .mime_str(audio_mime_type(audio_path))
        .map_err(|error| AsrError::RequestFailed(error.to_string()))?;
    let mut form = reqwest::blocking::multipart::Form::new()
        .text("model", config.model.trim().to_string())
        .text("stream", "false");
    if let Some(context_prompt) = request
        .context_prompt
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        form = form.text("prompt", context_prompt.to_string());
    }
    for hotword in normalize_hotwords(&request.hotwords).into_iter().take(100) {
        form = form.text("hotwords[]", hotword);
    }
    let form = form.part("file", file_part);
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|error| AsrError::RequestFailed(error.to_string()))?;
    let response = client
        .post(&url)
        .bearer_auth(config.api_key.trim())
        .multipart(form)
        .send()
        .map_err(|error| AsrError::RequestFailed(error.to_string()))?;

    let status = response.status();
    let status_code = status.as_u16();
    let body = response
        .text()
        .map_err(|error| AsrError::InvalidResponse(error.to_string()))?;
    if !status.is_success() {
        eprintln!("ASR Error Response: status={status_code}");
        return Err(AsrError::HttpStatus(status_code));
    }

    let transcription: ZhipuTranscriptionResponse = serde_json::from_str(&body)
        .map_err(|error| AsrError::InvalidResponse(error.to_string()))?;

    Ok(AsrTranscription {
        text: transcription.text.trim().to_string(),
        provider: "zhipu".to_string(),
        model: config.model.clone(),
        used_fallback: false,
    })
}

fn audio_mime_type(audio_path: &Path) -> &'static str {
    match audio_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("mp3") => "audio/mpeg",
        _ => "audio/wav",
    }
}

fn transcribe_with_local(
    audio_path: &Path,
    request: &AsrRequest,
    config: &AsrConfig,
) -> Result<AsrTranscription, AsrError> {
    let model_path = config
        .local_model_path
        .as_deref()
        .ok_or(AsrError::MissingLocalModel)?;
    let prompt = build_soft_prompt(request.context_prompt.as_deref(), &request.hotwords);
    let text =
        crate::local_asr::transcribe_with_initial_prompt(audio_path, model_path, prompt.as_deref())
            .map_err(AsrError::RequestFailed)?;
    Ok(AsrTranscription {
        text,
        provider: "local".to_string(),
        model: config.model.clone(),
        used_fallback: false,
    })
}

pub fn build_asr_config(
    app: &tauri::AppHandle,
    config: &crate::data::AppConfig,
) -> Result<AsrConfig, String> {
    let provider = config.asr.primary.trim().to_string();
    let settings = config
        .asr
        .settings
        .get(&provider)
        .cloned()
        .ok_or_else(|| format!("缺少 ASR Provider 配置：{provider}"))?;
    if provider == "local" {
        let fallback_provider = config.asr.fallbacks.first().cloned().unwrap_or_default();
        let fallback = config
            .asr
            .settings
            .get(&fallback_provider)
            .cloned()
            .unwrap_or_default();
        return Ok(AsrConfig {
            provider: "local".to_string(),
            api_key: String::new(),
            base_url: String::new(),
            model: settings.model,
            local_model_path: Some(crate::local_asr_model::model_path(app)?),
            allow_cloud_fallback: !fallback_provider.is_empty(),
            fallback_provider,
            fallback_api_key: fallback.api_key,
            fallback_base_url: fallback.base_url,
            fallback_model: fallback.model,
        });
    }

    Ok(AsrConfig {
        provider,
        api_key: settings.api_key,
        base_url: settings.base_url,
        model: settings.model,
        local_model_path: None,
        allow_cloud_fallback: false,
        fallback_provider: String::new(),
        fallback_api_key: String::new(),
        fallback_base_url: String::new(),
        fallback_model: String::new(),
    })
}

fn validate_audio_file(audio_path: &Path, config: &AsrConfig) -> Result<(), AsrError> {
    if config.provider != "local" && config.api_key.trim().is_empty() {
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
    if config.provider == "local" && extension != "wav" {
        return Err(AsrError::UnsupportedAudioFormat(
            "本地 ASR 首版仅支持 WAV".to_string(),
        ));
    }

    Ok(())
}

fn transcriptions_url(base_url: &str) -> String {
    format!("{}/audio/transcriptions", base_url.trim_end_matches('/'))
}
