use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPolishConfig {
    pub provider: String,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextPolishRequest {
    pub raw_text: String,
    pub persona_description: String,
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
            Self::MissingApiKey => write!(formatter, "文本处理 API Key 不能为空"),
            Self::MissingRawText => write!(formatter, "原始识别文本不能为空"),
            Self::RequestFailed(message) => write!(formatter, "文本整理请求失败：{message}"),
            Self::InvalidResponse(message) => {
                write!(formatter, "文本整理响应解析失败：{message}")
            }
        }
    }
}

impl std::error::Error for TextPolishError {}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionsRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<ThinkingConfig>,
}

#[derive(Debug, Serialize)]
struct ThinkingConfig {
    r#type: &'static str,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionsResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

pub fn polish_text_with_provider(
    request: &TextPolishRequest,
    config: &TextPolishConfig,
) -> Result<TextPolishResult, TextPolishError> {
    validate_request(request, config)?;

    match send_polish_request(request, config) {
        Ok(final_text) => {
            eprintln!("[⏱️ 文本润色] ✅ 润色成功");
            Ok(TextPolishResult {
                final_text,
                used_fallback: false,
                error_message: None,
            })
        }
        Err(error @ (TextPolishError::RequestFailed(_) | TextPolishError::InvalidResponse(_))) => {
            eprintln!("[⏱️ 文本润色] ❌ 润色失败，使用降级方案: {}", error);
            Ok(TextPolishResult {
                final_text: request.raw_text.trim().to_string(),
                used_fallback: true,
                error_message: Some(error.to_string()),
            })
        }
        Err(error) => {
            eprintln!("[⏱️ 文本润色] ❌ 验证失败: {}", error);
            Err(error)
        }
    }
}

fn send_polish_request(
    request: &TextPolishRequest,
    config: &TextPolishConfig,
) -> Result<String, TextPolishError> {
    let start_time = std::time::Instant::now();
    eprintln!("[⏱️ 文本润色] 开始构建请求");

    let step1_start = std::time::Instant::now();
    let system_message = build_instructions(request);
    let user_message = build_input(request);
    eprintln!("[⏱️ 文本润色] 构建消息 - 耗时 {:?}", step1_start.elapsed());

    let step2_start = std::time::Instant::now();
    let body = ChatCompletionsRequest {
        model: config.model.trim().to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_message,
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_message,
            },
        ],
        temperature: 0.3,
        // 语音整理通常只需要短输出。限制生成长度可避免模型过度推理。
        max_tokens: 512,
        // GLM-4.7 默认会生成较长 reasoning_content；实测短句可从约 21 秒
        // 降到约 2.6 秒。其他 OpenAI 兼容服务不发送该扩展字段。
        thinking: config
            .provider
            .trim()
            .eq_ignore_ascii_case("zhipu")
            .then_some(ThinkingConfig { r#type: "disabled" }),
    };
    eprintln!(
        "[⏱️ 文本润色] 构建请求体 - 耗时 {:?}",
        step2_start.elapsed()
    );

    let step3_start = std::time::Instant::now();
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(12)))
        .http_status_as_error(false)
        .build()
        .new_agent();
    let response = agent
        .post(&chat_completions_url(&config.base_url))
        .header(
            "Authorization",
            &format!("Bearer {}", config.api_key.trim()),
        )
        .header("Content-Type", "application/json")
        .send_json(&body)
        .map_err(|error| TextPolishError::RequestFailed(error.to_string()))?;
    eprintln!(
        "[⏱️ 文本润色] 发送 HTTP 请求并等待响应 - 耗时 {:?}",
        step3_start.elapsed()
    );

    let status_code = response.status().as_u16();
    if !response.status().is_success() {
        let response_body = response.into_body().read_to_string().unwrap_or_default();
        return Err(TextPolishError::RequestFailed(format!(
            "http status: {status_code}, body: {response_body}"
        )));
    }

    let step4_start = std::time::Instant::now();
    let response: ChatCompletionsResponse = response
        .into_body()
        .read_json()
        .map_err(|error| TextPolishError::InvalidResponse(error.to_string()))?;
    eprintln!(
        "[⏱️ 文本润色] 解析响应 JSON - 耗时 {:?}",
        step4_start.elapsed()
    );

    let step5_start = std::time::Instant::now();
    let final_text = response
        .choices
        .first()
        .map(|choice| choice.message.content.trim().to_string())
        .unwrap_or_default();

    if final_text.is_empty() {
        return Err(TextPolishError::InvalidResponse(
            "响应缺少文本内容".to_string(),
        ));
    }
    eprintln!(
        "[⏱️ 文本润色] 提取文本内容 - 耗时 {:?}",
        step5_start.elapsed()
    );

    eprintln!("[⏱️ 文本润色] 总耗时: {:?}", start_time.elapsed());

    Ok(final_text)
}

fn validate_request(
    request: &TextPolishRequest,
    config: &TextPolishConfig,
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
    let mut instructions = format!(
        "你是 AI 语音输入助手，负责把 ASR 原始识别文本整理成可直接使用的文本。\n\
        人格风格要求：{}\n\n\
        处理原则：\n\
        1. 必须按人格风格要求进行风格化改写，让输出呈现该人格指定的表达方式\n\
        2. 不要原样返回原始识别文本，也不要只做标点或语病清理\n\
        3. 保留用户原意，只保留用户表达的事实、意图和关键信息，不要编造用户没有表达的新事实\n",
        request.persona_description.trim()
    );

    // 注入热词到 instructions
    if !request.hotword_context.trim().is_empty() {
        instructions.push_str("\n用户定义了以下热词：\n");
        instructions.push_str(request.hotword_context.trim());
        instructions.push_str("\n\n处理语音识别文本时，优先识别和使用这些热词。");
    }

    instructions.push_str(
        "\n\n通用要求：\n\
        1. 自动补标点和断句，使语句通顺\n\
        2. 去除口头禅（如：嗯、啊、那个、就是说、然后呢）和无意义的重复表达\n\
        3. 修正明显的语法错误和不通顺的表达\n\
        4. 当人格风格要求与通用清理要求冲突时，优先满足人格风格要求\n\
        5. 只输出改写后的最终文本，不要添加任何解释或说明",
    );
    instructions
}

fn build_input(request: &TextPolishRequest) -> String {
    format!(
        "原始识别文本：\n{}\n\n输出要求：按人格风格要求改写为可直接复制使用的文本。不要原样返回原文。",
        request.raw_text.trim()
    )
}

fn chat_completions_url(base_url: &str) -> String {
    format!("{}/chat/completions", base_url.trim_end_matches('/'))
}

#[tauri::command]
pub fn polish_text(
    request: TextPolishRequest,
    provider: String,
    api_key: String,
    base_url: String,
    model: String,
) -> Result<TextPolishResult, String> {
    let config = TextPolishConfig {
        provider,
        api_key,
        base_url,
        model,
    };

    polish_text_with_provider(&request, &config).map_err(|error| error.to_string())
}
