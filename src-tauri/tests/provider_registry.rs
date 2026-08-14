use std::{collections::BTreeMap, sync::Arc};

use xiluolin_lib::providers::{
    asr::{route_asr, AsrInput, AsrOutput, AsrProvider, AsrProviderRegistry},
    catalog::{provider_catalog, ProviderOptionValue, ProviderRoutingConfig, ProviderSettings},
    error::{ProviderError, ProviderErrorKind},
    text::{route_text, TextInput, TextOutput, TextProvider, TextProviderRegistry},
};

fn settings() -> ProviderSettings {
    ProviderSettings {
        api_key: "secret".to_string(),
        base_url: "https://example.test/v1".to_string(),
        model: "model-a".to_string(),
        options: BTreeMap::from([(
            "enable_itn".to_string(),
            ProviderOptionValue::Boolean(false),
        )]),
    }
}

fn route(primary: &str, fallbacks: &[&str]) -> ProviderRoutingConfig {
    let ids = std::iter::once(primary)
        .chain(fallbacks.iter().copied())
        .map(str::to_string)
        .collect::<Vec<_>>();
    ProviderRoutingConfig {
        primary: primary.to_string(),
        fallbacks: fallbacks.iter().map(|value| value.to_string()).collect(),
        settings: ids
            .into_iter()
            .map(|id| (id, settings()))
            .collect::<BTreeMap<_, _>>(),
    }
}

#[test]
fn catalog_provider_ids_are_unique_per_capability() {
    let catalog = provider_catalog();
    let asr_ids = catalog
        .asr
        .iter()
        .map(|provider| provider.id.as_str())
        .collect::<Vec<_>>();
    let text_ids = catalog
        .text
        .iter()
        .map(|provider| provider.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        asr_ids,
        vec!["zhipu", "openai", "local", "qwen-audio", "qwen3-asr"]
    );
    assert_eq!(text_ids, vec!["zhipu", "openai", "qwen"]);
    assert_eq!(
        catalog
            .asr
            .iter()
            .find(|provider| provider.id == "qwen-audio")
            .expect("qwen-audio descriptor")
            .default_model,
        "qwen-audio-3.0-asr-flash"
    );
}

#[test]
fn route_rejects_duplicate_or_more_than_three_providers() {
    let duplicate = route("local", &["zhipu", "local"]);
    assert_eq!(
        duplicate.validate().unwrap_err(),
        "Provider 调用链不能包含重复项：local"
    );

    let too_long = route("local", &["zhipu", "openai", "qwen-audio"]);
    assert_eq!(
        too_long.validate().unwrap_err(),
        "Provider 调用链最多包含 3 项"
    );
}

struct FailingAsr(&'static str);

impl AsrProvider for FailingAsr {
    fn id(&self) -> &'static str {
        self.0
    }

    fn transcribe(
        &self,
        _input: &AsrInput,
        settings: &ProviderSettings,
    ) -> Result<AsrOutput, ProviderError> {
        Err(ProviderError::new(
            self.0,
            &settings.model,
            ProviderErrorKind::Network,
            None,
            "网络不可用",
        ))
    }
}

struct SuccessfulAsr(&'static str);

impl AsrProvider for SuccessfulAsr {
    fn id(&self) -> &'static str {
        self.0
    }

    fn transcribe(
        &self,
        _input: &AsrInput,
        settings: &ProviderSettings,
    ) -> Result<AsrOutput, ProviderError> {
        Ok(AsrOutput {
            text: "secondary text".to_string(),
            provider: self.0.to_string(),
            model: settings.model.clone(),
        })
    }
}

#[test]
fn asr_router_uses_secondary_once_and_records_actual_provider() {
    let mut registry = AsrProviderRegistry::default();
    registry.register(Arc::new(FailingAsr("primary"))).unwrap();
    registry
        .register(Arc::new(SuccessfulAsr("secondary")))
        .unwrap();
    let routing = route("primary", &["secondary"]);

    let input = AsrInput {
        audio_path: "fixture.wav".into(),
        hotwords: Vec::new(),
        context_prompt: None,
        local_model_path: None,
    };
    let result = route_asr(&input, &routing, &registry).unwrap();

    assert_eq!(result.output.text, "secondary text");
    assert_eq!(result.output.provider, "secondary");
    assert_eq!(result.output.model, "model-a");
    assert!(result.used_fallback);
    assert_eq!(result.attempts.len(), 2);
}

struct FailingText(&'static str);

impl TextProvider for FailingText {
    fn id(&self) -> &'static str {
        self.0
    }

    fn polish(
        &self,
        _input: &TextInput,
        settings: &ProviderSettings,
    ) -> Result<TextOutput, ProviderError> {
        Err(ProviderError::new(
            self.0,
            &settings.model,
            ProviderErrorKind::RateLimited,
            Some(429),
            "请求过于频繁",
        ))
    }
}

#[test]
fn text_router_returns_raw_text_after_all_providers_fail() {
    let mut registry = TextProviderRegistry::default();
    registry.register(Arc::new(FailingText("first"))).unwrap();
    registry.register(Arc::new(FailingText("second"))).unwrap();
    let routing = route("first", &["second"]);
    let input = TextInput {
        raw_text: "原始 ASR 文本".to_string(),
        persona_id: "general".to_string(),
        persona_description: "自然清晰".to_string(),
        hotword_context: String::new(),
    };

    let result = route_text(&input, &routing, &registry).unwrap();

    assert_eq!(result.output.text, "原始 ASR 文本");
    assert_eq!(result.output.provider, "");
    assert_eq!(result.output.model, "");
    assert!(result.used_text_fallback);
    assert_eq!(result.attempts.len(), 2);
}
