use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::focus_capture::{capture_focus, FocusSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureSource {
    Hotkey,
    App,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureStatus {
    Recording,
    Transcribing,
    Refining,
    Delivering,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
}

pub struct CaptureSessionState {
    current: Mutex<Option<CaptureSession>>,
}

impl CaptureSessionState {
    pub fn new() -> Self {
        Self {
            current: Mutex::new(None),
        }
    }

    pub fn begin(&self, source: CaptureSource) -> Result<CaptureSessionStart, String> {
        let focus = if source == CaptureSource::Hotkey {
            capture_focus()?
        } else {
            None
        };
        self.begin_with_focus(source, focus)
    }

    fn begin_with_focus(
        &self,
        source: CaptureSource,
        focus: Option<FocusSnapshot>,
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
        });
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
        Ok(())
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
        *current = None;
        Ok(())
    }

    pub fn has_active(&self) -> bool {
        self.current
            .lock()
            .map(|current| current.is_some())
            .unwrap_or(true)
    }

    pub fn cancel_current(&self) {
        if let Ok(mut current) = self.current.lock() {
            *current = None;
        }
    }

    pub fn cancel(&self, session_id: &str) {
        if let Ok(mut current) = self.current.lock() {
            if current
                .as_ref()
                .is_some_and(|session| session.id == session_id)
            {
                *current = None;
            }
        }
    }
}

#[tauri::command]
pub fn abort_capture_session(
    app: tauri::AppHandle,
    state: tauri::State<'_, CaptureSessionState>,
    session_id: String,
) -> Result<(), String> {
    state.cancel(&session_id);
    let _ = crate::indicator::finish_indicator(&app, "failed");
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
        assert!(state.begin_with_focus(CaptureSource::App, None).is_ok());
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
}
