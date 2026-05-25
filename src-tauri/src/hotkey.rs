use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent};
use std::sync::Arc;
use tokio::sync::Mutex;

// 快捷键状态管理
pub struct HotkeyState {
    pub longpress_registered: bool,
    pub toggle_registered: bool,
    pub longpress_shortcut: Option<String>,
    pub toggle_shortcut: Option<String>,
    pub is_recording_via_hotkey: bool,  // 跟踪通过快捷键触发的录音状态
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
        }
    }
}

// 注册全局快捷键
#[tauri::command]
pub async fn register_hotkey(
    app: AppHandle,
    shortcut: String,
    mode: String,
) -> Result<(), String> {
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
                    let old_shortcut_obj: Shortcut = old_shortcut.parse()
                        .map_err(|e| format!("解析旧快捷键失败: {}", e))?;
                    let _ = app.global_shortcut().unregister(old_shortcut_obj);
                }
            }
        }
        RecordingMode::Toggle => {
            if state.toggle_registered {
                if let Some(old_shortcut) = &state.toggle_shortcut {
                    let old_shortcut_obj: Shortcut = old_shortcut.parse()
                        .map_err(|e| format!("解析旧快捷键失败: {}", e))?;
                    let _ = app.global_shortcut().unregister(old_shortcut_obj);
                }
            }
        }
    }

    // 注册新快捷键
    let shortcut_obj: Shortcut = shortcut.parse()
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
pub async fn register_both_hotkeys(
    app: AppHandle,
    longpress_shortcut: Option<String>,
    toggle_shortcut: Option<String>,
) -> Result<(), String> {
    let state = app.state::<Arc<Mutex<HotkeyState>>>();
    let mut state = state.lock().await;

    // 先注销所有已注册的快捷键
    if state.longpress_registered {
        if let Some(shortcut) = &state.longpress_shortcut {
            if let Ok(shortcut_obj) = shortcut.parse::<Shortcut>() {
                let _ = app.global_shortcut().unregister(shortcut_obj);
            }
        }
    }
    if state.toggle_registered {
        if let Some(shortcut) = &state.toggle_shortcut {
            if let Ok(shortcut_obj) = shortcut.parse::<Shortcut>() {
                let _ = app.global_shortcut().unregister(shortcut_obj);
            }
        }
    }

    // 重置状态
    state.longpress_registered = false;
    state.toggle_registered = false;
    state.longpress_shortcut = None;
    state.toggle_shortcut = None;
    state.is_recording_via_hotkey = false;

    // 注册长按模式快捷键
    if let Some(shortcut) = longpress_shortcut {
        if !shortcut.is_empty() {
            let shortcut_obj: Shortcut = shortcut.parse()
                .map_err(|e| format!("长按模式快捷键格式错误: {}", e))?;

            let app_clone = app.clone();
            app.global_shortcut()
                .on_shortcut(shortcut_obj, move |_app_handle, _shortcut, event| {
                    handle_hotkey_event(&app_clone, event, &RecordingMode::LongPress);
                })
                .map_err(|e| format!("长按模式快捷键注册失败: {}. 可能与其他应用冲突", e))?;

            state.longpress_registered = true;
            state.longpress_shortcut = Some(shortcut);
        }
    }

    // 注册切换模式快捷键
    if let Some(shortcut) = toggle_shortcut {
        if !shortcut.is_empty() {
            let shortcut_obj: Shortcut = shortcut.parse()
                .map_err(|e| format!("切换模式快捷键格式错误: {}", e))?;

            let app_clone = app.clone();
            app.global_shortcut()
                .on_shortcut(shortcut_obj, move |_app_handle, _shortcut, event| {
                    handle_hotkey_event(&app_clone, event, &RecordingMode::Toggle);
                })
                .map_err(|e| format!("切换模式快捷键注册失败: {}. 可能与其他应用冲突", e))?;

            state.toggle_registered = true;
            state.toggle_shortcut = Some(shortcut);
        }
    }

    Ok(())
}

// 注销全局快捷键
#[tauri::command]
pub async fn unregister_hotkey(app: AppHandle) -> Result<(), String> {
    let state = app.state::<Arc<Mutex<HotkeyState>>>();
    let mut state = state.lock().await;

    // 注销长按模式快捷键
    if state.longpress_registered {
        if let Some(shortcut) = &state.longpress_shortcut {
            let shortcut_obj: Shortcut = shortcut.parse()
                .map_err(|e| format!("解析快捷键失败: {}", e))?;
            let _ = app.global_shortcut().unregister(shortcut_obj);
        }
    }

    // 注销切换模式快捷键
    if state.toggle_registered {
        if let Some(shortcut) = &state.toggle_shortcut {
            let shortcut_obj: Shortcut = shortcut.parse()
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
fn handle_hotkey_event(
    app: &AppHandle,
    event: ShortcutEvent,
    mode: &RecordingMode,
) {
    let app = app.clone();
    let mode = mode.clone();

    tauri::async_runtime::spawn(async move {
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
async fn handle_long_press_mode(
    app: &AppHandle,
    event: ShortcutEvent,
) {
    use crate::recording::{start_recording, stop_recording, RecordingState};

    let hotkey_state = app.state::<Arc<Mutex<HotkeyState>>>();

    match event.state {
        tauri_plugin_global_shortcut::ShortcutState::Pressed => {
            // 按下:开始录音
            let recording_state = app.state::<RecordingState>();
            match start_recording(recording_state, app.clone()).await {
                Ok(_) => {
                    // 更新快捷键状态
                    let mut state = hotkey_state.lock().await;
                    state.is_recording_via_hotkey = true;
                }
                Err(e) => {
                    eprintln!("启动录音失败: {:?}", e);
                    let _ = app.emit("recording-error", e);
                }
            }
        }
        tauri_plugin_global_shortcut::ShortcutState::Released => {
            // 松开:停止录音
            let recording_state = app.state::<RecordingState>();
            match stop_recording(recording_state).await {
                Ok(result) => {
                    // 更新快捷键状态
                    let mut state = hotkey_state.lock().await;
                    state.is_recording_via_hotkey = false;
                    // 触发后续处理流程
                    let _ = app.emit("recording-completed", result);
                }
                Err(e) => {
                    eprintln!("停止录音失败: {:?}", e);
                    let _ = app.emit("recording-error", e);
                }
            }
        }
    }
}

// 切换模式处理
async fn handle_toggle_mode(
    app: &AppHandle,
    event: ShortcutEvent,
) {
    use crate::recording::{start_recording, stop_recording, RecordingState};

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
        match stop_recording(recording_state).await {
            Ok(result) => {
                // 更新快捷键状态
                let mut state = hotkey_state.lock().await;
                state.is_recording_via_hotkey = false;
                let _ = app.emit("recording-completed", result);
            }
            Err(e) => {
                eprintln!("停止录音失败: {:?}", e);
                let _ = app.emit("recording-error", e);
            }
        }
    } else {
        // 未录音:开始录音
        match start_recording(recording_state, app.clone()).await {
            Ok(_) => {
                // 更新快捷键状态
                let mut state = hotkey_state.lock().await;
                state.is_recording_via_hotkey = true;
            }
            Err(e) => {
                eprintln!("启动录音失败: {:?}", e);
                let _ = app.emit("recording-error", e);
            }
        }
    }
}


