use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionStatus {
    Authorized,
    Denied,
    Restricted,
    NotDetermined,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MacosPermissionKind {
    Microphone,
    Accessibility,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MacosPermissionState {
    pub microphone: PermissionStatus,
    pub accessibility: PermissionStatus,
}

#[cfg(target_os = "macos")]
mod platform {
    use std::{process::Command, sync::mpsc, time::Duration};

    use block2::RcBlock;
    use macos_accessibility_client::accessibility::{
        application_is_trusted, application_is_trusted_with_prompt,
    };
    use objc2_av_foundation::{AVAuthorizationStatus, AVCaptureDevice, AVMediaTypeAudio};

    use super::{MacosPermissionKind, MacosPermissionState, PermissionStatus};

    pub fn accessibility_status() -> PermissionStatus {
        if application_is_trusted() {
            PermissionStatus::Authorized
        } else {
            PermissionStatus::Denied
        }
    }

    pub fn microphone_status() -> PermissionStatus {
        let Some(media_type) = (unsafe { AVMediaTypeAudio }) else {
            return PermissionStatus::Unknown;
        };
        let status = unsafe { AVCaptureDevice::authorizationStatusForMediaType(media_type) };
        match status {
            AVAuthorizationStatus::Authorized => PermissionStatus::Authorized,
            AVAuthorizationStatus::Denied => PermissionStatus::Denied,
            AVAuthorizationStatus::Restricted => PermissionStatus::Restricted,
            AVAuthorizationStatus::NotDetermined => PermissionStatus::NotDetermined,
            _ => PermissionStatus::Unknown,
        }
    }

    pub fn state() -> MacosPermissionState {
        MacosPermissionState {
            microphone: microphone_status(),
            accessibility: accessibility_status(),
        }
    }

    pub fn request_accessibility() -> PermissionStatus {
        if application_is_trusted_with_prompt() {
            PermissionStatus::Authorized
        } else {
            accessibility_status()
        }
    }

    pub fn request_microphone() -> Result<PermissionStatus, String> {
        let Some(media_type) = (unsafe { AVMediaTypeAudio }) else {
            return Err("无法读取 macOS 麦克风媒体类型".to_string());
        };
        if microphone_status() != PermissionStatus::NotDetermined {
            return Ok(microphone_status());
        }

        let (sender, receiver) = mpsc::channel();
        let handler = RcBlock::new(move |granted: objc2::runtime::Bool| {
            let _ = sender.send(granted.as_bool());
        });
        unsafe {
            AVCaptureDevice::requestAccessForMediaType_completionHandler(media_type, &handler)
        };

        receiver
            .recv_timeout(Duration::from_secs(120))
            .map_err(|_| "等待麦克风权限结果超时".to_string())?;
        Ok(microphone_status())
    }

    pub fn open_settings(kind: MacosPermissionKind) -> Result<(), String> {
        let pane = match kind {
            MacosPermissionKind::Microphone => "Privacy_Microphone",
            MacosPermissionKind::Accessibility => "Privacy_Accessibility",
        };
        let url = format!("x-apple.systempreferences:com.apple.preference.security?{pane}");
        Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|error| format!("打开 macOS 隐私设置失败：{error}"))?;
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::{MacosPermissionKind, MacosPermissionState, PermissionStatus};

    pub fn accessibility_status() -> PermissionStatus {
        PermissionStatus::Unsupported
    }

    pub fn microphone_status() -> PermissionStatus {
        PermissionStatus::Unsupported
    }

    pub fn state() -> MacosPermissionState {
        MacosPermissionState {
            microphone: PermissionStatus::Unsupported,
            accessibility: PermissionStatus::Unsupported,
        }
    }

    pub fn request_accessibility() -> PermissionStatus {
        PermissionStatus::Unsupported
    }

    pub fn request_microphone() -> Result<PermissionStatus, String> {
        Ok(PermissionStatus::Unsupported)
    }

    pub fn open_settings(_kind: MacosPermissionKind) -> Result<(), String> {
        Err("当前平台不支持 macOS 隐私设置".to_string())
    }
}

pub use platform::{accessibility_status, microphone_status, state};

#[tauri::command]
pub async fn request_macos_permission(
    permission: MacosPermissionKind,
) -> Result<PermissionStatus, String> {
    match permission {
        MacosPermissionKind::Accessibility => Ok(platform::request_accessibility()),
        MacosPermissionKind::Microphone => {
            tokio::task::spawn_blocking(platform::request_microphone)
                .await
                .map_err(|error| format!("请求麦克风权限任务失败：{error}"))?
        }
    }
}

#[tauri::command]
pub fn open_macos_privacy_settings(permission: MacosPermissionKind) -> Result<(), String> {
    platform::open_settings(permission)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_status_serializes_for_the_frontend() {
        assert_eq!(
            serde_json::to_string(&PermissionStatus::NotDetermined).unwrap(),
            "\"not_determined\""
        );
        assert_eq!(
            serde_json::to_string(&MacosPermissionKind::Accessibility).unwrap(),
            "\"accessibility\""
        );
    }
}
