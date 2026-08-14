use serde::{Deserialize, Serialize};
use tauri_specta::Event;

use crate::{
    capture_session::CaptureSnapshot, local_asr_model::LocalAsrDownloadProgress,
    realtime_asr_model::RealtimeModelDownloadProgress,
};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, Event)]
#[serde(transparent)]
#[tauri_specta(event_name = "capture-snapshot")]
pub struct CaptureSnapshotEvent(pub CaptureSnapshot);

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, Event)]
#[tauri_specta(event_name = "history-changed")]
pub struct HistoryChangedEvent {
    pub history_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, Event)]
#[serde(transparent)]
#[tauri_specta(event_name = "realtime-asr-download-progress")]
pub struct RealtimeAsrDownloadProgressEvent(pub RealtimeModelDownloadProgress);

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, Event)]
#[serde(transparent)]
#[tauri_specta(event_name = "recording-error")]
pub struct RecordingErrorEvent(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, Event)]
#[tauri_specta(event_name = "recording-limit-warning")]
pub struct RecordingLimitWarningEvent {
    pub session_id: String,
    #[specta(type = specta_typescript::Number)]
    pub remaining_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, Event)]
#[serde(transparent)]
#[tauri_specta(event_name = "local-asr-download-progress")]
pub struct LocalAsrDownloadProgressEvent(pub LocalAsrDownloadProgress);
