use enigo::{Enigo, Key, Keyboard, Settings};
use arboard::Clipboard;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputMethod {
    Keyboard,
    Clipboard,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputResult {
    pub method: OutputMethod,
    pub success: bool,
    pub message: String,
}

// 主输出命令:自动选择最佳输出方式
#[tauri::command]
pub async fn output_text(text: String) -> Result<OutputResult, String> {
    // 1. 优先尝试直接键盘注入
    if let Ok(_) = keyboard_inject(&text).await {
        return Ok(OutputResult {
            method: OutputMethod::Keyboard,
            success: true,
            message: "已自动输入到光标位置".to_string(),
        });
    }

    // 2. 降级到剪贴板粘贴
    if let Ok(_) = clipboard_paste(&text).await {
        return Ok(OutputResult {
            method: OutputMethod::Clipboard,
            success: true,
            message: "已通过剪贴板输入".to_string(),
        });
    }

    // 3. 最终兜底:至少复制到剪贴板
    clipboard_copy(&text).await?;
    Ok(OutputResult {
        method: OutputMethod::Manual,
        success: false,
        message: "已复制到剪贴板,请手动粘贴 (Ctrl+V)".to_string(),
    })
}

// 直接键盘注入
async fn keyboard_inject(text: &str) -> Result<(), String> {
    let text = text.to_string();
    tokio::task::spawn_blocking(move || {
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("初始化键盘模拟失败: {}", e))?;

        // 直接输入整个文本，让 enigo 处理
        enigo.text(&text)
            .map_err(|e| format!("输入文本失败: {}", e))?;

        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("键盘注入任务失败: {}", e))?
}

// 剪贴板粘贴
async fn clipboard_paste(text: &str) -> Result<(), String> {
    // 先复制到剪贴板
    clipboard_copy(text).await?;

    // 模拟粘贴快捷键
    tokio::task::spawn_blocking(|| {
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("初始化键盘模拟失败: {}", e))?;

        #[cfg(target_os = "macos")]
        {
            enigo.key(Key::Meta, enigo::Direction::Press)
                .map_err(|e| format!("按下Meta键失败: {}", e))?;
            enigo.key(Key::Unicode('v'), enigo::Direction::Click)
                .map_err(|e| format!("按下V键失败: {}", e))?;
            enigo.key(Key::Meta, enigo::Direction::Release)
                .map_err(|e| format!("释放Meta键失败: {}", e))?;
        }

        #[cfg(not(target_os = "macos"))]
        {
            enigo.key(Key::Control, enigo::Direction::Press)
                .map_err(|e| format!("按下Ctrl键失败: {}", e))?;
            enigo.key(Key::Unicode('v'), enigo::Direction::Click)
                .map_err(|e| format!("按下V键失败: {}", e))?;
            enigo.key(Key::Control, enigo::Direction::Release)
                .map_err(|e| format!("释放Ctrl键失败: {}", e))?;
        }

        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("粘贴操作失败: {}", e))?
}

// 仅复制到剪贴板
async fn clipboard_copy(text: &str) -> Result<(), String> {
    let text = text.to_string();
    tokio::task::spawn_blocking(move || {
        let mut clipboard = Clipboard::new()
            .map_err(|e| format!("无法访问剪贴板: {}", e))?;
        clipboard
            .set_text(text)
            .map_err(|e| format!("写入剪贴板失败: {}", e))?;
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("剪贴板操作任务失败: {}", e))?
}
