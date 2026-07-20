use arboard::{Clipboard, ImageData};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    capture_session::{CaptureSessionState, CaptureSource, CaptureStatus},
    focus_capture::{restore_focus, FocusRestoreLevel, FocusSnapshot},
    indicator,
};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum OutputMethod {
    Paste,
    Clipboard,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OutputResult {
    pub method: OutputMethod,
    pub success: bool,
    pub message: String,
    pub target_restored: bool,
    pub target_restore_level: FocusRestoreLevel,
    pub clipboard_restored: bool,
    pub used_fallback: bool,
}

#[derive(Debug)]
enum ClipboardBackup {
    Text(String),
    Image(ImageData<'static>),
}

impl ClipboardBackup {
    fn read(clipboard: &mut Clipboard) -> Option<Self> {
        clipboard
            .get_text()
            .map(Self::Text)
            .or_else(|_| clipboard.get_image().map(Self::Image))
            .ok()
    }

    fn restore(self, clipboard: &mut Clipboard) -> bool {
        match self {
            Self::Text(text) => clipboard.set_text(text).is_ok(),
            Self::Image(image) => clipboard.set_image(image).is_ok(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PasteOutcome {
    target_restore_level: FocusRestoreLevel,
    clipboard_restored: bool,
}

#[tauri::command]
#[specta::specta]
pub async fn deliver_text(
    app: tauri::AppHandle,
    sessions: State<'_, CaptureSessionState>,
    session_id: Option<String>,
    history_id: Option<String>,
    text: String,
) -> Result<OutputResult, String> {
    eprintln!(
        "[文本投递] 开始：session={}, chars={}",
        session_id.as_deref().unwrap_or("none"),
        text.chars().count()
    );

    let Some(session_id) = session_id else {
        clipboard_copy(&text).await?;
        if let Some(history_id) = history_id {
            if let Err(error) =
                crate::data::update_history_delivery_for_app(&app, &history_id, "copy")
            {
                eprintln!("更新历史投递方式失败：{error}");
            }
        }
        return Ok(OutputResult {
            method: OutputMethod::Clipboard,
            success: true,
            message: "结果已复制到剪贴板".to_string(),
            target_restored: false,
            target_restore_level: FocusRestoreLevel::None,
            clipboard_restored: false,
            used_fallback: false,
        });
    };

    let context = sessions.delivery_context(&session_id)?;
    sessions.update_status(&session_id, CaptureStatus::Delivering)?;
    if context.source == CaptureSource::Hotkey {
        let _ = indicator::update_indicator(&app, "delivering");
    }

    if context.source == CaptureSource::App {
        let result = clipboard_copy(&text).await;
        return finish_delivery(
            &app,
            &sessions,
            &session_id,
            result.map(|_| OutputResult {
                method: OutputMethod::Clipboard,
                success: true,
                message: "结果已复制到剪贴板".to_string(),
                target_restored: false,
                target_restore_level: FocusRestoreLevel::None,
                clipboard_restored: false,
                used_fallback: false,
            }),
            false,
        );
    }

    eprintln!("[文本投递] 快捷键录音：准备恢复目标窗口并自动粘贴");
    match clipboard_paste(&text, context.focus).await {
        Ok(outcome) => {
            eprintln!(
                "[文本投递] 自动粘贴成功：restore_level={:?}, clipboard_restored={}",
                outcome.target_restore_level, outcome.clipboard_restored
            );
            finish_delivery(
                &app,
                &sessions,
                &session_id,
                Ok(OutputResult {
                    method: OutputMethod::Paste,
                    success: true,
                    message: match outcome.target_restore_level {
                        FocusRestoreLevel::Window => "已输入到录音开始时的窗口".to_string(),
                        FocusRestoreLevel::Application => {
                            "未能精确定位原窗口，已输入到录音开始时的应用".to_string()
                        }
                        FocusRestoreLevel::None => "已发送自动粘贴".to_string(),
                    },
                    target_restored: outcome.target_restore_level != FocusRestoreLevel::None,
                    target_restore_level: outcome.target_restore_level,
                    clipboard_restored: outcome.clipboard_restored,
                    used_fallback: false,
                }),
                true,
            )
        }
        Err(paste_error) => {
            eprintln!("[文本投递] 自动粘贴失败，降级为剪贴板：{paste_error}");
            let fallback = clipboard_copy(&text).await.map(|_| OutputResult {
                method: OutputMethod::Manual,
                success: false,
                message: format!(
                    "自动粘贴失败，结果已复制到剪贴板，可手动按 {}：{paste_error}",
                    manual_paste_shortcut()
                ),
                target_restored: false,
                target_restore_level: FocusRestoreLevel::None,
                clipboard_restored: false,
                used_fallback: true,
            });
            finish_delivery(&app, &sessions, &session_id, fallback, true)
        }
    }
}

fn manual_paste_shortcut() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "Command+V"
    }
    #[cfg(not(target_os = "macos"))]
    {
        "Ctrl+V"
    }
}

fn finish_delivery(
    app: &tauri::AppHandle,
    sessions: &CaptureSessionState,
    session_id: &str,
    result: Result<OutputResult, String>,
    show_indicator: bool,
) -> Result<OutputResult, String> {
    match result {
        Ok(result) => {
            if let Ok(context) = sessions.delivery_context(session_id) {
                if let Some(history_id) = context.history_id {
                    let delivery_method = match result.method {
                        OutputMethod::Paste => "paste",
                        OutputMethod::Clipboard => "copy",
                        OutputMethod::Manual => "manual",
                    };
                    if let Err(error) = crate::data::update_history_delivery_for_app(
                        app,
                        &history_id,
                        delivery_method,
                    ) {
                        eprintln!("更新历史投递方式失败：{error}");
                    }
                }
            }
            sessions.finish(session_id, CaptureStatus::Completed)?;
            if show_indicator {
                let _ = indicator::finish_indicator(app, "completed");
            }
            Ok(result)
        }
        Err(error) => {
            let _ = sessions.finish(session_id, CaptureStatus::Failed);
            if show_indicator {
                let _ = indicator::finish_indicator(app, "failed");
            }
            Err(error)
        }
    }
}

async fn clipboard_paste(text: &str, focus: Option<FocusSnapshot>) -> Result<PasteOutcome, String> {
    let text = text.to_string();
    tokio::task::spawn_blocking(move || {
        let mut clipboard = Clipboard::new().map_err(|error| format!("无法访问剪贴板：{error}"))?;
        let previous_clipboard = ClipboardBackup::read(&mut clipboard);
        clipboard
            .set_text(text)
            .map_err(|error| format!("写入剪贴板失败：{error}"))?;
        drop(clipboard);

        let target_restore_level = restore_focus(focus.as_ref())?;
        std::thread::sleep(std::time::Duration::from_millis(40));
        send_paste_shortcut()?;
        std::thread::sleep(std::time::Duration::from_millis(180));

        let clipboard_restored = if let Some(previous_clipboard) = previous_clipboard {
            Clipboard::new()
                .map(|mut clipboard| previous_clipboard.restore(&mut clipboard))
                .unwrap_or(false)
        } else {
            false
        };

        Ok(PasteOutcome {
            target_restore_level,
            clipboard_restored,
        })
    })
    .await
    .map_err(|error| format!("自动粘贴任务失败：{error}"))?
}

fn send_paste_shortcut() -> Result<(), String> {
    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|error| format!("初始化键盘模拟失败：{error}"))?;

    #[cfg(target_os = "macos")]
    let modifier = Key::Meta;
    #[cfg(not(target_os = "macos"))]
    let modifier = Key::Control;

    enigo
        .key(modifier, Direction::Press)
        .map_err(|error| format!("按下粘贴修饰键失败：{error}"))?;
    let click_result = click_paste_key(&mut enigo);
    let release_result = enigo
        .key(modifier, Direction::Release)
        .map_err(|error| format!("释放粘贴修饰键失败：{error}"));

    click_result?;
    release_result
}

fn click_paste_key(enigo: &mut Enigo) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // macOS ANSI V 的虚拟键码为 9。不要在 Tokio blocking worker 中使用
        // Key::Unicode('v')：enigo 会查询 TSM 当前输入源，而该 API 要求主队列，
        // 在后台线程调用会触发 dispatch_assert_queue 并以 SIGTRAP 终止整个进程。
        enigo
            .raw(9, Direction::Click)
            .map_err(|error| format!("触发粘贴快捷键失败：{error}"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|error| format!("触发粘贴快捷键失败：{error}"))
    }
}

#[cfg(test)]
fn macos_paste_key_code() -> u16 {
    9
}

async fn clipboard_copy(text: &str) -> Result<(), String> {
    let text = text.to_string();
    tokio::task::spawn_blocking(move || {
        let mut clipboard = Clipboard::new().map_err(|error| format!("无法访问剪贴板：{error}"))?;
        clipboard
            .set_text(text)
            .map_err(|error| format!("写入剪贴板失败：{error}"))
    })
    .await
    .map_err(|error| format!("剪贴板任务失败：{error}"))?
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "macos")]
    #[test]
    fn macos_paste_uses_ansi_v_keycode_without_layout_lookup() {
        assert_eq!(super::macos_paste_key_code(), 9);
    }
}
