use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::Manager;
use tauri_specta::Event;

use crate::{capture_session::CaptureSessionState, events::RealtimeAsrDownloadProgressEvent};

const MODEL_ID: &str = "csukuangfj/sherpa-onnx-streaming-zipformer-bilingual-zh-en-2023-02-20";
const MODEL_NAME: &str = "Zipformer 中英双语混合量化实验版";
const MODEL_REVISION: &str = "98590b7ed6443e77b714204da2757d75e1a642f4";
const MODEL_DIRECTORY: &str = "sherpa-onnx-streaming-zipformer-bilingual-zh-en-mixed-int8";
const VERIFIED_MARKER: &str = ".verified-revision";
static DOWNLOAD_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy)]
pub struct ModelArtifact {
    pub name: &'static str,
    pub size: u64,
    pub sha256: &'static str,
}

pub const MODEL_ARTIFACTS: &[ModelArtifact] = &[
    ModelArtifact {
        name: "encoder-epoch-99-avg-1.int8.onnx",
        size: 181_895_032,
        sha256: "8fa764187a261844f859d7143ebaa563af5d10adfece4c18a8f414c88cba2a9b",
    },
    ModelArtifact {
        name: "decoder-epoch-99-avg-1.onnx",
        size: 13_876_452,
        sha256: "2e3b5ec371f8899ee6acd829fd753ba45772df57a91bdf37cde3136354e7db7d",
    },
    ModelArtifact {
        name: "joiner-epoch-99-avg-1.int8.onnx",
        size: 3_228_404,
        sha256: "1ed689c5ed19dbaa725d9d191bb4822b5f4855a39e1ffd28cbc1f340d25b2ee0",
    },
    ModelArtifact {
        name: "tokens.txt",
        size: 56_317,
        sha256: "a8e0e4ec53810e433789b54a5c0134a7eaa2ffca595a6334d54c00da858841d3",
    },
    ModelArtifact {
        name: "bpe.model",
        size: 244_836,
        sha256: "bcae393dbc5611be5ffa4c7ae0841558978a5a4f484008cb9dff3a2cc97ebe01",
    },
    ModelArtifact {
        name: "bpe.vocab",
        size: 12_564,
        sha256: "d0b642f3a2eacd5fadefdeff9e0e1358cab729647cbb7fe58cf738e1f7407029",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum RealtimeModelState {
    NotDownloaded,
    Ready,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct RealtimeModelInfo {
    pub name: String,
    pub revision: String,
    pub path: String,
    pub state: RealtimeModelState,
    pub enabled: bool,
    #[specta(type = specta_typescript::Number)]
    pub total_size_bytes: u64,
    #[specta(type = specta_typescript::Number)]
    pub downloaded_size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct RealtimeModelDownloadProgress {
    pub file_name: String,
    pub file_index: u8,
    pub file_count: u8,
    #[specta(type = specta_typescript::Number)]
    pub downloaded_bytes: u64,
    #[specta(type = specta_typescript::Number)]
    pub total_bytes: u64,
    pub percent: u8,
}

struct TemporaryDirectory {
    path: PathBuf,
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

struct DownloadGuard;

impl DownloadGuard {
    fn acquire() -> Result<Self, String> {
        DOWNLOAD_ACTIVE
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| Self)
            .map_err(|_| "实时预览模型正在下载".to_string())
    }
}

impl Drop for DownloadGuard {
    fn drop(&mut self) {
        DOWNLOAD_ACTIVE.store(false, Ordering::Release);
    }
}

pub fn model_directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("models")
        .join(MODEL_DIRECTORY))
}

fn total_size() -> u64 {
    MODEL_ARTIFACTS.iter().map(|artifact| artifact.size).sum()
}

fn downloaded_size(directory: &Path) -> u64 {
    MODEL_ARTIFACTS
        .iter()
        .filter_map(|artifact| std::fs::metadata(directory.join(artifact.name)).ok())
        .map(|metadata| metadata.len())
        .sum()
}

pub(crate) fn has_verified_install(directory: &Path) -> bool {
    let files_match = MODEL_ARTIFACTS.iter().all(|artifact| {
        std::fs::metadata(directory.join(artifact.name))
            .map(|metadata| metadata.len() == artifact.size)
            .unwrap_or(false)
    });
    files_match
        && std::fs::read_to_string(directory.join(VERIFIED_MARKER))
            .map(|revision| revision.trim() == MODEL_REVISION)
            .unwrap_or(false)
}

fn write_verified_marker(directory: &Path) -> Result<(), String> {
    std::fs::write(directory.join(VERIFIED_MARKER), MODEL_REVISION)
        .map_err(|error| error.to_string())
}

pub(crate) fn verify_model_directory(directory: &Path) -> Result<(), String> {
    for artifact in MODEL_ARTIFACTS {
        verify_file(
            &directory.join(artifact.name),
            artifact.size,
            artifact.sha256,
        )?;
    }
    Ok(())
}

fn verify_file(path: &Path, expected_size: u64, expected_sha256: &str) -> Result<(), String> {
    let metadata =
        std::fs::metadata(path).map_err(|_| format!("模型文件缺失：{}", path.display()))?;
    if metadata.len() != expected_size {
        return Err(format!(
            "模型文件大小异常：{}（应为 {expected_size}，实际为 {}）",
            path.display(),
            metadata.len()
        ));
    }
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let actual = format!("{:x}", hasher.finalize());
    if actual != expected_sha256 {
        return Err(format!("模型文件校验失败：{}", path.display()));
    }
    Ok(())
}

fn read_info(app: &tauri::AppHandle) -> Result<RealtimeModelInfo, String> {
    let directory = recover_interrupted_install(app, !DOWNLOAD_ACTIVE.load(Ordering::Acquire))?;
    let downloaded_size_bytes = downloaded_size(&directory);
    let state = if !directory.exists() {
        RealtimeModelState::NotDownloaded
    } else if has_verified_install(&directory) {
        RealtimeModelState::Ready
    } else {
        RealtimeModelState::Invalid
    };
    let enabled = crate::data::read_app_config(app.clone())?.realtime_preview_enabled
        && state == RealtimeModelState::Ready;
    Ok(RealtimeModelInfo {
        name: MODEL_NAME.to_string(),
        revision: MODEL_REVISION.to_string(),
        path: directory.to_string_lossy().to_string(),
        state,
        enabled,
        total_size_bytes: total_size(),
        downloaded_size_bytes,
    })
}

fn recover_interrupted_install(
    app: &tauri::AppHandle,
    cleanup_staging: bool,
) -> Result<PathBuf, String> {
    let target = model_directory(app)?;
    let Some(models_directory) = target.parent() else {
        return Err("模型目录无效".to_string());
    };
    std::fs::create_dir_all(models_directory).map_err(|error| error.to_string())?;
    let backup = models_directory.join(format!("{MODEL_DIRECTORY}.backup"));
    if !target.exists() && has_verified_install(&backup) {
        std::fs::rename(&backup, &target).map_err(|error| error.to_string())?;
    } else if target.exists() && backup.exists() {
        let _ = std::fs::remove_dir_all(&backup);
    }
    if cleanup_staging {
        if let Ok(entries) = std::fs::read_dir(models_directory) {
            for entry in entries.flatten() {
                if entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(&format!("{MODEL_DIRECTORY}.download-"))
                {
                    let _ = std::fs::remove_dir_all(entry.path());
                }
            }
        }
    }
    Ok(target)
}

fn persist_enabled(app: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let mut config = crate::data::read_app_config(app.clone())?;
    config.realtime_preview_enabled = enabled;
    crate::data::update_app_config(app.clone(), config)?;
    Ok(())
}

fn download_model(app: &tauri::AppHandle) -> Result<(), String> {
    let target = recover_interrupted_install(app, true)?;
    let models_directory = target.parent().ok_or_else(|| "模型目录无效".to_string())?;
    std::fs::create_dir_all(models_directory).map_err(|error| error.to_string())?;
    let staging = models_directory.join(format!(
        "{MODEL_DIRECTORY}.download-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&staging).map_err(|error| error.to_string())?;
    let _staging_guard = TemporaryDirectory {
        path: staging.clone(),
    };
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(15 * 60))
        .build()
        .map_err(|error| error.to_string())?;
    let total = total_size();
    let mut completed = 0_u64;

    for (index, artifact) in MODEL_ARTIFACTS.iter().enumerate() {
        let url = format!(
            "https://huggingface.co/{MODEL_ID}/resolve/{MODEL_REVISION}/{}",
            artifact.name
        );
        let mut response = client
            .get(url)
            .header("User-Agent", "XiLuoLin/0.1")
            .send()
            .map_err(|error| format!("下载实时预览模型失败：{error}"))?
            .error_for_status()
            .map_err(|error| format!("下载实时预览模型失败：{error}"))?;
        let path = staging.join(artifact.name);
        let mut file = File::create(&path).map_err(|error| error.to_string())?;
        let mut buffer = [0_u8; 64 * 1024];
        let mut file_downloaded = 0_u64;
        loop {
            let read = response
                .read(&mut buffer)
                .map_err(|error| error.to_string())?;
            if read == 0 {
                break;
            }
            file.write_all(&buffer[..read])
                .map_err(|error| error.to_string())?;
            file_downloaded += read as u64;
            let downloaded = completed.saturating_add(file_downloaded);
            let _ = RealtimeAsrDownloadProgressEvent(RealtimeModelDownloadProgress {
                file_name: artifact.name.to_string(),
                file_index: (index + 1) as u8,
                file_count: MODEL_ARTIFACTS.len() as u8,
                downloaded_bytes: downloaded,
                total_bytes: total,
                percent: ((downloaded.saturating_mul(100) / total.max(1)).min(100)) as u8,
            })
            .emit(app);
        }
        file.sync_all().map_err(|error| error.to_string())?;
        verify_file(&path, artifact.size, artifact.sha256)?;
        completed = completed.saturating_add(artifact.size);
    }
    verify_model_directory(&staging)?;
    write_verified_marker(&staging)?;

    activate_staging_directory(&target, &staging)?;
    Ok(())
}

fn activate_staging_directory(target: &Path, staging: &Path) -> Result<(), String> {
    let models_directory = target.parent().ok_or_else(|| "模型目录无效".to_string())?;
    let backup = models_directory.join(format!("{MODEL_DIRECTORY}.backup"));
    let _ = std::fs::remove_dir_all(&backup);
    if target.exists() {
        std::fs::rename(target, &backup).map_err(|error| error.to_string())?;
    }
    if let Err(error) = std::fs::rename(staging, target) {
        if backup.exists() {
            let _ = std::fs::rename(&backup, target);
        }
        return Err(error.to_string());
    }
    let _ = std::fs::remove_dir_all(backup);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn realtime_asr_model_info(app: tauri::AppHandle) -> Result<RealtimeModelInfo, String> {
    read_info(&app)
}

#[tauri::command]
#[specta::specta]
pub async fn download_realtime_asr_model(
    app: tauri::AppHandle,
) -> Result<RealtimeModelInfo, String> {
    let _download_guard = DownloadGuard::acquire()?;
    if app.state::<CaptureSessionState>().has_active() {
        return Err("语音输入正在进行中，请完成后再下载模型".to_string());
    }
    let app_for_download = app.clone();
    tauri::async_runtime::spawn_blocking(move || download_model(&app_for_download))
        .await
        .map_err(|error| format!("实时模型下载任务失败：{error}"))??;
    read_info(&app)
}

#[tauri::command]
#[specta::specta]
pub async fn verify_realtime_asr_model(app: tauri::AppHandle) -> Result<(), String> {
    if DOWNLOAD_ACTIVE.load(Ordering::Acquire) {
        return Err("实时预览模型正在下载".to_string());
    }
    if app.state::<CaptureSessionState>().has_active() {
        return Err("语音输入正在进行中，请完成后再校验模型".to_string());
    }
    let directory = model_directory(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        verify_model_directory(&directory)?;
        write_verified_marker(&directory)
    })
    .await
    .map_err(|error| format!("实时模型验证任务失败：{error}"))?
}

#[tauri::command]
#[specta::specta]
pub fn set_realtime_preview_enabled(
    app: tauri::AppHandle,
    enabled: bool,
) -> Result<RealtimeModelInfo, String> {
    if DOWNLOAD_ACTIVE.load(Ordering::Acquire) {
        return Err("实时预览模型正在下载".to_string());
    }
    if enabled && !has_verified_install(&model_directory(&app)?) {
        return Err("实时预览模型未安装或尚未通过校验".to_string());
    }
    persist_enabled(&app, enabled)?;
    read_info(&app)
}

#[tauri::command]
#[specta::specta]
pub fn delete_realtime_asr_model(app: tauri::AppHandle) -> Result<RealtimeModelInfo, String> {
    if DOWNLOAD_ACTIVE.load(Ordering::Acquire) {
        return Err("实时预览模型正在下载，请完成后再删除".to_string());
    }
    if app.state::<CaptureSessionState>().has_active() {
        return Err("语音输入正在进行中，请完成后再删除模型".to_string());
    }
    persist_enabled(&app, false)?;
    let directory = model_directory(&app)?;
    if directory.exists() {
        std::fs::remove_dir_all(directory).map_err(|error| error.to_string())?;
    }
    read_info(&app)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::{
        activate_staging_directory, total_size, verify_file, MODEL_ARTIFACTS, MODEL_REVISION,
    };

    #[test]
    fn model_manifest_uses_the_evaluated_mixed_quantization_artifacts() {
        let expected = [
            (
                "encoder-epoch-99-avg-1.int8.onnx",
                181_895_032,
                "8fa764187a261844f859d7143ebaa563af5d10adfece4c18a8f414c88cba2a9b",
            ),
            (
                "decoder-epoch-99-avg-1.onnx",
                13_876_452,
                "2e3b5ec371f8899ee6acd829fd753ba45772df57a91bdf37cde3136354e7db7d",
            ),
            (
                "joiner-epoch-99-avg-1.int8.onnx",
                3_228_404,
                "1ed689c5ed19dbaa725d9d191bb4822b5f4855a39e1ffd28cbc1f340d25b2ee0",
            ),
            (
                "tokens.txt",
                56_317,
                "a8e0e4ec53810e433789b54a5c0134a7eaa2ffca595a6334d54c00da858841d3",
            ),
            (
                "bpe.model",
                244_836,
                "bcae393dbc5611be5ffa4c7ae0841558978a5a4f484008cb9dff3a2cc97ebe01",
            ),
            (
                "bpe.vocab",
                12_564,
                "d0b642f3a2eacd5fadefdeff9e0e1358cab729647cbb7fe58cf738e1f7407029",
            ),
        ];

        assert_eq!(MODEL_REVISION, "98590b7ed6443e77b714204da2757d75e1a642f4");
        assert_eq!(MODEL_ARTIFACTS.len(), expected.len());
        assert_eq!(total_size(), 199_313_605);
        for (artifact, (name, size, sha256)) in MODEL_ARTIFACTS.iter().zip(expected) {
            assert_eq!(artifact.name, name);
            assert_eq!(artifact.size, size);
            assert_eq!(artifact.sha256, sha256);
        }
    }

    #[test]
    fn artifact_verification_rejects_missing_size_and_hash_mismatches() {
        let directory = std::env::temp_dir().join(format!(
            "xiluolin-realtime-model-test-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("artifact.bin");

        assert!(verify_file(
            &path,
            3,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        )
        .is_err());

        std::fs::File::create(&path)
            .unwrap()
            .write_all(b"ab")
            .unwrap();
        assert!(verify_file(
            &path,
            3,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        )
        .is_err());

        std::fs::write(&path, b"abc").unwrap();
        assert!(verify_file(&path, 3, "incorrect").is_err());
        verify_file(
            &path,
            3,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        )
        .unwrap();
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn verified_staging_directory_atomically_replaces_the_previous_install() {
        let directory = std::env::temp_dir().join(format!(
            "xiluolin-realtime-model-atomic-test-{}",
            uuid::Uuid::new_v4()
        ));
        let target = directory.join(super::MODEL_DIRECTORY);
        let staging = directory.join("staging");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::create_dir_all(&staging).unwrap();
        std::fs::write(target.join("version"), "old").unwrap();
        std::fs::write(staging.join("version"), "new").unwrap();

        activate_staging_directory(&target, &staging).unwrap();

        assert_eq!(
            std::fs::read_to_string(target.join("version")).unwrap(),
            "new"
        );
        assert!(!staging.exists());
        assert!(!directory
            .join(format!("{}.backup", super::MODEL_DIRECTORY))
            .exists());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
