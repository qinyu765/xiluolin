use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
};

use sherpa_onnx::{OnlineRecognizer, OnlineRecognizerConfig, OnlineStream};
use tauri::Manager;

use crate::{
    capture_session::{CaptureSessionState, PreviewState},
    realtime_asr_model::{has_verified_install, model_directory},
};

const PREVIEW_QUEUE_CAPACITY: usize = 128;

#[derive(Debug)]
enum PreviewMessage {
    Audio { sample_rate: u32, samples: Vec<f32> },
    Finish,
    Cancel,
}

#[derive(Clone)]
pub(crate) struct PreviewAudioSink {
    sender: mpsc::SyncSender<PreviewMessage>,
    overflowed: Arc<AtomicBool>,
}

impl PreviewAudioSink {
    pub(crate) fn push(&self, sample_rate: u32, samples: Vec<f32>) -> Result<(), ()> {
        match self.sender.try_send(PreviewMessage::Audio {
            sample_rate,
            samples,
        }) {
            Ok(()) => Ok(()),
            Err(mpsc::TrySendError::Full(_)) => {
                self.overflowed.store(true, Ordering::Release);
                Err(())
            }
            Err(mpsc::TrySendError::Disconnected(_)) => Err(()),
        }
    }

    #[cfg(test)]
    fn has_overflowed(&self) -> bool {
        self.overflowed.load(Ordering::Acquire)
    }

    #[cfg(test)]
    fn test_channel(capacity: usize) -> (Self, mpsc::Receiver<PreviewMessage>) {
        let (sender, receiver) = mpsc::sync_channel(capacity);
        (
            Self {
                sender,
                overflowed: Arc::new(AtomicBool::new(false)),
            },
            receiver,
        )
    }
}

pub(crate) struct RealtimePreviewSession {
    sender: mpsc::SyncSender<PreviewMessage>,
    overflowed: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl RealtimePreviewSession {
    pub(crate) fn start_if_enabled(
        app: &tauri::AppHandle,
        session_id: &str,
        hotwords: Vec<String>,
    ) -> Option<(Self, PreviewAudioSink)> {
        let enabled = crate::data::read_app_config(app.clone())
            .map(|config| config.realtime_preview_enabled)
            .unwrap_or(false);
        if !enabled {
            return None;
        }

        let directory = match model_directory(app) {
            Ok(directory) if has_verified_install(&directory) => directory,
            _ => {
                mark_preview_unavailable(app, session_id, "实时预览模型不可用");
                return None;
            }
        };
        let sessions = app.state::<CaptureSessionState>();
        let _ = sessions.update_preview(
            session_id,
            PreviewState::Loading,
            String::new(),
            String::new(),
        );
        let _ = sessions.emit_snapshot(app);

        let (sender, receiver) = mpsc::sync_channel(PREVIEW_QUEUE_CAPACITY);
        let overflowed = Arc::new(AtomicBool::new(false));
        let sink = PreviewAudioSink {
            sender: sender.clone(),
            overflowed: Arc::clone(&overflowed),
        };
        let overflowed_for_thread = Arc::clone(&overflowed);
        let app_for_thread = app.clone();
        let session_for_thread = session_id.to_string();
        let thread = match std::thread::Builder::new()
            .name("xiluolin-realtime-asr".to_string())
            .spawn(move || {
                if let Err(error) = run_preview_worker(
                    &app_for_thread,
                    &session_for_thread,
                    &directory,
                    hotwords,
                    receiver,
                    overflowed_for_thread,
                ) {
                    mark_preview_unavailable(&app_for_thread, &session_for_thread, &error);
                }
            }) {
            Ok(thread) => thread,
            Err(error) => {
                mark_preview_unavailable(app, session_id, &error.to_string());
                return None;
            }
        };
        Some((
            Self {
                sender,
                overflowed,
                thread: Some(thread),
            },
            sink,
        ))
    }

    pub(crate) fn finish(mut self) -> Result<(), String> {
        self.signal(PreviewMessage::Finish);
        self.join()
    }

    pub(crate) fn cancel(mut self) -> Result<(), String> {
        self.signal(PreviewMessage::Cancel);
        self.join()
    }

    fn signal(&self, message: PreviewMessage) {
        if matches!(
            self.sender.try_send(message),
            Err(mpsc::TrySendError::Full(_))
        ) {
            self.overflowed.store(true, Ordering::Release);
        }
    }

    fn join(&mut self) -> Result<(), String> {
        if let Some(thread) = self.thread.take() {
            thread
                .join()
                .map_err(|_| "实时预览线程异常退出".to_string())?;
        }
        Ok(())
    }
}

#[derive(Default)]
struct TranscriptAccumulator {
    stable: String,
    tentative: String,
}

impl TranscriptAccumulator {
    fn update(&mut self, text: String, endpoint: bool) {
        let text = text.trim().to_string();
        if endpoint {
            if !text.is_empty() {
                append_segment(&mut self.stable, &text);
            }
            self.tentative.clear();
        } else {
            self.tentative = text;
        }
    }

    fn stable(&self) -> &str {
        &self.stable
    }

    fn tentative(&self) -> &str {
        &self.tentative
    }

    fn display_text(&self) -> String {
        let mut display = self.stable.clone();
        append_segment(&mut display, &self.tentative);
        display
    }
}

fn append_segment(target: &mut String, segment: &str) {
    if segment.is_empty() {
        return;
    }
    let needs_space = target
        .chars()
        .last()
        .zip(segment.chars().next())
        .is_some_and(|(left, right)| left.is_ascii_alphanumeric() && right.is_ascii_alphanumeric());
    if needs_space {
        target.push(' ');
    }
    target.push_str(segment);
}

fn create_recognizer(directory: &Path, hotwords: &[String]) -> Result<OnlineRecognizer, String> {
    let path = |name: &str| directory.join(name).to_string_lossy().to_string();
    let mut config = OnlineRecognizerConfig::default();
    config.model_config.transducer.encoder = Some(path("encoder-epoch-99-avg-1.int8.onnx"));
    config.model_config.transducer.decoder = Some(path("decoder-epoch-99-avg-1.int8.onnx"));
    config.model_config.transducer.joiner = Some(path("joiner-epoch-99-avg-1.int8.onnx"));
    config.model_config.tokens = Some(path("tokens.txt"));
    config.model_config.bpe_vocab = Some(path("bpe.vocab"));
    config.model_config.model_type = Some("zipformer".to_string());
    config.model_config.modeling_unit = Some("cjkchar+bpe".to_string());
    config.model_config.num_threads = std::thread::available_parallelism()
        .map(|threads| threads.get().min(4) as i32)
        .unwrap_or(2);
    config.decoding_method = Some("modified_beam_search".to_string());
    config.max_active_paths = 4;
    config.enable_endpoint = true;
    config.rule1_min_trailing_silence = 2.4;
    config.rule2_min_trailing_silence = 0.8;
    config.rule3_min_utterance_length = 20.0;
    if !hotwords.is_empty() {
        config.hotwords_buf = Some(hotwords.join("\n").into_bytes());
        config.hotwords_score = 1.5;
    }
    OnlineRecognizer::create(&config).ok_or_else(|| "初始化实时预览模型失败".to_string())
}

fn run_preview_worker(
    app: &tauri::AppHandle,
    session_id: &str,
    directory: &Path,
    hotwords: Vec<String>,
    receiver: mpsc::Receiver<PreviewMessage>,
    overflowed: Arc<AtomicBool>,
) -> Result<(), String> {
    let recognizer = create_recognizer(directory, &hotwords)?;
    let stream = recognizer.create_stream();
    let mut transcript = TranscriptAccumulator::default();
    update_snapshot(app, session_id, &transcript, PreviewState::Active);

    while let Ok(message) = receiver.recv() {
        if overflowed.load(Ordering::Acquire) {
            return Err("实时预览处理过慢，已为本次录音关闭预览".to_string());
        }
        match message {
            PreviewMessage::Audio {
                sample_rate,
                samples,
            } => {
                stream.accept_waveform(sample_rate as i32, &samples);
                decode_available(&recognizer, &stream);
                update_from_recognizer(app, session_id, &recognizer, &stream, &mut transcript);
            }
            PreviewMessage::Finish => {
                stream.input_finished();
                decode_available(&recognizer, &stream);
                if let Some(result) = recognizer.get_result(&stream) {
                    transcript.update(result.text, false);
                    update_snapshot(app, session_id, &transcript, PreviewState::Active);
                }
                break;
            }
            PreviewMessage::Cancel => break,
        }
    }
    Ok(())
}

fn decode_available(recognizer: &OnlineRecognizer, stream: &OnlineStream) {
    while recognizer.is_ready(stream) {
        recognizer.decode(stream);
    }
}

fn update_from_recognizer(
    app: &tauri::AppHandle,
    session_id: &str,
    recognizer: &OnlineRecognizer,
    stream: &OnlineStream,
    transcript: &mut TranscriptAccumulator,
) {
    let Some(result) = recognizer.get_result(stream) else {
        return;
    };
    let endpoint = recognizer.is_endpoint(stream);
    let before = transcript.display_text();
    transcript.update(result.text, endpoint);
    if endpoint {
        recognizer.reset(stream);
    }
    if transcript.display_text() != before {
        update_snapshot(app, session_id, transcript, PreviewState::Active);
    }
}

fn update_snapshot(
    app: &tauri::AppHandle,
    session_id: &str,
    transcript: &TranscriptAccumulator,
    preview_state: PreviewState,
) {
    let sessions = app.state::<CaptureSessionState>();
    if sessions
        .update_preview(
            session_id,
            preview_state,
            transcript.stable().to_string(),
            transcript.tentative().to_string(),
        )
        .is_ok()
    {
        let _ = sessions.emit_snapshot(app);
    }
}

fn mark_preview_unavailable(app: &tauri::AppHandle, session_id: &str, error: &str) {
    let sessions = app.state::<CaptureSessionState>();
    let snapshot = sessions.current_snapshot().ok();
    let stable = snapshot
        .as_ref()
        .map(|snapshot| snapshot.stable_text.clone())
        .unwrap_or_default();
    let tentative = snapshot
        .as_ref()
        .map(|snapshot| snapshot.tentative_text.clone())
        .unwrap_or_default();
    if sessions
        .update_preview(session_id, PreviewState::Unavailable, stable, tentative)
        .is_ok()
    {
        let _ = sessions.emit_snapshot(app);
    }
    eprintln!("[实时预览] session={session_id}, unavailable={error}");
}

#[cfg(test)]
mod tests {
    use super::{PreviewAudioSink, TranscriptAccumulator};

    #[test]
    fn endpoint_commits_tentative_text_without_inserting_spaces_between_chinese_segments() {
        let mut transcript = TranscriptAccumulator::default();

        transcript.update("你好".to_string(), false);
        assert_eq!(transcript.stable(), "");
        assert_eq!(transcript.tentative(), "你好");

        transcript.update("你好".to_string(), true);
        transcript.update("世界".to_string(), false);
        assert_eq!(transcript.stable(), "你好");
        assert_eq!(transcript.tentative(), "世界");
        assert_eq!(transcript.display_text(), "你好世界");
    }

    #[test]
    fn audio_sink_marks_overflow_instead_of_blocking_or_dropping_silently() {
        let (sink, _receiver) = PreviewAudioSink::test_channel(1);

        assert!(sink.push(48_000, vec![0.0; 480]).is_ok());
        assert!(sink.push(48_000, vec![0.0; 480]).is_err());
        assert!(sink.has_overflowed());
    }
}
