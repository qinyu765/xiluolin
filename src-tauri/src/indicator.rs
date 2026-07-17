use std::sync::atomic::{AtomicU64, Ordering};

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const INDICATOR_LABEL: &str = "recording-indicator";
static INDICATOR_REVISION: AtomicU64 = AtomicU64::new(0);
const VALID_STATUSES: [&str; 6] = [
    "recording",
    "transcribing",
    "refining",
    "delivering",
    "completed",
    "failed",
];

pub fn ensure_indicator(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window(INDICATOR_LABEL) {
        return Ok(window);
    }

    let window_builder = WebviewWindowBuilder::new(
        app,
        INDICATOR_LABEL,
        WebviewUrl::App("indicator.html".into()),
    )
    .title("语音输入状态")
    .inner_size(220.0, 54.0)
    .resizable(false)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .focusable(false);

    #[cfg(not(target_os = "macos"))]
    let window_builder = window_builder.transparent(true);

    let window = window_builder
        .visible(false)
        .build()
        .map_err(|error| format!("创建录音指示器失败：{error}"))?;
    let _ = window.set_ignore_cursor_events(true);

    if let Ok(Some(monitor)) = window.current_monitor() {
        let size = monitor.size();
        let x = (size.width as f64 - 220.0) / 2.0;
        let y = size.height as f64 * 0.7;
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: x as i32,
            y: y as i32,
        }));
    }

    Ok(window)
}

pub fn show_indicator(app: &AppHandle) -> Result<(), String> {
    let window = ensure_indicator(app)?;
    INDICATOR_REVISION.fetch_add(1, Ordering::SeqCst);
    update_window(&window, "recording")?;
    window
        .show()
        .map_err(|error| format!("显示录音指示器失败：{error}"))
}

pub fn update_indicator(app: &AppHandle, status: &str) -> Result<(), String> {
    let window = ensure_indicator(app)?;
    INDICATOR_REVISION.fetch_add(1, Ordering::SeqCst);
    update_window(&window, status)
}

pub fn finish_indicator(app: &AppHandle, status: &str) -> Result<(), String> {
    update_indicator(app, status)?;
    let revision = INDICATOR_REVISION.load(Ordering::SeqCst);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(900)).await;
        if INDICATOR_REVISION.load(Ordering::SeqCst) == revision {
            let _ = hide_indicator(&app);
        }
    });
    Ok(())
}

fn update_window(window: &WebviewWindow, status: &str) -> Result<(), String> {
    if !VALID_STATUSES.contains(&status) {
        return Err(format!("未知的录音指示器状态：{status}"));
    }
    let status = serde_json::to_string(status).map_err(|error| error.to_string())?;
    window
        .eval(format!("window.setIndicatorStatus({status});"))
        .map_err(|error| format!("更新录音指示器失败：{error}"))
}

pub fn hide_indicator(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(INDICATOR_LABEL) {
        window
            .hide()
            .map_err(|error| format!("隐藏录音指示器失败：{error}"))?;
    }
    Ok(())
}

#[tauri::command]
pub fn update_indicator_status(app: AppHandle, status: String) -> Result<(), String> {
    update_indicator(&app, &status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indicator_statuses_cover_the_capture_pipeline() {
        assert_eq!(
            VALID_STATUSES,
            [
                "recording",
                "transcribing",
                "refining",
                "delivering",
                "completed",
                "failed"
            ]
        );
    }
}
