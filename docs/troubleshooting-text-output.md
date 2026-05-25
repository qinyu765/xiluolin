# 光标位置文本输出问题排查指南

## 问题描述

使用 `enigo` 库在 Windows 11 上进行键盘注入时，文本无法输出到光标位置。

## 可能的原因

### 1. Windows UIPI 权限隔离

**问题**：Windows 的用户界面特权隔离（User Interface Privilege Isolation, UIPI）会阻止非提权应用向提权应用注入输入。

**表现**：
- 在普通应用（如记事本、浏览器）中可以正常输入
- 在管理员权限运行的应用（如 VS Code、PowerShell）中无法输入
- 没有错误提示，输入静默失败

**解决方案**：
- **方案 A**：以管理员权限运行本应用
- **方案 B**：降低目标应用的权限（不以管理员身份运行）
- **方案 C**：使用剪贴板粘贴作为降级方案（已实现）

**参考资料**：
- [SendInput fails on UAC prompt - Stack Overflow](https://stackoverflow.com/questions/56595640/sendinput-fails-on-uac-prompt)
- [LowLevelKeyboardProc and elevated applications](https://stackoverflow.com/questions/52696285/lowlevelkeyboardproc-being-called-for-elevated-applications-when-run-as-non-elev)

### 2. 焦点和时序问题

**问题**：在异步环境中，文本注入可能在目标窗口失去焦点后执行。

**表现**：
- 偶尔能输入，偶尔不能
- 快速切换窗口后输入失败
- 在 Tauri 窗口关闭/最小化后输入失败

**解决方案**：
- 在 `keyboard_inject` 函数中添加 100ms 延迟（已实现）
- 确保在调用 `output_text` 前目标窗口已获得焦点
- 考虑在前端添加"点击目标位置后再录音"的提示

### 3. enigo 版本兼容性

**当前版本**：`enigo = "0.2"`

**已知问题**：
- Tauri + enigo 在某些平台上有崩溃问题（主要是 macOS）
- enigo 0.2 是较旧的版本，可能存在 Windows 11 兼容性问题

**解决方案**：
- 考虑升级到最新版本（需要 API 迁移）
- 监控 [enigo GitHub issues](https://github.com/enigo-rs/enigo/issues)

**参考资料**：
- [[bug] use `enigo` in tauri cause app crashed · Issue #6421](https://github.com/tauri-apps/tauri/issues/6421)

### 4. Windows 11 SendInput 限制

**问题**：Windows 11 对 SendInput API 有更严格的限制，某些应用可能拒绝接收模拟输入。

**表现**：
- 在特定应用（如 UWP 应用、某些游戏）中无法输入
- 在 Windows 安全模式下无法输入

**解决方案**：
- 使用剪贴板粘贴作为降级方案（已实现）
- 提示用户手动粘贴（已实现）

**参考资料**：
- [SendInput text is "halted" in Win 11 Notepad](https://www.autohotkey.com/boards/viewtopic.php?t=118816)

## 当前实现的降级策略

代码已实现三级降级策略：

```rust
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
```

## 排查步骤

### 步骤 1：确认基本功能

1. 打开记事本（非管理员权限）
2. 点击记事本窗口，确保光标在文本区域
3. 使用快捷键录音并识别
4. 观察是否能自动输入

**预期结果**：应该能正常输入
**如果失败**：继续步骤 2

### 步骤 2：检查权限问题

1. 右键点击目标应用，查看是否以管理员身份运行
2. 如果是，关闭后以普通权限重新打开
3. 重复步骤 1

**预期结果**：应该能正常输入
**如果失败**：继续步骤 3

### 步骤 3：测试剪贴板降级

1. 打开任意文本编辑器
2. 使用快捷键录音并识别
3. 观察提示信息：
   - "已自动输入到光标位置" → 键盘注入成功
   - "已通过剪贴板输入" → 键盘注入失败，剪贴板粘贴成功
   - "已复制到剪贴板,请手动粘贴" → 两种方式都失败

**如果显示第三种提示**：
- 手动按 Ctrl+V，确认剪贴板中有内容
- 如果有内容，说明识别成功，只是输出失败
- 如果没有内容，说明剪贴板访问也失败

### 步骤 4：检查 Tauri 权限配置

查看 `src-tauri/capabilities/default.json`，确认没有遗漏必要的权限。

当前配置：
```json
{
  "permissions": [
    "opener:default",
    "sql:default",
    "store:default",
    "core:event:allow-listen",
    "core:event:allow-emit"
  ]
}
```

注意：enigo 和 arboard 是 Rust 原生库，不需要 Tauri 权限配置。

### 步骤 5：查看控制台日志

打开开发者工具（F12），查看控制台输出：

```
开始自动输出文本到光标位置...
✅ 文本输出完成: { method: "keyboard", success: true, message: "..." }
```

如果看到错误信息，记录完整的错误堆栈。

## 临时解决方案

如果键盘注入持续失败，可以：

1. **使用剪贴板模式**：识别完成后手动按 Ctrl+V
2. **以管理员权限运行应用**：右键应用图标 → "以管理员身份运行"
3. **降低目标应用权限**：不以管理员身份运行目标应用

## 长期优化方向

1. **升级 enigo 版本**：迁移到最新版本，获得更好的 Windows 11 支持
2. **添加焦点检测**：在输入前检测目标窗口是否有焦点
3. **用户配置选项**：允许用户选择默认输出方式（键盘/剪贴板/手动）
4. **更详细的错误提示**：区分不同的失败原因，给出针对性建议

## 社区参考

- [enigo GitHub 仓库](https://github.com/enigo-rs/enigo)
- [enigo 文档](https://docs.rs/enigo)
- [Tauri 全局快捷键文档](https://tauri.app/v1/guides/features/global-shortcut/)
- [Windows SendInput API 文档](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)

## 更新日志

- 2026-05-25：添加 100ms 延迟以改善焦点时序问题
- 2026-05-25：创建本排查文档
