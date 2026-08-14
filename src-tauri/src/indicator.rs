use std::sync::atomic::{AtomicU64, Ordering};

use tauri::{AppHandle, Manager, Monitor, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const INDICATOR_LABEL: &str = "recording-indicator";
const INDICATOR_WIDTH: f64 = 480.0;
const INDICATOR_HEIGHT: f64 = 72.0;
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
const SNAPSHOT_REFRESH_SCRIPT: &str =
    "window.dispatchEvent(new Event('capture-snapshot-refresh'));";

#[cfg(target_os = "macos")]
tauri_nspanel::tauri_panel! {
    panel!(RecordingIndicatorPanel {
        config: {
            can_become_key_window: false,
            can_become_main_window: false,
        }
    })
}

pub fn ensure_indicator(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window(INDICATOR_LABEL) {
        return Ok(window);
    }

    let window_builder = WebviewWindowBuilder::new(
        app,
        INDICATOR_LABEL,
        WebviewUrl::App("index.html?window=indicator".into()),
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
    convert_macos_indicator_to_panel(&window)?;
    configure_macos_fullscreen_overlay(&window)?;
    let _ = window.set_ignore_cursor_events(true);

    position_indicator(app, &window);

    Ok(window)
}

#[cfg(target_os = "macos")]
fn convert_macos_indicator_to_panel(window: &WebviewWindow) -> Result<(), String> {
    use tauri_nspanel::{StyleMask, WebviewWindowExt};

    let panel = window
        .to_panel::<RecordingIndicatorPanel>()
        .map_err(|error| format!("创建录音指示器原生面板失败：{error}"))?;
    panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());
    panel.set_floating_panel(true);
    panel.set_hides_on_deactivate(false);
    panel.set_becomes_key_only_if_needed(true);
    panel.set_ignores_mouse_events(true);
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn convert_macos_indicator_to_panel(_window: &WebviewWindow) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn configure_macos_fullscreen_overlay(window: &WebviewWindow) -> Result<(), String> {
    use objc2_app_kit::{NSScreenSaverWindowLevel, NSWindow};

    let ns_window = window
        .ns_window()
        .map_err(|error| format!("读取录音指示器原生窗口失败：{error}"))?
        .cast::<NSWindow>();
    let ns_window = unsafe { ns_window.as_ref() }
        .ok_or_else(|| "读取录音指示器原生窗口失败：窗口指针为空".to_string())?;

    let behavior = fullscreen_overlay_behavior(ns_window.collectionBehavior());
    ns_window.setCollectionBehavior(behavior);
    ns_window.setLevel(NSScreenSaverWindowLevel);
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn configure_macos_fullscreen_overlay(_window: &WebviewWindow) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn fullscreen_overlay_behavior(
    current: objc2_app_kit::NSWindowCollectionBehavior,
) -> objc2_app_kit::NSWindowCollectionBehavior {
    use objc2_app_kit::NSWindowCollectionBehavior;

    let incompatible =
        NSWindowCollectionBehavior::FullScreenPrimary | NSWindowCollectionBehavior::FullScreenNone;
    (current & !incompatible)
        | NSWindowCollectionBehavior::CanJoinAllSpaces
        | NSWindowCollectionBehavior::CanJoinAllApplications
        | NSWindowCollectionBehavior::FullScreenAuxiliary
        | NSWindowCollectionBehavior::Stationary
}

#[cfg(target_os = "macos")]
fn order_macos_indicator_front(window: &WebviewWindow) -> Result<(), String> {
    let dispatcher = window.clone();
    let native_window = window.clone();
    dispatcher
        .run_on_main_thread(move || {
            use objc2_app_kit::NSWindow;

            let Ok(ns_window) = native_window.ns_window() else {
                return;
            };
            let Some(ns_window) = (unsafe { ns_window.cast::<NSWindow>().as_ref() }) else {
                return;
            };
            ns_window.orderFrontRegardless();
        })
        .map_err(|error| format!("置顶录音指示器失败：{error}"))
}

#[cfg(not(target_os = "macos"))]
fn order_macos_indicator_front(_window: &WebviewWindow) -> Result<(), String> {
    Ok(())
}

pub fn show_indicator(app: &AppHandle) -> Result<(), String> {
    let window = ensure_indicator(app)?;
    INDICATOR_REVISION.fetch_add(1, Ordering::SeqCst);
    position_indicator(app, &window);
    window
        .show()
        .map_err(|error| format!("显示录音指示器失败：{error}"))?;
    order_macos_indicator_front(&window)?;
    update_window(&window, "recording")
}

fn position_indicator(app: &AppHandle, window: &WebviewWindow) {
    let monitor = cursor_monitor(app)
        .or_else(|| app.primary_monitor().ok().flatten())
        .or_else(|| window.current_monitor().ok().flatten());
    if let Some(monitor) = monitor {
        let scale = monitor.scale_factor();
        let window_width = indicator_width(monitor.size().width, scale);
        let window_height = (INDICATOR_HEIGHT * scale).round() as u32;
        let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(
            window_width,
            window_height,
        )));
        let monitor_position = monitor.position();
        let position = indicator_position(
            monitor_position.x,
            monitor_position.y,
            monitor.size().width,
            monitor.size().height,
            window_width,
        );
        let _ = window.set_position(tauri::Position::Physical(position));
    }
}

fn indicator_width(monitor_width: u32, scale_factor: f64) -> u32 {
    let desired = (INDICATOR_WIDTH * scale_factor).round() as u32;
    let margin = (32.0 * scale_factor).round() as u32;
    desired.min(monitor_width.saturating_sub(margin).max(1))
}

fn cursor_monitor(app: &AppHandle) -> Option<Monitor> {
    let cursor = app.cursor_position().ok()?;
    app.monitor_from_point(cursor.x, cursor.y).ok().flatten()
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
    indicator_refresh_script(status)?;
    let refresh_result = update_indicator(app, status);
    let delay_ms = indicator_hide_delay(status);
    let revision = INDICATOR_REVISION.load(Ordering::SeqCst);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
        if INDICATOR_REVISION.load(Ordering::SeqCst) == revision {
            let _ = hide_indicator(&app);
        }
    });
    // 即使窗口脚本暂时不可用，也必须保留兜底隐藏，避免悬浮窗永久停在处理中。
    refresh_result
}

fn indicator_hide_delay(status: &str) -> u64 {
    if status == "failed" {
        4_000
    } else {
        1_200
    }
}

fn indicator_refresh_script(status: &str) -> Result<&'static str, String> {
    if !VALID_STATUSES.contains(&status) {
        return Err(format!("未知的录音指示器状态：{status}"));
    }
    Ok(SNAPSHOT_REFRESH_SCRIPT)
}

fn update_window(window: &WebviewWindow, status: &str) -> Result<(), String> {
    window
        .eval(indicator_refresh_script(status)?)
        .map_err(|error| format!("刷新录音指示器状态失败：{error}"))
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
            indicator_position(0, 0, 1920, 1080, 480),
            tauri::PhysicalPosition { x: 720, y: 43 }
        );
    }

    #[test]
    fn indicator_position_includes_secondary_monitor_origin() {
        assert_eq!(
            indicator_position(-2560, -120, 2560, 1440, 480),
            tauri::PhysicalPosition { x: -1520, y: -62 }
        );
    }

    #[test]
    fn indicator_width_leaves_a_margin_on_narrow_displays() {
        assert_eq!(indicator_width(460, 1.0), 428);
        assert_eq!(indicator_width(1920, 2.0), 960);
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

    #[test]
    fn failed_indicator_stays_visible_longer_than_success() {
        assert_eq!(indicator_hide_delay("completed"), 1_200);
        assert_eq!(indicator_hide_delay("failed"), 4_000);
    }

    #[test]
    fn status_update_requests_a_canonical_snapshot_refresh() {
        assert_eq!(
            indicator_refresh_script("completed").unwrap(),
            "window.dispatchEvent(new Event('capture-snapshot-refresh'));"
        );
        assert!(indicator_refresh_script("unknown").is_err());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn fullscreen_overlay_joins_fullscreen_spaces_as_an_auxiliary_window() {
        use objc2_app_kit::NSWindowCollectionBehavior;

        let behavior = fullscreen_overlay_behavior(
            NSWindowCollectionBehavior::FullScreenPrimary
                | NSWindowCollectionBehavior::FullScreenNone,
        );

        assert!(behavior.contains(NSWindowCollectionBehavior::CanJoinAllSpaces));
        assert!(behavior.contains(NSWindowCollectionBehavior::CanJoinAllApplications));
        assert!(behavior.contains(NSWindowCollectionBehavior::FullScreenAuxiliary));
        assert!(behavior.contains(NSWindowCollectionBehavior::Stationary));
        assert!(!behavior.contains(NSWindowCollectionBehavior::FullScreenPrimary));
        assert!(!behavior.contains(NSWindowCollectionBehavior::FullScreenNone));
    }
}
