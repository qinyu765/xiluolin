use cpal::traits::{DeviceTrait, HostTrait};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{Manager, State};
use tauri_specta::Event;

use crate::audio_control::windows_audio;
use crate::capture_session::{
    CaptureSessionStart, CaptureSessionState, CaptureSource, CaptureStatus,
};
use crate::data;
use crate::events::{RecordingCompletedEvent, RecordingErrorEvent, RecordingLimitWarningEvent};
use crate::recording_worker::AudioWorker;

const RECORDING_WARNING_MS: u64 = 25_000;
const RECORDING_AUTO_STOP_MS: u64 = 28_000;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct RecordingResult {
    pub session_id: String,
    pub file_path: String,
    #[specta(type = specta_typescript::Number)]
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingError {
    AlreadyRecording,
    NoRecordingInProgress,
    MicrophonePermissionDenied,
    NoInputDeviceAvailable,
    DeviceConfigFailed(String),
    FileCreationFailed(String),
    StreamBuildFailed(String),
    StreamStartFailed(String),
    UnsupportedSampleFormat(String),
    StateLockFailed(String),
}

impl fmt::Display for RecordingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyRecording => write!(formatter, "录音已在进行中，请先停止当前录音"),
            Self::NoRecordingInProgress => write!(formatter, "当前没有正在进行的录音"),
            Self::MicrophonePermissionDenied => {
                write!(formatter, "麦克风权限缺失，请在系统设置中开启麦克风权限")
            }
            Self::NoInputDeviceAvailable => {
                write!(formatter, "未找到可用的音频输入设备，请检查麦克风连接")
            }
            Self::DeviceConfigFailed(message) => {
                write!(formatter, "获取音频设备配置失败：{message}")
            }
            Self::FileCreationFailed(message) => write!(formatter, "创建录音文件失败：{message}"),
            Self::StreamBuildFailed(message) => write!(formatter, "构建录音流失败：{message}"),
            Self::StreamStartFailed(message) => write!(formatter, "启动录音流失败：{message}"),
            Self::UnsupportedSampleFormat(format) => {
                write!(formatter, "不支持的音频采样格式：{format}")
            }
            Self::StateLockFailed(message) => write!(formatter, "录音状态锁定失败：{message}"),
        }
    }
}

impl std::error::Error for RecordingError {}

impl From<RecordingError> for String {
    fn from(error: RecordingError) -> Self {
        error.to_string()
    }
}

struct ActiveRecording {
    start_time: Instant,
    output_path: PathBuf,
    session_id: String,
    worker: AudioWorker,
}

pub struct RecordingState {
    active: Mutex<Option<ActiveRecording>>,
}

impl RecordingState {
    pub fn new() -> Self {
        Self {
            active: Mutex::new(None),
        }
    }

    fn session_id(&self) -> Option<String> {
        self.active.lock().ok().and_then(|active| {
            active
                .as_ref()
                .map(|recording| recording.session_id.clone())
        })
    }

    fn take(&self, expected_session_id: Option<&str>) -> Result<ActiveRecording, RecordingError> {
        let mut active = self
            .active
            .lock()
            .map_err(|error| RecordingError::StateLockFailed(error.to_string()))?;
        let recording = active
            .as_ref()
            .ok_or(RecordingError::NoRecordingInProgress)?;
        if expected_session_id.is_some_and(|expected| recording.session_id != expected) {
            return Err(RecordingError::NoRecordingInProgress);
        }
        active.take().ok_or(RecordingError::NoRecordingInProgress)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeadlineAction {
    None,
    Warn,
    Stop,
}

enum StopRecordingFailure {
    StaleSession,
    Failed(String),
}

impl StopRecordingFailure {
    fn into_message(self) -> String {
        match self {
            Self::StaleSession => RecordingError::NoRecordingInProgress.to_string(),
            Self::Failed(message) => message,
        }
    }
}

fn deadline_action(elapsed_ms: u64, warning_sent: bool) -> DeadlineAction {
    if elapsed_ms >= RECORDING_AUTO_STOP_MS {
        DeadlineAction::Stop
    } else if elapsed_ms >= RECORDING_WARNING_MS && !warning_sent {
        DeadlineAction::Warn
    } else {
        DeadlineAction::None
    }
}

fn recording_session_matches(active_session_id: Option<&str>, expected_session_id: &str) -> bool {
    active_session_id == Some(expected_session_id)
}

#[tauri::command]
#[specta::specta]
pub async fn start_recording(
    state: State<'_, RecordingState>,
    app_handle: tauri::AppHandle,
) -> Result<CaptureSessionStart, String> {
    start_recording_for_source(&state, &app_handle, CaptureSource::App)
}

pub fn start_recording_for_source(
    state: &RecordingState,
    app_handle: &tauri::AppHandle,
    source: CaptureSource,
) -> Result<CaptureSessionStart, String> {
    let session_state = app_handle.state::<CaptureSessionState>();
    let started = session_state.begin(source)?;

    if let Err(error) = start_audio_capture(state, app_handle, &started.session_id) {
        session_state.cancel(&started.session_id);
        return Err(error);
    }

    Ok(started)
}

fn start_audio_capture(
    state: &RecordingState,
    app_handle: &tauri::AppHandle,
    session_id: &str,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    if crate::macos_permissions::microphone_status()
        != crate::macos_permissions::PermissionStatus::Authorized
    {
        return Err(RecordingError::MicrophonePermissionDenied.into());
    }

    let mut active = state
        .active
        .lock()
        .map_err(|e| RecordingError::StateLockFailed(e.to_string()))?;

    if active.is_some() {
        return Err(RecordingError::AlreadyRecording.into());
    }

    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| RecordingError::FileCreationFailed(e.to_string()))?;
    let recordings_dir = app_data_dir.join("recordings");
    fs::create_dir_all(&recordings_dir)
        .map_err(|e| RecordingError::FileCreationFailed(e.to_string()))?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S_%3f").to_string();
    let output_path = recordings_dir.join(format!("recording_{timestamp}.wav"));

    let config = data::read_app_config(app_handle.clone()).ok();
    let selected_microphone = config
        .as_ref()
        .map(|config| config.selected_microphone.clone())
        .unwrap_or_default();
    let worker = AudioWorker::start(output_path.clone(), selected_microphone)?;

    *active = Some(ActiveRecording {
        start_time: Instant::now(),
        output_path,
        session_id: session_id.to_string(),
        worker,
    });
    drop(active);

    if let Some(config) = config {
        if config.mute_system_audio {
            let _ = windows_audio::mute_all_sessions();
        }
    }

    schedule_recording_deadlines(app_handle.clone(), session_id.to_string());

    Ok(())
}

fn schedule_recording_deadlines(app: tauri::AppHandle, session_id: String) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(RECORDING_WARNING_MS)).await;
        let recording_state = app.state::<RecordingState>();
        let active_session_id = recording_state.session_id();
        if !recording_session_matches(active_session_id.as_deref(), &session_id) {
            return;
        }
        if deadline_action(RECORDING_WARNING_MS, false) == DeadlineAction::Warn {
            let _ = RecordingLimitWarningEvent {
                session_id: session_id.clone(),
                remaining_ms: RECORDING_AUTO_STOP_MS - RECORDING_WARNING_MS,
            }
            .emit(&app);
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(
            RECORDING_AUTO_STOP_MS - RECORDING_WARNING_MS,
        ))
        .await;
        let active_session_id = recording_state.session_id();
        if !recording_session_matches(active_session_id.as_deref(), &session_id) {
            return;
        }
        if deadline_action(RECORDING_AUTO_STOP_MS, true) != DeadlineAction::Stop {
            return;
        }

        if let Some(hotkey_state) =
            app.try_state::<Arc<tokio::sync::Mutex<crate::hotkey::HotkeyState>>>()
        {
            hotkey_state.lock().await.is_recording_via_hotkey = false;
        }
        match stop_recording_for_session_id(&recording_state, &app, Some(&session_id)).await {
            Ok(result) => {
                let _ = crate::indicator::update_indicator(&app, "transcribing");
                let _ = RecordingCompletedEvent(result).emit(&app);
            }
            Err(StopRecordingFailure::StaleSession) => {}
            Err(StopRecordingFailure::Failed(error)) => {
                app.state::<CaptureSessionState>().cancel(&session_id);
                let _ = crate::indicator::finish_indicator(&app, "failed");
                let _ = RecordingErrorEvent(error).emit(&app);
            }
        }
    });
}

#[tauri::command]
#[specta::specta]
pub async fn stop_recording(
    state: State<'_, RecordingState>,
    app_handle: tauri::AppHandle,
) -> Result<RecordingResult, String> {
    stop_recording_for_session(&state, &app_handle).await
}

pub async fn stop_recording_for_session(
    state: &RecordingState,
    app_handle: &tauri::AppHandle,
) -> Result<RecordingResult, String> {
    stop_recording_for_session_id(state, app_handle, None)
        .await
        .map_err(StopRecordingFailure::into_message)
}

pub async fn stop_recording_for_expected_session(
    state: &RecordingState,
    app_handle: &tauri::AppHandle,
    expected_session_id: &str,
) -> Result<Option<RecordingResult>, String> {
    match stop_recording_for_session_id(state, app_handle, Some(expected_session_id)).await {
        Ok(result) => Ok(Some(result)),
        Err(StopRecordingFailure::StaleSession) => Ok(None),
        Err(StopRecordingFailure::Failed(error)) => Err(error),
    }
}

async fn stop_recording_for_session_id(
    state: &RecordingState,
    app_handle: &tauri::AppHandle,
    expected_session_id: Option<&str>,
) -> Result<RecordingResult, StopRecordingFailure> {
    let recording = match state.take(expected_session_id) {
        Ok(recording) => recording,
        Err(RecordingError::NoRecordingInProgress) => {
            return Err(StopRecordingFailure::StaleSession)
        }
        Err(error) => return Err(StopRecordingFailure::Failed(error.to_string())),
    };
    let duration_ms = recording.start_time.elapsed().as_millis() as u64;
    let session_id = recording.session_id;
    let output_path = recording.output_path;
    // cpal::Stream 由创建它的工作线程持有；Stop 命令会先释放流，再 finalize WAV。
    if let Err(error) = recording.worker.stop() {
        let _ = fs::remove_file(&output_path);
        let _ = windows_audio::unmute_all_sessions();
        return Err(StopRecordingFailure::Failed(error));
    }

    let _ = windows_audio::unmute_all_sessions();
    app_handle
        .state::<CaptureSessionState>()
        .update_status(&session_id, CaptureStatus::Transcribing)
        .map_err(StopRecordingFailure::Failed)?;

    Ok(RecordingResult {
        session_id,
        file_path: output_path.to_string_lossy().to_string(),
        duration_ms,
    })
}

pub async fn cancel_recording_for_session(
    state: &RecordingState,
    app_handle: &tauri::AppHandle,
    expected_session_id: &str,
) -> Result<(), String> {
    let recording = state
        .take(Some(expected_session_id))
        .map_err(String::from)?;
    let session_id = recording.session_id;
    let output_path = recording.output_path;
    let worker_result = recording.worker.cancel();
    let _ = fs::remove_file(&output_path);
    let _ = windows_audio::unmute_all_sessions();
    app_handle
        .state::<CaptureSessionState>()
        .cancel(&session_id);
    worker_result
}

#[tauri::command]
#[specta::specta]
pub fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();

    let default_device_name = host
        .default_input_device()
        .and_then(|device| device.description().ok())
        .map(|description| description.name().to_string());

    let devices = host
        .input_devices()
        .map_err(|e| format!("获取音频设备列表失败: {}", e))?;

    let mut result = Vec::new();
    for device in devices {
        if let Ok(description) = device.description() {
            let name = description.name().to_string();
            let is_default = default_device_name.as_ref() == Some(&name);
            result.push(AudioDevice { name, is_default });
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_deadline_warns_once_then_stops_at_twenty_eight_seconds() {
        assert_eq!(deadline_action(24_999, false), DeadlineAction::None);
        assert_eq!(deadline_action(25_000, false), DeadlineAction::Warn);
        assert_eq!(deadline_action(27_999, true), DeadlineAction::None);
        assert_eq!(deadline_action(28_000, false), DeadlineAction::Stop);
        assert_eq!(deadline_action(30_000, true), DeadlineAction::Stop);
    }

    #[test]
    fn stale_deadline_cannot_stop_a_new_recording_session() {
        assert!(recording_session_matches(Some("session-a"), "session-a"));
        assert!(!recording_session_matches(Some("session-b"), "session-a"));
        assert!(!recording_session_matches(None, "session-a"));
    }
}
