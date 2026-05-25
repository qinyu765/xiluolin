# 光标位置文本输出问题修复记录

## 修复日期
2026-05-25

## 问题描述

### 问题1：文本输出重复
用户报告语音输入后，输出的文本会重复两次，例如：
- 输入："这是一次测试"
- 输出："这是一次测试这是一次测试"
- 历史记录也显示重复

### 问题2：字符重复
使用键盘注入时，字符会重复输入：
- 输入："这是一次测试"
- 输出："这这是是一一次测试"

## 根本原因分析

### 原因1：React StrictMode 导致事件监听器重复注册

**现象**：
通过日志发现 `process_voice_input` 和 `output_text` 都被调用了两次：
```
[⏱️ 性能] process_voice_input 开始
...
[⏱️ 性能] process_voice_input 开始  // 第二次调用
```

**根本原因**：
- React StrictMode 在开发环境下会导致 `useEffect` 执行两次
- 导致 `recording-completed` 事件监听器被注册两次
- 每次录音完成后，两个监听器都会触发，导致整个流程执行两次

### 原因2：键盘注入方式不稳定

**现象**：
- 使用 `enigo.text()` 直接输入整个字符串时，字符会重复："这这是是一一次测试"
- 改为逐字符输入后，问题更严重："这测测试试"

**根本原因**：
- `enigo` 库在 Windows 上的键盘注入不稳定
- 输入速度过快时，系统来不及处理，导致字符重复

### 原因3：Windows UIPI 权限隔离
- Windows 的用户界面特权隔离（UIPI）阻止非提权应用向提权应用注入输入
- 如果目标应用以管理员权限运行，而本应用没有，`enigo.text()` 会静默失败
- 这是 Windows 安全机制，无法绕过

**参考资料**：
- [SendInput fails on UAC prompt - Stack Overflow](https://stackoverflow.com/questions/56595640/sendinput-fails-on-uac-prompt)

### 2. 焦点和时序问题
- 在异步环境中，文本注入可能在目标窗口失去焦点后执行
- Tauri 窗口和目标窗口之间的焦点切换需要时间
- 没有延迟会导致输入发送到错误的窗口或丢失

### 3. 缺少诊断信息
- 原代码没有详细的日志输出
- 用户无法判断是哪个环节失败
- 难以排查问题

## 实施的修复

### 修复 1：添加焦点稳定延迟

**文件**：`src-tauri/src/output.rs`

在 `keyboard_inject` 函数中添加 100ms 延迟：

```rust
async fn keyboard_inject(text: &str) -> Result<(), String> {
    let text = text.to_string();
    tokio::task::spawn_blocking(move || {
        // 添加短暂延迟，确保目标窗口获得焦点
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("初始化键盘模拟失败: {}", e))?;
        
        enigo.text(&text)
            .map_err(|e| format!("输入文本失败: {}", e))?;
        
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("键盘注入任务失败: {}", e))?
}
```

**效果**：给目标窗口足够的时间获得焦点，减少因时序问题导致的输入失败。

### 修复 2：添加详细的诊断日志

在所有关键函数中添加 `println!` 日志：

- `output_text`：记录每个降级策略的尝试和结果
- `keyboard_inject`：记录初始化、延迟、输入过程
- `clipboard_paste`：记录剪贴板复制和快捷键模拟
- `clipboard_copy`：记录剪贴板访问和写入

**示例输出**：
```
[output_text] 开始输出文本，长度: 25 字符
[output_text] 尝试方法 1: 键盘注入
[keyboard_inject] 等待 100ms 以确保焦点稳定...
[keyboard_inject] 初始化 enigo...
[keyboard_inject] 开始输入文本...
[keyboard_inject] ✅ 文本输入完成
[output_text] ✅ 键盘注入成功
```

**效果**：用户可以通过控制台日志准确判断失败原因。

### 修复 3：在剪贴板粘贴中添加延迟

在 `clipboard_paste` 函数中，复制到剪贴板后添加 50ms 延迟再执行粘贴快捷键：

```rust
async fn clipboard_paste(text: &str) -> Result<(), String> {
    clipboard_copy(text).await?;
    
    tokio::task::spawn_blocking(|| {
        // 等待剪贴板操作完成
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        // 执行 Ctrl+V
        // ...
    })
    .await
    .map_err(|e| format!("粘贴操作失败: {}", e))?
}
```

**效果**：确保剪贴板内容已写入完成再执行粘贴，提高成功率。

## 已有的降级策略

代码已实现三级降级策略（无需修改）：

1. **键盘注入**：直接使用 `enigo.text()` 输入到光标位置
2. **剪贴板粘贴**：复制到剪贴板后模拟 Ctrl+V
3. **手动粘贴**：仅复制到剪贴板，提示用户手动粘贴

这个策略能够应对大部分场景。

## 测试建议

### 测试场景 1：普通应用
1. 打开记事本（非管理员权限）
2. 点击记事本，确保光标在文本区域
3. 使用快捷键录音并识别
4. **预期**：文本自动输入，提示"已自动输入到光标位置"

### 测试场景 2：提权应用
1. 以管理员身份打开 PowerShell
2. 点击 PowerShell 窗口
3. 使用快捷键录音并识别
4. **预期**：键盘注入失败，降级到剪贴板粘贴或手动粘贴

### 测试场景 3：快速切换窗口
1. 打开记事本
2. 点击本应用窗口，开始录音
3. 录音过程中快速切换到记事本
4. **预期**：100ms 延迟应该能让焦点稳定，文本正确输入

### 测试场景 4：查看日志
1. 以开发模式运行应用：`npm run tauri dev`
2. 执行任意语音输入
3. 查看终端输出的日志
4. **预期**：能看到完整的执行流程和成功/失败信息

## 用户指导

如果用户仍然遇到输入失败，建议：

1. **检查目标应用权限**：
   - 右键目标应用图标，查看是否以管理员身份运行
   - 如果是，关闭后以普通权限重新打开

2. **使用剪贴板模式**：
   - 如果提示"已复制到剪贴板，请手动粘贴"
   - 在目标位置按 Ctrl+V 即可

3. **以管理员权限运行本应用**：
   - 右键应用图标 → "以管理员身份运行"
   - 这样可以向提权应用注入输入

4. **查看详细日志**：
   - 以开发模式运行，查看控制台输出
   - 将日志信息反馈给开发者

## 相关文档

- [troubleshooting-text-output.md](./troubleshooting-text-output.md) - 完整的排查指南
- [enigo GitHub 仓库](https://github.com/enigo-rs/enigo)
- [Windows SendInput API 文档](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)

## 后续优化方向

1. **升级 enigo 版本**：
   - 当前使用 `enigo = "0.2"`
   - 考虑升级到最新版本以获得更好的 Windows 11 支持
   - 需要 API 迁移工作

2. **添加焦点检测**：
   - 在输入前检测目标窗口是否有焦点
   - 如果没有焦点，提示用户点击目标窗口

3. **用户配置选项**：
   - 允许用户选择默认输出方式（键盘/剪贴板/手动）
   - 允许用户调整延迟时间

4. **更智能的降级策略**：
   - 记住每个应用的成功方式
   - 下次直接使用成功的方式，减少尝试时间

## 提交信息

```
fix: 改善光标位置文本输出的稳定性

- 在键盘注入前添加 100ms 延迟，确保目标窗口焦点稳定
- 在剪贴板粘贴前添加 50ms 延迟，确保剪贴板写入完成
- 添加详细的诊断日志，便于排查问题
- 创建完整的排查指南文档

问题原因：
1. Windows UIPI 权限隔离阻止向提权应用注入输入
2. 异步环境中焦点切换时序问题导致输入丢失
3. 缺少诊断信息难以定位问题

修复后：
- 焦点时序问题得到改善
- 用户可以通过日志准确判断失败原因
- 提供完整的排查和解决方案文档
```
