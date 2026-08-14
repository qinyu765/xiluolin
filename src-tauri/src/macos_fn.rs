use std::sync::Mutex;

use tauri::AppHandle;

const FN_HOLD_THRESHOLD_MS: u64 = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FnAction {
    Start,
    Cancel,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FnDecision {
    action: Option<FnAction>,
    suppress: bool,
}

impl FnDecision {
    const KEEP: Self = Self {
        action: None,
        suppress: false,
    };
}

#[derive(Debug, Default)]
struct FnGestureState {
    fn_down: bool,
    started_at_ms: Option<u64>,
}

impl FnGestureState {
    fn flags_changed(
        &mut self,
        fn_down: bool,
        has_other_modifiers: bool,
        now_ms: u64,
    ) -> FnDecision {
        if fn_down == self.fn_down {
            if fn_down && has_other_modifiers && self.started_at_ms.take().is_some() {
                return FnDecision {
                    action: Some(FnAction::Cancel),
                    suppress: false,
                };
            }
            return FnDecision::KEEP;
        }

        self.fn_down = fn_down;
        if fn_down {
            if has_other_modifiers {
                self.started_at_ms = None;
                return FnDecision::KEEP;
            }
            self.started_at_ms = Some(now_ms);
            // 独立手势的按下与松开必须成对拦截，避免下游只看到半个修饰键事件。
            return FnDecision {
                action: Some(FnAction::Start),
                suppress: true,
            };
        }

        let Some(started_at_ms) = self.started_at_ms.take() else {
            return FnDecision::KEEP;
        };
        let action = if now_ms.saturating_sub(started_at_ms) < FN_HOLD_THRESHOLD_MS {
            FnAction::Cancel
        } else {
            FnAction::Complete
        };
        // 独立 Fn 手势由应用消费松开事件，避免同时触发系统的单按 Fn 行为。
        FnDecision {
            action: Some(action),
            suppress: true,
        }
    }

    fn reset(&mut self) -> FnDecision {
        self.fn_down = false;
        if self.started_at_ms.take().is_some() {
            FnDecision {
                action: Some(FnAction::Cancel),
                suppress: false,
            }
        } else {
            FnDecision::KEEP
        }
    }
}

pub struct FnHoldManager {
    monitor: Mutex<Option<platform::FnMonitor>>,
}

impl FnHoldManager {
    pub fn new() -> Self {
        Self {
            monitor: Mutex::new(None),
        }
    }
}

pub fn configure_fn_hold(
    app: &AppHandle,
    manager: &FnHoldManager,
    enabled: bool,
) -> Result<(), String> {
    let mut monitor = manager
        .monitor
        .lock()
        .map_err(|error| format!("Fn 监听状态锁定失败：{error}"))?;
    if let Some(previous) = monitor.take() {
        previous.stop();
    }
    if enabled {
        *monitor = Some(platform::FnMonitor::start(app.clone())?);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
mod platform {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc, Mutex};
    use std::thread::{self, JoinHandle};
    use std::time::{Duration, Instant};

    use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
    use core_graphics::event::{
        CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
        CGEventType, CallbackResult,
    };
    use tauri::Manager;
    use tauri_specta::Event;
    use tokio::sync::mpsc as tokio_mpsc;

    use super::{FnAction, FnDecision, FnGestureState};
    use crate::capture_session::{CaptureSessionState, CaptureSource};
    use crate::events::RecordingErrorEvent;
    use crate::macos_permissions::{accessibility_status, PermissionStatus};
    use crate::recording::{
        cancel_recording_for_session, start_recording_for_source,
        stop_recording_for_expected_session, RecordingState,
    };

    struct FnRecordingRuntime {
        session_id: Option<String>,
    }

    pub struct FnMonitor {
        stop_requested: Arc<AtomicBool>,
        actions: Option<tokio_mpsc::UnboundedSender<FnAction>>,
        thread: Option<JoinHandle<()>>,
    }

    impl FnMonitor {
        pub fn start(app: tauri::AppHandle) -> Result<Self, String> {
            if accessibility_status() != PermissionStatus::Authorized {
                return Err("独立 Fn 录音需要辅助功能权限；现有组合快捷键仍可继续使用".to_string());
            }

            let (action_sender, mut action_receiver) = tokio_mpsc::unbounded_channel();
            let action_app = app.clone();
            tauri::async_runtime::spawn(async move {
                let mut runtime = FnRecordingRuntime { session_id: None };
                while let Some(action) = action_receiver.recv().await {
                    handle_action(&action_app, &mut runtime, action).await;
                }
            });

            let stop_requested = Arc::new(AtomicBool::new(false));
            let thread_stop = Arc::clone(&stop_requested);
            let thread_actions = action_sender.clone();
            let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
            let thread = thread::Builder::new()
                .name("xiluolin-fn-event-tap".to_string())
                .spawn(move || {
                    run_event_tap(thread_stop, thread_actions, ready_sender);
                })
                .map_err(|error| format!("创建 Fn 监听线程失败：{error}"))?;

            match ready_receiver.recv_timeout(Duration::from_secs(3)) {
                Ok(Ok(())) => Ok(Self {
                    stop_requested,
                    actions: Some(action_sender),
                    thread: Some(thread),
                }),
                Ok(Err(error)) => {
                    stop_requested.store(true, Ordering::SeqCst);
                    let _ = thread.join();
                    Err(error)
                }
                Err(_) => {
                    stop_requested.store(true, Ordering::SeqCst);
                    let _ = thread.join();
                    Err("等待 Fn 监听器启动超时".to_string())
                }
            }
        }

        pub fn stop(mut self) {
            if let Some(actions) = self.actions.take() {
                let _ = actions.send(FnAction::Cancel);
            }
            self.stop_requested.store(true, Ordering::SeqCst);
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
        }
    }

    impl Drop for FnMonitor {
        fn drop(&mut self) {
            if let Some(actions) = self.actions.take() {
                let _ = actions.send(FnAction::Cancel);
            }
            self.stop_requested.store(true, Ordering::SeqCst);
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
        }
    }

    fn run_event_tap(
        stop_requested: Arc<AtomicBool>,
        actions: tokio_mpsc::UnboundedSender<FnAction>,
        ready: mpsc::SyncSender<Result<(), String>>,
    ) {
        let gesture = Arc::new(Mutex::new(FnGestureState::default()));
        let callback_gesture = Arc::clone(&gesture);
        let callback_actions = actions.clone();
        let reenable_requested = Arc::new(AtomicBool::new(false));
        let callback_reenable = Arc::clone(&reenable_requested);
        let started_at = Instant::now();

        let event_tap = CGEventTap::new(
            CGEventTapLocation::Session,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            vec![CGEventType::FlagsChanged],
            move |_proxy, event_type, event| match event_type {
                CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput => {
                    callback_reenable.store(true, Ordering::SeqCst);
                    let decision = callback_gesture
                        .lock()
                        .map(|mut gesture| gesture.reset())
                        .unwrap_or(FnDecision::KEEP);
                    dispatch_decision(&callback_actions, decision)
                }
                CGEventType::FlagsChanged => {
                    let flags = event.get_flags();
                    let fn_down = flags.contains(CGEventFlags::CGEventFlagSecondaryFn);
                    let has_other_modifiers = flags.intersects(
                        CGEventFlags::CGEventFlagCommand
                            | CGEventFlags::CGEventFlagAlternate
                            | CGEventFlags::CGEventFlagShift
                            | CGEventFlags::CGEventFlagControl,
                    );
                    let decision = callback_gesture
                        .lock()
                        .map(|mut gesture| {
                            gesture.flags_changed(
                                fn_down,
                                has_other_modifiers,
                                started_at.elapsed().as_millis() as u64,
                            )
                        })
                        .unwrap_or(FnDecision::KEEP);
                    dispatch_decision(&callback_actions, decision)
                }
                _ => CallbackResult::Keep,
            },
        );
        let event_tap = match event_tap {
            Ok(event_tap) => event_tap,
            Err(()) => {
                let _ = ready.send(Err(
                    "注册 macOS Fn 事件监听失败；请检查辅助功能权限".to_string()
                ));
                return;
            }
        };
        let source = match event_tap.mach_port().create_runloop_source(0) {
            Ok(source) => source,
            Err(()) => {
                let _ = ready.send(Err("创建 macOS Fn 事件循环失败".to_string()));
                return;
            }
        };
        let run_loop = CFRunLoop::get_current();
        run_loop.add_source(&source, unsafe { kCFRunLoopDefaultMode });
        event_tap.enable();
        let _ = ready.send(Ok(()));

        while !stop_requested.load(Ordering::SeqCst) {
            CFRunLoop::run_in_mode(
                unsafe { kCFRunLoopDefaultMode },
                Duration::from_millis(100),
                true,
            );
            if reenable_requested.swap(false, Ordering::SeqCst) {
                event_tap.enable();
            }
        }
    }

    fn dispatch_decision(
        actions: &tokio_mpsc::UnboundedSender<FnAction>,
        decision: FnDecision,
    ) -> CallbackResult {
        if let Some(action) = decision.action {
            let _ = actions.send(action);
        }
        if decision.suppress {
            CallbackResult::Drop
        } else {
            CallbackResult::Keep
        }
    }

    async fn handle_action(
        app: &tauri::AppHandle,
        runtime: &mut FnRecordingRuntime,
        action: FnAction,
    ) {
        match action {
            FnAction::Start => {
                if runtime.session_id.is_some() {
                    return;
                }
                let recording_state = app.state::<RecordingState>();
                match start_recording_for_source(&recording_state, app, CaptureSource::Hotkey) {
                    Ok(started) => {
                        runtime.session_id = Some(started.session_id);
                        let _ = crate::indicator::show_indicator(app);
                    }
                    Err(error) => {
                        if !error.contains("上一条语音输入仍在处理中") {
                            let _ = crate::indicator::finish_indicator(app, "failed");
                            let _ = RecordingErrorEvent(error).emit(app);
                        }
                    }
                }
            }
            FnAction::Cancel => {
                let Some(session_id) = runtime.session_id.take() else {
                    return;
                };
                let recording_state = app.state::<RecordingState>();
                if let Err(error) =
                    cancel_recording_for_session(&recording_state, app, &session_id).await
                {
                    app.state::<CaptureSessionState>().cancel(&session_id);
                    let _ = RecordingErrorEvent(error).emit(app);
                }
                let _ = crate::indicator::hide_indicator(app);
            }
            FnAction::Complete => {
                let Some(session_id) = runtime.session_id.take() else {
                    return;
                };
                let recording_state = app.state::<RecordingState>();
                match stop_recording_for_expected_session(&recording_state, app, &session_id).await
                {
                    Ok(Some(result)) => {
                        let _ = crate::indicator::update_indicator(app, "transcribing");
                        crate::capture_coordinator::spawn_recording_pipeline(app.clone(), result);
                    }
                    Ok(None) => {
                        // 28 秒自动停止已接管该会话，无需重复发送完成事件。
                    }
                    Err(error) => {
                        app.state::<CaptureSessionState>().cancel(&session_id);
                        let _ = crate::indicator::finish_indicator(app, "failed");
                        let _ = RecordingErrorEvent(error).emit(app);
                    }
                }
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    pub struct FnMonitor;

    impl FnMonitor {
        pub fn start(_app: tauri::AppHandle) -> Result<Self, String> {
            Ok(Self)
        }

        pub fn stop(self) {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_fn_starts_immediately_and_completes_after_threshold() {
        let mut state = FnGestureState::default();
        assert_eq!(
            state.flags_changed(true, false, 1_000),
            FnDecision {
                action: Some(FnAction::Start),
                suppress: true,
            }
        );
        assert_eq!(
            state.flags_changed(false, false, 1_300),
            FnDecision {
                action: Some(FnAction::Complete),
                suppress: true,
            }
        );
    }

    #[test]
    fn short_fn_tap_cancels_recording() {
        let mut state = FnGestureState::default();
        state.flags_changed(true, false, 10);
        assert_eq!(
            state.flags_changed(false, false, 309),
            FnDecision {
                action: Some(FnAction::Cancel),
                suppress: true,
            }
        );
    }

    #[test]
    fn modifier_combination_is_kept_and_cancels_started_gesture() {
        let mut state = FnGestureState::default();
        state.flags_changed(true, false, 0);
        assert_eq!(
            state.flags_changed(true, true, 20),
            FnDecision {
                action: Some(FnAction::Cancel),
                suppress: false,
            }
        );
        assert_eq!(state.flags_changed(false, true, 30), FnDecision::KEEP);
    }

    #[test]
    fn fn_pressed_as_part_of_existing_modifier_combination_is_ignored() {
        let mut state = FnGestureState::default();
        assert_eq!(state.flags_changed(true, true, 0), FnDecision::KEEP);
        assert_eq!(state.flags_changed(false, false, 500), FnDecision::KEEP);
    }

    #[test]
    fn disabled_event_tap_resets_and_cancels_active_gesture() {
        let mut state = FnGestureState::default();
        state.flags_changed(true, false, 0);
        assert_eq!(
            state.reset(),
            FnDecision {
                action: Some(FnAction::Cancel),
                suppress: false,
            }
        );
        assert_eq!(state.flags_changed(false, false, 500), FnDecision::KEEP);
    }
}
