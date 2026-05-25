use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{Manager, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingResult {
    pub file_path: String,
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
            Self::MicrophonePermissionDenied => write!(formatter, "麦克风权限缺失，请在系统设置中开启麦克风权限"),
            Self::NoInputDeviceAvailable => write!(formatter, "未找到可用的音频输入设备，请检查麦克风连接"),
            Self::DeviceConfigFailed(message) => write!(formatter, "获取音频设备配置失败：{message}"),
            Self::FileCreationFailed(message) => write!(formatter, "创建录音文件失败：{message}"),
            Self::StreamBuildFailed(message) => write!(formatter, "构建录音流失败：{message}"),
            Self::StreamStartFailed(message) => write!(formatter, "启动录音流失败：{message}"),
            Self::UnsupportedSampleFormat(format) => write!(formatter, "不支持的音频采样格式：{format}"),
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

pub struct RecordingState {
    is_recording: Arc<Mutex<bool>>,
    start_time: Arc<Mutex<Option<Instant>>>,
    output_path: Arc<Mutex<Option<PathBuf>>>,
}

impl RecordingState {
    pub fn new() -> Self {
        RecordingState {
            is_recording: Arc::new(Mutex::new(false)),
            start_time: Arc::new(Mutex::new(None)),
            output_path: Arc::new(Mutex::new(None)),
        }
    }
}

#[tauri::command]
pub async fn start_recording(
    state: State<'_, RecordingState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let mut is_recording = state
        .is_recording
        .lock()
        .map_err(|e| RecordingError::StateLockFailed(e.to_string()))?;

    if *is_recording {
        return Err(RecordingError::AlreadyRecording.into());
    }

    // 创建临时录音文件路径
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| RecordingError::FileCreationFailed(e.to_string()))?;

    let recordings_dir = app_data_dir.join("recordings");
    fs::create_dir_all(&recordings_dir)
        .map_err(|e| RecordingError::FileCreationFailed(e.to_string()))?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let output_path = recordings_dir.join(format!("recording_{}.wav", timestamp));

    // 获取默认音频输入设备
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| RecordingError::NoInputDeviceAvailable)?;

    // 获取设备支持的配置
    let config = device
        .default_input_config()
        .map_err(|e| {
            // 检查是否是权限问题
            let error_msg = e.to_string().to_lowercase();
            if error_msg.contains("permission") || error_msg.contains("access") {
                RecordingError::MicrophonePermissionDenied
            } else {
                RecordingError::DeviceConfigFailed(e.to_string())
            }
        })?;

    let sample_rate = config.sample_rate().0;
    let channels = config.channels();

    // 创建 WAV 文件写入器
    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let writer = WavWriter::create(&output_path, spec)
        .map_err(|e| RecordingError::FileCreationFailed(e.to_string()))?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    // 构建音频流
    let writer_clone = Arc::clone(&writer);
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Ok(mut writer_guard) = writer_clone.lock() {
                    if let Some(writer) = writer_guard.as_mut() {
                        for &sample in data {
                            let amplitude = (sample * i16::MAX as f32) as i16;
                            let _ = writer.write_sample(amplitude);
                        }
                    }
                }
            },
            move |err| {
                eprintln!("录音流错误: {}", err);
            },
            None,
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                if let Ok(mut writer_guard) = writer_clone.lock() {
                    if let Some(writer) = writer_guard.as_mut() {
                        for &sample in data {
                            let _ = writer.write_sample(sample);
                        }
                    }
                }
            },
            move |err| {
                eprintln!("录音流错误: {}", err);
            },
            None,
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config.into(),
            move |data: &[u16], _: &cpal::InputCallbackInfo| {
                if let Ok(mut writer_guard) = writer_clone.lock() {
                    if let Some(writer) = writer_guard.as_mut() {
                        for &sample in data {
                            let amplitude = (sample as i32 - 32768) as i16;
                            let _ = writer.write_sample(amplitude);
                        }
                    }
                }
            },
            move |err| {
                eprintln!("录音流错误: {}", err);
            },
            None,
        ),
        _ => {
            return Err(RecordingError::UnsupportedSampleFormat(format!("{:?}", config.sample_format())).into())
        }
    }
    .map_err(|e| RecordingError::StreamBuildFailed(e.to_string()))?;

    // 启动录音流
    stream
        .play()
        .map_err(|e| RecordingError::StreamStartFailed(e.to_string()))?;

    // 保存状态
    *is_recording = true;
    *state
        .start_time
        .lock()
        .map_err(|e| RecordingError::StateLockFailed(e.to_string()))? = Some(Instant::now());
    *state
        .output_path
        .lock()
        .map_err(|e| RecordingError::StateLockFailed(e.to_string()))? = Some(output_path.clone());

    // 将 stream 泄漏以保持录音持续进行
    // 这是一个临时方案，后续需要改进为使用全局状态管理 stream
    std::mem::forget(stream);
    std::mem::forget(writer);

    Ok("recording_started".to_string())
}

#[tauri::command]
pub async fn stop_recording(
    state: State<'_, RecordingState>,
) -> Result<RecordingResult, String> {
    // 检查录音状态并获取必要信息
    let (duration_ms, output_path) = {
        let mut is_recording = state
            .is_recording
            .lock()
            .map_err(|e| RecordingError::StateLockFailed(e.to_string()))?;

        if !*is_recording {
            return Err(RecordingError::NoRecordingInProgress.into());
        }

        // 计算录音时长
        let start_time = state
            .start_time
            .lock()
            .map_err(|e| RecordingError::StateLockFailed(e.to_string()))?
            .ok_or_else(|| RecordingError::StateLockFailed("开始时间未找到".to_string()))?;
        let duration_ms = start_time.elapsed().as_millis() as u64;

        // 获取输出路径
        let output_path = state
            .output_path
            .lock()
            .map_err(|e| RecordingError::StateLockFailed(e.to_string()))?
            .clone()
            .ok_or_else(|| RecordingError::StateLockFailed("输出路径未找到".to_string()))?;

        // 重置状态
        *is_recording = false;
        *state
            .start_time
            .lock()
            .map_err(|e| RecordingError::StateLockFailed(e.to_string()))? = None;
        *state
            .output_path
            .lock()
            .map_err(|e| RecordingError::StateLockFailed(e.to_string()))? = None;

        (duration_ms, output_path)
    }; // MutexGuard 在这里被释放

    // 等待一小段时间确保所有音频数据已写入
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    Ok(RecordingResult {
        file_path: output_path.to_string_lossy().to_string(),
        duration_ms,
    })
}

#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();

    let default_device_name = host
        .default_input_device()
        .and_then(|d| d.name().ok());

    let devices = host
        .input_devices()
        .map_err(|e| format!("获取音频设备列表失败: {}", e))?;

    let mut result = Vec::new();
    for device in devices {
        if let Ok(name) = device.name() {
            let is_default = default_device_name.as_ref() == Some(&name);
            result.push(AudioDevice {
                name,
                is_default,
            });
        }
    }

    Ok(result)
}
