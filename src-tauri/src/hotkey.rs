use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent};
use std::sync::Arc;
use tokio::sync::Mutex;

// 快捷键状态管理
pub struct HotkeyState {
    pub is_registered: bool,
    pub current_shortcut: Option<String>,
    pub recording_mode: RecordingMode,
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
            is_registered: false,
            current_shortcut: None,
            recording_mode: RecordingMode::Toggle,
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

    // 如果已注册,先注销
    if state.is_registered {
        if let Some(old_shortcut) = &state.current_shortcut {
            let old_shortcut_obj: Shortcut = old_shortcut.parse()
                .map_err(|e| format!("解析旧快捷键失败: {}", e))?;
            let _ = app.global_shortcut().unregister(old_shortcut_obj);
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
    state.is_registered = true;
    state.current_shortcut = Some(shortcut);
    state.recording_mode = recording_mode;
    state.is_recording_via_hotkey = false;

    Ok(())
}

// 注销全局快捷键
#[tauri::command]
pub async fn unregister_hotkey(app: AppHandle) -> Result<(), String> {
    let state = app.state::<Arc<Mutex<HotkeyState>>>();
    let mut state = state.lock().await;

    if let Some(shortcut) = &state.current_shortcut {
        let shortcut_obj: Shortcut = shortcut.parse()
            .map_err(|e| format!("解析快捷键失败: {}", e))?;
        app.global_shortcut()
            .unregister(shortcut_obj)
            .map_err(|e| format!("快捷键注销失败: {}", e))?;
    }

    state.is_registered = false;
    state.current_shortcut = None;
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


