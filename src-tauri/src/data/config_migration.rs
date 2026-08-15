//! One-time upgrade path for settings written before the nested Provider schema.
//!
//! `AppConfig` is intentionally v2-only.  The legacy shape lives in this private
//! module so normal runtime code cannot accidentally depend on flat Provider fields.

use std::collections::BTreeMap;

use serde::Deserialize;

use super::{default_app_config, AppConfig};
use crate::providers::catalog::{ProviderRoutingConfig, ProviderSettings};

#[derive(Debug, Deserialize)]
struct LegacyAppConfig {
    #[serde(default = "default_persona_id")]
    default_persona_id: String,
    #[serde(default = "default_asr_provider")]
    asr_provider: String,
    #[serde(default)]
    asr_api_key: String,
    #[serde(default = "default_asr_base_url")]
    asr_base_url: String,
    #[serde(default = "default_asr_model")]
    asr_model: String,
    #[serde(default = "default_openai_asr_model")]
    openai_asr_model: String,
    #[serde(default)]
    openai_api_key: String,
    #[serde(default = "default_openai_base_url")]
    openai_base_url: String,
    #[serde(default = "default_openai_model")]
    openai_model: String,
    #[serde(default = "default_text_provider")]
    text_provider: String,
    #[serde(default)]
    zhipu_api_key: String,
    #[serde(default = "default_zhipu_base_url")]
    zhipu_base_url: String,
    #[serde(default = "default_zhipu_model")]
    zhipu_model: String,
    #[serde(default)]
    longpress_shortcut: String,
    #[serde(default)]
    toggle_shortcut: String,
    #[serde(default)]
    fn_hold_enabled: bool,
    #[serde(default = "default_auto_save_history")]
    auto_save_history: bool,
    #[serde(default)]
    mute_system_audio: bool,
    #[serde(default)]
    selected_microphone: String,
    #[serde(default)]
    retain_recordings: bool,
    #[serde(default = "default_local_asr_model")]
    local_asr_model: String,
    #[serde(default)]
    allow_cloud_fallback: bool,
    #[serde(default = "default_fallback_asr_provider")]
    fallback_asr_provider: String,
    #[serde(default)]
    realtime_preview_enabled: bool,
}

/// Decode either the final v2 shape or the old flat shape.
///
/// The boolean reports whether a one-time migration occurred and is used by the
/// caller to create a sanitized backup before replacing the stored value.
pub fn decode(raw: serde_json::Value) -> Result<(AppConfig, bool), String> {
    let is_v2 = raw
        .get("config_version")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|version| version >= 2)
        || raw.get("asr").is_some()
        || raw.get("text").is_some();

    if is_v2 {
        let mut config: AppConfig =
            serde_json::from_value(raw).map_err(|error| format!("解析 v2 配置失败：{error}"))?;
        config.config_version = 2;
        config.asr.validate()?;
        config.text.validate()?;
        return Ok((config, false));
    }

    let legacy: LegacyAppConfig =
        serde_json::from_value(raw).map_err(|error| format!("解析旧版配置失败：{error}"))?;
    let config = migrate_legacy(legacy)?;
    Ok((config, true))
}

/// Remove every credential-shaped value before writing a human-readable v1 backup.
pub fn sanitized_legacy_backup(raw: &serde_json::Value) -> serde_json::Value {
    fn redact(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(object) => serde_json::Value::Object(
                object
                    .iter()
                    .filter_map(|(key, value)| {
                        let normalized = key.to_ascii_lowercase();
                        if normalized.contains("api_key")
                            || normalized == "apikey"
                            || normalized == "key"
                        {
                            None
                        } else {
                            Some((key.clone(), redact(value)))
                        }
                    })
                    .collect(),
            ),
            serde_json::Value::Array(values) => {
                serde_json::Value::Array(values.iter().map(redact).collect())
            }
            _ => value.clone(),
        }
    }

    redact(raw)
}

fn migrate_legacy(legacy: LegacyAppConfig) -> Result<AppConfig, String> {
    let defaults = default_app_config();
    let mut asr_settings = defaults.asr.settings;
    asr_settings.insert(
        "zhipu".to_string(),
        provider_settings(legacy.asr_api_key, legacy.asr_base_url, legacy.asr_model),
    );
    asr_settings.insert(
        "openai".to_string(),
        provider_settings(
            legacy.openai_api_key.clone(),
            legacy.openai_base_url.clone(),
            legacy.openai_asr_model,
        ),
    );
    asr_settings.insert(
        "local".to_string(),
        provider_settings(String::new(), String::new(), legacy.local_asr_model),
    );

    let fallbacks = if legacy.asr_provider == "local"
        && legacy.allow_cloud_fallback
        && !legacy.fallback_asr_provider.trim().is_empty()
        && legacy.fallback_asr_provider != "local"
    {
        vec![legacy.fallback_asr_provider]
    } else {
        Vec::new()
    };
    let asr = ProviderRoutingConfig {
        primary: legacy.asr_provider.trim().to_string(),
        fallbacks,
        settings: asr_settings,
    };

    let mut text_settings = defaults.text.settings;
    text_settings.insert(
        "zhipu".to_string(),
        provider_settings(
            legacy.zhipu_api_key,
            legacy.zhipu_base_url,
            legacy.zhipu_model,
        ),
    );
    text_settings.insert(
        "openai".to_string(),
        provider_settings(
            legacy.openai_api_key,
            legacy.openai_base_url,
            legacy.openai_model,
        ),
    );
    let text = ProviderRoutingConfig {
        primary: legacy.text_provider.trim().to_string(),
        fallbacks: Vec::new(),
        settings: text_settings,
    };

    let config = AppConfig {
        config_version: 2,
        asr,
        text,
        default_persona_id: legacy.default_persona_id,
        longpress_shortcut: legacy.longpress_shortcut,
        toggle_shortcut: legacy.toggle_shortcut,
        fn_hold_enabled: legacy.fn_hold_enabled,
        auto_save_history: legacy.auto_save_history,
        mute_system_audio: legacy.mute_system_audio,
        selected_microphone: legacy.selected_microphone,
        retain_recordings: legacy.retain_recordings,
        realtime_preview_enabled: legacy.realtime_preview_enabled,
    };
    config.asr.validate()?;
    config.text.validate()?;
    Ok(config)
}

fn provider_settings(api_key: String, base_url: String, model: String) -> ProviderSettings {
    ProviderSettings {
        api_key,
        base_url,
        model,
        options: BTreeMap::new(),
    }
}

fn default_persona_id() -> String {
    super::GENERAL_PERSONA_ID.to_string()
}

fn default_asr_provider() -> String {
    "zhipu".to_string()
}

fn default_asr_base_url() -> String {
    "https://open.bigmodel.cn/api/paas/v4".to_string()
}

fn default_asr_model() -> String {
    "glm-asr-2512".to_string()
}

fn default_openai_asr_model() -> String {
    "whisper-1".to_string()
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

fn default_auto_save_history() -> bool {
    true
}
