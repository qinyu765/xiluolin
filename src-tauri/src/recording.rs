use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{Manager, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingResult {
    pub file_path: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecordingError {
    pub message: String,
}

impl From<String> for RecordingError {
    fn from(message: String) -> Self {
        RecordingError { message }
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
) -> Result<String, RecordingError> {
    let mut is_recording = state
        .is_recording
        .lock()
        .map_err(|e| format!("Failed to lock recording state: {}", e))?;

    if *is_recording {
        return Err("Recording is already in progress".to_string().into());
    }

    // 创建临时录音文件路径
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let recordings_dir = app_data_dir.join("recordings");
    fs::create_dir_all(&recordings_dir)
        .map_err(|e| format!("Failed to create recordings directory: {}", e))?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let output_path = recordings_dir.join(format!("recording_{}.wav", timestamp));

    // 获取默认音频输入设备
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "No input device available".to_string())?;

    // 获取设备支持的配置
    let config = device
        .default_input_config()
        .map_err(|e| format!("Failed to get default input config: {}", e))?;

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
        .map_err(|e| format!("Failed to create WAV writer: {}", e))?;
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
                eprintln!("Recording stream error: {}", err);
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
                eprintln!("Recording stream error: {}", err);
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
                eprintln!("Recording stream error: {}", err);
            },
            None,
        ),
        _ => {
            return Err(format!("Unsupported sample format: {:?}", config.sample_format()).into())
        }
    }
    .map_err(|e| format!("Failed to build input stream: {}", e))?;

    // 启动录音流
    stream
        .play()
        .map_err(|e| format!("Failed to start recording stream: {}", e))?;

    // 保存状态
    *is_recording = true;
    *state
        .start_time
        .lock()
        .map_err(|e| format!("Failed to lock start time: {}", e))? = Some(Instant::now());
    *state
        .output_path
        .lock()
        .map_err(|e| format!("Failed to lock output path: {}", e))? = Some(output_path.clone());

    // 将 stream 泄漏以保持录音持续进行
    // 这是一个临时方案，后续需要改进为使用全局状态管理 stream
    std::mem::forget(stream);
    std::mem::forget(writer);

    Ok("recording_started".to_string())
}

#[tauri::command]
pub async fn stop_recording(
    state: State<'_, RecordingState>,
) -> Result<RecordingResult, RecordingError> {
    // 检查录音状态并获取必要信息
    let (duration_ms, output_path) = {
        let mut is_recording = state
            .is_recording
            .lock()
            .map_err(|e| format!("Failed to lock recording state: {}", e))?;

        if !*is_recording {
            return Err("No recording in progress".to_string().into());
        }

        // 计算录音时长
        let start_time = state
            .start_time
            .lock()
            .map_err(|e| format!("Failed to lock start time: {}", e))?
            .ok_or_else(|| "Start time not found".to_string())?;
        let duration_ms = start_time.elapsed().as_millis() as u64;

        // 获取输出路径
        let output_path = state
            .output_path
            .lock()
            .map_err(|e| format!("Failed to lock output path: {}", e))?
            .clone()
            .ok_or_else(|| "Output path not found".to_string())?;

        // 重置状态
        *is_recording = false;
        *state
            .start_time
            .lock()
            .map_err(|e| format!("Failed to lock start time: {}", e))? = None;
        *state
            .output_path
            .lock()
            .map_err(|e| format!("Failed to lock output path: {}", e))? = None;

        (duration_ms, output_path)
    }; // MutexGuard 在这里被释放

    // 等待一小段时间确保所有音频数据已写入
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    Ok(RecordingResult {
        file_path: output_path.to_string_lossy().to_string(),
        duration_ms,
    })
}
