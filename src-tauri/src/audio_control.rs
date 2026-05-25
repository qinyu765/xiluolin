// Windows 音频控制模块
// 用于在录音时静音其他应用的音频播放

#[cfg(target_os = "windows")]
pub mod windows_audio {
    use windows::{
        core::*,
        Win32::Media::Audio::*,
        Win32::System::Com::*,
    };

    /// 静音所有其他应用的音频会话
    pub fn mute_all_sessions() -> Result<()> {
        unsafe {
            // 初始化 COM（忽略已初始化的错误）
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            let result = (|| -> Result<()> {
                // 获取设备枚举器
                let enumerator: IMMDeviceEnumerator = CoCreateInstance(
                    &MMDeviceEnumerator,
                    None,
                    CLSCTX_ALL,
                )?;

                // 获取默认音频输出设备
                let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

                // 获取会话管理器
                let session_manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;
                let session_enum = session_manager.GetSessionEnumerator()?;
                let count = session_enum.GetCount()?;

                // 遍历所有音频会话并静音
                for i in 0..count {
                    if let Ok(session_control) = session_enum.GetSession(i) {
                        if let Ok(simple_volume) = session_control.cast::<ISimpleAudioVolume>() {
                            // 获取当前静音状态
                            if let Ok(is_muted) = simple_volume.GetMute() {
                                // 如果当前未静音，则静音它
                                if !is_muted.as_bool() {
                                    let _ = simple_volume.SetMute(true, std::ptr::null());
                                }
                            }
                        }
                    }
                }

                Ok(())
            })();

            CoUninitialize();
            result
        }
    }

    /// 恢复所有音频会话（取消静音）
    pub fn unmute_all_sessions() -> Result<()> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            let result = (|| -> Result<()> {
                let enumerator: IMMDeviceEnumerator = CoCreateInstance(
                    &MMDeviceEnumerator,
                    None,
                    CLSCTX_ALL,
                )?;

                let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
                let session_manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;
                let session_enum = session_manager.GetSessionEnumerator()?;
                let count = session_enum.GetCount()?;

                // 遍历所有音频会话并取消静音
                for i in 0..count {
                    if let Ok(session_control) = session_enum.GetSession(i) {
                        if let Ok(simple_volume) = session_control.cast::<ISimpleAudioVolume>() {
                            // 取消静音
                            let _ = simple_volume.SetMute(false, std::ptr::null());
                        }
                    }
                }

                Ok(())
            })();

            CoUninitialize();
            result
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub mod windows_audio {
    pub fn mute_all_sessions() -> Result<(), String> {
        // 非 Windows 平台不支持
        Ok(())
    }

    pub fn unmute_all_sessions() -> Result<(), String> {
        // 非 Windows 平台不支持
        Ok(())
    }
}
