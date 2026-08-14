//! Compatibility adapters between the flat `AppConfig` used by the current
//! settings UI and the provider routing model used by the voice platform.
//!
//! The UI keeps its existing storage contract for now. This module is the
//! only place that translates that contract into a primary/fallback route, so
//! the frontend can migrate independently from the capture and provider
//! runtime.

use std::collections::BTreeMap;

use super::{
    asr::{route_asr, AsrInput},
    catalog::{ProviderRoutingConfig, ProviderSettings},
    text::{route_text, TextInput},
};
use crate::{
    asr::{AsrConfig, AsrError, AsrRequest, AsrTranscription},
    text_polish::{TextPolishConfig, TextPolishError, TextPolishRequest},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutedTextResult {
    pub final_text: String,
    pub used_fallback: bool,
    pub error_message: Option<String>,
    pub provider: String,
    pub model: String,
}

pub fn transcribe_audio_file(
    request: &AsrRequest,
    config: &AsrConfig,
) -> Result<AsrTranscription, AsrError> {
    validate_legacy_asr_config(config)?;

    let routing = asr_route_from_legacy_config(config);
    let route = route_asr(
        &AsrInput {
            audio_path: request.audio_path.clone(),
            hotwords: request.hotwords.clone(),
            context_prompt: request.context_prompt.clone(),
            local_model_path: config.local_model_path.clone(),
        },
        &routing,
        &super::asr::default_asr_registry(),
    )
    .map_err(|error| map_asr_route_error(config, error.to_string()))?;

    Ok(AsrTranscription {
        text: route.output.text,
        provider: route.output.provider,
        model: route.output.model,
        used_fallback: route.used_fallback,
    })
}

pub fn polish_text_with_provider(
    request: &TextPolishRequest,
    config: &TextPolishConfig,
) -> Result<RoutedTextResult, TextPolishError> {
    if config.api_key.trim().is_empty() {
        return Err(TextPolishError::MissingApiKey);
    }
    if request.raw_text.trim().is_empty() {
        return Err(TextPolishError::MissingRawText);
    }

    let route = route_text(
        &TextInput {
            raw_text: request.raw_text.clone(),
            persona_id: request.persona_id.clone(),
            persona_description: request.persona_description.clone(),
            hotword_context: request.hotword_context.clone(),
        },
        &text_route_from_legacy_config(config),
        &super::text::default_text_registry(),
    )
    .map_err(|error| TextPolishError::RequestFailed(error.to_string()))?;

    let error_message = route
        .used_text_fallback
        .then(|| "文本 Provider 全部失败，已保留原始识别文本".to_string());
    Ok(RoutedTextResult {
        final_text: route.output.text,
        used_fallback: route.used_text_fallback,
        error_message,
        provider: route.output.provider,
        model: route.output.model,
    })
}

pub fn asr_route_from_legacy_config(config: &AsrConfig) -> ProviderRoutingConfig {
    let mut settings = BTreeMap::new();
    settings.insert(
        config.provider.clone(),
        ProviderSettings {
            api_key: config.api_key.clone(),
            base_url: config.base_url.clone(),
            model: config.model.clone(),
            options: BTreeMap::new(),
        },
    );

    let mut fallbacks = Vec::new();
    if config.allow_cloud_fallback
        && !config.fallback_provider.trim().is_empty()
        && config.fallback_provider != config.provider
    {
        fallbacks.push(config.fallback_provider.clone());
        settings.insert(
            config.fallback_provider.clone(),
            ProviderSettings {
                api_key: config.fallback_api_key.clone(),
                base_url: config.fallback_base_url.clone(),
                model: config.fallback_model.clone(),
                options: BTreeMap::new(),
            },
        );
    }

    ProviderRoutingConfig {
        primary: config.provider.clone(),
        fallbacks,
        settings,
    }
}

fn text_route_from_legacy_config(config: &TextPolishConfig) -> ProviderRoutingConfig {
    ProviderRoutingConfig {
        primary: config.provider.clone(),
        fallbacks: Vec::new(),
        settings: BTreeMap::from([(
            config.provider.clone(),
            ProviderSettings {
                api_key: config.api_key.clone(),
                base_url: config.base_url.clone(),
                model: config.model.clone(),
                options: BTreeMap::new(),
            },
        )]),
    }
}

fn validate_legacy_asr_config(config: &AsrConfig) -> Result<(), AsrError> {
    if config.provider == "local" && config.local_model_path.is_none() {
        return Err(AsrError::MissingLocalModel);
    }
    if config.provider != "local" && config.api_key.trim().is_empty() {
        return Err(AsrError::MissingApiKey);
    }
    Ok(())
}

fn map_asr_route_error(config: &AsrConfig, message: String) -> AsrError {
    if config.provider == "local" && config.local_model_path.is_none() {
        AsrError::MissingLocalModel
    } else if config.provider != "local" && config.api_key.trim().is_empty() {
        AsrError::MissingApiKey
    } else {
        AsrError::RequestFailed(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_legacy_cloud_fallback_into_a_two_step_route() {
        let config = AsrConfig {
            provider: "local".to_string(),
            api_key: String::new(),
            base_url: String::new(),
            model: "whisper-local".to_string(),
            local_model_path: Some("/tmp/model".into()),
            allow_cloud_fallback: true,
            fallback_provider: "zhipu".to_string(),
            fallback_api_key: "secret".to_string(),
            fallback_base_url: "https://example.test".to_string(),
            fallback_model: "glm-asr".to_string(),
        };

        let route = asr_route_from_legacy_config(&config);
        assert_eq!(route.primary, "local");
        assert_eq!(route.fallbacks, ["zhipu"]);
        assert_eq!(route.settings["zhipu"].model, "glm-asr");
    }

    #[test]
    fn does_not_duplicate_a_fallback_that_matches_the_primary() {
        let config = AsrConfig {
            provider: "zhipu".to_string(),
            api_key: "secret".to_string(),
            base_url: "https://example.test".to_string(),
            model: "glm-asr".to_string(),
            local_model_path: None,
            allow_cloud_fallback: true,
            fallback_provider: "zhipu".to_string(),
            fallback_api_key: "secret".to_string(),
            fallback_base_url: "https://example.test".to_string(),
            fallback_model: "glm-asr".to_string(),
        };

        let route = asr_route_from_legacy_config(&config);
        assert!(route.fallbacks.is_empty());
        assert!(route.validate().is_ok());
    }
}
