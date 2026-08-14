use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};

use serde::{Deserialize, Serialize};

use crate::data::AppConfig;

const CREDENTIAL_SERVICE: &str = "com.xiluolin.desktop";
const LEGACY_CREDENTIAL_SERVICE: &str = "com.xiluolin.app";
const BUNDLED_CREDENTIAL_ACCOUNT: &str = "app_credentials_v2";
const LEGACY_BUNDLED_CREDENTIAL_ACCOUNT: &str = "app_credentials_v1";

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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct AppCredentials {
    #[serde(default)]
    pub asr: BTreeMap<String, String>,
    #[serde(default)]
    pub text: BTreeMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
struct LegacyBundledCredentials {
    #[serde(default)]
    asr_api_key: String,
    #[serde(default)]
    openai_api_key: String,
    #[serde(default)]
    zhipu_api_key: String,
}

impl AppCredentials {
    pub fn from_config(config: &AppConfig) -> Self {
        let credentials = Self {
            asr: config
                .asr
                .settings
                .iter()
                .filter(|(_, settings)| !settings.api_key.trim().is_empty())
                .map(|(provider, settings)| (provider.clone(), settings.api_key.trim().to_string()))
                .collect(),
            text: config
                .text
                .settings
                .iter()
                .filter(|(_, settings)| !settings.api_key.trim().is_empty())
                .map(|(provider, settings)| (provider.clone(), settings.api_key.trim().to_string()))
                .collect(),
            ..Self::default()
        };
        credentials
    }

    pub fn apply_to(&self, config: &mut AppConfig) {
        for (provider, api_key) in &self.asr {
            if let Some(settings) = config.asr.settings.get_mut(provider) {
                settings.api_key.clone_from(api_key);
            }
        }
        for (provider, api_key) in &self.text {
            if let Some(settings) = config.text.settings.get_mut(provider) {
                settings.api_key.clone_from(api_key);
            }
        }
    }

    fn get_legacy(&self, key: CredentialKey) -> &str {
        match key {
            CredentialKey::Asr => self.asr.get("zhipu").map(String::as_str).unwrap_or(""),
            CredentialKey::OpenAi => self
                .text
                .get("openai")
                .or_else(|| self.asr.get("openai"))
                .map(String::as_str)
                .unwrap_or(""),
            CredentialKey::Zhipu => self.text.get("zhipu").map(String::as_str).unwrap_or(""),
        }
    }

    fn set_legacy(&mut self, key: CredentialKey, value: String) {
        if value.is_empty() {
            return;
        }
        match key {
            CredentialKey::Asr => {
                self.asr.insert("zhipu".to_string(), value);
            }
            CredentialKey::OpenAi => {
                self.asr.insert("openai".to_string(), value.clone());
                self.text.insert("openai".to_string(), value);
            }
            CredentialKey::Zhipu => {
                self.text.insert("zhipu".to_string(), value);
            }
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

    fn bundled_entry(account: &str) -> Result<keyring::Entry, String> {
        keyring::Entry::new(CREDENTIAL_SERVICE, account)
            .map_err(|error| format!("初始化系统凭据库失败：{error}"))
    }

    fn read_bundled(account: &str) -> Result<Option<AppCredentials>, String> {
        match Self::bundled_entry(account)?.get_password() {
            Ok(value) => {
                let raw: serde_json::Value = serde_json::from_str(&value)
                    .map_err(|error| format!("解析系统凭据失败：{error}"))?;
                if let Ok(credentials) = serde_json::from_value::<AppCredentials>(raw.clone()) {
                    return Ok(Some(credentials));
                }
                let legacy: LegacyBundledCredentials = serde_json::from_value(raw)
                    .map_err(|error| format!("解析旧版系统凭据失败：{error}"))?;
                let mut credentials = AppCredentials::default();
                credentials.set_legacy(CredentialKey::Asr, legacy.asr_api_key);
                credentials.set_legacy(CredentialKey::OpenAi, legacy.openai_api_key);
                credentials.set_legacy(CredentialKey::Zhipu, legacy.zhipu_api_key);
                Ok(Some(credentials))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(format!("读取系统凭据失败：{error}")),
        }
    }

    fn write_bundled(credentials: &AppCredentials) -> Result<(), String> {
        let value = serde_json::to_string(credentials)
            .map_err(|error| format!("序列化系统凭据失败：{error}"))?;
        let entry = Self::bundled_entry(BUNDLED_CREDENTIAL_ACCOUNT)?;
        entry
            .set_password(&value)
            .map_err(|error| format!("保存系统凭据失败：{error}"))?;
        let persisted = entry
            .get_password()
            .map_err(|error| format!("校验系统凭据失败：{error}"))?;
        if persisted != value {
            return Err("校验系统凭据失败：回读内容不一致".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LegacyCredentialLocation {
    BundledV1,
    CurrentSplit(CredentialKey),
    LegacySplit(CredentialKey),
}

impl LegacyCredentialLocation {
    fn all() -> Vec<Self> {
        let mut locations = vec![Self::BundledV1];
        locations.extend(CredentialKey::ALL.map(Self::CurrentSplit));
        locations.extend(CredentialKey::ALL.map(Self::LegacySplit));
        locations
    }
}

trait LegacyCredentialCleanupStore {
    fn read(&self, location: LegacyCredentialLocation) -> Result<Option<String>, String>;
    fn write(&self, location: LegacyCredentialLocation, value: &str) -> Result<(), String>;
    fn delete(&self, location: LegacyCredentialLocation) -> Result<(), String>;
}

impl SystemCredentialStore {
    fn legacy_location_entry(location: LegacyCredentialLocation) -> Result<keyring::Entry, String> {
        match location {
            LegacyCredentialLocation::BundledV1 => {
                Self::bundled_entry(LEGACY_BUNDLED_CREDENTIAL_ACCOUNT)
            }
            LegacyCredentialLocation::CurrentSplit(key) => Self::primary_entry(key),
            LegacyCredentialLocation::LegacySplit(key) => Self::legacy_entry(key),
        }
    }
}

impl LegacyCredentialCleanupStore for SystemCredentialStore {
    fn read(&self, location: LegacyCredentialLocation) -> Result<Option<String>, String> {
        match Self::legacy_location_entry(location)?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(format!("读取旧版系统凭据失败：{error}")),
        }
    }

    fn write(&self, location: LegacyCredentialLocation, value: &str) -> Result<(), String> {
        Self::legacy_location_entry(location)?
            .set_password(value)
            .map_err(|error| format!("恢复旧版系统凭据失败：{error}"))
    }

    fn delete(&self, location: LegacyCredentialLocation) -> Result<(), String> {
        match Self::legacy_location_entry(location)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(format!("清理旧版系统凭据失败：{error}")),
        }
    }
}

fn cleanup_legacy_credentials(store: &impl LegacyCredentialCleanupStore) -> Result<(), String> {
    let snapshot = LegacyCredentialLocation::all()
        .into_iter()
        .filter_map(|location| match store.read(location) {
            Ok(Some(value)) => Some(Ok((location, value))),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<_>, String>>()?;

    for (location, _) in &snapshot {
        if let Err(delete_error) = store.delete(*location) {
            let restore_errors = snapshot
                .iter()
                .filter_map(|(restore_location, value)| store.write(*restore_location, value).err())
                .collect::<Vec<_>>();
            return if restore_errors.is_empty() {
                Err(delete_error)
            } else {
                Err(format!(
                    "{delete_error}; 旧凭据恢复失败：{}",
                    restore_errors.join("; ")
                ))
            };
        }
    }
    Ok(())
}

pub fn finalize_system_credentials_migration() -> Result<(), String> {
    cleanup_legacy_credentials(&SystemCredentialStore)
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

    let credentials = match SystemCredentialStore::read_bundled(BUNDLED_CREDENTIAL_ACCOUNT)? {
        Some(credentials) => credentials,
        None => {
            let credentials =
                match SystemCredentialStore::read_bundled(LEGACY_BUNDLED_CREDENTIAL_ACCOUNT)? {
                    Some(credentials) => credentials,
                    None => load_credentials(legacy, &SystemCredentialStore)?,
                };
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
                let legacy_value = legacy.get_legacy(key);
                if !legacy_value.is_empty() {
                    store.set(key, legacy_value)?;
                }
                legacy_value.to_string()
            }
        };
        loaded.set_legacy(key, value);
    }

    Ok(loaded)
}

pub fn save_credentials(
    credentials: &AppCredentials,
    store: &impl CredentialStore,
) -> Result<(), String> {
    for key in CredentialKey::ALL {
        let value = credentials.get_legacy(key);
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
    for settings in sanitized.asr.settings.values_mut() {
        settings.api_key.clear();
    }
    for settings in sanitized.text.settings.values_mut() {
        settings.api_key.clear();
    }
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

    #[derive(Default)]
    struct MemoryLegacyCleanupStore {
        values: RefCell<HashMap<LegacyCredentialLocation, String>>,
        fail_on_delete: RefCell<Option<LegacyCredentialLocation>>,
    }

    impl LegacyCredentialCleanupStore for MemoryLegacyCleanupStore {
        fn read(&self, location: LegacyCredentialLocation) -> Result<Option<String>, String> {
            Ok(self.values.borrow().get(&location).cloned())
        }

        fn write(&self, location: LegacyCredentialLocation, value: &str) -> Result<(), String> {
            self.values.borrow_mut().insert(location, value.to_string());
            Ok(())
        }

        fn delete(&self, location: LegacyCredentialLocation) -> Result<(), String> {
            if self.fail_on_delete.borrow().as_ref() == Some(&location) {
                return Err("模拟旧凭据清理失败".to_string());
            }
            self.values.borrow_mut().remove(&location);
            Ok(())
        }
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
        config.asr.settings.get_mut("zhipu").unwrap().api_key = "asr-secret".to_string();
        config.asr.settings.get_mut("openai").unwrap().api_key = "openai-secret".to_string();
        config.text.settings.get_mut("openai").unwrap().api_key = "openai-secret".to_string();
        config.text.settings.get_mut("zhipu").unwrap().api_key = "zhipu-secret".to_string();
        config
    }

    #[test]
    fn sanitizing_config_removes_all_api_keys() {
        let config = config_with_credentials();
        let sanitized = sanitized_config(&config);

        assert!(sanitized
            .asr
            .settings
            .values()
            .all(|settings| settings.api_key.is_empty()));
        assert!(sanitized
            .text
            .settings
            .values()
            .all(|settings| settings.api_key.is_empty()));
        assert_eq!(
            sanitized.asr.settings["zhipu"].model,
            config.asr.settings["zhipu"].model
        );

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

        assert_eq!(loaded.asr["openai"], "secure-openai-secret");
        assert_eq!(loaded.text["openai"], "secure-openai-secret");
    }

    #[test]
    fn failed_migration_returns_error_without_mutating_config() {
        let store = MemoryCredentialStore::default();
        *store.fail_on_set.borrow_mut() = Some(CredentialKey::OpenAi);
        let config = config_with_credentials();
        let legacy = AppCredentials::from_config(&config);

        let result = load_credentials(&legacy, &store);

        assert!(result.is_err());
        assert_eq!(config.asr.settings["openai"].api_key, "openai-secret");
        assert_eq!(config.text.settings["zhipu"].api_key, "zhipu-secret");
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

    #[test]
    fn successful_v2_migration_cleans_all_legacy_credential_shapes() {
        let store = MemoryLegacyCleanupStore::default();
        for (index, location) in LegacyCredentialLocation::all().into_iter().enumerate() {
            store
                .values
                .borrow_mut()
                .insert(location, format!("secret-{index}"));
        }

        cleanup_legacy_credentials(&store).expect("cleanup should pass");

        assert!(store.values.borrow().is_empty());
    }

    #[test]
    fn failed_legacy_cleanup_restores_every_old_credential() {
        let store = MemoryLegacyCleanupStore::default();
        let expected = LegacyCredentialLocation::all()
            .into_iter()
            .enumerate()
            .map(|(index, location)| (location, format!("secret-{index}")))
            .collect::<HashMap<_, _>>();
        *store.values.borrow_mut() = expected.clone();
        *store.fail_on_delete.borrow_mut() = Some(LegacyCredentialLocation::CurrentSplit(
            CredentialKey::OpenAi,
        ));

        let error = cleanup_legacy_credentials(&store).expect_err("cleanup should fail");

        assert!(error.contains("模拟旧凭据清理失败"));
        assert_eq!(*store.values.borrow(), expected);
    }
}
