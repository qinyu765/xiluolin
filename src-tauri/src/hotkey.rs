use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::events::RecordingErrorEvent;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent};
use tokio::sync::Mutex;

static HOTKEY_REGISTRATION_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static HOTKEY_LATEST_PUBLISHED_REVISION: AtomicU64 = AtomicU64::new(0);

pub fn next_hotkey_registration_revision() -> u64 {
    HOTKEY_REGISTRATION_SEQUENCE
        .fetch_add(1, Ordering::Relaxed)
        .wrapping_add(1)
}

pub fn publish_hotkey_registration_revision(revision: u64) {
    let mut current = HOTKEY_LATEST_PUBLISHED_REVISION.load(Ordering::Acquire);
    while revision > current {
        match HOTKEY_LATEST_PUBLISHED_REVISION.compare_exchange(
            current,
            revision,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => return,
            Err(next) => current = next,
        }
    }
}

fn hotkey_registration_is_current(revision: u64) -> bool {
    HOTKEY_LATEST_PUBLISHED_REVISION.load(Ordering::Acquire) == revision
}

// 快捷键状态管理
pub struct HotkeyState {
    pub longpress_registered: bool,
    pub toggle_registered: bool,
    pub longpress_shortcut: Option<String>,
    pub toggle_shortcut: Option<String>,
    pub is_recording_via_hotkey: bool, // 跟踪通过快捷键触发的录音状态
    event_gate: Arc<Mutex<()>>,        // 串行处理 Press/Released，避免启动与停止乱序
}

#[derive(Clone, Debug)]
pub enum RecordingMode {
    LongPress,
    Toggle,
}

impl Default for HotkeyState {
    fn default() -> Self {
        Self {
            longpress_registered: false,
            toggle_registered: false,
            longpress_shortcut: None,
            toggle_shortcut: None,
            is_recording_via_hotkey: false,
            event_gate: Arc::new(Mutex::new(())),
        }
    }
}

// 注册全局快捷键
#[tauri::command]
#[specta::specta]
pub async fn register_hotkey(app: AppHandle, shortcut: String, mode: String) -> Result<(), String> {
    let state = app.state::<Arc<Mutex<HotkeyState>>>();
    let mut state = state.lock().await;

    // 解析录音模式
    let recording_mode = match mode.as_str() {
        "long_press" => RecordingMode::LongPress,
        "toggle" => RecordingMode::Toggle,
        _ => return Err("无效的录音模式".to_string()),
    };

    // 根据模式注销对应的快捷键
    match recording_mode {
        RecordingMode::LongPress => {
            if state.longpress_registered {
                if let Some(old_shortcut) = &state.longpress_shortcut {
                    let old_shortcut_obj: Shortcut = old_shortcut
                        .parse()
                        .map_err(|e| format!("解析旧快捷键失败: {}", e))?;
                    let _ = app.global_shortcut().unregister(old_shortcut_obj);
                }
            }
        }
        RecordingMode::Toggle => {
            if state.toggle_registered {
                if let Some(old_shortcut) = &state.toggle_shortcut {
                    let old_shortcut_obj: Shortcut = old_shortcut
                        .parse()
                        .map_err(|e| format!("解析旧快捷键失败: {}", e))?;
                    let _ = app.global_shortcut().unregister(old_shortcut_obj);
                }
            }
        }
    }

    // 注册新快捷键
    let shortcut_obj: Shortcut = shortcut
        .parse()
        .map_err(|e| format!("快捷键格式错误: {}", e))?;

    let app_clone = app.clone();
    let mode_clone = recording_mode.clone();

    app.global_shortcut()
        .on_shortcut(shortcut_obj, move |_app_handle, _shortcut, event| {
            handle_hotkey_event(&app_clone, event, &mode_clone);
        })
        .map_err(|e| format!("快捷键注册失败: {}. 可能与其他应用冲突", e))?;

    // 更新状态
    match recording_mode {
        RecordingMode::LongPress => {
            state.longpress_registered = true;
            state.longpress_shortcut = Some(shortcut);
        }
        RecordingMode::Toggle => {
            state.toggle_registered = true;
            state.toggle_shortcut = Some(shortcut);
        }
    }
    state.is_recording_via_hotkey = false;

    Ok(())
}

// 同时注册长按和切换两种模式的快捷键
#[tauri::command]
#[specta::specta]
pub async fn register_both_hotkeys(
    app: AppHandle,
    longpress_shortcut: Option<String>,
    toggle_shortcut: Option<String>,
) -> Result<(), String> {
    let revision = next_hotkey_registration_revision();
    publish_hotkey_registration_revision(revision);
    register_both_hotkeys_if_current(app, longpress_shortcut, toggle_shortcut, revision).await
}

pub async fn register_both_hotkeys_if_current(
    app: AppHandle,
    longpress_shortcut: Option<String>,
    toggle_shortcut: Option<String>,
    revision: u64,
) -> Result<(), String> {
    if !hotkey_registration_is_current(revision) {
        return Ok(());
    }

    let state = app.state::<Arc<Mutex<HotkeyState>>>();
    let mut state = state.lock().await;
    if !hotkey_registration_is_current(revision) {
        return Ok(());
    }

    register_both_hotkeys_locked(&app, longpress_shortcut, toggle_shortcut, &mut state)
}

fn parse_configured_shortcut(
    shortcut: Option<String>,
    label: &str,
) -> Result<Option<(String, Shortcut)>, String> {
    let Some(shortcut) = shortcut.filter(|shortcut| !shortcut.is_empty()) else {
        return Ok(None);
    };
    let parsed = shortcut
        .parse::<Shortcut>()
        .map_err(|error| format!("{label}快捷键格式错误: {error}"))?;
    Ok(Some((shortcut, parsed)))
}

fn parse_configured_shortcuts(
    longpress_shortcut: Option<String>,
    toggle_shortcut: Option<String>,
) -> Result<(Option<(String, Shortcut)>, Option<(String, Shortcut)>), String> {
    let longpress = parse_configured_shortcut(longpress_shortcut, "长按模式")?;
    let toggle = parse_configured_shortcut(toggle_shortcut, "切换模式")?;
    if let (Some((_, longpress)), Some((_, toggle))) = (&longpress, &toggle) {
        if longpress == toggle {
            return Err("长按模式和切换模式不能使用相同快捷键".to_string());
        }
    }
    Ok((longpress, toggle))
}

fn register_configured_shortcut(
    app: &AppHandle,
    shortcut: Shortcut,
    mode: RecordingMode,
    label: &str,
) -> Result<(), String> {
    let app_clone = app.clone();
    app.global_shortcut()
        .on_shortcut(shortcut, move |_app_handle, _shortcut, event| {
            handle_hotkey_event(&app_clone, event, &mode);
        })
        .map_err(|error| format!("{label}快捷键注册失败: {error}. 可能与其他应用冲突"))
}

fn unregister_configured_shortcut(app: &AppHandle, shortcut: &str) {
    if let Ok(shortcut) = shortcut.parse::<Shortcut>() {
        let _ = app.global_shortcut().unregister(shortcut);
    }
}

fn restore_previous_hotkeys(
    app: &AppHandle,
    state: &mut HotkeyState,
    previous_longpress: Option<String>,
    previous_toggle: Option<String>,
    previous_is_recording: bool,
) -> Result<(), String> {
    if state.longpress_registered {
        if let Some(shortcut) = state.longpress_shortcut.as_deref() {
            unregister_configured_shortcut(app, shortcut);
        }
    }
    if state.toggle_registered {
        if let Some(shortcut) = state.toggle_shortcut.as_deref() {
            unregister_configured_shortcut(app, shortcut);
        }
    }
    state.longpress_registered = false;
    state.toggle_registered = false;
    state.longpress_shortcut = None;
    state.toggle_shortcut = None;
    state.is_recording_via_hotkey = false;

    if let Some(shortcut) = previous_longpress.as_deref() {
        let parsed = shortcut
            .parse::<Shortcut>()
            .map_err(|error| format!("恢复长按模式快捷键失败: {error}"))?;
        if let Err(error) =
            register_configured_shortcut(app, parsed, RecordingMode::LongPress, "恢复长按模式")
        {
            return Err(error);
        }
    }
    if let Some(shortcut) = previous_toggle.as_deref() {
        let parsed = shortcut
            .parse::<Shortcut>()
            .map_err(|error| format!("恢复切换模式快捷键失败: {error}"))?;
        if let Err(error) =
            register_configured_shortcut(app, parsed, RecordingMode::Toggle, "恢复切换模式")
        {
            if let Some(longpress) = previous_longpress.as_deref() {
                unregister_configured_shortcut(app, longpress);
            }
            return Err(error);
        }
    }

    state.longpress_registered = previous_longpress.is_some();
    state.toggle_registered = previous_toggle.is_some();
    state.longpress_shortcut = previous_longpress;
    state.toggle_shortcut = previous_toggle;
    state.is_recording_via_hotkey = previous_is_recording;
    Ok(())
}

fn rollback_hotkey_update(
    app: &AppHandle,
    state: &mut HotkeyState,
    previous_longpress: Option<String>,
    previous_toggle: Option<String>,
    previous_is_recording: bool,
    error: String,
) -> String {
    match restore_previous_hotkeys(
        app,
        state,
        previous_longpress,
        previous_toggle,
        previous_is_recording,
    ) {
        Ok(()) => error,
        Err(restore_error) => format!("{error}; 快捷键恢复失败：{restore_error}"),
    }
}

fn register_both_hotkeys_locked(
    app: &AppHandle,
    longpress_shortcut: Option<String>,
    toggle_shortcut: Option<String>,
    state: &mut HotkeyState,
) -> Result<(), String> {
    println!(
        "register_both_hotkeys 被调用: longpress={:?}, toggle={:?}",
        longpress_shortcut, toggle_shortcut
    );
    let (longpress, toggle) = parse_configured_shortcuts(longpress_shortcut, toggle_shortcut)?;
    let previous_longpress = state
        .longpress_registered
        .then(|| state.longpress_shortcut.clone())
        .flatten();
    let previous_toggle = state
        .toggle_registered
        .then(|| state.toggle_shortcut.clone())
        .flatten();
    let previous_is_recording = state.is_recording_via_hotkey;

    if let Some(shortcut) = previous_longpress.as_deref() {
        unregister_configured_shortcut(app, shortcut);
        println!("已注销长按模式快捷键: {shortcut}");
    }
    if let Some(shortcut) = previous_toggle.as_deref() {
        unregister_configured_shortcut(app, shortcut);
        println!("已注销切换模式快捷键: {shortcut}");
    }
    state.longpress_registered = false;
    state.toggle_registered = false;
    state.longpress_shortcut = None;
    state.toggle_shortcut = None;
    state.is_recording_via_hotkey = false;

    if let Some((shortcut, parsed)) = longpress {
        println!("尝试注册长按模式快捷键: {shortcut}");
        if let Err(error) =
            register_configured_shortcut(app, parsed, RecordingMode::LongPress, "长按模式")
        {
            return Err(rollback_hotkey_update(
                app,
                state,
                previous_longpress,
                previous_toggle,
                previous_is_recording,
                error,
            ));
        }
        state.longpress_registered = true;
        state.longpress_shortcut = Some(shortcut);
    }

    if let Some((shortcut, parsed)) = toggle {
        println!("尝试注册切换模式快捷键: {shortcut}");
        if let Err(error) =
            register_configured_shortcut(app, parsed, RecordingMode::Toggle, "切换模式")
        {
            return Err(rollback_hotkey_update(
                app,
                state,
                previous_longpress,
                previous_toggle,
                previous_is_recording,
                error,
            ));
        }
        state.toggle_registered = true;
        state.toggle_shortcut = Some(shortcut);
    }

    println!("register_both_hotkeys 完成");
    Ok(())
}

// 注销全局快捷键
#[tauri::command]
#[specta::specta]
pub async fn unregister_hotkey(app: AppHandle) -> Result<(), String> {
    let state = app.state::<Arc<Mutex<HotkeyState>>>();
    let mut state = state.lock().await;

    // 注销长按模式快捷键
    if state.longpress_registered {
        if let Some(shortcut) = &state.longpress_shortcut {
            let shortcut_obj: Shortcut = shortcut
                .parse()
                .map_err(|e| format!("解析快捷键失败: {}", e))?;
            let _ = app.global_shortcut().unregister(shortcut_obj);
        }
    }

    // 注销切换模式快捷键
    if state.toggle_registered {
        if let Some(shortcut) = &state.toggle_shortcut {
            let shortcut_obj: Shortcut = shortcut
                .parse()
                .map_err(|e| format!("解析快捷键失败: {}", e))?;
            let _ = app.global_shortcut().unregister(shortcut_obj);
        }
    }

    state.longpress_registered = false;
    state.toggle_registered = false;
    state.longpress_shortcut = None;
    state.toggle_shortcut = None;
    state.is_recording_via_hotkey = false;

    Ok(())
}

// 处理快捷键事件
fn handle_hotkey_event(app: &AppHandle, event: ShortcutEvent, mode: &RecordingMode) {
    println!("快捷键事件触发: mode={:?}, state={:?}", mode, event.state);
    let app = app.clone();
    let mode = mode.clone();

    tauri::async_runtime::spawn(async move {
        // 全局快捷键插件会连续回调 Pressed/Released。录音启动包含设备初始化，
        // 如果两个回调各自并发执行，Released 可能在 Pressed 写入状态前先返回，
        // 留下一条无法通过长按释放停止的录音。用独立 gate 保证事件按进入顺序执行。
        let event_gate = {
            let state = app.state::<Arc<Mutex<HotkeyState>>>();
            let gate = state.lock().await.event_gate.clone();
            gate
        };
        let _event_guard = event_gate.lock().await;

        match mode {
            RecordingMode::LongPress => {
                handle_long_press_mode(&app, event).await;
            }
            RecordingMode::Toggle => {
                handle_toggle_mode(&app, event).await;
            }
        }
    });
}

// 长按模式处理
async fn handle_long_press_mode(app: &AppHandle, event: ShortcutEvent) {
    use crate::capture_session::CaptureSource;
    use crate::recording::{
        start_recording_for_source, stop_recording_for_session, RecordingState,
    };

    let hotkey_state = app.state::<Arc<Mutex<HotkeyState>>>();

    match event.state {
        tauri_plugin_global_shortcut::ShortcutState::Pressed => {
            println!("长按模式: 按键按下，准备开始录音");
            let recording_state = app.state::<RecordingState>();
            match start_recording_for_source(&recording_state, app, CaptureSource::Hotkey) {
                Ok(_) => {
                    let _ = crate::indicator::show_indicator(app);
                    println!("长按模式: 录音启动成功");
                    // 更新快捷键状态
                    let mut state = hotkey_state.lock().await;
                    state.is_recording_via_hotkey = true;
                }
                Err(e) => {
                    eprintln!("长按模式: 启动录音失败: {:?}", e);
                    if !e.contains("上一条语音输入仍在处理中") {
                        let _ = crate::indicator::finish_indicator(app, "failed");
                        let _ = RecordingErrorEvent(e).emit(app);
                    }
                }
            }
        }
        tauri_plugin_global_shortcut::ShortcutState::Released => {
            let is_recording = hotkey_state.lock().await.is_recording_via_hotkey;
            if !is_recording {
                return;
            }
            println!("长按模式: 按键松开，准备停止录音");
            let recording_state = app.state::<RecordingState>();
            match stop_recording_for_session(&recording_state, app).await {
                Ok(result) => {
                    println!("长按模式: 录音停止成功，时长: {}ms", result.duration_ms);
                    let _ = crate::indicator::update_indicator(app, "transcribing");
                    // 更新快捷键状态
                    let mut state = hotkey_state.lock().await;
                    state.is_recording_via_hotkey = false;
                    crate::capture_coordinator::spawn_recording_pipeline(app.clone(), result);
                }
                Err(e) => {
                    eprintln!("长按模式: 停止录音失败: {:?}", e);
                    app.state::<crate::capture_session::CaptureSessionState>()
                        .cancel_current();
                    let _ = crate::indicator::finish_indicator(app, "failed");
                    let _ = RecordingErrorEvent(e).emit(app);
                }
            }
        }
    }
}

// 切换模式处理
async fn handle_toggle_mode(app: &AppHandle, event: ShortcutEvent) {
    use crate::capture_session::CaptureSource;
    use crate::recording::{
        start_recording_for_source, stop_recording_for_session, RecordingState,
    };

    // 只响应按下事件
    if event.state != tauri_plugin_global_shortcut::ShortcutState::Pressed {
        return;
    }

    let hotkey_state = app.state::<Arc<Mutex<HotkeyState>>>();
    let recording_state = app.state::<RecordingState>();

    // 检查当前是否正在录音
    let is_recording = {
        let state = hotkey_state.lock().await;
        state.is_recording_via_hotkey
    };

    if is_recording {
        // 正在录音:停止录音
        println!("切换模式: 当前正在录音，准备停止");
        match stop_recording_for_session(&recording_state, app).await {
            Ok(result) => {
                println!("切换模式: 录音停止成功，时长: {}ms", result.duration_ms);
                let _ = crate::indicator::update_indicator(app, "transcribing");
                // 更新快捷键状态
                let mut state = hotkey_state.lock().await;
                state.is_recording_via_hotkey = false;
                crate::capture_coordinator::spawn_recording_pipeline(app.clone(), result);
            }
            Err(e) => {
                eprintln!("切换模式: 停止录音失败: {:?}", e);
                app.state::<crate::capture_session::CaptureSessionState>()
                    .cancel_current();
                let _ = crate::indicator::finish_indicator(app, "failed");
                let _ = RecordingErrorEvent(e).emit(app);
            }
        }
    } else {
        // 未录音:开始录音
        println!("切换模式: 当前未录音，准备开始");
        match start_recording_for_source(&recording_state, app, CaptureSource::Hotkey) {
            Ok(_) => {
                let _ = crate::indicator::show_indicator(app);
                println!("切换模式: 录音启动成功");
                // 更新快捷键状态
                let mut state = hotkey_state.lock().await;
                state.is_recording_via_hotkey = true;
            }
            Err(e) => {
                eprintln!("切换模式: 启动录音失败: {:?}", e);
                if !e.contains("上一条语音输入仍在处理中") {
                    let _ = crate::indicator::finish_indicator(app, "failed");
                    let _ = RecordingErrorEvent(e).emit(app);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_shortcuts_reject_duplicate_modes() {
        let error = parse_configured_shortcuts(
            Some("Alt+Space".to_string()),
            Some("Alt+Space".to_string()),
        )
        .expect_err("the two recording modes must not share one shortcut");

        assert!(error.contains("不能使用相同快捷键"));
    }

    #[test]
    fn configured_shortcuts_allow_disabled_mode() {
        let (longpress, toggle) =
            parse_configured_shortcuts(None, Some("Alt+Space".to_string())).unwrap();

        assert!(longpress.is_none());
        assert_eq!(
            toggle.expect("toggle shortcut should be parsed").0,
            "Alt+Space"
        );
    }
}
