use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::Manager;
use tauri_plugin_opener::OpenerExt;

use crate::{capture_session::CaptureSessionState, data::LocalDatabase};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordingStorageInfo {
    pub file_count: u64,
    pub total_bytes: u64,
    pub directory: String,
}

fn recordings_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("recordings");
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    std::fs::canonicalize(directory).map_err(|error| error.to_string())
}

fn database_for_app(app: &tauri::AppHandle) -> Result<LocalDatabase, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    let database = LocalDatabase::open(app_data_dir.join("xiluolin.sqlite"))
        .map_err(|error| error.to_string())?;
    database.initialize().map_err(|error| error.to_string())?;
    Ok(database)
}

fn managed_recording_path(root: &Path, path: &Path) -> Result<Option<PathBuf>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let canonical = std::fs::canonicalize(path).map_err(|error| error.to_string())?;
    if !canonical.starts_with(root)
        || !canonical.is_file()
        || !canonical
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("wav"))
    {
        return Err("录音文件不属于应用管理目录".to_string());
    }
    Ok(Some(canonical))
}

pub fn remove_managed_recording(app: &tauri::AppHandle, path: &str) -> Result<(), String> {
    let root = recordings_dir(app)?;
    if let Some(path) = managed_recording_path(&root, Path::new(path))? {
        std::fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn recording_storage_info(app: tauri::AppHandle) -> Result<RecordingStorageInfo, String> {
    let root = recordings_dir(&app)?;
    let mut file_count = 0;
    let mut total_bytes = 0;

    for entry in std::fs::read_dir(&root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if let Some(path) = managed_recording_path(&root, &entry.path())? {
            let metadata = std::fs::metadata(path).map_err(|error| error.to_string())?;
            file_count += 1;
            total_bytes += metadata.len();
        }
    }

    Ok(RecordingStorageInfo {
        file_count,
        total_bytes,
        directory: root.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub fn open_recordings_directory(app: tauri::AppHandle) -> Result<(), String> {
    let root = recordings_dir(&app)?;
    app.opener()
        .open_path(root.to_string_lossy().to_string(), None::<String>)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn clear_retained_recordings(app: tauri::AppHandle) -> Result<RecordingStorageInfo, String> {
    if app.state::<CaptureSessionState>().has_active() {
        return Err("语音输入正在进行中，请完成后再清理录音".to_string());
    }

    let root = recordings_dir(&app)?;
    for entry in std::fs::read_dir(&root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if let Some(path) = managed_recording_path(&root, &entry.path())? {
            std::fs::remove_file(path).map_err(|error| error.to_string())?;
        }
    }

    database_for_app(&app)?
        .clear_history_audio_paths()
        .map_err(|error| error.to_string())?;
    recording_storage_info(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_path_is_rejected() {
        let root =
            std::env::temp_dir().join(format!("xiluolin-storage-root-{}", uuid::Uuid::new_v4()));
        let outside = std::env::temp_dir().join(format!(
            "xiluolin-storage-outside-{}.wav",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&outside, b"audio").unwrap();
        let root = std::fs::canonicalize(root).unwrap();

        assert!(managed_recording_path(&root, &outside).is_err());
        assert!(outside.exists());
        let _ = std::fs::remove_file(outside);
    }
}
