use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiTextConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextPolishRequest {
    pub raw_text: String,
    pub persona_name: String,
    pub persona_prompt: String,
    pub hotword_context: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextPolishResult {
    pub final_text: String,
    pub used_fallback: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextPolishError {
    MissingApiKey,
    MissingRawText,
    RequestFailed(String),
    InvalidResponse(String),
}

impl fmt::Display for TextPolishError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingApiKey => write!(formatter, "OpenAI API Key 不能为空"),
            Self::MissingRawText => write!(formatter, "原始识别文本不能为空"),
            Self::RequestFailed(message) => write!(formatter, "OpenAI 文本整理请求失败：{message}"),
            Self::InvalidResponse(message) => write!(formatter, "OpenAI 文本整理响应解析失败：{message}"),
        }
    }
}

impl std::error::Error for TextPolishError {}

#[derive(Debug, Serialize)]
struct OpenAiResponsesRequest {
    model: String,
    instructions: String,
    input: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponsesResponse {
    output_text: Option<String>,
}

pub fn polish_text_with_openai(
    request: &TextPolishRequest,
    config: &OpenAiTextConfig,
) -> Result<TextPolishResult, TextPolishError> {
    validate_request(request, config)?;

    match send_polish_request(request, config) {
        Ok(final_text) => Ok(TextPolishResult {
            final_text,
            used_fallback: false,
            error_message: None,
        }),
        Err(error @ TextPolishError::RequestFailed(_)) => Ok(TextPolishResult {
            final_text: request.raw_text.trim().to_string(),
            used_fallback: true,
            error_message: Some(error.to_string()),
        }),
        Err(error) => Err(error),
    }
}

fn send_polish_request(
    request: &TextPolishRequest,
    config: &OpenAiTextConfig,
) -> Result<String, TextPolishError> {
    let body = OpenAiResponsesRequest {
        model: config.model.trim().to_string(),
        instructions: build_instructions(request),
        input: build_input(request),
    };
    let response = ureq::post(&responses_url(&config.base_url))
        .header("Authorization", format!("Bearer {}", config.api_key.trim()))
        .header("Content-Type", "application/json")
        .send_json(&body)
        .map_err(|error| TextPolishError::RequestFailed(error.to_string()))?;

    let response = response
        .into_body()
        .read_json::<OpenAiResponsesResponse>()
        .map_err(|error| TextPolishError::InvalidResponse(error.to_string()))?;

    let final_text = response
        .output_text
        .unwrap_or_default()
        .trim()
        .to_string();
    if final_text.is_empty() {
        return Err(TextPolishError::InvalidResponse(
            "响应缺少 output_text".to_string(),
        ));
    }

    Ok(final_text)
}

fn validate_request(
    request: &TextPolishRequest,
    config: &OpenAiTextConfig,
) -> Result<(), TextPolishError> {
    if config.api_key.trim().is_empty() {
        return Err(TextPolishError::MissingApiKey);
    }
    if request.raw_text.trim().is_empty() {
        return Err(TextPolishError::MissingRawText);
    }

    Ok(())
}

fn build_instructions(request: &TextPolishRequest) -> String {
    format!(
        "你是 AI 语音输入助手，负责把 ASR 原始识别文本整理成可直接使用的文本。\n\
        当前人格：{}\n\
        人格要求：{}\n\
        通用要求：保留用户原意；自动补标点和断句；去除明显口头禅和重复表达；不要编造用户没有表达的信息；只输出整理后的文本。",
        request.persona_name.trim(),
        request.persona_prompt.trim()
    )
}

fn build_input(request: &TextPolishRequest) -> String {
    let mut input = format!("原始识别文本：\n{}", request.raw_text.trim());
    if !request.hotword_context.trim().is_empty() {
        input.push_str("\n\n热词词典：\n");
        input.push_str(request.hotword_context.trim());
    }
    input.push_str("\n\n输出要求：按当前人格整理为可直接复制使用的中文文本。");
    input
}

fn responses_url(base_url: &str) -> String {
    format!("{}/responses", base_url.trim_end_matches('/'))
}

#[tauri::command]
pub fn polish_text(
    request: TextPolishRequest,
    api_key: String,
    base_url: String,
    model: String,
) -> Result<TextPolishResult, String> {
    let config = OpenAiTextConfig {
        api_key,
        base_url,
        model,
    };

    polish_text_with_openai(&request, &config).map_err(|error| error.to_string())
}
