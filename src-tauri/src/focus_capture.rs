use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum FocusRestoreLevel {
    Window,
    Application,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FocusSnapshot {
    #[cfg(target_os = "windows")]
    hwnd: isize,
    #[cfg(target_os = "macos")]
    pid: i32,
    #[cfg(target_os = "macos")]
    bundle_id: Option<String>,
    #[cfg(target_os = "macos")]
    app_name: String,
    #[cfg(target_os = "macos")]
    window: Option<macos::WindowFingerprint>,
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

#[cfg(target_os = "macos")]
pub fn capture_focus() -> Result<Option<FocusSnapshot>, String> {
    macos::capture_focus()
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn capture_focus() -> Result<Option<FocusSnapshot>, String> {
    Ok(None)
}

#[cfg(target_os = "windows")]
pub fn restore_focus(snapshot: Option<&FocusSnapshot>) -> Result<FocusRestoreLevel, String> {
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
            Ok(FocusRestoreLevel::Window)
        } else {
            Err("系统拒绝恢复录音开始时的目标窗口".to_string())
        }
    }
}

#[cfg(target_os = "macos")]
pub fn restore_focus(snapshot: Option<&FocusSnapshot>) -> Result<FocusRestoreLevel, String> {
    macos::restore_focus(snapshot)
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn restore_focus(_snapshot: Option<&FocusSnapshot>) -> Result<FocusRestoreLevel, String> {
    Ok(FocusRestoreLevel::None)
}

#[cfg(target_os = "macos")]
mod macos {
    use std::{ffi::c_void, ptr, thread, time::Duration};

    use accessibility_sys::{
        kAXFocusedAttribute, kAXFocusedWindowAttribute, kAXMainAttribute, kAXPositionAttribute,
        kAXRaiseAction, kAXRoleAttribute, kAXSizeAttribute, kAXSubroleAttribute, kAXTitleAttribute,
        kAXValueTypeCGPoint, kAXValueTypeCGSize, kAXWindowsAttribute,
        AXUIElementCopyAttributeValue, AXUIElementCreateApplication, AXUIElementPerformAction,
        AXUIElementRef, AXUIElementSetAttributeValue, AXValueGetType, AXValueGetTypeID,
        AXValueGetValue, AXValueRef,
    };
    use core_foundation::{base::TCFType, string::CFString};
    use core_foundation_sys::{
        array::{CFArrayGetCount, CFArrayGetTypeID, CFArrayGetValueAtIndex, CFArrayRef},
        base::{CFGetTypeID, CFRelease, CFTypeRef},
        number::kCFBooleanTrue,
        string::{CFStringGetTypeID, CFStringRef},
    };
    use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication, NSWorkspace};

    use super::{FocusRestoreLevel, FocusSnapshot};
    use crate::macos_permissions::{accessibility_status, PermissionStatus};

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Point {
        x: f64,
        y: f64,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Size {
        width: f64,
        height: f64,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub(super) struct WindowFingerprint {
        title: Option<String>,
        role: Option<String>,
        subrole: Option<String>,
        position: Option<Point>,
        size: Option<Size>,
    }

    pub(super) fn capture_focus() -> Result<Option<FocusSnapshot>, String> {
        let workspace = NSWorkspace::sharedWorkspace();
        let Some(application) = workspace.frontmostApplication() else {
            return Ok(None);
        };
        let pid = application.processIdentifier();
        if pid <= 0 || pid as u32 == std::process::id() {
            return Ok(None);
        }

        let bundle_id = application
            .bundleIdentifier()
            .map(|value| value.to_string());
        let app_name = application
            .localizedName()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "目标应用".to_string());
        let window = if accessibility_status() == PermissionStatus::Authorized {
            focused_window(pid)
        } else {
            None
        };

        Ok(Some(FocusSnapshot {
            pid,
            bundle_id,
            app_name,
            window,
        }))
    }

    pub(super) fn restore_focus(
        snapshot: Option<&FocusSnapshot>,
    ) -> Result<FocusRestoreLevel, String> {
        if accessibility_status() != PermissionStatus::Authorized {
            return Err(
                "未授予辅助功能权限，请在系统设置 → 隐私与安全性 → 辅助功能中允许 XiLuoLin"
                    .to_string(),
            );
        }
        let snapshot = snapshot.ok_or_else(|| "未记录录音开始时的目标应用".to_string())?;
        let application =
            NSRunningApplication::runningApplicationWithProcessIdentifier(snapshot.pid)
                .ok_or_else(|| format!("录音开始时的目标应用“{}”已退出", snapshot.app_name))?;
        if application.isTerminated() {
            return Err(format!("录音开始时的目标应用“{}”已退出", snapshot.app_name));
        }
        if let (Some(expected), Some(actual)) = (
            snapshot.bundle_id.as_deref(),
            application.bundleIdentifier(),
        ) {
            if expected != actual.to_string() {
                return Err(format!("录音开始时的目标应用“{}”已退出", snapshot.app_name));
            }
        }

        let activated = application.activateWithOptions(NSApplicationActivationOptions(0));
        if !activated {
            return Err(format!("macOS 拒绝激活目标应用“{}”", snapshot.app_name));
        }

        let requested_window_restore = snapshot
            .window
            .as_ref()
            .is_some_and(|fingerprint| restore_window(snapshot.pid, fingerprint));

        let deadline = std::time::Instant::now() + Duration::from_millis(800);
        while std::time::Instant::now() < deadline {
            if frontmost_pid() == Some(snapshot.pid) {
                let window_restored = requested_window_restore
                    && snapshot.window.as_ref().is_some_and(|expected| {
                        focused_window(snapshot.pid)
                            .is_some_and(|actual| match_score(expected, &actual) >= 3)
                    });
                return Ok(if window_restored {
                    FocusRestoreLevel::Window
                } else {
                    FocusRestoreLevel::Application
                });
            }
            thread::sleep(Duration::from_millis(25));
        }

        Err(format!(
            "未能确认目标应用“{}”已恢复到前台，为避免误粘贴已取消自动输入",
            snapshot.app_name
        ))
    }

    fn frontmost_pid() -> Option<i32> {
        NSWorkspace::sharedWorkspace()
            .frontmostApplication()
            .map(|application| application.processIdentifier())
    }

    fn focused_window(pid: i32) -> Option<WindowFingerprint> {
        let application = unsafe { AXUIElementCreateApplication(pid) };
        if application.is_null() {
            return None;
        }
        let window = copy_element_attribute(application, kAXFocusedWindowAttribute);
        unsafe { CFRelease(application.cast()) };
        let window = window?;
        let fingerprint = fingerprint(window);
        unsafe { CFRelease(window.cast()) };
        Some(fingerprint)
    }

    fn restore_window(pid: i32, expected: &WindowFingerprint) -> bool {
        let application = unsafe { AXUIElementCreateApplication(pid) };
        if application.is_null() {
            return false;
        }
        let windows = copy_array_attribute(application, kAXWindowsAttribute);
        let Some(windows) = windows else {
            unsafe { CFRelease(application.cast()) };
            return false;
        };

        let mut best: Option<(AXUIElementRef, i32)> = None;
        let count = unsafe { CFArrayGetCount(windows) };
        for index in 0..count {
            let window = unsafe { CFArrayGetValueAtIndex(windows, index) as AXUIElementRef };
            if window.is_null() {
                continue;
            }
            let score = match_score(expected, &fingerprint(window));
            if score > best.map(|(_, best_score)| best_score).unwrap_or(-1) {
                best = Some((window, score));
            }
        }

        let restored = best.is_some_and(|(window, score)| {
            if score < 3 {
                return false;
            }
            let main_result = set_boolean_attribute(window, kAXMainAttribute);
            let focused_result = set_boolean_attribute(window, kAXFocusedAttribute);
            let raised = perform_action(window, kAXRaiseAction);
            main_result || focused_result || raised
        });

        unsafe {
            CFRelease(windows.cast());
            CFRelease(application.cast());
        }
        restored
    }

    fn match_score(expected: &WindowFingerprint, actual: &WindowFingerprint) -> i32 {
        let mut score = 0;
        if expected.title.is_some() && expected.title == actual.title {
            score += 5;
        }
        if expected.role.is_some() && expected.role == actual.role {
            score += 2;
        }
        if expected.subrole.is_some() && expected.subrole == actual.subrole {
            score += 1;
        }
        if close_point(expected.position, actual.position) {
            score += 2;
        }
        if close_size(expected.size, actual.size) {
            score += 2;
        }
        score
    }

    fn close_point(left: Option<Point>, right: Option<Point>) -> bool {
        matches!((left, right), (Some(left), Some(right)) if (left.x - right.x).abs() <= 4.0 && (left.y - right.y).abs() <= 4.0)
    }

    fn close_size(left: Option<Size>, right: Option<Size>) -> bool {
        matches!((left, right), (Some(left), Some(right)) if (left.width - right.width).abs() <= 4.0 && (left.height - right.height).abs() <= 4.0)
    }

    fn fingerprint(window: AXUIElementRef) -> WindowFingerprint {
        WindowFingerprint {
            title: copy_string_attribute(window, kAXTitleAttribute),
            role: copy_string_attribute(window, kAXRoleAttribute),
            subrole: copy_string_attribute(window, kAXSubroleAttribute),
            position: copy_point_attribute(window, kAXPositionAttribute),
            size: copy_size_attribute(window, kAXSizeAttribute),
        }
    }

    fn attribute_name(name: &str) -> CFString {
        CFString::new(name)
    }

    fn copy_attribute(element: AXUIElementRef, name: &str) -> Option<CFTypeRef> {
        let name = attribute_name(name);
        let mut value: CFTypeRef = ptr::null();
        let result = unsafe {
            AXUIElementCopyAttributeValue(element, name.as_concrete_TypeRef(), &mut value)
        };
        if result == 0 && !value.is_null() {
            Some(value)
        } else {
            None
        }
    }

    fn copy_element_attribute(element: AXUIElementRef, name: &str) -> Option<AXUIElementRef> {
        copy_attribute(element, name).map(|value| value as AXUIElementRef)
    }

    fn copy_array_attribute(element: AXUIElementRef, name: &str) -> Option<CFArrayRef> {
        let value = copy_attribute(element, name)?;
        if unsafe { CFGetTypeID(value) } == unsafe { CFArrayGetTypeID() } {
            Some(value as CFArrayRef)
        } else {
            unsafe { CFRelease(value) };
            None
        }
    }

    fn copy_string_attribute(element: AXUIElementRef, name: &str) -> Option<String> {
        let value = copy_attribute(element, name)?;
        if unsafe { CFGetTypeID(value) } != unsafe { CFStringGetTypeID() } {
            unsafe { CFRelease(value) };
            return None;
        }
        let value = unsafe { CFString::wrap_under_create_rule(value as CFStringRef) };
        Some(value.to_string())
    }

    fn copy_point_attribute(element: AXUIElementRef, name: &str) -> Option<Point> {
        let raw_value = copy_attribute(element, name)?;
        if unsafe { CFGetTypeID(raw_value) } != unsafe { AXValueGetTypeID() } {
            unsafe { CFRelease(raw_value) };
            return None;
        }
        let value = raw_value as AXValueRef;
        let mut point = Point { x: 0.0, y: 0.0 };
        let valid = unsafe {
            AXValueGetType(value) == kAXValueTypeCGPoint
                && AXValueGetValue(
                    value,
                    kAXValueTypeCGPoint,
                    (&mut point as *mut Point).cast::<c_void>(),
                )
        };
        unsafe { CFRelease(value.cast()) };
        valid.then_some(point)
    }

    fn copy_size_attribute(element: AXUIElementRef, name: &str) -> Option<Size> {
        let raw_value = copy_attribute(element, name)?;
        if unsafe { CFGetTypeID(raw_value) } != unsafe { AXValueGetTypeID() } {
            unsafe { CFRelease(raw_value) };
            return None;
        }
        let value = raw_value as AXValueRef;
        let mut size = Size {
            width: 0.0,
            height: 0.0,
        };
        let valid = unsafe {
            AXValueGetType(value) == kAXValueTypeCGSize
                && AXValueGetValue(
                    value,
                    kAXValueTypeCGSize,
                    (&mut size as *mut Size).cast::<c_void>(),
                )
        };
        unsafe { CFRelease(value.cast()) };
        valid.then_some(size)
    }

    fn set_boolean_attribute(element: AXUIElementRef, name: &str) -> bool {
        let name = attribute_name(name);
        unsafe {
            AXUIElementSetAttributeValue(element, name.as_concrete_TypeRef(), kCFBooleanTrue.cast())
                == 0
        }
    }

    fn perform_action(element: AXUIElementRef, action: &str) -> bool {
        let action = attribute_name(action);
        unsafe { AXUIElementPerformAction(element, action.as_concrete_TypeRef()) == 0 }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn fingerprint_with(
            title: &str,
            x: f64,
            y: f64,
            width: f64,
            height: f64,
        ) -> WindowFingerprint {
            WindowFingerprint {
                title: Some(title.to_string()),
                role: Some("AXWindow".to_string()),
                subrole: Some("AXStandardWindow".to_string()),
                position: Some(Point { x, y }),
                size: Some(Size { width, height }),
            }
        }

        #[test]
        fn exact_window_scores_higher_than_another_window() {
            let expected = fingerprint_with("文档", 10.0, 20.0, 800.0, 600.0);
            let exact = fingerprint_with("文档", 11.0, 19.0, 800.0, 600.0);
            let other = fingerprint_with("设置", 300.0, 200.0, 500.0, 400.0);

            assert!(match_score(&expected, &exact) > match_score(&expected, &other));
            assert!(match_score(&expected, &exact) >= 3);
        }

        #[test]
        fn geometry_can_match_untitled_windows() {
            let mut expected = fingerprint_with("", 10.0, 20.0, 800.0, 600.0);
            expected.title = None;
            let mut actual = expected.clone();
            actual.position = Some(Point { x: 12.0, y: 22.0 });

            assert!(match_score(&expected, &actual) >= 3);
        }
    }
}
