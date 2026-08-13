use xiluolin_lib::{
    credentials::{sanitized_config, AppCredentials},
    data::{migrate_config_to_v2, AppConfig},
};

fn legacy_config_json(allow_cloud_fallback: bool) -> serde_json::Value {
    serde_json::json!({
        "default_persona_id": "general",
        "asr_provider": "local",
        "asr_api_key": "zhipu-asr-key",
        "asr_base_url": "https://zhipu.example/v4",
        "asr_model": "glm-asr-2512",
        "openai_asr_model": "whisper-1",
        "openai_api_key": "openai-shared-key",
        "openai_base_url": "https://openai.example/v1",
        "openai_model": "gpt-4o-mini",
        "text_provider": "openai",
        "zhipu_api_key": "zhipu-text-key",
        "zhipu_base_url": "https://zhipu.example/v4",
        "zhipu_model": "glm-4.7-flash",
        "longpress_shortcut": "CommandOrControl+Shift+R",
        "toggle_shortcut": "Alt+Space",
        "auto_save_history": true,
        "local_asr_model": "ggml-small.bin",
        "allow_cloud_fallback": allow_cloud_fallback,
        "fallback_asr_provider": "openai"
    })
}

#[test]
fn legacy_config_migrates_routes_without_losing_provider_settings() {
    let legacy: AppConfig = serde_json::from_value(legacy_config_json(true)).unwrap();
    let migrated = migrate_config_to_v2(legacy).expect("v1 config should migrate");

    assert_eq!(migrated.config_version, 2);
    assert_eq!(migrated.asr.primary, "local");
    assert_eq!(migrated.asr.fallbacks, vec!["openai"]);
    assert_eq!(migrated.asr.settings["local"].model, "ggml-small.bin");
    assert_eq!(migrated.asr.settings["openai"].model, "whisper-1");
    assert_eq!(
        migrated.asr.settings["openai"].base_url,
        "https://openai.example/v1"
    );
    assert_eq!(migrated.text.primary, "openai");
    assert_eq!(migrated.text.settings["openai"].model, "gpt-4o-mini");

    let credentials = AppCredentials::from_config(&migrated);
    assert_eq!(credentials.asr["zhipu"], "zhipu-asr-key");
    assert_eq!(credentials.asr["openai"], "openai-shared-key");
    assert_eq!(credentials.text["openai"], "openai-shared-key");
    assert_eq!(credentials.text["zhipu"], "zhipu-text-key");
}

#[test]
fn disabled_legacy_cloud_fallback_does_not_create_route_fallback() {
    let legacy: AppConfig = serde_json::from_value(legacy_config_json(false)).unwrap();
    let migrated = migrate_config_to_v2(legacy).expect("v1 config should migrate");

    assert!(migrated.asr.fallbacks.is_empty());
}

#[test]
fn sanitized_v2_config_contains_no_keys_or_legacy_provider_fields() {
    let legacy: AppConfig = serde_json::from_value(legacy_config_json(true)).unwrap();
    let migrated = migrate_config_to_v2(legacy).expect("v1 config should migrate");
    let persisted = serde_json::to_value(sanitized_config(&migrated)).unwrap();
    let encoded = persisted.to_string();

    assert_eq!(persisted["config_version"], 2);
    assert_eq!(persisted["asr"]["primary"], "local");
    assert!(!encoded.contains("zhipu-asr-key"));
    assert!(!encoded.contains("openai-shared-key"));
    assert!(!encoded.contains("zhipu-text-key"));
    assert!(persisted.get("asr_provider").is_none());
    assert!(persisted.get("openai_api_key").is_none());
    assert!(persisted.get("allow_cloud_fallback").is_none());
}
