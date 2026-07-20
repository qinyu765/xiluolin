use std::sync::atomic::{AtomicU64, Ordering};

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const INDICATOR_LABEL: &str = "recording-indicator";
const INDICATOR_WIDTH: f64 = 320.0;
const INDICATOR_HEIGHT: f64 = 64.0;
const INDICATOR_TOP_RATIO: f64 = 0.04;
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
    .inner_size(INDICATOR_WIDTH, INDICATOR_HEIGHT)
    .resizable(false)
    .decorations(false)
    .always_on_top(true)
    .visible_on_all_workspaces(true)
    .skip_taskbar(true)
    .shadow(false)
    .focusable(false)
    .transparent(true);

    let window = window_builder
        .visible(false)
        .build()
        .map_err(|error| format!("创建录音指示器失败：{error}"))?;
    let _ = window.set_ignore_cursor_events(true);

    position_indicator(&window);

    Ok(window)
}

pub fn show_indicator(app: &AppHandle) -> Result<(), String> {
    let window = ensure_indicator(app)?;
    INDICATOR_REVISION.fetch_add(1, Ordering::SeqCst);
    position_indicator(&window);
    update_window(&window, "recording")?;
    window
        .show()
        .map_err(|error| format!("显示录音指示器失败：{error}"))
}

fn position_indicator(window: &WebviewWindow) {
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());
    let window_size = window.outer_size().ok();

    if let (Some(monitor), Some(window_size)) = (monitor, window_size) {
        let monitor_position = monitor.position();
        let position = indicator_position(
            monitor_position.x,
            monitor_position.y,
            monitor.size().width,
            monitor.size().height,
            window_size.width,
        );
        let _ = window.set_position(tauri::Position::Physical(position));
    }
}

fn indicator_position(
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: u32,
    monitor_height: u32,
    window_width: u32,
) -> tauri::PhysicalPosition<i32> {
    let x = monitor_x + (monitor_width as i32 - window_width as i32) / 2;
    let y = monitor_y + (monitor_height as f64 * INDICATOR_TOP_RATIO).round() as i32;
    tauri::PhysicalPosition { x, y }
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
#[specta::specta]
pub fn update_indicator_status(app: AppHandle, status: String) -> Result<(), String> {
    update_indicator(&app, &status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indicator_is_centered_near_the_top_of_the_primary_monitor() {
        assert_eq!(
            indicator_position(0, 0, 1920, 1080, 320),
            tauri::PhysicalPosition { x: 800, y: 43 }
        );
    }

    #[test]
    fn indicator_position_includes_secondary_monitor_origin() {
        assert_eq!(
            indicator_position(-2560, -120, 2560, 1440, 320),
            tauri::PhysicalPosition { x: -1440, y: -62 }
        );
    }

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
