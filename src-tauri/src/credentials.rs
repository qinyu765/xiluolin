use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};

use crate::data::AppConfig;

const CREDENTIAL_SERVICE: &str = "com.xiluolin.desktop";
const LEGACY_CREDENTIAL_SERVICE: &str = "com.xiluolin.app";
const BUNDLED_CREDENTIAL_ACCOUNT: &str = "app_credentials_v1";

static SYSTEM_CREDENTIAL_CACHE: OnceLock<Mutex<Option<AppCredentials>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CredentialKey {
    Asr,
    OpenAi,
    Zhipu,
}

impl CredentialKey {
    const ALL: [Self; 3] = [Self::Asr, Self::OpenAi, Self::Zhipu];

    fn account(self) -> &'static str {
        match self {
            Self::Asr => "asr_api_key",
            Self::OpenAi => "openai_api_key",
            Self::Zhipu => "zhipu_api_key",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppCredentials {
    pub asr_api_key: String,
    pub openai_api_key: String,
    pub zhipu_api_key: String,
}

impl AppCredentials {
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            asr_api_key: config.asr_api_key.clone(),
            openai_api_key: config.openai_api_key.clone(),
            zhipu_api_key: config.zhipu_api_key.clone(),
        }
    }

    pub fn apply_to(&self, config: &mut AppConfig) {
        config.asr_api_key.clone_from(&self.asr_api_key);
        config.openai_api_key.clone_from(&self.openai_api_key);
        config.zhipu_api_key.clone_from(&self.zhipu_api_key);
    }

    fn get(&self, key: CredentialKey) -> &str {
        match key {
            CredentialKey::Asr => &self.asr_api_key,
            CredentialKey::OpenAi => &self.openai_api_key,
            CredentialKey::Zhipu => &self.zhipu_api_key,
        }
    }

    fn set(&mut self, key: CredentialKey, value: String) {
        match key {
            CredentialKey::Asr => self.asr_api_key = value,
            CredentialKey::OpenAi => self.openai_api_key = value,
            CredentialKey::Zhipu => self.zhipu_api_key = value,
        }
    }
}

pub trait CredentialStore {
    fn get(&self, key: CredentialKey) -> Result<Option<String>, String>;
    fn set(&self, key: CredentialKey, value: &str) -> Result<(), String>;
    fn delete(&self, key: CredentialKey) -> Result<(), String>;
}

pub struct SystemCredentialStore;

impl SystemCredentialStore {
    fn entry(service: &str, key: CredentialKey) -> Result<keyring::Entry, String> {
        keyring::Entry::new(service, key.account())
            .map_err(|error| format!("初始化系统凭据库失败：{error}"))
    }

    fn primary_entry(key: CredentialKey) -> Result<keyring::Entry, String> {
        Self::entry(CREDENTIAL_SERVICE, key)
    }

    fn legacy_entry(key: CredentialKey) -> Result<keyring::Entry, String> {
        Self::entry(LEGACY_CREDENTIAL_SERVICE, key)
    }

    fn bundled_entry() -> Result<keyring::Entry, String> {
        keyring::Entry::new(CREDENTIAL_SERVICE, BUNDLED_CREDENTIAL_ACCOUNT)
            .map_err(|error| format!("初始化系统凭据库失败：{error}"))
    }

    fn read_bundled() -> Result<Option<AppCredentials>, String> {
        match Self::bundled_entry()?.get_password() {
            Ok(value) => serde_json::from_str(&value)
                .map(Some)
                .map_err(|error| format!("解析系统凭据失败：{error}")),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(format!("读取系统凭据失败：{error}")),
        }
    }

    fn write_bundled(credentials: &AppCredentials) -> Result<(), String> {
        let value = serde_json::to_string(credentials)
            .map_err(|error| format!("序列化系统凭据失败：{error}"))?;
        Self::bundled_entry()?
            .set_password(&value)
            .map_err(|error| format!("保存系统凭据失败：{error}"))
    }
}

impl CredentialStore for SystemCredentialStore {
    fn get(&self, key: CredentialKey) -> Result<Option<String>, String> {
        match Self::primary_entry(key)?.get_password() {
            Ok(value) => return Ok(Some(value)),
            Err(keyring::Error::NoEntry) => {}
            Err(error) => return Err(format!("读取系统凭据失败：{error}")),
        }

        match Self::legacy_entry(key)?.get_password() {
            Ok(value) => {
                Self::primary_entry(key)?
                    .set_password(&value)
                    .map_err(|error| format!("迁移系统凭据失败：{error}"))?;
                Ok(Some(value))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(format!("读取旧版系统凭据失败：{error}")),
        }
    }

    fn set(&self, key: CredentialKey, value: &str) -> Result<(), String> {
        Self::primary_entry(key)?
            .set_password(value)
            .map_err(|error| format!("保存系统凭据失败：{error}"))
    }

    fn delete(&self, key: CredentialKey) -> Result<(), String> {
        for entry in [Self::primary_entry(key)?, Self::legacy_entry(key)?] {
            match entry.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => {}
                Err(error) => return Err(format!("删除系统凭据失败：{error}")),
            }
        }
        Ok(())
    }
}

/// Loads all secrets through one Keychain item and caches them for the lifetime of the
/// process. Older releases stored three separate items and called this path from several
/// startup commands, which could produce six or more macOS authorization dialogs.
pub fn load_system_credentials(legacy: &AppCredentials) -> Result<AppCredentials, String> {
    let cache = SYSTEM_CREDENTIAL_CACHE.get_or_init(|| Mutex::new(None));
    let mut cached = cache
        .lock()
        .map_err(|error| format!("系统凭据缓存锁定失败：{error}"))?;
    if let Some(credentials) = cached.as_ref() {
        return Ok(credentials.clone());
    }

    let credentials = match SystemCredentialStore::read_bundled()? {
        Some(credentials) => credentials,
        None => {
            // One-time migration from the three legacy Keychain entries (or the old
            // plaintext config). Future launches only read the bundled entry.
            let credentials = load_credentials(legacy, &SystemCredentialStore)?;
            if credentials != AppCredentials::default() {
                SystemCredentialStore::write_bundled(&credentials)?;
            }
            credentials
        }
    };
    *cached = Some(credentials.clone());
    Ok(credentials)
}

pub fn save_system_credentials(credentials: &AppCredentials) -> Result<(), String> {
    let cache = SYSTEM_CREDENTIAL_CACHE.get_or_init(|| Mutex::new(None));
    let mut cached = cache
        .lock()
        .map_err(|error| format!("系统凭据缓存锁定失败：{error}"))?;

    // Persist an empty bundle as well. It is an explicit tombstone that prevents old
    // split entries from being migrated back after the user clears all API keys.
    SystemCredentialStore::write_bundled(credentials)?;
    *cached = Some(credentials.clone());
    Ok(())
}

pub fn load_credentials(
    legacy: &AppCredentials,
    store: &impl CredentialStore,
) -> Result<AppCredentials, String> {
    let mut loaded = AppCredentials::default();

    for key in CredentialKey::ALL {
        let value = match store.get(key)? {
            Some(value) => value,
            None => {
                let legacy_value = legacy.get(key);
                if !legacy_value.is_empty() {
                    store.set(key, legacy_value)?;
                }
                legacy_value.to_string()
            }
        };
        loaded.set(key, value);
    }

    Ok(loaded)
}

pub fn save_credentials(
    credentials: &AppCredentials,
    store: &impl CredentialStore,
) -> Result<(), String> {
    for key in CredentialKey::ALL {
        let value = credentials.get(key);
        if value.is_empty() {
            store.delete(key)?;
        } else {
            store.set(key, value)?;
        }
    }

    Ok(())
}

pub fn sanitized_config(config: &AppConfig) -> AppConfig {
    let mut sanitized = config.clone();
    sanitized.asr_api_key.clear();
    sanitized.openai_api_key.clear();
    sanitized.zhipu_api_key.clear();
    sanitized
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap};

    use super::*;
    use crate::data::default_app_config;

    #[derive(Default)]
    struct MemoryCredentialStore {
        values: RefCell<HashMap<CredentialKey, String>>,
        fail_on_set: RefCell<Option<CredentialKey>>,
    }

    impl CredentialStore for MemoryCredentialStore {
        fn get(&self, key: CredentialKey) -> Result<Option<String>, String> {
            Ok(self.values.borrow().get(&key).cloned())
        }

        fn set(&self, key: CredentialKey, value: &str) -> Result<(), String> {
            if self.fail_on_set.borrow().as_ref() == Some(&key) {
                return Err("模拟凭据写入失败".to_string());
            }
            self.values.borrow_mut().insert(key, value.to_string());
            Ok(())
        }

        fn delete(&self, key: CredentialKey) -> Result<(), String> {
            self.values.borrow_mut().remove(&key);
            Ok(())
        }
    }

    fn config_with_credentials() -> AppConfig {
        let mut config = default_app_config();
        config.asr_api_key = "asr-secret".to_string();
        config.openai_api_key = "openai-secret".to_string();
        config.zhipu_api_key = "zhipu-secret".to_string();
        config
    }

    #[test]
    fn sanitizing_config_removes_all_api_keys() {
        let config = config_with_credentials();
        let sanitized = sanitized_config(&config);

        assert!(sanitized.asr_api_key.is_empty());
        assert!(sanitized.openai_api_key.is_empty());
        assert!(sanitized.zhipu_api_key.is_empty());
        assert_eq!(sanitized.asr_model, config.asr_model);

        let persisted_json = serde_json::to_string(&sanitized).expect("config should serialize");
        assert!(!persisted_json.contains("asr-secret"));
        assert!(!persisted_json.contains("openai-secret"));
        assert!(!persisted_json.contains("zhipu-secret"));
    }

    #[test]
    fn legacy_plaintext_credentials_are_migrated() {
        let store = MemoryCredentialStore::default();
        let legacy = AppCredentials::from_config(&config_with_credentials());

        let loaded = load_credentials(&legacy, &store).expect("migration should pass");

        assert_eq!(loaded, legacy);
        assert_eq!(
            store.get(CredentialKey::Asr).unwrap(),
            Some("asr-secret".to_string())
        );
        assert_eq!(
            store.get(CredentialKey::OpenAi).unwrap(),
            Some("openai-secret".to_string())
        );
        assert_eq!(
            store.get(CredentialKey::Zhipu).unwrap(),
            Some("zhipu-secret".to_string())
        );
    }

    #[test]
    fn secure_credentials_take_precedence_over_legacy_values() {
        let store = MemoryCredentialStore::default();
        store
            .set(CredentialKey::OpenAi, "secure-openai-secret")
            .unwrap();
        let legacy = AppCredentials::from_config(&config_with_credentials());

        let loaded = load_credentials(&legacy, &store).expect("loading should pass");

        assert_eq!(loaded.openai_api_key, "secure-openai-secret");
    }

    #[test]
    fn failed_migration_returns_error_without_mutating_config() {
        let store = MemoryCredentialStore::default();
        *store.fail_on_set.borrow_mut() = Some(CredentialKey::OpenAi);
        let config = config_with_credentials();
        let legacy = AppCredentials::from_config(&config);

        let result = load_credentials(&legacy, &store);

        assert!(result.is_err());
        assert_eq!(config.openai_api_key, "openai-secret");
        assert_eq!(config.zhipu_api_key, "zhipu-secret");
    }

    #[test]
    fn bundled_credentials_round_trip_as_one_value() {
        let credentials = AppCredentials::from_config(&config_with_credentials());
        let encoded = serde_json::to_string(&credentials).expect("credentials should serialize");
        let decoded: AppCredentials =
            serde_json::from_str(&encoded).expect("credentials should deserialize");

        assert_eq!(decoded, credentials);
        assert!(encoded.contains("asr-secret"));
        assert!(encoded.contains("openai-secret"));
        assert!(encoded.contains("zhipu-secret"));
    }

    #[test]
    fn saving_empty_credentials_deletes_existing_entries() {
        let store = MemoryCredentialStore::default();
        store.set(CredentialKey::Asr, "secret").unwrap();

        save_credentials(&AppCredentials::default(), &store).expect("deletion should pass");

        assert_eq!(store.get(CredentialKey::Asr).unwrap(), None);
    }
}
