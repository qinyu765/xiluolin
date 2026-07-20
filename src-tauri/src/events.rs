use serde::{Deserialize, Serialize};
use tauri_specta::Event;

use crate::{local_asr_model::LocalAsrDownloadProgress, recording::RecordingResult};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, Event)]
#[serde(transparent)]
#[tauri_specta(event_name = "recording-completed")]
pub struct RecordingCompletedEvent(pub RecordingResult);

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, Event)]
#[serde(transparent)]
#[tauri_specta(event_name = "recording-error")]
pub struct RecordingErrorEvent(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, Event)]
#[serde(transparent)]
#[tauri_specta(event_name = "local-asr-download-progress")]
pub struct LocalAsrDownloadProgressEvent(pub LocalAsrDownloadProgress);
