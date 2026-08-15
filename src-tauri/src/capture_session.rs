use std::{sync::Mutex, time::Instant};

use serde::{Deserialize, Serialize};
use tauri_specta::Event;
use uuid::Uuid;

use crate::data::{AppConfig, Persona};
use crate::focus_capture::FocusSnapshot;

#[derive(Debug, Clone)]
pub struct CapturedSessionContext {
    pub config: AppConfig,
    pub persona: Persona,
    pub asr_hotwords: Vec<String>,
    pub hotword_context: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum CaptureSource {
    Hotkey,
    App,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum CaptureStatus {
    Recording,
    Transcribing,
    Refining,
    Delivering,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum CapturePhase {
    Idle,
    Recording,
    Transcribing,
    Refining,
    Delivering,
    Completed,
    Failed,
}

impl From<CaptureStatus> for CapturePhase {
    fn from(status: CaptureStatus) -> Self {
        match status {
            CaptureStatus::Recording => Self::Recording,
            CaptureStatus::Transcribing => Self::Transcribing,
            CaptureStatus::Refining => Self::Refining,
            CaptureStatus::Delivering => Self::Delivering,
            CaptureStatus::Completed => Self::Completed,
            CaptureStatus::Failed => Self::Failed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum PreviewState {
    Disabled,
    Loading,
    Active,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct CaptureFailure {
    pub code: String,
    pub stage: CapturePhase,
    pub recoverable: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct CaptureSnapshot {
    pub session_id: Option<String>,
    #[specta(type = specta_typescript::Number)]
    pub revision: u64,
    pub source: Option<CaptureSource>,
    pub phase: CapturePhase,
    #[specta(type = specta_typescript::Number)]
    pub elapsed_ms: u64,
    pub stable_text: String,
    pub tentative_text: String,
    pub preview_state: PreviewState,
    pub history_id: Option<String>,
    pub failure: Option<CaptureFailure>,
}

impl CaptureSnapshot {
    fn idle() -> Self {
        Self {
            session_id: None,
            revision: 0,
            source: None,
            phase: CapturePhase::Idle,
            elapsed_ms: 0,
            stable_text: String::new(),
            tentative_text: String::new(),
            preview_state: PreviewState::Disabled,
            history_id: None,
            failure: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
pub struct CaptureSessionStart {
    pub session_id: String,
}

#[derive(Debug, Clone)]
pub struct DeliveryContext {
    pub source: CaptureSource,
    pub focus: Option<FocusSnapshot>,
    pub history_id: Option<String>,
}

#[derive(Debug, Clone)]
struct CaptureSession {
    id: String,
    source: CaptureSource,
    status: CaptureStatus,
    focus: Option<FocusSnapshot>,
    history_id: Option<String>,
    started_at: Instant,
    context: Option<CapturedSessionContext>,
}

pub struct CaptureSessionState {
    current: Mutex<Option<CaptureSession>>,
    snapshot: Mutex<CaptureSnapshot>,
}

impl Default for CaptureSessionState {
    fn default() -> Self {
        Self::new()
    }
}

impl CaptureSessionState {
    pub fn new() -> Self {
        Self {
            current: Mutex::new(None),
            snapshot: Mutex::new(CaptureSnapshot::idle()),
        }
    }

    pub fn begin_with_captured_context(
        &self,
        source: CaptureSource,
        focus: Option<FocusSnapshot>,
        context: CapturedSessionContext,
    ) -> Result<CaptureSessionStart, String> {
        self.begin_with_focus_and_context(source, focus, Some(context))
    }

    #[cfg(test)]
    fn begin_with_focus(
        &self,
        source: CaptureSource,
        focus: Option<FocusSnapshot>,
    ) -> Result<CaptureSessionStart, String> {
        self.begin_with_focus_and_context(source, focus, None)
    }

    fn begin_with_focus_and_context(
        &self,
        source: CaptureSource,
        focus: Option<FocusSnapshot>,
        context: Option<CapturedSessionContext>,
    ) -> Result<CaptureSessionStart, String> {
        let mut current = self
            .current
            .lock()
            .map_err(|error| format!("CaptureSession 状态锁定失败：{error}"))?;
        if current.is_some() {
            return Err("上一条语音输入仍在处理中".to_string());
        }

        let session_id = Uuid::new_v4().to_string();
        *current = Some(CaptureSession {
            id: session_id.clone(),
            source,
            status: CaptureStatus::Recording,
            focus,
            history_id: None,
            started_at: Instant::now(),
            context,
        });
        self.replace_snapshot(|snapshot| {
            let revision = snapshot.revision.saturating_add(1);
            *snapshot = CaptureSnapshot {
                session_id: Some(session_id.clone()),
                revision,
                source: Some(source),
                phase: CapturePhase::Recording,
                elapsed_ms: 0,
                stable_text: String::new(),
                tentative_text: String::new(),
                preview_state: PreviewState::Disabled,
                history_id: None,
                failure: None,
            };
        })?;
        Ok(CaptureSessionStart { session_id })
    }

    pub fn update_status(&self, session_id: &str, next: CaptureStatus) -> Result<(), String> {
        let mut current = self
            .current
            .lock()
            .map_err(|error| format!("CaptureSession 状态锁定失败：{error}"))?;
        let session = current
            .as_mut()
            .filter(|session| session.id == session_id)
            .ok_or_else(|| "CaptureSession 不存在或已经结束".to_string())?;

        if session.status == next {
            return Ok(());
        }
        if !is_valid_transition(session.status, next) {
            return Err(format!(
                "CaptureSession 状态不能从 {:?} 切换到 {:?}",
                session.status, next
            ));
        }
        session.status = next;
        let elapsed_ms = session.started_at.elapsed().as_millis() as u64;
        self.replace_snapshot(|snapshot| {
            snapshot.revision = snapshot.revision.saturating_add(1);
            snapshot.phase = next.into();
            snapshot.elapsed_ms = elapsed_ms;
        })?;
        Ok(())
    }

    pub fn update_preview(
        &self,
        session_id: &str,
        preview_state: PreviewState,
        stable_text: String,
        tentative_text: String,
    ) -> Result<(), String> {
        let current = self
            .current
            .lock()
            .map_err(|error| format!("CaptureSession 状态锁定失败：{error}"))?;
        let session = current
            .as_ref()
            .filter(|session| session.id == session_id)
            .ok_or_else(|| "CaptureSession 不存在或已经结束".to_string())?;
        let elapsed_ms = session.started_at.elapsed().as_millis() as u64;
        self.replace_snapshot(|snapshot| {
            snapshot.revision = snapshot.revision.saturating_add(1);
            snapshot.elapsed_ms = elapsed_ms;
            snapshot.preview_state = preview_state;
            snapshot.stable_text = stable_text;
            snapshot.tentative_text = tentative_text;
        })
    }

    pub fn attach_history(&self, session_id: &str, history_id: String) -> Result<(), String> {
        let mut current = self
            .current
            .lock()
            .map_err(|error| format!("CaptureSession 状态锁定失败：{error}"))?;
        let session = current
            .as_mut()
            .filter(|session| session.id == session_id)
            .ok_or_else(|| "CaptureSession 不存在或已经结束".to_string())?;
        session.history_id = Some(history_id);
        let history_id = session.history_id.clone();
        self.replace_snapshot(|snapshot| {
            snapshot.revision = snapshot.revision.saturating_add(1);
            snapshot.history_id = history_id;
        })?;
        Ok(())
    }

    pub fn delivery_context(&self, session_id: &str) -> Result<DeliveryContext, String> {
        let current = self
            .current
            .lock()
            .map_err(|error| format!("CaptureSession 状态锁定失败：{error}"))?;
        let session = current
            .as_ref()
            .filter(|session| session.id == session_id)
            .ok_or_else(|| "CaptureSession 不存在或已经结束".to_string())?;
        Ok(DeliveryContext {
            source: session.source,
            focus: session.focus.clone(),
            history_id: session.history_id.clone(),
        })
    }

    pub fn processing_context(&self, session_id: &str) -> Result<CapturedSessionContext, String> {
        let current = self
            .current
            .lock()
            .map_err(|error| format!("CaptureSession 状态锁定失败：{error}"))?;
        current
            .as_ref()
            .filter(|session| session.id == session_id)
            .and_then(|session| session.context.clone())
            .ok_or_else(|| "CaptureSession 缺少固定处理配置".to_string())
    }

    pub fn finish(&self, session_id: &str, status: CaptureStatus) -> Result<(), String> {
        if !matches!(status, CaptureStatus::Completed | CaptureStatus::Failed) {
            return Err("结束状态必须是 completed 或 failed".to_string());
        }

        let mut current = self
            .current
            .lock()
            .map_err(|error| format!("CaptureSession 状态锁定失败：{error}"))?;
        let session = current
            .as_ref()
            .filter(|session| session.id == session_id)
            .ok_or_else(|| "CaptureSession 不存在或已经结束".to_string())?;
        if !is_valid_transition(session.status, status) {
            return Err(format!(
                "CaptureSession 状态不能从 {:?} 切换到 {:?}",
                session.status, status
            ));
        }
        let elapsed_ms = session.started_at.elapsed().as_millis() as u64;
        self.replace_snapshot(|snapshot| {
            snapshot.revision = snapshot.revision.saturating_add(1);
            snapshot.phase = status.into();
            snapshot.elapsed_ms = elapsed_ms;
        })?;
        *current = None;
        Ok(())
    }

    pub fn current_snapshot(&self) -> Result<CaptureSnapshot, String> {
        let mut snapshot = self
            .snapshot
            .lock()
            .map_err(|error| format!("CaptureSnapshot 状态锁定失败：{error}"))?
            .clone();
        if let Ok(current) = self.current.lock() {
            if let Some(session) = current.as_ref() {
                snapshot.elapsed_ms = session.started_at.elapsed().as_millis() as u64;
            }
        }
        Ok(snapshot)
    }

    pub fn fail(&self, session_id: &str, failure: CaptureFailure) -> Result<(), String> {
        let mut current = self
            .current
            .lock()
            .map_err(|error| format!("CaptureSession 状态锁定失败：{error}"))?;
        let session = current
            .as_ref()
            .filter(|session| session.id == session_id)
            .ok_or_else(|| "CaptureSession 不存在或已经结束".to_string())?;
        if !is_valid_transition(session.status, CaptureStatus::Failed) {
            return Err(format!(
                "CaptureSession 状态不能从 {:?} 切换到 Failed",
                session.status
            ));
        }
        let elapsed_ms = session.started_at.elapsed().as_millis() as u64;
        self.replace_snapshot(|snapshot| {
            snapshot.revision = snapshot.revision.saturating_add(1);
            snapshot.phase = CapturePhase::Failed;
            snapshot.elapsed_ms = elapsed_ms;
            snapshot.failure = Some(failure);
        })?;
        *current = None;
        Ok(())
    }

    fn replace_snapshot(&self, update: impl FnOnce(&mut CaptureSnapshot)) -> Result<(), String> {
        let mut snapshot = self
            .snapshot
            .lock()
            .map_err(|error| format!("CaptureSnapshot 状态锁定失败：{error}"))?;
        update(&mut snapshot);
        Ok(())
    }

    pub fn emit_snapshot(&self, app: &tauri::AppHandle) -> Result<CaptureSnapshot, String> {
        let snapshot = self.current_snapshot()?;
        crate::events::CaptureSnapshotEvent(snapshot.clone())
            .emit(app)
            .map_err(|error| format!("发送 CaptureSnapshot 失败：{error}"))?;
        Ok(snapshot)
    }

    pub fn has_active(&self) -> bool {
        self.current
            .lock()
            .map(|current| current.is_some())
            .unwrap_or(true)
    }

    pub fn cancel_current(&self) {
        let session_id = self
            .current
            .lock()
            .ok()
            .and_then(|current| current.as_ref().map(|session| session.id.clone()));
        if let Some(session_id) = session_id {
            self.cancel(&session_id);
        }
    }

    pub fn cancel(&self, session_id: &str) {
        if let Ok(mut current) = self.current.lock() {
            if current
                .as_ref()
                .is_some_and(|session| session.id == session_id)
            {
                let _ = self.replace_snapshot(|snapshot| {
                    let revision = snapshot.revision.saturating_add(1);
                    *snapshot = CaptureSnapshot::idle();
                    snapshot.revision = revision;
                });
                *current = None;
            }
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn read_capture_snapshot(
    state: tauri::State<'_, CaptureSessionState>,
) -> Result<CaptureSnapshot, String> {
    state.current_snapshot()
}

#[tauri::command]
#[specta::specta]
pub async fn abort_capture_session(
    app: tauri::AppHandle,
    state: tauri::State<'_, CaptureSessionState>,
    recording_state: tauri::State<'_, crate::recording::RecordingState>,
    session_id: String,
) -> Result<(), String> {
    if crate::recording::cancel_recording_for_session(&recording_state, &app, &session_id)
        .await
        .is_err()
    {
        // Processing may already own the finalized WAV. Clearing the session
        // prevents delivery even though an in-flight provider call cannot be interrupted.
        state.cancel(&session_id);
    }
    let _ = state.emit_snapshot(&app);
    let _ = crate::indicator::hide_indicator(&app);
    Ok(())
}

fn is_valid_transition(current: CaptureStatus, next: CaptureStatus) -> bool {
    matches!(
        (current, next),
        (CaptureStatus::Recording, CaptureStatus::Transcribing)
            | (CaptureStatus::Recording, CaptureStatus::Failed)
            | (CaptureStatus::Transcribing, CaptureStatus::Refining)
            | (CaptureStatus::Transcribing, CaptureStatus::Delivering)
            | (CaptureStatus::Transcribing, CaptureStatus::Failed)
            | (CaptureStatus::Refining, CaptureStatus::Delivering)
            | (CaptureStatus::Refining, CaptureStatus::Failed)
            | (CaptureStatus::Delivering, CaptureStatus::Completed)
            | (CaptureStatus::Delivering, CaptureStatus::Failed)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn captured_context() -> CapturedSessionContext {
        CapturedSessionContext {
            config: crate::data::default_app_config(),
            persona: Persona {
                id: "general".to_string(),
                name: "通用".to_string(),
                description: "固定的人格".to_string(),
                icon: "sparkles".to_string(),
                is_default: true,
                processing_mode: "polish".to_string(),
                created_at: String::new(),
                updated_at: String::new(),
            },
            asr_hotwords: vec!["XiLuoLin".to_string()],
            hotword_context: "术语：XiLuoLin".to_string(),
        }
    }

    #[test]
    fn session_follows_the_expected_state_machine() {
        let state = CaptureSessionState::new();
        let started = state
            .begin_with_focus(CaptureSource::Hotkey, None)
            .expect("session should start");

        state
            .update_status(&started.session_id, CaptureStatus::Transcribing)
            .unwrap();
        state
            .update_status(&started.session_id, CaptureStatus::Refining)
            .unwrap();
        state
            .update_status(&started.session_id, CaptureStatus::Delivering)
            .unwrap();
        state
            .finish(&started.session_id, CaptureStatus::Completed)
            .unwrap();

        assert!(state.delivery_context(&started.session_id).is_err());
    }

    #[test]
    fn second_session_is_rejected_until_the_first_finishes() {
        let state = CaptureSessionState::new();
        let started = state
            .begin_with_focus(CaptureSource::App, None)
            .expect("session should start");

        assert!(state.begin_with_focus(CaptureSource::App, None).is_err());
        state.cancel(&started.session_id);
        let cancelled = state.current_snapshot().unwrap();
        assert_eq!(cancelled.phase, CapturePhase::Idle);
        assert!(cancelled.revision > 0);
        assert!(state.begin_with_focus(CaptureSource::App, None).is_ok());
    }

    #[test]
    fn processing_configuration_is_frozen_for_the_session() {
        let state = CaptureSessionState::new();
        let expected = captured_context();
        let started = state
            .begin_with_focus_and_context(CaptureSource::App, None, Some(expected.clone()))
            .unwrap();

        let actual = state.processing_context(&started.session_id).unwrap();
        assert_eq!(actual.config, expected.config);
        assert_eq!(actual.persona, expected.persona);
        assert_eq!(actual.asr_hotwords, expected.asr_hotwords);
        assert_eq!(actual.hotword_context, expected.hotword_context);
    }

    #[test]
    fn invalid_transition_is_rejected() {
        let state = CaptureSessionState::new();
        let started = state
            .begin_with_focus(CaptureSource::App, None)
            .expect("session should start");

        let error = state
            .update_status(&started.session_id, CaptureStatus::Delivering)
            .unwrap_err();
        assert!(error.contains("状态不能"));
    }

    #[test]
    fn delivery_context_keeps_source_private_to_rust() {
        let state = CaptureSessionState::new();
        let started = state
            .begin_with_focus(CaptureSource::Hotkey, None)
            .expect("session should start");
        let context = state.delivery_context(&started.session_id).unwrap();

        assert_eq!(context.source, CaptureSource::Hotkey);
        assert!(context.focus.is_none());
        assert!(context.history_id.is_none());
    }

    #[test]
    fn current_snapshot_exposes_idle_and_active_session_state() {
        let state = CaptureSessionState::new();

        let idle = state.current_snapshot().unwrap();
        assert_eq!(idle.phase, CapturePhase::Idle);
        assert_eq!(idle.preview_state, PreviewState::Disabled);
        assert!(idle.session_id.is_none());

        let started = state
            .begin_with_focus(CaptureSource::Hotkey, None)
            .expect("session should start");
        let recording = state.current_snapshot().unwrap();

        assert_eq!(
            recording.session_id.as_deref(),
            Some(started.session_id.as_str())
        );
        assert_eq!(recording.source, Some(CaptureSource::Hotkey));
        assert_eq!(recording.phase, CapturePhase::Recording);
        assert!(recording.revision > idle.revision);
    }

    #[test]
    fn status_and_preview_updates_advance_revision_without_losing_stable_text() {
        let state = CaptureSessionState::new();
        let started = state
            .begin_with_focus(CaptureSource::App, None)
            .expect("session should start");
        let initial_revision = state.current_snapshot().unwrap().revision;

        state
            .update_preview(
                &started.session_id,
                PreviewState::Active,
                "你好".to_string(),
                "世界".to_string(),
            )
            .unwrap();
        let preview = state.current_snapshot().unwrap();
        assert_eq!(preview.stable_text, "你好");
        assert_eq!(preview.tentative_text, "世界");
        assert!(preview.revision > initial_revision);

        state
            .update_status(&started.session_id, CaptureStatus::Transcribing)
            .unwrap();
        let transcribing = state.current_snapshot().unwrap();
        assert_eq!(transcribing.phase, CapturePhase::Transcribing);
        assert_eq!(transcribing.stable_text, "你好");
        assert_eq!(transcribing.tentative_text, "世界");
        assert!(transcribing.revision > preview.revision);
    }

    #[test]
    fn failing_a_session_releases_it_and_keeps_structured_failure_for_late_windows() {
        let state = CaptureSessionState::new();
        let started = state
            .begin_with_focus(CaptureSource::Hotkey, None)
            .expect("session should start");

        state
            .fail(
                &started.session_id,
                CaptureFailure {
                    code: "preview_overflow".to_string(),
                    stage: CapturePhase::Recording,
                    recoverable: true,
                    detail: "实时预览处理过慢".to_string(),
                },
            )
            .unwrap();

        let failed = state.current_snapshot().unwrap();
        assert_eq!(failed.phase, CapturePhase::Failed);
        assert_eq!(failed.failure.unwrap().code, "preview_overflow");
        assert!(!state.has_active());
    }
}
