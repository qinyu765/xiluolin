#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusSnapshot {
    #[cfg(target_os = "windows")]
    hwnd: isize,
}

#[cfg(target_os = "windows")]
pub fn capture_focus() -> Result<Option<FocusSnapshot>, String> {
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return Ok(None);
    }

    let mut process_id = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut process_id)) };
    if process_id == std::process::id() {
        return Ok(None);
    }

    Ok(Some(FocusSnapshot {
        hwnd: hwnd.0 as isize,
    }))
}

#[cfg(not(target_os = "windows"))]
pub fn capture_focus() -> Result<Option<FocusSnapshot>, String> {
    Ok(None)
}

#[cfg(target_os = "windows")]
pub fn restore_focus(snapshot: Option<&FocusSnapshot>) -> Result<bool, String> {
    use windows::Win32::{
        Foundation::HWND,
        System::Threading::{AttachThreadInput, GetCurrentThreadId},
        UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowThreadProcessId, IsWindow, SetForegroundWindow,
        },
    };

    let snapshot = snapshot.ok_or_else(|| "未记录录音开始时的目标窗口".to_string())?;
    let hwnd = HWND(snapshot.hwnd as *mut std::ffi::c_void);

    if !unsafe { IsWindow(hwnd) }.as_bool() {
        return Err("录音开始时的目标窗口已关闭".to_string());
    }

    let current_thread = unsafe { GetCurrentThreadId() };
    let foreground = unsafe { GetForegroundWindow() };
    let foreground_thread = if foreground.0.is_null() {
        0
    } else {
        unsafe { GetWindowThreadProcessId(foreground, None) }
    };
    let target_thread = unsafe { GetWindowThreadProcessId(hwnd, None) };

    if target_thread == 0 {
        return Err("无法获取目标窗口线程".to_string());
    }

    let attach_to_foreground = foreground_thread != 0 && foreground_thread != current_thread;
    let attach_to_target = target_thread != current_thread && target_thread != foreground_thread;

    unsafe {
        if attach_to_foreground {
            let _ = AttachThreadInput(current_thread, foreground_thread, true);
        }
        if attach_to_target {
            let _ = AttachThreadInput(current_thread, target_thread, true);
        }

        let restored = SetForegroundWindow(hwnd).as_bool();

        if attach_to_target {
            let _ = AttachThreadInput(current_thread, target_thread, false);
        }
        if attach_to_foreground {
            let _ = AttachThreadInput(current_thread, foreground_thread, false);
        }

        if restored {
            Ok(true)
        } else {
            Err("系统拒绝恢复录音开始时的目标窗口".to_string())
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn restore_focus(_snapshot: Option<&FocusSnapshot>) -> Result<bool, String> {
    Ok(false)
}
