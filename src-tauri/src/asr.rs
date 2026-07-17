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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
            Self::RequestFailed(message) => write!(formatter, "ASR 请求失败：{message}"),
            Self::InvalidResponse(message) => write!(formatter, "ASR 响应解析失败：{message}"),
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
        "local" => match transcribe_with_local(audio_path, config) {
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
                    "openai" => transcribe_with_openai(audio_path, &fallback),
                    "zhipu" => transcribe_with_zhipu(audio_path, &fallback),
                    _ => Err(AsrError::RequestFailed(
                        "云端降级 Provider 无效".to_string(),
                    )),
                }?;
                result.used_fallback = true;
                Ok(result)
            }
            Err(error) => Err(error),
        },
        "openai" => transcribe_with_openai(audio_path, config),
        "zhipu" => transcribe_with_zhipu(audio_path, config),
        _ => Err(AsrError::RequestFailed("未知 ASR Provider".to_string())),
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
        provider: "openai".to_string(),
        model: config.model.clone(),
        used_fallback: false,
    })
}

fn transcribe_with_zhipu(
    audio_path: &Path,
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
        provider: "zhipu".to_string(),
        model: config.model.clone(),
        used_fallback: false,
    })
}

fn transcribe_with_local(
    audio_path: &Path,
    config: &AsrConfig,
) -> Result<AsrTranscription, AsrError> {
    let model_path = config
        .local_model_path
        .as_deref()
        .ok_or(AsrError::MissingLocalModel)?;
    let text =
        crate::local_asr::transcribe(audio_path, model_path).map_err(AsrError::RequestFailed)?;
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
    if config.asr_provider == "local" {
        let (fallback_api_key, fallback_base_url, fallback_model) =
            config.cloud_asr_config(&config.fallback_asr_provider);
        return Ok(AsrConfig {
            provider: "local".to_string(),
            api_key: String::new(),
            base_url: String::new(),
            model: config.local_asr_model.clone(),
            local_model_path: Some(crate::local_asr_model::model_path(app)?),
            allow_cloud_fallback: config.allow_cloud_fallback,
            fallback_provider: config.fallback_asr_provider.clone(),
            fallback_api_key: fallback_api_key.to_string(),
            fallback_base_url: fallback_base_url.to_string(),
            fallback_model: fallback_model.to_string(),
        });
    }

    let (api_key, base_url, model) = config.selected_asr_config();
    Ok(AsrConfig {
        provider: config.asr_provider.clone(),
        api_key: api_key.to_string(),
        base_url: base_url.to_string(),
        model: model.to_string(),
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
        local_model_path: None,
        allow_cloud_fallback: false,
        fallback_provider: String::new(),
        fallback_api_key: String::new(),
        fallback_base_url: String::new(),
        fallback_model: String::new(),
    };

    transcribe_audio_file(Path::new(&audio_path), &config).map_err(|error| error.to_string())
}
