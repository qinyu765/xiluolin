use std::fs;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};

use crate::recording::RecordingError;

type SharedWavWriter = Arc<Mutex<Option<WavWriter<BufWriter<std::fs::File>>>>>;

enum AudioWorkerCommand {
    Stop(mpsc::Sender<Result<(), String>>),
    Cancel(mpsc::Sender<()>),
}

pub(super) struct AudioWorker {
    command_sender: mpsc::Sender<AudioWorkerCommand>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl AudioWorker {
    pub(super) fn start(
        output_path: PathBuf,
        preferred_microphone: String,
    ) -> Result<Self, RecordingError> {
        let (command_sender, command_receiver) = mpsc::channel();
        let (ready_sender, ready_receiver) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("xiluolin-audio-capture".to_string())
            .spawn(move || {
                run_audio_worker(
                    output_path,
                    preferred_microphone,
                    command_receiver,
                    ready_sender,
                );
            })
            .map_err(|error| RecordingError::StreamBuildFailed(error.to_string()))?;

        match ready_receiver.recv() {
            Ok(Ok(())) => Ok(Self {
                command_sender,
                thread: Some(thread),
            }),
            Ok(Err(error)) => {
                let _ = thread.join();
                Err(error)
            }
            Err(error) => {
                let _ = thread.join();
                Err(RecordingError::StreamStartFailed(error.to_string()))
            }
        }
    }

    pub(super) fn stop(mut self) -> Result<(), String> {
        let (result_sender, result_receiver) = mpsc::channel();
        self.command_sender
            .send(AudioWorkerCommand::Stop(result_sender))
            .map_err(|error| format!("停止录音工作线程失败：{error}"))?;
        let result = result_receiver
            .recv()
            .map_err(|error| format!("等待录音封装失败：{error}"))?;
        self.join()?;
        result
    }

    pub(super) fn cancel(mut self) -> Result<(), String> {
        let (done_sender, done_receiver) = mpsc::channel();
        self.command_sender
            .send(AudioWorkerCommand::Cancel(done_sender))
            .map_err(|error| format!("取消录音工作线程失败：{error}"))?;
        done_receiver
            .recv()
            .map_err(|error| format!("等待录音取消失败：{error}"))?;
        self.join()
    }

    fn join(&mut self) -> Result<(), String> {
        if let Some(thread) = self.thread.take() {
            thread
                .join()
                .map_err(|_| "录音工作线程异常退出".to_string())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeviceSelection {
    name: Option<String>,
    fell_back: bool,
}

fn select_input_device_name(
    preferred: &str,
    available: &[&str],
    default: Option<&str>,
) -> DeviceSelection {
    let preferred = preferred.trim();
    if !preferred.is_empty() && available.contains(&preferred) {
        return DeviceSelection {
            name: Some(preferred.to_string()),
            fell_back: false,
        };
    }
    DeviceSelection {
        name: default
            .or_else(|| available.first().copied())
            .map(str::to_string),
        fell_back: !preferred.is_empty(),
    }
}

fn select_input_device(
    host: &cpal::Host,
    preferred: &str,
) -> Result<(cpal::Device, bool), RecordingError> {
    let default_device = host.default_input_device();
    let default_name = default_device
        .as_ref()
        .and_then(|device| device.name().ok());
    let devices = host
        .input_devices()
        .map_err(|error| RecordingError::DeviceConfigFailed(error.to_string()))?
        .filter_map(|device| device.name().ok().map(|name| (name, device)))
        .collect::<Vec<_>>();
    let names = devices
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    let selection = select_input_device_name(preferred, &names, default_name.as_deref());

    if let Some(selected_name) = selection.name.as_deref() {
        if let Some((_, device)) = devices.into_iter().find(|(name, _)| name == selected_name) {
            return Ok((device, selection.fell_back));
        }
    }
    default_device
        .map(|device| (device, selection.fell_back))
        .ok_or(RecordingError::NoInputDeviceAvailable)
}

fn downmix_to_i16<T: Copy>(data: &[T], channels: usize, normalize: impl Fn(T) -> f64) -> Vec<i16> {
    if channels == 0 {
        return Vec::new();
    }
    data.chunks_exact(channels)
        .map(|frame| {
            let average = frame.iter().copied().map(&normalize).sum::<f64>() / channels as f64;
            let average = average.clamp(-1.0, 1.0);
            if average <= -1.0 {
                i16::MIN
            } else if average >= 1.0 {
                i16::MAX
            } else {
                (average * i16::MAX as f64).round() as i16
            }
        })
        .collect()
}

fn downmix_f32_to_i16(data: &[f32], channels: usize) -> Vec<i16> {
    downmix_to_i16(data, channels, |sample| sample as f64)
}

fn downmix_i16_to_i16(data: &[i16], channels: usize) -> Vec<i16> {
    downmix_to_i16(data, channels, |sample| {
        if sample < 0 {
            sample as f64 / -(i16::MIN as f64)
        } else {
            sample as f64 / i16::MAX as f64
        }
    })
}

fn downmix_u16_to_i16(data: &[u16], channels: usize) -> Vec<i16> {
    downmix_to_i16(data, channels, |sample| {
        let centered = sample as f64 - 32768.0;
        if centered < 0.0 {
            centered / 32768.0
        } else {
            centered / 32767.0
        }
    })
}

fn write_samples(writer: &SharedWavWriter, samples: Vec<i16>) {
    if let Ok(mut writer_guard) = writer.lock() {
        if let Some(writer) = writer_guard.as_mut() {
            for sample in samples {
                if writer.write_sample(sample).is_err() {
                    eprintln!("录音数据写入失败");
                    break;
                }
            }
        }
    }
}

fn finalize_writer(writer: &SharedWavWriter) -> Result<(), RecordingError> {
    let mut writer_guard = writer
        .lock()
        .map_err(|error| RecordingError::StateLockFailed(error.to_string()))?;
    if let Some(writer) = writer_guard.take() {
        writer
            .finalize()
            .map_err(|error| RecordingError::FileCreationFailed(error.to_string()))?;
    }
    Ok(())
}

fn discard_recording_file(writer: &SharedWavWriter, output_path: &Path) {
    let _ = finalize_writer(writer);
    let _ = fs::remove_file(output_path);
}

fn finish_audio_resources<T>(stop_stream: impl FnOnce(), finalize: impl FnOnce() -> T) -> T {
    stop_stream();
    finalize()
}

fn run_audio_worker(
    output_path: PathBuf,
    preferred_microphone: String,
    command_receiver: mpsc::Receiver<AudioWorkerCommand>,
    ready_sender: mpsc::Sender<Result<(), RecordingError>>,
) {
    let setup = (|| -> Result<(cpal::Stream, SharedWavWriter), RecordingError> {
        let host = cpal::default_host();
        let (device, fell_back) = select_input_device(&host, &preferred_microphone)?;
        if fell_back {
            eprintln!("已配置的麦克风不可用，已回退到系统默认输入设备");
        }
        let config = device.default_input_config().map_err(|error| {
            let message = error.to_string();
            let normalized = message.to_lowercase();
            if normalized.contains("permission") || normalized.contains("access") {
                RecordingError::MicrophonePermissionDenied
            } else {
                RecordingError::DeviceConfigFailed(message)
            }
        })?;
        let channels = config.channels() as usize;
        let writer = WavWriter::create(
            &output_path,
            WavSpec {
                channels: 1,
                sample_rate: config.sample_rate().0,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
        )
        .map_err(|error| RecordingError::FileCreationFailed(error.to_string()))?;
        let writer = Arc::new(Mutex::new(Some(writer)));
        let writer_clone = Arc::clone(&writer);
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.clone().into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    write_samples(&writer_clone, downmix_f32_to_i16(data, channels));
                },
                |error| eprintln!("录音流错误：{error}"),
                None,
            ),
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.clone().into(),
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    write_samples(&writer_clone, downmix_i16_to_i16(data, channels));
                },
                |error| eprintln!("录音流错误：{error}"),
                None,
            ),
            cpal::SampleFormat::U16 => device.build_input_stream(
                &config.clone().into(),
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    write_samples(&writer_clone, downmix_u16_to_i16(data, channels));
                },
                |error| eprintln!("录音流错误：{error}"),
                None,
            ),
            format => {
                discard_recording_file(&writer, &output_path);
                return Err(RecordingError::UnsupportedSampleFormat(format!(
                    "{format:?}"
                )));
            }
        }
        .map_err(|error| {
            discard_recording_file(&writer, &output_path);
            RecordingError::StreamBuildFailed(error.to_string())
        })?;
        if let Err(error) = stream.play() {
            drop(stream);
            discard_recording_file(&writer, &output_path);
            return Err(RecordingError::StreamStartFailed(error.to_string()));
        }
        Ok((stream, writer))
    })();

    let (stream, writer) = match setup {
        Ok(resources) => resources,
        Err(error) => {
            let _ = ready_sender.send(Err(error));
            return;
        }
    };
    if ready_sender.send(Ok(())).is_err() {
        drop(stream);
        discard_recording_file(&writer, &output_path);
        return;
    }

    match command_receiver.recv() {
        Ok(AudioWorkerCommand::Stop(result_sender)) => {
            let result = finish_audio_resources(
                || drop(stream),
                || finalize_writer(&writer).map_err(String::from),
            );
            if result.is_err() {
                let _ = fs::remove_file(&output_path);
            }
            let _ = result_sender.send(result);
        }
        Ok(AudioWorkerCommand::Cancel(done_sender)) => {
            finish_audio_resources(
                || drop(stream),
                || discard_recording_file(&writer, &output_path),
            );
            let _ = done_sender.send(());
        }
        Err(_) => {
            drop(stream);
            discard_recording_file(&writer, &output_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_microphone_is_used_and_missing_device_falls_back_to_default() {
        let selected = select_input_device_name(
            "USB Microphone",
            &["MacBook Microphone", "USB Microphone"],
            Some("MacBook Microphone"),
        );
        assert_eq!(selected.name.as_deref(), Some("USB Microphone"));
        assert!(!selected.fell_back);

        let fallback = select_input_device_name(
            "Disconnected Microphone",
            &["MacBook Microphone", "USB Microphone"],
            Some("MacBook Microphone"),
        );
        assert_eq!(fallback.name.as_deref(), Some("MacBook Microphone"));
        assert!(fallback.fell_back);

        let default =
            select_input_device_name("", &["MacBook Microphone"], Some("MacBook Microphone"));
        assert_eq!(default.name.as_deref(), Some("MacBook Microphone"));
        assert!(!default.fell_back);

        let first_available =
            select_input_device_name("Disconnected Microphone", &["Only Microphone"], None);
        assert_eq!(first_available.name.as_deref(), Some("Only Microphone"));
        assert!(first_available.fell_back);
    }

    #[test]
    fn all_supported_sample_formats_are_averaged_to_mono_and_clamped() {
        assert_eq!(
            downmix_f32_to_i16(&[1.0, -1.0, 0.5, 0.5], 2),
            vec![0, 16384]
        );
        assert_eq!(
            downmix_i16_to_i16(&[i16::MAX, i16::MAX, i16::MIN, i16::MIN], 2),
            vec![i16::MAX, i16::MIN]
        );
        assert_eq!(
            downmix_u16_to_i16(&[u16::MAX, u16::MAX, 0, 0], 2),
            vec![i16::MAX, i16::MIN]
        );
        assert_eq!(
            downmix_f32_to_i16(&[2.0, 2.0, -2.0, -2.0], 2),
            vec![i16::MAX, i16::MIN]
        );
    }

    #[test]
    fn incomplete_multichannel_frames_are_not_written() {
        assert_eq!(downmix_i16_to_i16(&[100, 300, 999], 2), vec![200]);
        assert!(downmix_f32_to_i16(&[0.5], 2).is_empty());
    }

    #[test]
    fn stream_is_stopped_before_wav_is_finalized() {
        let operations = std::cell::RefCell::new(Vec::new());
        finish_audio_resources(
            || operations.borrow_mut().push("stop-stream"),
            || operations.borrow_mut().push("finalize-wav"),
        );
        assert_eq!(operations.into_inner(), vec!["stop-stream", "finalize-wav"]);
    }

    #[test]
    fn cancel_path_stops_resources_and_deletes_the_temporary_wav() {
        let path = std::env::temp_dir().join(format!(
            "xiluolin-cancel-recording-{}.wav",
            uuid::Uuid::new_v4()
        ));
        let writer = WavWriter::create(
            &path,
            WavSpec {
                channels: 1,
                sample_rate: 48_000,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
        )
        .unwrap();
        let writer = Arc::new(Mutex::new(Some(writer)));

        finish_audio_resources(|| {}, || discard_recording_file(&writer, &path));

        assert!(!path.exists());
    }
}
