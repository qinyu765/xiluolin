use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};

use super::{
    catalog::{ProviderRoutingConfig, ProviderSettings},
    error::{ProviderError, ProviderErrorKind, ProviderErrorScope, ProviderRouteError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsrInput {
    pub audio_path: PathBuf,
    pub hotwords: Vec<String>,
    pub context_prompt: Option<String>,
    pub local_model_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct AsrOutput {
    pub text: String,
    pub provider: String,
    pub model: String,
}

pub trait AsrProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn transcribe(
        &self,
        input: &AsrInput,
        settings: &ProviderSettings,
    ) -> Result<AsrOutput, ProviderError>;
}

#[derive(Default)]
pub struct AsrProviderRegistry {
    providers: BTreeMap<String, Arc<dyn AsrProvider>>,
}

impl AsrProviderRegistry {
    pub fn register(&mut self, provider: Arc<dyn AsrProvider>) -> Result<(), String> {
        let id = provider.id().to_string();
        if self.providers.insert(id.clone(), provider).is_some() {
            return Err(format!("ASR Provider ID 重复：{id}"));
        }
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn AsrProvider>> {
        self.providers.get(id)
    }
}

pub fn default_asr_registry() -> AsrProviderRegistry {
    let mut registry = AsrProviderRegistry::default();
    for provider in super::asr_adapters::built_in_asr_providers() {
        registry
            .register(provider)
            .expect("内置 ASR Provider ID 必须唯一");
    }
    registry
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ProviderAttempt {
    pub provider: String,
    pub model: String,
    pub error_kind: Option<ProviderErrorKind>,
    pub http_status: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct AsrRouteResult {
    pub output: AsrOutput,
    pub used_fallback: bool,
    pub attempts: Vec<ProviderAttempt>,
}

pub fn route_asr(
    input: &AsrInput,
    routing: &ProviderRoutingConfig,
    registry: &AsrProviderRegistry,
) -> Result<AsrRouteResult, ProviderRouteError> {
    if let Err(message) = routing.validate() {
        return Err(ProviderRouteError {
            capability: "ASR",
            errors: vec![ProviderError::new(
                "route",
                "",
                ProviderErrorKind::Configuration,
                None,
                message,
            )],
        });
    }

    let mut errors = Vec::new();
    let mut attempts = Vec::new();
    for (index, provider_id) in routing.provider_ids().enumerate() {
        let settings = match routing.settings.get(provider_id) {
            Some(settings) => settings,
            None => {
                let error = ProviderError::new(
                    provider_id,
                    "",
                    ProviderErrorKind::Configuration,
                    None,
                    "缺少 Provider 配置",
                );
                attempts.push(attempt_from_error(&error));
                errors.push(error);
                continue;
            }
        };
        let Some(provider) = registry.get(provider_id) else {
            let error = ProviderError::new(
                provider_id,
                &settings.model,
                ProviderErrorKind::Configuration,
                None,
                "未知 ASR Provider",
            );
            attempts.push(attempt_from_error(&error));
            errors.push(error);
            continue;
        };
        match provider.transcribe(input, settings) {
            Ok(output) => {
                attempts.push(ProviderAttempt {
                    provider: output.provider.clone(),
                    model: output.model.clone(),
                    error_kind: None,
                    http_status: None,
                });
                return Ok(AsrRouteResult {
                    output,
                    used_fallback: index > 0,
                    attempts,
                });
            }
            Err(error) => {
                let global = error.scope == ProviderErrorScope::Global;
                attempts.push(attempt_from_error(&error));
                errors.push(error);
                if global {
                    break;
                }
            }
        }
    }

    Err(ProviderRouteError {
        capability: "ASR",
        errors,
    })
}

fn attempt_from_error(error: &ProviderError) -> ProviderAttempt {
    ProviderAttempt {
        provider: error.provider.clone(),
        model: error.model.clone(),
        error_kind: Some(error.kind),
        http_status: error.http_status,
    }
}
