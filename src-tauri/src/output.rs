use arboard::Clipboard;
use enigo::{Enigo, Key, Keyboard, Settings};
use serde::{Deserialize, Serialize};

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
    let start_time = std::time::Instant::now();
    println!("[⏱️ 输出] ========== 开始新的输出请求 ==========");
    println!("[⏱️ 输出] 文本内容: {}", text);
    println!("[⏱️ 输出] 文本长度: {} 字符", text.len());

    // 1. 优先尝试剪贴板粘贴（更可靠，不会出现字符重复）
    let step1_start = std::time::Instant::now();
    println!("[⏱️ 输出] 尝试方法 1: 剪贴板粘贴");
    match clipboard_paste(&text).await {
        Ok(_) => {
            println!(
                "[⏱️ 输出] ✅ 剪贴板粘贴成功 - 耗时 {:?}",
                step1_start.elapsed()
            );
            println!("[⏱️ 输出] 总耗时: {:?}", start_time.elapsed());
            println!("[⏱️ 输出] ========== 输出请求完成 ==========");
            return Ok(OutputResult {
                method: OutputMethod::Clipboard,
                success: true,
                message: "已通过剪贴板输入".to_string(),
            });
        }
        Err(e) => {
            println!(
                "[⏱️ 输出] ❌ 剪贴板粘贴失败 - 耗时 {:?}: {}",
                step1_start.elapsed(),
                e
            );
        }
    }

    // 2. 最终兜底:至少复制到剪贴板
    let step2_start = std::time::Instant::now();
    println!("[⏱️ 输出] 尝试方法 2: 仅复制到剪贴板");
    clipboard_copy(&text).await?;
    println!(
        "[⏱️ 输出] ⚠️ 已复制到剪贴板，需要手动粘贴 - 耗时 {:?}",
        step2_start.elapsed()
    );
    println!("[⏱️ 输出] 总耗时: {:?}", start_time.elapsed());
    println!("[⏱️ 输出] ========== 输出请求完成 ==========");
    Ok(OutputResult {
        method: OutputMethod::Manual,
        success: false,
        message: "已复制到剪贴板,请手动粘贴 (Ctrl+V)".to_string(),
    })
}

// 剪贴板粘贴
async fn clipboard_paste(text: &str) -> Result<(), String> {
    println!("[clipboard_paste] ========== 开始剪贴板粘贴流程 ==========");
    println!("[clipboard_paste] 文本内容: {}", text);

    // 先复制到剪贴板
    clipboard_copy(text).await?;

    // 模拟粘贴快捷键
    tokio::task::spawn_blocking(|| {
        println!("[clipboard_paste] 等待 20ms 后执行粘贴快捷键...");
        std::thread::sleep(std::time::Duration::from_millis(20));

        println!("[clipboard_paste] 初始化 enigo...");
        let mut enigo = Enigo::new(&Settings::default()).map_err(|e| {
            let err_msg = format!("初始化键盘模拟失败: {}", e);
            println!("[clipboard_paste] ❌ {}", err_msg);
            err_msg
        })?;

        #[cfg(target_os = "macos")]
        {
            println!("[clipboard_paste] 执行 Cmd+V (macOS)...");
            enigo
                .key(Key::Meta, enigo::Direction::Press)
                .map_err(|e| format!("按下Meta键失败: {}", e))?;
            enigo
                .key(Key::Unicode('v'), enigo::Direction::Click)
                .map_err(|e| format!("按下V键失败: {}", e))?;
            enigo
                .key(Key::Meta, enigo::Direction::Release)
                .map_err(|e| format!("释放Meta键失败: {}", e))?;
        }

        #[cfg(not(target_os = "macos"))]
        {
            println!("[clipboard_paste] 准备执行 Ctrl+V (Windows/Linux)...");
            println!("[clipboard_paste] 步骤1: 按下 Ctrl 键");
            enigo
                .key(Key::Control, enigo::Direction::Press)
                .map_err(|e| {
                    let err_msg = format!("按下Ctrl键失败: {}", e);
                    println!("[clipboard_paste] ❌ {}", err_msg);
                    err_msg
                })?;

            println!("[clipboard_paste] 步骤2: 点击 V 键");
            enigo
                .key(Key::Unicode('v'), enigo::Direction::Click)
                .map_err(|e| {
                    let err_msg = format!("按下V键失败: {}", e);
                    println!("[clipboard_paste] ❌ {}", err_msg);
                    err_msg
                })?;

            println!("[clipboard_paste] 步骤3: 释放 Ctrl 键");
            enigo
                .key(Key::Control, enigo::Direction::Release)
                .map_err(|e| {
                    let err_msg = format!("释放Ctrl键失败: {}", e);
                    println!("[clipboard_paste] ❌ {}", err_msg);
                    err_msg
                })?;
        }

        println!("[clipboard_paste] ✅ 粘贴快捷键执行完成");
        println!("[clipboard_paste] ========== 剪贴板粘贴流程结束 ==========");
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| {
        let err_msg = format!("粘贴操作失败: {}", e);
        println!("[clipboard_paste] ❌ {}", err_msg);
        err_msg
    })?
}

// 仅复制到剪贴板
async fn clipboard_copy(text: &str) -> Result<(), String> {
    let text = text.to_string();
    println!(
        "[clipboard_copy] 复制文本到剪贴板，长度: {} 字符",
        text.len()
    );
    tokio::task::spawn_blocking(move || {
        let mut clipboard = Clipboard::new().map_err(|e| {
            let err_msg = format!("无法访问剪贴板: {}", e);
            println!("[clipboard_copy] ❌ {}", err_msg);
            err_msg
        })?;
        clipboard.set_text(text).map_err(|e| {
            let err_msg = format!("写入剪贴板失败: {}", e);
            println!("[clipboard_copy] ❌ {}", err_msg);
            err_msg
        })?;
        println!("[clipboard_copy] ✅ 文本已复制到剪贴板");
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| {
        let err_msg = format!("剪贴板操作任务失败: {}", e);
        println!("[clipboard_copy] ❌ {}", err_msg);
        err_msg
    })?
}
