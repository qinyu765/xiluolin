use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::providers::catalog::{ProviderRoutingConfig, ProviderSettings};

pub const GENERAL_PERSONA_ID: &str = "general";
pub const VERBATIM_PERSONA_ID: &str = "verbatim";
pub const POLISH_PROCESSING_MODE: &str = "polish";
pub const VERBATIM_PROCESSING_MODE: &str = "verbatim";
const DEFAULT_PERSONA_ID: &str = GENERAL_PERSONA_ID;
pub(crate) const APP_CONFIG_STORE: &str = "settings.json";
pub(crate) const APP_CONFIG_KEY: &str = "app_config";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct AppConfig {
    #[serde(default = "legacy_config_version")]
    pub config_version: u16,
    #[serde(default)]
    pub asr: ProviderRoutingConfig,
    #[serde(default)]
    pub text: ProviderRoutingConfig,
    pub default_persona_id: String,
    #[serde(default = "default_asr_provider", skip_serializing)]
    #[specta(skip)]
    pub asr_provider: String,
    #[serde(default, skip_serializing)]
    #[specta(skip)]
    pub asr_api_key: String,
    #[serde(default = "default_asr_base_url", skip_serializing)]
    #[specta(skip)]
    pub asr_base_url: String,
    #[serde(default = "default_asr_model", skip_serializing)]
    #[specta(skip)]
    pub asr_model: String,
    #[serde(default = "default_openai_asr_model", skip_serializing)]
    #[specta(skip)]
    pub openai_asr_model: String,
    #[serde(default, skip_serializing)]
    #[specta(skip)]
    pub openai_api_key: String,
    #[serde(default = "default_openai_base_url", skip_serializing)]
    #[specta(skip)]
    pub openai_base_url: String,
    #[serde(default = "default_openai_model", skip_serializing)]
    #[specta(skip)]
    pub openai_model: String,
    #[serde(default = "default_text_provider", skip_serializing)]
    #[specta(skip)]
    pub text_provider: String,
    #[serde(default, skip_serializing)]
    #[specta(skip)]
    pub zhipu_api_key: String,
    #[serde(default = "default_zhipu_base_url", skip_serializing)]
    #[specta(skip)]
    pub zhipu_base_url: String,
    #[serde(default = "default_zhipu_model", skip_serializing)]
    #[specta(skip)]
    pub zhipu_model: String,
    #[serde(default)]
    pub longpress_shortcut: String,
    #[serde(default)]
    pub toggle_shortcut: String,
    #[serde(default)]
    pub fn_hold_enabled: bool,
    pub auto_save_history: bool,
    #[serde(default)]
    pub mute_system_audio: bool,
    #[serde(default)]
    pub selected_microphone: String,
    #[serde(default)]
    pub retain_recordings: bool,
    #[serde(default = "default_local_asr_model", skip_serializing)]
    #[specta(skip)]
    pub local_asr_model: String,
    #[serde(default, skip_serializing)]
    #[specta(skip)]
    pub allow_cloud_fallback: bool,
    #[serde(default = "default_fallback_asr_provider", skip_serializing)]
    #[specta(skip)]
    pub fallback_asr_provider: String,
}

impl AppConfig {
    pub fn selected_asr_config(&self) -> (&str, &str, &str) {
        self.cloud_asr_config(&self.asr_provider)
    }

    pub fn cloud_asr_config(&self, provider: &str) -> (&str, &str, &str) {
        if provider == "openai" {
            (
                &self.openai_api_key,
                &self.openai_base_url,
                &self.openai_asr_model,
            )
        } else {
            (&self.asr_api_key, &self.asr_base_url, &self.asr_model)
        }
    }

    pub fn selected_text_config(&self) -> (&str, &str, &str) {
        if self.text_provider == "zhipu" {
            (&self.zhipu_api_key, &self.zhipu_base_url, &self.zhipu_model)
        } else {
            (
                &self.openai_api_key,
                &self.openai_base_url,
                &self.openai_model,
            )
        }
    }

    pub fn selected_asr_settings(&self) -> Option<&ProviderSettings> {
        self.asr.settings.get(&self.asr.primary)
    }

    pub fn selected_text_settings(&self) -> Option<&ProviderSettings> {
        self.text.settings.get(&self.text.primary)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct Persona {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub is_default: bool,
    pub processing_mode: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct PersonaDraft {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub processing_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct DefaultPersonaUpdate {
    pub personas: Vec<Persona>,
    pub config: AppConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct Hotword {
    pub id: String,
    pub text: String,
    pub category: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct HotwordDraft {
    pub text: String,
    pub category: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct HistoryRecord {
    pub id: String,
    pub raw_text: String,
    pub final_text: String,
    pub persona_id: String,
    pub persona_name: String,
    #[specta(type = specta_typescript::Number)]
    pub duration_ms: i64,
    #[specta(type = specta_typescript::Number)]
    pub output_chars: i64,
    pub output_mode: String,
    pub source: String,
    pub asr_provider: String,
    pub asr_model: String,
    pub text_provider: String,
    pub text_model: String,
    pub text_processing_mode: String,
    pub used_asr_fallback: bool,
    pub used_fallback: bool,
    pub delivery_method: String,
    pub audio_path: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct HistoryRecordDraft {
    pub raw_text: String,
    pub final_text: String,
    pub persona_id: String,
    pub persona_name: String,
    #[specta(type = specta_typescript::Number)]
    pub duration_ms: i64,
    pub output_mode: String,
    pub source: String,
    pub asr_provider: String,
    pub asr_model: String,
    pub text_provider: String,
    pub text_model: String,
    pub text_processing_mode: String,
    pub used_asr_fallback: bool,
    pub used_fallback: bool,
    pub delivery_method: String,
    pub audio_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct HistoryStatistics {
    #[specta(type = specta_typescript::Number)]
    pub total_count: i64,
    #[specta(type = specta_typescript::Number)]
    pub total_duration_ms: i64,
    #[specta(type = specta_typescript::Number)]
    pub total_output_chars: i64,
    #[specta(type = specta_typescript::Number)]
    pub estimated_saved_ms: i64,
    pub top_persona_name: Option<String>,
    #[specta(type = specta_typescript::Number)]
    pub top_persona_count: i64,
}

fn default_asr_provider() -> String {
    "zhipu".to_string()
}

fn legacy_config_version() -> u16 {
    1
}

pub fn normalized_processing_mode(mode: &str) -> &'static str {
    if mode == VERBATIM_PROCESSING_MODE {
        VERBATIM_PROCESSING_MODE
    } else {
        POLISH_PROCESSING_MODE
    }
}

fn default_openai_asr_model() -> String {
    "whisper-1".to_string()
}

fn default_asr_base_url() -> String {
    "https://open.bigmodel.cn/api/paas/v4".to_string()
}

fn default_asr_model() -> String {
    "glm-asr-2512".to_string()
}

fn default_openai_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_openai_model() -> String {
    "gpt-4o-mini".to_string()
}

fn default_text_provider() -> String {
    "zhipu".to_string()
}

fn default_zhipu_base_url() -> String {
    "https://open.bigmodel.cn/api/paas/v4".to_string()
}

fn default_zhipu_model() -> String {
    "glm-4.7-flash".to_string()
}

fn default_local_asr_model() -> String {
    crate::local_asr_model::LOCAL_ASR_MODEL_NAME.to_string()
}

fn default_fallback_asr_provider() -> String {
    "zhipu".to_string()
}

pub fn default_app_config() -> AppConfig {
    let asr = ProviderRoutingConfig {
        primary: "zhipu".to_string(),
        fallbacks: Vec::new(),
        settings: BTreeMap::from([
            (
                "zhipu".to_string(),
                provider_settings("https://open.bigmodel.cn/api/paas/v4", "glm-asr-2512"),
            ),
            (
                "openai".to_string(),
                provider_settings("https://api.openai.com/v1", "whisper-1"),
            ),
            (
                "local".to_string(),
                provider_settings("", &default_local_asr_model()),
            ),
            (
                "qwen-audio".to_string(),
                provider_settings("https://dashscope.aliyuncs.com", "qwen-audio-3.0-asr-flash"),
            ),
            (
                "qwen3-asr".to_string(),
                provider_settings(
                    "https://dashscope.aliyuncs.com/compatible-mode/v1",
                    "qwen3-asr-flash",
                ),
            ),
        ]),
    };
    let text = ProviderRoutingConfig {
        primary: "zhipu".to_string(),
        fallbacks: Vec::new(),
        settings: BTreeMap::from([
            (
                "zhipu".to_string(),
                provider_settings(&default_zhipu_base_url(), &default_zhipu_model()),
            ),
            (
                "openai".to_string(),
                provider_settings("https://api.openai.com/v1", "gpt-4o-mini"),
            ),
            (
                "qwen".to_string(),
                provider_settings(
                    "https://dashscope.aliyuncs.com/compatible-mode/v1",
                    "qwen3.7-flash",
                ),
            ),
        ]),
    };
    AppConfig {
        config_version: 2,
        asr,
        text,
        default_persona_id: DEFAULT_PERSONA_ID.to_string(),
        asr_provider: default_asr_provider(),
        asr_api_key: "".to_string(),
        asr_base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
        asr_model: "glm-asr-2512".to_string(),
        openai_asr_model: default_openai_asr_model(),
        openai_api_key: "".to_string(),
        openai_base_url: "https://api.openai.com/v1".to_string(),
        openai_model: "gpt-4o-mini".to_string(),
        text_provider: default_text_provider(),
        zhipu_api_key: "".to_string(),
        zhipu_base_url: default_zhipu_base_url(),
        zhipu_model: default_zhipu_model(),
        longpress_shortcut: "CommandOrControl+Shift+R".to_string(),
        toggle_shortcut: "Alt+Space".to_string(),
        fn_hold_enabled: false,
        auto_save_history: true,
        mute_system_audio: false,
        selected_microphone: "".to_string(),
        retain_recordings: false,
        local_asr_model: default_local_asr_model(),
        allow_cloud_fallback: false,
        fallback_asr_provider: default_fallback_asr_provider(),
    }
}

pub fn migrate_config_to_v2(mut config: AppConfig) -> Result<AppConfig, String> {
    if config.config_version >= 2 {
        config.asr.validate()?;
        config.text.validate()?;
        return Ok(config);
    }

    let asr_primary = config.asr_provider.trim().to_string();
    let text_primary = config.text_provider.trim().to_string();
    let mut asr_settings = BTreeMap::from([
        (
            "zhipu".to_string(),
            ProviderSettings {
                api_key: config.asr_api_key.clone(),
                base_url: config.asr_base_url.clone(),
                model: config.asr_model.clone(),
                options: BTreeMap::new(),
            },
        ),
        (
            "openai".to_string(),
            ProviderSettings {
                api_key: config.openai_api_key.clone(),
                base_url: config.openai_base_url.clone(),
                model: config.openai_asr_model.clone(),
                options: BTreeMap::new(),
            },
        ),
        (
            "local".to_string(),
            provider_settings("", &config.local_asr_model),
        ),
    ]);
    for (provider, settings) in default_app_config().asr.settings {
        asr_settings.entry(provider).or_insert(settings);
    }
    let fallbacks = if asr_primary == "local" && config.allow_cloud_fallback {
        vec![config.fallback_asr_provider.clone()]
    } else {
        Vec::new()
    };
    config.asr = ProviderRoutingConfig {
        primary: asr_primary,
        fallbacks,
        settings: asr_settings,
    };

    let mut text_settings = BTreeMap::from([
        (
            "zhipu".to_string(),
            ProviderSettings {
                api_key: config.zhipu_api_key.clone(),
                base_url: config.zhipu_base_url.clone(),
                model: config.zhipu_model.clone(),
                options: BTreeMap::new(),
            },
        ),
        (
            "openai".to_string(),
            ProviderSettings {
                api_key: config.openai_api_key.clone(),
                base_url: config.openai_base_url.clone(),
                model: config.openai_model.clone(),
                options: BTreeMap::new(),
            },
        ),
    ]);
    for (provider, settings) in default_app_config().text.settings {
        text_settings.entry(provider).or_insert(settings);
    }
    config.text = ProviderRoutingConfig {
        primary: text_primary,
        fallbacks: Vec::new(),
        settings: text_settings,
    };
    config.config_version = 2;
    config.asr.validate()?;
    config.text.validate()?;
    Ok(config)
}

fn provider_settings(base_url: &str, model: &str) -> ProviderSettings {
    ProviderSettings {
        api_key: String::new(),
        base_url: base_url.to_string(),
        model: model.to_string(),
        options: BTreeMap::new(),
    }
}
