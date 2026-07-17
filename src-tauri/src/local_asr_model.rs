use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use serde::Serialize;
use tauri::{Emitter, Manager};

use crate::capture_session::CaptureSessionState;

pub const LOCAL_ASR_MODEL_NAME: &str = "ggml-base-q5_1.bin";
pub const LOCAL_ASR_MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base-q5_1.bin";
const MIN_MODEL_BYTES: u64 = 10 * 1024 * 1024;

struct TemporaryDownload {
    path: PathBuf,
    keep: bool,
}

impl Drop for TemporaryDownload {
    fn drop(&mut self) {
        if !self.keep {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalAsrModelInfo {
    pub name: String,
    pub path: String,
    pub exists: bool,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalAsrDownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub percent: Option<u8>,
}

pub fn model_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("models");
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory.join(LOCAL_ASR_MODEL_NAME))
}

fn model_info(app: &tauri::AppHandle) -> Result<LocalAsrModelInfo, String> {
    let path = model_path(app)?;
    let metadata = std::fs::metadata(&path).ok();
    Ok(LocalAsrModelInfo {
        name: LOCAL_ASR_MODEL_NAME.to_string(),
        path: path.to_string_lossy().to_string(),
        exists: metadata.is_some(),
        size_bytes: metadata.map(|value| value.len()).unwrap_or(0),
    })
}

#[tauri::command]
pub fn local_asr_model_info(app: tauri::AppHandle) -> Result<LocalAsrModelInfo, String> {
    model_info(&app)
}

#[tauri::command]
pub async fn download_local_asr_model(app: tauri::AppHandle) -> Result<LocalAsrModelInfo, String> {
    if app.state::<CaptureSessionState>().has_active() {
        return Err("语音输入正在进行中，请完成后再下载模型".to_string());
    }
    let app_for_download = app.clone();
    tauri::async_runtime::spawn_blocking(move || download_model(&app_for_download))
        .await
        .map_err(|error| format!("模型下载任务失败：{error}"))??;
    model_info(&app)
}

fn download_model(app: &tauri::AppHandle) -> Result<(), String> {
    let target = model_path(app)?;
    let temporary = target.with_extension("download");
    let _ = std::fs::remove_file(&temporary);
    let mut temporary_guard = TemporaryDownload {
        path: temporary.clone(),
        keep: false,
    };

    let mut response = reqwest::blocking::Client::new()
        .get(LOCAL_ASR_MODEL_URL)
        .header("User-Agent", "XiLuoLin/0.1")
        .send()
        .map_err(|error| format!("下载本地 ASR 模型失败：{error}"))?
        .error_for_status()
        .map_err(|error| format!("下载本地 ASR 模型失败：{error}"))?;
    let total = response.content_length();
    let mut file = File::create(&temporary).map_err(|error| error.to_string())?;
    let mut buffer = [0_u8; 64 * 1024];
    let mut downloaded = 0_u64;

    loop {
        let read = response
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])
            .map_err(|error| error.to_string())?;
        downloaded += read as u64;
        let percent = total.map(|total| ((downloaded * 100 / total.max(1)).min(100)) as u8);
        let _ = app.emit(
            "local-asr-download-progress",
            LocalAsrDownloadProgress {
                downloaded_bytes: downloaded,
                total_bytes: total,
                percent,
            },
        );
    }
    file.sync_all().map_err(|error| error.to_string())?;

    if downloaded < MIN_MODEL_BYTES {
        return Err("下载的模型文件大小异常".to_string());
    }
    if target.exists() {
        std::fs::remove_file(&target).map_err(|error| error.to_string())?;
    }
    std::fs::rename(&temporary, &target).map_err(|error| error.to_string())?;
    temporary_guard.keep = true;
    Ok(())
}

#[tauri::command]
pub fn delete_local_asr_model(app: tauri::AppHandle) -> Result<LocalAsrModelInfo, String> {
    if app.state::<CaptureSessionState>().has_active() {
        return Err("语音输入正在进行中，请完成后再删除模型".to_string());
    }
    let path = model_path(&app)?;
    if path.exists() {
        std::fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    crate::local_asr::clear_model_cache();
    model_info(&app)
}

#[tauri::command]
pub async fn verify_local_asr_model(app: tauri::AppHandle) -> Result<(), String> {
    let path = model_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || crate::local_asr::verify_model(&path))
        .await
        .map_err(|error| format!("模型验证任务失败：{error}"))?
}
