use std::{collections::BTreeMap, sync::Arc, time::Duration};

use serde::Serialize;

use super::{
    catalog::ProviderSettings,
    error::{ProviderError, ProviderErrorKind},
    text::{TextInput, TextOutput, TextProvider},
    transport,
};

pub fn built_in_text_providers() -> Vec<Arc<dyn TextProvider>> {
    vec![
        Arc::new(OpenAiText("zhipu")),
        Arc::new(OpenAiText("openai")),
        Arc::new(OpenAiText("qwen")),
    ]
}

struct OpenAiText(&'static str);

impl TextProvider for OpenAiText {
    fn id(&self) -> &'static str {
        self.0
    }

    fn polish(
        &self,
        input: &TextInput,
        settings: &ProviderSettings,
    ) -> Result<TextOutput, ProviderError> {
        if input.raw_text.trim().is_empty() {
            return Err(ProviderError::new(
                self.id(),
                &settings.model,
                ProviderErrorKind::IncompatibleInput,
                None,
                "ASR 原始文本不能为空",
            )
            .global());
        }
        let legacy_request = crate::text_polish::TextPolishRequest {
            raw_text: input.raw_text.clone(),
            persona_id: input.persona_id.clone(),
            persona_description: input.persona_description.clone(),
            hotword_context: input.hotword_context.clone(),
        };
        let body = ChatRequest {
            model: settings.model.trim(),
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: crate::text_polish::build_instructions(&legacy_request),
                },
                ChatMessage {
                    role: "user",
                    content: crate::text_polish::build_input(&legacy_request),
                },
            ],
            temperature: 0.3,
            max_tokens: 512,
            thinking: (self.id() == "zhipu").then_some(ThinkingConfig { r#type: "disabled" }),
            enable_thinking: (self.id() == "qwen").then_some(false),
        };
        let response = transport::post_json(
            self.id(),
            &settings.model,
            &settings.base_url,
            "chat/completions",
            &settings.api_key,
            &BTreeMap::new(),
            &body,
            Duration::from_secs(12),
        )?;
        let text = response
            .pointer("/choices/0/message/content")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                ProviderError::new(
                    self.id(),
                    &settings.model,
                    ProviderErrorKind::InvalidResponse,
                    Some(200),
                    "响应缺少文本内容",
                )
            })?;
        crate::text_polish::validate_model_output(&legacy_request, text).map_err(|_| {
            ProviderError::new(
                self.id(),
                &settings.model,
                ProviderErrorKind::InvalidResponse,
                Some(200),
                "模型返回了内部整理指令",
            )
        })?;
        Ok(TextOutput {
            text: crate::text_polish::finalize_text(&legacy_request, text),
            provider: self.id().to_string(),
            model: settings.model.clone(),
        })
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_thinking: Option<bool>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct ThinkingConfig {
    r#type: &'static str,
}
