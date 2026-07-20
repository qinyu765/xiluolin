use serde::{Deserialize, Serialize};

pub const GENERAL_PERSONA_ID: &str = "general";
const DEFAULT_PERSONA_ID: &str = GENERAL_PERSONA_ID;
pub(crate) const APP_CONFIG_STORE: &str = "settings.json";
pub(crate) const APP_CONFIG_KEY: &str = "app_config";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct AppConfig {
    pub default_persona_id: String,
    #[serde(default = "default_asr_provider")]
    pub asr_provider: String,
    #[serde(default)]
    pub asr_api_key: String,
    pub asr_base_url: String,
    pub asr_model: String,
    #[serde(default = "default_openai_asr_model")]
    pub openai_asr_model: String,
    #[serde(default)]
    pub openai_api_key: String,
    pub openai_base_url: String,
    pub openai_model: String,
    #[serde(default = "default_text_provider")]
    pub text_provider: String,
    #[serde(default)]
    pub zhipu_api_key: String,
    #[serde(default = "default_zhipu_base_url")]
    pub zhipu_base_url: String,
    #[serde(default = "default_zhipu_model")]
    pub zhipu_model: String,
    #[serde(default)]
    pub longpress_shortcut: String,
    #[serde(default)]
    pub toggle_shortcut: String,
    pub auto_save_history: bool,
    #[serde(default)]
    pub mute_system_audio: bool,
    #[serde(default)]
    pub selected_microphone: String,
    #[serde(default)]
    pub retain_recordings: bool,
    #[serde(default = "default_local_asr_model")]
    pub local_asr_model: String,
    #[serde(default)]
    pub allow_cloud_fallback: bool,
    #[serde(default = "default_fallback_asr_provider")]
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct Persona {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct PersonaDraft {
    pub name: String,
    pub description: String,
    pub icon: String,
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

fn default_openai_asr_model() -> String {
    "whisper-1".to_string()
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
    AppConfig {
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
        auto_save_history: true,
        mute_system_audio: false,
        selected_microphone: "".to_string(),
        retain_recordings: false,
        local_asr_model: default_local_asr_model(),
        allow_cloud_fallback: false,
        fallback_asr_provider: default_fallback_asr_provider(),
    }
}
