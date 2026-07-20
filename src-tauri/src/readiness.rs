use cpal::traits::HostTrait;
use serde::Serialize;
use tauri::Manager;

use crate::{
    data::{read_app_config, AppConfig},
    hotkey::HotkeyState,
    macos_permissions::{MacosPermissionState, PermissionStatus},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessAction {
    RequestMicrophone,
    OpenMicrophoneSettings,
    RequestAccessibility,
    OpenAccessibilitySettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
pub struct ReadinessCheck {
    pub ready: bool,
    pub blocking: bool,
    pub detail: String,
    pub actions: Vec<ReadinessAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
pub struct InputReadiness {
    pub platform: String,
    pub macos_permissions: Option<MacosPermissionState>,
    pub microphone: ReadinessCheck,
    pub asr: ReadinessCheck,
    pub text_processing: ReadinessCheck,
    pub hotkey: ReadinessCheck,
    pub auto_paste: ReadinessCheck,
    pub models_ready: bool,
    pub can_process: bool,
    pub can_dictate: bool,
}

#[tauri::command]
#[specta::specta]
pub async fn read_input_readiness(app: tauri::AppHandle) -> Result<InputReadiness, String> {
    let config = read_app_config(app.clone())?;
    let permissions = macos_permission_state();
    let microphone = microphone_check(permissions.as_ref());
    let local_model_exists = crate::local_asr_model::model_path(&app)
        .map(|path| path.exists())
        .unwrap_or(false);
    let asr = asr_check(&config, local_model_exists);
    let text_processing = text_processing_check(&config);

    let hotkey_state = app.state::<std::sync::Arc<tokio::sync::Mutex<HotkeyState>>>();
    let hotkey_state = hotkey_state.lock().await;
    let hotkey_ready = hotkey_state.longpress_registered || hotkey_state.toggle_registered;
    let hotkey = check(
        hotkey_ready,
        true,
        if hotkey_ready {
            "至少一个全局快捷键已注册"
        } else {
            "未注册可用的全局快捷键，请保存通用设置并检查快捷键冲突"
        },
    );

    let auto_paste = auto_paste_check(permissions.as_ref());
    let models_ready = asr.ready && text_processing.ready;
    let can_process = microphone.ready && models_ready;
    let can_dictate = can_process && hotkey.ready;

    Ok(InputReadiness {
        platform: std::env::consts::OS.to_string(),
        macos_permissions: permissions,
        microphone,
        asr,
        text_processing,
        hotkey,
        auto_paste,
        models_ready,
        can_process,
        can_dictate,
    })
}

fn check(ready: bool, blocking: bool, detail: impl Into<String>) -> ReadinessCheck {
    ReadinessCheck {
        ready,
        blocking,
        detail: detail.into(),
        actions: Vec::new(),
    }
}

fn macos_permission_state() -> Option<MacosPermissionState> {
    #[cfg(target_os = "macos")]
    {
        Some(crate::macos_permissions::state())
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

fn microphone_check(permissions: Option<&MacosPermissionState>) -> ReadinessCheck {
    #[cfg(target_os = "macos")]
    if let Some(permissions) = permissions {
        match permissions.microphone {
            PermissionStatus::Authorized => {}
            PermissionStatus::NotDetermined => {
                return ReadinessCheck {
                    ready: false,
                    blocking: true,
                    detail: "尚未请求麦克风权限".to_string(),
                    actions: vec![
                        ReadinessAction::RequestMicrophone,
                        ReadinessAction::OpenMicrophoneSettings,
                    ],
                };
            }
            PermissionStatus::Denied => {
                return ReadinessCheck {
                    ready: false,
                    blocking: true,
                    detail: "麦克风权限已被拒绝，请在 macOS 隐私与安全性设置中允许 XiLuoLin"
                        .to_string(),
                    actions: vec![ReadinessAction::OpenMicrophoneSettings],
                };
            }
            PermissionStatus::Restricted => {
                return ReadinessCheck {
                    ready: false,
                    blocking: true,
                    detail: "麦克风访问受到系统策略限制".to_string(),
                    actions: vec![ReadinessAction::OpenMicrophoneSettings],
                };
            }
            PermissionStatus::Unknown | PermissionStatus::Unsupported => {
                return ReadinessCheck {
                    ready: false,
                    blocking: true,
                    detail: "无法确认 macOS 麦克风权限状态".to_string(),
                    actions: vec![ReadinessAction::OpenMicrophoneSettings],
                };
            }
        }
    }

    let ready = cpal::default_host().default_input_device().is_some();
    check(
        ready,
        true,
        if ready {
            "已检测到默认麦克风"
        } else {
            "未检测到默认麦克风，请检查设备连接和系统权限"
        },
    )
}

fn asr_check(config: &AppConfig, local_model_exists: bool) -> ReadinessCheck {
    let provider = config.asr_provider.trim();
    if provider == "local" {
        return check(
            local_model_exists,
            true,
            if local_model_exists {
                format!("本地 ASR 模型 {} 已就绪", config.local_asr_model)
            } else {
                "本地 ASR 模型尚未下载".to_string()
            },
        );
    }

    let (api_key, base_url, model) = config.selected_asr_config();
    let ready = matches!(provider, "zhipu" | "openai")
        && !api_key.trim().is_empty()
        && !base_url.trim().is_empty()
        && !model.trim().is_empty();

    check(
        ready,
        true,
        if ready {
            format!("{} ASR 配置完整", provider_name(provider))
        } else {
            "ASR Provider、API Key、Base URL 或模型配置不完整".to_string()
        },
    )
}

fn text_processing_check(config: &AppConfig) -> ReadinessCheck {
    let provider = config.text_provider.trim();
    let (api_key, base_url, model) = config.selected_text_config();
    let ready = matches!(provider, "zhipu" | "openai")
        && !api_key.is_empty()
        && !base_url.is_empty()
        && !model.is_empty();

    check(
        ready,
        true,
        if ready {
            format!("{} 文本处理配置完整", provider_name(provider))
        } else {
            "文本 Provider、API Key、Base URL 或模型配置不完整".to_string()
        },
    )
}

fn provider_name(provider: &str) -> &'static str {
    match provider {
        "zhipu" => "智谱",
        "openai" => "OpenAI",
        _ => "未知 Provider",
    }
}

fn auto_paste_check(permissions: Option<&MacosPermissionState>) -> ReadinessCheck {
    #[cfg(target_os = "windows")]
    {
        let _ = permissions;
        check(
            true,
            false,
            "支持目标窗口恢复和 Ctrl+V；提升权限窗口可能被 UIPI 阻止",
        )
    }

    #[cfg(target_os = "macos")]
    {
        let authorized =
            permissions.is_some_and(|state| state.accessibility == PermissionStatus::Authorized);
        ReadinessCheck {
            ready: authorized,
            blocking: false,
            detail: if authorized {
                "辅助功能权限已授权，支持恢复录音开始时的应用窗口并发送 Command+V".to_string()
            } else {
                "需要辅助功能权限；未授权时识别结果仍会复制到剪贴板".to_string()
            },
            actions: if authorized {
                Vec::new()
            } else {
                vec![
                    ReadinessAction::RequestAccessibility,
                    ReadinessAction::OpenAccessibilitySettings,
                ]
            },
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = permissions;
        check(
            false,
            false,
            "当前平台未完成自动粘贴兼容验证，结果仍会复制到剪贴板",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::default_app_config;

    #[test]
    fn default_config_is_not_ready_without_credentials() {
        let config = default_app_config();
        assert!(!asr_check(&config, false).ready);
        assert!(!text_processing_check(&config).ready);
    }

    #[test]
    fn zhipu_configuration_is_ready_when_required_fields_exist() {
        let mut config = default_app_config();
        config.asr_api_key = "asr-key".to_string();
        config.zhipu_api_key = "text-key".to_string();

        assert!(asr_check(&config, false).ready);
        assert!(text_processing_check(&config).ready);
    }

    #[test]
    fn openai_configuration_uses_the_selected_provider_fields() {
        let mut config = default_app_config();
        config.asr_provider = "openai".to_string();
        config.openai_api_key = "openai-key".to_string();
        config.text_provider = "openai".to_string();

        assert!(asr_check(&config, false).ready);
        assert!(text_processing_check(&config).ready);
    }

    #[test]
    fn local_provider_requires_downloaded_model() {
        let mut config = default_app_config();
        config.asr_provider = "local".to_string();

        assert!(!asr_check(&config, false).ready);
        assert!(asr_check(&config, true).ready);
    }

    #[test]
    fn unsupported_provider_is_not_ready() {
        let mut config = default_app_config();
        config.asr_provider = "unknown".to_string();
        config.asr_api_key = "asr-key".to_string();
        config.text_provider = "unknown".to_string();
        config.openai_api_key = "text-key".to_string();

        assert!(!asr_check(&config, false).ready);
        assert!(!text_processing_check(&config).ready);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn denied_accessibility_permission_is_non_blocking() {
        let permissions = MacosPermissionState {
            microphone: PermissionStatus::Authorized,
            accessibility: PermissionStatus::Denied,
        };
        let check = auto_paste_check(Some(&permissions));

        assert!(!check.ready);
        assert!(!check.blocking);
        assert!(check
            .actions
            .contains(&ReadinessAction::RequestAccessibility));
    }
}
