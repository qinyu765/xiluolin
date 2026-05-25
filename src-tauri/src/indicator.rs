use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

const INDICATOR_LABEL: &str = "recording-indicator";

/// 显示录音指示器窗口
pub fn show_indicator(app: &AppHandle) -> Result<(), String> {
    // 检查窗口是否已存在
    if let Some(window) = app.get_webview_window(INDICATOR_LABEL) {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    // 获取 indicator.html 的路径
    let indicator_url = if cfg!(dev) {
        // 开发模式：使用项目根目录的 indicator.html
        let path = std::env::current_dir()
            .map_err(|e| format!("获取当前目录失败: {}", e))?
            .join("indicator.html");

        println!("指示器 HTML 路径: {:?}", path);

        if !path.exists() {
            return Err(format!("指示器文件不存在: {:?}", path));
        }

        // Windows 路径转换为 file:// URL
        let path_str = path.to_string_lossy().replace("\\", "/");
        let url_string = if path_str.starts_with("/") {
            format!("file://{}", path_str)
        } else {
            format!("file:///{}", path_str)
        };

        println!("指示器 URL: {}", url_string);

        WebviewUrl::External(
            url_string.parse()
                .map_err(|e| format!("解析 URL 失败: {}", e))?
        )
    } else {
        // 生产模式：使用打包的资源
        WebviewUrl::App("indicator.html".into())
    };

    // 创建新的指示器窗口
    let window = WebviewWindowBuilder::new(
        app,
        INDICATOR_LABEL,
        indicator_url,
    )
    .title("录音中")
    .inner_size(200.0, 80.0)
    .resizable(false)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .transparent(true)
    .visible(false)
    .build()
    .map_err(|e| format!("创建指示器窗口失败: {}", e))?;

    // 窗口居中显示
    let _ = window.center();

    // 显示窗口
    let _ = window.show();

    Ok(())
}

/// 隐藏录音指示器窗口
pub fn hide_indicator(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(INDICATOR_LABEL) {
        let _ = window.hide();
    }
    Ok(())
}

/// 更新指示器状态
#[tauri::command]
pub fn update_indicator_status(app: AppHandle, status: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(INDICATOR_LABEL) {
        let _ = window.emit("indicator-status", status);
    }
    Ok(())
}
