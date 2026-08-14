use std::future::Future;

use tauri::Manager;
use tauri_specta::Event;

use crate::{
    capture_session::{
        CaptureFailure, CapturePhase, CaptureSessionStart, CaptureSessionState, CaptureSnapshot,
        CaptureSource,
    },
    events::HistoryChangedEvent,
    output::FallbackResultState,
    pipeline::VoiceInputResult,
    recording::{self, RecordingResult, RecordingState},
};

async fn run_pipeline_steps<T, R, Process, SelectText, Deliver, DeliveryFuture>(
    process: Process,
    select_text: SelectText,
    deliver: Deliver,
) -> Result<(T, R), String>
where
    Process: FnOnce() -> Result<T, String>,
    SelectText: FnOnce(&T) -> String,
    Deliver: FnOnce(String) -> DeliveryFuture,
    DeliveryFuture: Future<Output = Result<R, String>>,
{
    let processed = process()?;
    let text = select_text(&processed);
    let delivered = deliver(text).await?;
    Ok((processed, delivered))
}

#[tauri::command]
#[specta::specta]
pub async fn start_capture(
    state: tauri::State<'_, RecordingState>,
    app: tauri::AppHandle,
) -> Result<CaptureSessionStart, String> {
    start_capture_for_source(&state, &app, CaptureSource::App)
}

pub fn start_capture_for_source(
    state: &RecordingState,
    app: &tauri::AppHandle,
    source: CaptureSource,
) -> Result<CaptureSessionStart, String> {
    let started = recording::start_recording_for_source(state, app, source)?;
    if source == CaptureSource::Hotkey {
        if let Err(error) = crate::indicator::show_indicator(app) {
            eprintln!("录音悬浮窗显示失败，录音继续：{error}");
        }
    }
    Ok(started)
}

#[tauri::command]
#[specta::specta]
pub async fn stop_capture(
    state: tauri::State<'_, RecordingState>,
    app: tauri::AppHandle,
) -> Result<CaptureSnapshot, String> {
    let recording = recording::stop_recording_for_session(&state, &app).await?;
    spawn_recording_pipeline(app.clone(), recording);
    app.state::<CaptureSessionState>().current_snapshot()
}

pub async fn stop_capture_for_session(
    state: &RecordingState,
    app: &tauri::AppHandle,
) -> Result<RecordingResult, String> {
    let recording = recording::stop_recording_for_session(state, app).await?;
    spawn_recording_pipeline(app.clone(), recording.clone());
    Ok(recording)
}

pub fn spawn_recording_pipeline(app: tauri::AppHandle, recording: RecordingResult) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = process_recording(app.clone(), recording.clone()).await {
            let sessions = app.state::<CaptureSessionState>();
            if sessions.has_active() {
                let snapshot = sessions.current_snapshot().ok();
                let stage = snapshot
                    .as_ref()
                    .map(|snapshot| snapshot.phase)
                    .unwrap_or(CapturePhase::Transcribing);
                let _ = sessions.fail(
                    &recording.session_id,
                    CaptureFailure {
                        code: "capture_pipeline_failed".to_string(),
                        stage,
                        recoverable: false,
                        detail: error.clone(),
                    },
                );
                let _ = sessions.emit_snapshot(&app);
            }
            let _ = crate::indicator::finish_indicator(&app, "failed");
            eprintln!("语音输入后台处理失败：{error}");
        }
    });
}

async fn process_recording(
    app: tauri::AppHandle,
    recording: RecordingResult,
) -> Result<(), String> {
    let process_app = app.clone();
    let session_id = recording.session_id.clone();
    let process_session_id = session_id.clone();
    let duration_ms = u32::try_from(recording.duration_ms).unwrap_or(u32::MAX);
    let file_path = recording.file_path.clone();
    let processed = tauri::async_runtime::spawn_blocking(move || {
        crate::pipeline::process_recording_file(
            process_app,
            process_session_id,
            file_path,
            duration_ms,
        )
    })
    .await
    .map_err(|error| format!("录音处理线程失败：{error}"))?;

    let processed = processed?;
    let history_id = processed
        .history_record
        .as_ref()
        .map(|history| history.id.clone());
    if let Err(error) = (HistoryChangedEvent {
        history_id: history_id.clone(),
    })
    .emit(&app)
    {
        eprintln!("发送历史更新事件失败：{error}");
    }

    let sessions = app.state::<CaptureSessionState>();
    let fallback = app.state::<FallbackResultState>();
    let (processed, _) = run_pipeline_steps(
        || Ok(processed),
        |result: &VoiceInputResult| result.final_text.clone(),
        |text| {
            crate::output::deliver_text_internal(
                &app,
                &sessions,
                &fallback,
                Some(session_id),
                None,
                text,
            )
        },
    )
    .await?;
    let _ = processed;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::run_pipeline_steps;

    #[tokio::test]
    async fn successful_processing_delivers_exactly_once_without_a_webview_listener() {
        let delivered = Rc::new(RefCell::new(Vec::new()));
        let delivered_for_step = Rc::clone(&delivered);

        let result = run_pipeline_steps(
            || Ok::<_, String>("最终文本".to_string()),
            |text| text.clone(),
            move |text| {
                delivered_for_step.borrow_mut().push(text);
                async { Ok::<_, String>("paste") }
            },
        )
        .await
        .unwrap();

        assert_eq!(result.0, "最终文本");
        assert_eq!(result.1, "paste");
        assert_eq!(delivered.borrow().as_slice(), ["最终文本"]);
    }

    #[tokio::test]
    async fn failed_processing_never_attempts_delivery() {
        let delivery_attempted = Rc::new(RefCell::new(false));
        let attempted_for_step = Rc::clone(&delivery_attempted);

        let result = run_pipeline_steps(
            || Err::<String, _>("asr failed".to_string()),
            |text| text.clone(),
            move |_text| {
                *attempted_for_step.borrow_mut() = true;
                async { Ok::<_, String>(()) }
            },
        )
        .await;

        assert_eq!(result.unwrap_err(), "asr failed");
        assert!(!*delivery_attempted.borrow());
    }
}
