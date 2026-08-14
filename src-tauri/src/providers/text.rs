use std::{collections::BTreeMap, sync::Arc};

use serde::{Deserialize, Serialize};

use super::{
    asr::ProviderAttempt,
    catalog::{ProviderRoutingConfig, ProviderSettings},
    error::{ProviderError, ProviderErrorKind, ProviderErrorScope, ProviderRouteError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInput {
    pub raw_text: String,
    pub persona_id: String,
    pub persona_description: String,
    pub hotword_context: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct TextOutput {
    pub text: String,
    pub provider: String,
    pub model: String,
}

pub trait TextProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn polish(
        &self,
        input: &TextInput,
        settings: &ProviderSettings,
    ) -> Result<TextOutput, ProviderError>;
}

#[derive(Default)]
pub struct TextProviderRegistry {
    providers: BTreeMap<String, Arc<dyn TextProvider>>,
}

impl TextProviderRegistry {
    pub fn register(&mut self, provider: Arc<dyn TextProvider>) -> Result<(), String> {
        let id = provider.id().to_string();
        if self.providers.insert(id.clone(), provider).is_some() {
            return Err(format!("Text Provider ID 重复：{id}"));
        }
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn TextProvider>> {
        self.providers.get(id)
    }
}

pub fn default_text_registry() -> TextProviderRegistry {
    let mut registry = TextProviderRegistry::default();
    for provider in super::text_adapters::built_in_text_providers() {
        registry
            .register(provider)
            .expect("内置 Text Provider ID 必须唯一");
    }
    registry
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct TextRouteResult {
    pub output: TextOutput,
    pub used_text_fallback: bool,
    pub attempts: Vec<ProviderAttempt>,
}

pub fn route_text(
    input: &TextInput,
    routing: &ProviderRoutingConfig,
    registry: &TextProviderRegistry,
) -> Result<TextRouteResult, ProviderRouteError> {
    if let Err(message) = routing.validate() {
        return Err(route_configuration_error(message));
    }
    if input.raw_text.trim().is_empty() {
        return Err(route_configuration_error("ASR 原始文本不能为空"));
    }

    let mut attempts = Vec::new();
    for (index, provider_id) in routing.provider_ids().enumerate() {
        let Some(settings) = routing.settings.get(provider_id) else {
            attempts.push(failed_attempt(
                provider_id,
                "",
                ProviderErrorKind::Configuration,
                None,
            ));
            continue;
        };
        let Some(provider) = registry.get(provider_id) else {
            attempts.push(failed_attempt(
                provider_id,
                &settings.model,
                ProviderErrorKind::Configuration,
                None,
            ));
            continue;
        };
        match provider.polish(input, settings) {
            Ok(output) => {
                attempts.push(ProviderAttempt {
                    provider: output.provider.clone(),
                    model: output.model.clone(),
                    error_kind: None,
                    http_status: None,
                });
                return Ok(TextRouteResult {
                    output,
                    used_text_fallback: index > 0,
                    attempts,
                });
            }
            Err(error) => {
                let global = error.scope == ProviderErrorScope::Global;
                attempts.push(failed_attempt(
                    &error.provider,
                    &error.model,
                    error.kind,
                    error.http_status,
                ));
                if global {
                    break;
                }
            }
        }
    }

    Ok(TextRouteResult {
        output: TextOutput {
            text: input.raw_text.trim().to_string(),
            provider: String::new(),
            model: String::new(),
        },
        used_text_fallback: true,
        attempts,
    })
}

fn route_configuration_error(message: impl AsRef<str>) -> ProviderRouteError {
    ProviderRouteError {
        capability: "Text",
        errors: vec![ProviderError::new(
            "route",
            "",
            ProviderErrorKind::Configuration,
            None,
            message,
        )],
    }
}

fn failed_attempt(
    provider: &str,
    model: &str,
    kind: ProviderErrorKind,
    http_status: Option<u16>,
) -> ProviderAttempt {
    ProviderAttempt {
        provider: provider.to_string(),
        model: model.to_string(),
        error_kind: Some(kind),
        http_status,
    }
}
