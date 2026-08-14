use std::{collections::BTreeMap, fs, path::Path, sync::Arc, time::Duration};

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;

use super::{
    asr::{AsrInput, AsrOutput, AsrProvider},
    catalog::{ProviderOptionValue, ProviderSettings},
    error::{ProviderError, ProviderErrorKind},
    transport,
};

const MAX_QWEN_AUDIO_BYTES: u64 = 10 * 1024 * 1024;

pub fn built_in_asr_providers() -> Vec<Arc<dyn AsrProvider>> {
    vec![
        Arc::new(LegacyCloudAsr("zhipu")),
        Arc::new(LegacyCloudAsr("openai")),
        Arc::new(LocalAsr),
        Arc::new(QwenAudioAsr),
        Arc::new(Qwen3Asr),
    ]
}

struct LegacyCloudAsr(&'static str);

impl AsrProvider for LegacyCloudAsr {
    fn id(&self) -> &'static str {
        self.0
    }

    fn transcribe(
        &self,
        input: &AsrInput,
        settings: &ProviderSettings,
    ) -> Result<AsrOutput, ProviderError> {
        let config = crate::asr::AsrConfig {
            provider: self.0.to_string(),
            api_key: settings.api_key.clone(),
            base_url: settings.base_url.clone(),
            model: settings.model.clone(),
            local_model_path: None,
            allow_cloud_fallback: false,
            fallback_provider: String::new(),
            fallback_api_key: String::new(),
            fallback_base_url: String::new(),
            fallback_model: String::new(),
        };
        let request = crate::asr::AsrRequest {
            audio_path: input.audio_path.clone(),
            hotwords: input.hotwords.clone(),
            context_prompt: input.context_prompt.clone(),
        };
        crate::asr::transcribe_audio_file(&request, &config)
            .map(|output| AsrOutput {
                text: output.text,
                provider: output.provider,
                model: output.model,
            })
            .map_err(|error| map_legacy_asr_error(self.0, &settings.model, error))
    }
}

struct LocalAsr;

impl AsrProvider for LocalAsr {
    fn id(&self) -> &'static str {
        "local"
    }

    fn transcribe(
        &self,
        input: &AsrInput,
        settings: &ProviderSettings,
    ) -> Result<AsrOutput, ProviderError> {
        let model_path = input.local_model_path.as_deref().ok_or_else(|| {
            ProviderError::new(
                self.id(),
                &settings.model,
                ProviderErrorKind::Configuration,
                None,
                "本地 ASR 模型尚未下载",
            )
        })?;
        let prompt =
            crate::asr::build_soft_prompt(input.context_prompt.as_deref(), &input.hotwords);
        let text = crate::local_asr::transcribe_with_initial_prompt(
            &input.audio_path,
            model_path,
            prompt.as_deref(),
        )
        .map_err(|_| {
            ProviderError::new(
                self.id(),
                &settings.model,
                ProviderErrorKind::LocalRuntime,
                None,
                "本地 ASR 运行失败",
            )
        })?;
        Ok(AsrOutput {
            text,
            provider: self.id().to_string(),
            model: settings.model.clone(),
        })
    }
}

struct QwenAudioAsr;

impl AsrProvider for QwenAudioAsr {
    fn id(&self) -> &'static str {
        "qwen-audio"
    }

    fn transcribe(
        &self,
        input: &AsrInput,
        settings: &ProviderSettings,
    ) -> Result<AsrOutput, ProviderError> {
        let audio = read_audio(self.id(), settings, &input.audio_path)?;
        if audio.bytes.len() as u64 > MAX_QWEN_AUDIO_BYTES {
            return Err(ProviderError::new(
                self.id(),
                &settings.model,
                ProviderErrorKind::IncompatibleInput,
                None,
                "Base64 音频源文件不能超过 10 MB",
            ));
        }
        let vocabulary = normalized_hotwords(&input.hotwords)
            .into_iter()
            .take(100)
            .map(|hotword| (hotword, 5_u8))
            .collect::<BTreeMap<_, _>>();
        let language_hints = settings
            .options
            .get("language_hints")
            .and_then(ProviderOptionValue::as_string_list)
            .unwrap_or_default()
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .take(4)
            .map(str::to_string)
            .collect::<Vec<_>>();
        let body = QwenAudioRequest {
            model: settings.model.trim(),
            input: QwenAudioInput {
                messages: vec![QwenAudioMessage {
                    role: "user",
                    content: vec![QwenAudioContent {
                        content_type: "input_audio",
                        input_audio: QwenAudioData {
                            data: audio.data_uri(),
                        },
                    }],
                }],
            },
            parameters: QwenAudioParameters {
                format: &audio.format,
                vocabulary,
                language_hints,
            },
        };
        let response = transport::post_json(
            self.id(),
            &settings.model,
            &settings.base_url,
            "api/v1/services/aigc/multimodal-generation/generation",
            &settings.api_key,
            &BTreeMap::from([("X-DashScope-SSE", "disable")]),
            &body,
            Duration::from_secs(60),
        )?;
        let text = response
            .pointer("/output/text")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| invalid_response(self.id(), &settings.model))?;
        Ok(AsrOutput {
            text: text.to_string(),
            provider: self.id().to_string(),
            model: settings.model.clone(),
        })
    }
}

struct Qwen3Asr;

impl AsrProvider for Qwen3Asr {
    fn id(&self) -> &'static str {
        "qwen3-asr"
    }

    fn transcribe(
        &self,
        input: &AsrInput,
        settings: &ProviderSettings,
    ) -> Result<AsrOutput, ProviderError> {
        let audio = read_audio(self.id(), settings, &input.audio_path)?;
        if audio.bytes.len() as u64 > MAX_QWEN_AUDIO_BYTES {
            return Err(ProviderError::new(
                self.id(),
                &settings.model,
                ProviderErrorKind::IncompatibleInput,
                None,
                "Base64 音频源文件不能超过 10 MB",
            ));
        }
        let mut messages = Vec::new();
        if let Some(glossary) = glossary_message(input) {
            messages.push(Qwen3Message::System {
                role: "system",
                content: glossary,
            });
        }
        messages.push(Qwen3Message::User {
            role: "user",
            content: vec![Qwen3AudioContent {
                content_type: "input_audio",
                input_audio: QwenAudioData {
                    data: audio.data_uri(),
                },
            }],
        });
        let body = Qwen3Request {
            model: settings.model.trim(),
            messages,
            stream: false,
            asr_options: Qwen3Options {
                language: settings
                    .options
                    .get("language")
                    .and_then(ProviderOptionValue::as_text)
                    .map(str::trim)
                    .filter(|value| !value.is_empty()),
                enable_itn: settings
                    .options
                    .get("enable_itn")
                    .and_then(ProviderOptionValue::as_boolean)
                    .unwrap_or(false),
            },
        };
        let response = transport::post_json(
            self.id(),
            &settings.model,
            &settings.base_url,
            "chat/completions",
            &settings.api_key,
            &BTreeMap::new(),
            &body,
            Duration::from_secs(60),
        )?;
        let text = response
            .pointer("/choices/0/message/content")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| invalid_response(self.id(), &settings.model))?;
        Ok(AsrOutput {
            text: text.to_string(),
            provider: self.id().to_string(),
            model: settings.model.clone(),
        })
    }
}

struct AudioData {
    bytes: Vec<u8>,
    mime: &'static str,
    format: String,
}

impl AudioData {
    fn data_uri(&self) -> String {
        format!("data:{};base64,{}", self.mime, STANDARD.encode(&self.bytes))
    }
}

fn read_audio(
    provider: &str,
    settings: &ProviderSettings,
    path: &Path,
) -> Result<AudioData, ProviderError> {
    transport::validate_cloud_settings(
        provider,
        &settings.model,
        &settings.base_url,
        &settings.api_key,
    )?;
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let (mime, format) = match extension.as_str() {
        "wav" => ("audio/wav", "wav"),
        "mp3" => ("audio/mpeg", "mp3"),
        _ => {
            return Err(ProviderError::new(
                provider,
                &settings.model,
                ProviderErrorKind::IncompatibleInput,
                None,
                "仅支持 WAV 或 MP3 音频",
            ))
        }
    };
    let bytes = fs::read(path).map_err(|_| {
        ProviderError::new(
            provider,
            &settings.model,
            ProviderErrorKind::IncompatibleInput,
            None,
            "音频文件不存在或不可读",
        )
        .global()
    })?;
    Ok(AudioData {
        bytes,
        mime,
        format: format.to_string(),
    })
}

fn normalized_hotwords(hotwords: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    for hotword in hotwords {
        let hotword = hotword.trim();
        if !hotword.is_empty() && !result.iter().any(|value| value == hotword) {
            result.push(hotword.to_string());
        }
    }
    result
}

fn glossary_message(input: &AsrInput) -> Option<String> {
    let hotwords = normalized_hotwords(&input.hotwords);
    let context = input
        .context_prompt
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    match (context, hotwords.is_empty()) {
        (None, true) => None,
        (Some(context), true) => Some(format!("识别背景：{context}")),
        (None, false) => Some(format!("实体词表：{}", hotwords.join("、"))),
        (Some(context), false) => Some(format!(
            "识别背景：{context}\n实体词表：{}",
            hotwords.join("、")
        )),
    }
}

fn invalid_response(provider: &str, model: &str) -> ProviderError {
    ProviderError::new(
        provider,
        model,
        ProviderErrorKind::InvalidResponse,
        Some(200),
        "响应缺少文本内容",
    )
}

fn map_legacy_asr_error(provider: &str, model: &str, error: crate::asr::AsrError) -> ProviderError {
    use crate::asr::AsrError;
    let (kind, scope, message) = match error {
        AsrError::MissingApiKey => (ProviderErrorKind::Configuration, false, "API Key 不能为空"),
        AsrError::MissingLocalModel => (ProviderErrorKind::Configuration, false, "本地模型缺失"),
        AsrError::MissingAudioFile(_) => (
            ProviderErrorKind::IncompatibleInput,
            true,
            "音频文件不存在或不可读",
        ),
        AsrError::UnsupportedAudioFormat(_) | AsrError::AudioTooLarge { .. } => (
            ProviderErrorKind::IncompatibleInput,
            false,
            "音频格式或大小不兼容",
        ),
        AsrError::RequestFailed(_) => (ProviderErrorKind::Network, false, "ASR 请求失败"),
        AsrError::InvalidResponse(_) => (ProviderErrorKind::InvalidResponse, false, "ASR 响应无效"),
    };
    let error = ProviderError::new(provider, model, kind, None, message);
    if scope {
        error.global()
    } else {
        error
    }
}

#[derive(Serialize)]
struct QwenAudioRequest<'a> {
    model: &'a str,
    input: QwenAudioInput,
    parameters: QwenAudioParameters<'a>,
}

#[derive(Serialize)]
struct QwenAudioInput {
    messages: Vec<QwenAudioMessage>,
}

#[derive(Serialize)]
struct QwenAudioMessage {
    role: &'static str,
    content: Vec<QwenAudioContent>,
}

#[derive(Serialize)]
struct QwenAudioContent {
    #[serde(rename = "type")]
    content_type: &'static str,
    input_audio: QwenAudioData,
}

#[derive(Serialize)]
struct QwenAudioData {
    data: String,
}

#[derive(Serialize)]
struct QwenAudioParameters<'a> {
    format: &'a str,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    vocabulary: BTreeMap<String, u8>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    language_hints: Vec<String>,
}

#[derive(Serialize)]
struct Qwen3Request<'a> {
    model: &'a str,
    messages: Vec<Qwen3Message>,
    stream: bool,
    asr_options: Qwen3Options<'a>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Qwen3Message {
    System {
        role: &'static str,
        content: String,
    },
    User {
        role: &'static str,
        content: Vec<Qwen3AudioContent>,
    },
}

#[derive(Serialize)]
struct Qwen3AudioContent {
    #[serde(rename = "type")]
    content_type: &'static str,
    input_audio: QwenAudioData,
}

#[derive(Serialize)]
struct Qwen3Options<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<&'a str>,
    enable_itn: bool,
}
