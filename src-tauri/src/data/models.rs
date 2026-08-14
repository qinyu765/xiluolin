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
    #[serde(default = "current_config_version")]
    pub config_version: u16,
    #[serde(default)]
    pub asr: ProviderRoutingConfig,
    #[serde(default)]
    pub text: ProviderRoutingConfig,
    pub default_persona_id: String,
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
    #[serde(default)]
    pub realtime_preview_enabled: bool,
}

impl AppConfig {
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

fn current_config_version() -> u16 {
    2
}

pub fn normalized_processing_mode(mode: &str) -> &'static str {
    if mode == VERBATIM_PROCESSING_MODE {
        VERBATIM_PROCESSING_MODE
    } else {
        POLISH_PROCESSING_MODE
    }
}

fn default_zhipu_base_url() -> String {
    "https://open.bigmodel.cn/api/paas/v4".to_string()
}

fn default_zhipu_model() -> String {
    "glm-4.7-flash".to_string()
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
                provider_settings("", crate::local_asr_model::LOCAL_ASR_MODEL_NAME),
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
        longpress_shortcut: "CommandOrControl+Shift+R".to_string(),
        toggle_shortcut: "Alt+Space".to_string(),
        fn_hold_enabled: false,
        auto_save_history: true,
        mute_system_audio: false,
        selected_microphone: "".to_string(),
        retain_recordings: false,
        realtime_preview_enabled: false,
    }
}

fn provider_settings(base_url: &str, model: &str) -> ProviderSettings {
    ProviderSettings {
        api_key: String::new(),
        base_url: base_url.to_string(),
        model: model.to_string(),
        options: BTreeMap::new(),
    }
}
