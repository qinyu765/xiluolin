# CaptureSession 与跨应用文本投递复盘

## 1. 背景：语音输入不是“录音 + ASR”

桌面语音输入的最终价值不是拿到一段文本，而是：

> 用户在某个输入框中开始说话，几秒后文本稳定地回到那个输入框。

这条链路涉及：

- 全局快捷键；
- 录音生命周期；
- 焦点变化；
- 状态悬浮窗；
- ASR 和文本整理延迟；
- 目标应用恢复；
- 剪贴板；
- 键盘模拟；
- 失败兜底。

原实现完成了每个独立模块，但缺少一个贯穿全流程的会话对象。

## 2. 原始实现的问题

### 2.1 处理结束时才决定粘贴目标

原始输出命令只有：

```text
output_text(text)
```

它不知道：

- 录音从哪个窗口开始；
- 用户处理期间是否切换应用；
- 状态窗是否抢过焦点；
- 当前窗口是不是 XiLuoLin 自身。

结果是文本可能被粘贴到：

- XiLuoLin 主窗口；
- 用户后来打开的聊天窗口；
- 一个没有文本焦点的应用；
- 错误的终端或编辑器。

### 2.2 状态窗主动获取焦点

原 `show_indicator` 在窗口已存在时调用：

```rust
window.set_focus()
```

这与输入法类产品的目标完全相反。状态窗应该告诉用户当前状态，而不是成为新的输入目标。

### 2.3 状态窗创建太晚

首次快捷键按下时才创建 WebViewWindow，会带来：

- 首次显示延迟；
- 窗口闪烁；
- 页面资源路径错误；
- 更高概率影响焦点。

### 2.4 剪贴板被永久覆盖

原流程：

```text
写入生成文本
  → Ctrl/Cmd + V
  → 结束
```

用户原来复制的文本或图片会丢失。

### 2.5 处理阶段没有统一状态

前端和 Rust 分别维护若干布尔值：

- `isRecording`；
- `isProcessing`；
- `is_recording_via_hotkey`；
- indicator show/hide。

这些状态不能明确表达当前处于 ASR、整理还是投递，也无法验证合法跳转。

## 3. 设计目标

1. 录音开始时锁定目标，而不是处理结束时推测。
2. 原生窗口句柄只存在 Rust 内存中，不暴露给 WebView。
3. 一次语音输入只能有一个明确的会话 ID。
4. 状态机拒绝非法跳转和并发会话。
5. 状态窗不可聚焦、不可点击、启动时预创建。
6. 自动粘贴成功后恢复用户原剪贴板。
7. 失败时结果仍保留在剪贴板。
8. 应用内录音不应该错误粘贴到外部应用。

## 4. CaptureSession 状态机

状态定义：

```text
Recording
   ↓
Transcribing
   ↓
Refining
   ↓
Delivering
   ↓
Completed
```

任意处理中状态都可以进入：

```text
Failed
```

禁止的示例：

- `Recording → Delivering`；
- `Completed → Recording`；
- 当前会话未结束时创建第二个会话。

状态对象保存：

```text
CaptureSession
  ├─ id: UUID
  ├─ source: Hotkey | App
  ├─ status
  ├─ focus snapshot
  └─ history_id（历史写入后附加）
```

前端只收到 `session_id`，不能读取 HWND、PID 等原生数据。

## 5. 为什么使用 session_id

如果只传 `file_path`：

- 无法确认该文件属于哪个录音；
- 无法把输出结果关联到开始时的焦点；
- 无法关联历史 ID；
- 重复事件可能处理同一文件；
- 失败后无法准确清理状态。

引入 UUID 后：

```text
start_recording
  → session_id

stop_recording
  → session_id + file_path + duration

process_recording_file
  → session_id + result + history_id

deliver_text
  → session_id + final_text
```

整个链路可以验证“是不是同一次语音输入”。

## 6. Windows 焦点快照

### 6.1 捕获时机

必须在快捷键开始时捕获，而不是停止录音或处理完成时。

Windows 使用：

- `GetForegroundWindow`；
- `GetWindowThreadProcessId`。

如果目标窗口属于 XiLuoLin 自身，不保存为外部投递目标，最终走复制兜底。

### 6.2 恢复目标

处理完成后：

1. 检查 HWND 仍然有效；
2. 获取当前线程、前台线程和目标线程；
3. 必要时使用 `AttachThreadInput`；
4. 调用 `SetForegroundWindow`；
5. 解除线程输入关联；
6. 等待焦点稳定；
7. 发送粘贴快捷键。

### 6.3 为什么不能只调用 SetForegroundWindow

Windows 有 foreground lock 规则。后台进程直接请求前台经常只会让任务栏闪烁，而不会真正恢复焦点。

`AttachThreadInput` 不是绝对保证，但可以提高同权限窗口恢复成功率。

### 6.4 UIPI 限制

普通权限应用不能向提升权限窗口注入输入。这是 Windows 安全机制，不应该通过无限重试规避。

正确行为：

```text
尝试恢复/粘贴
  → 失败
  → 生成文本留在剪贴板
  → 明确提示手动粘贴
```

## 7. macOS 策略

本轮优先保证：

- 状态窗不抢焦点；
- 当前活动窗口可以接收 Cmd+V；
- 无法粘贴时保留剪贴板。

尚未实现：

- 录音开始时的应用快照；
- NSRunningApplication 恢复；
- Accessibility 权限精确读取；
- Input Monitoring 精确读取。

这些属于后续平台增强，不能在文档中宣称已与 Windows 等价。

## 8. 剪贴板保护

### 8.1 备份内容

自动粘贴前尝试备份：

1. 文本；
2. 如果没有文本，备份图片。

arboard 无法保留任意应用私有剪贴板格式，因此当前保证文本和图片，不保证所有 MIME/自定义格式。

### 8.2 成功路径

```text
读取原剪贴板
  → 写入生成文本
  → 恢复目标窗口
  → 发送 Ctrl/Cmd + V
  → 等待目标应用读取
  → 恢复原剪贴板
```

### 8.3 失败路径

如果目标恢复或键盘模拟失败：

- 不恢复原剪贴板；
- 确保生成文本仍在剪贴板；
- 返回 `Manual`/fallback 结果；
- 用户仍可手动粘贴。

这里的优先级是：

> 先保证生成结果不丢失，再考虑恢复原剪贴板。

## 9. deliver_text 接口

旧接口：

```text
output_text(text)
```

新接口：

```text
deliver_text(session_id?, history_id?, text)
```

行为：

### 有快捷键 session

- 恢复目标；
- 自动粘贴；
- 更新历史 delivery_method；
- 完成 session。

### 应用内 session

- 不向外部应用粘贴；
- 复制结果；
- 完成 session。

### 没有 session

常用于上传音频后的“复制”按钮：

- 只写入剪贴板；
- 如果有 history_id，更新投递方式为 copy。

## 10. 状态悬浮窗

### 10.1 资源路径问题

原资源位于 `src-tauri/indicator.html`，开发与生产加载方式不同，打包路径长期未验证。

最终移动到：

```text
public/indicator.html
```

由 Vite：

- 开发时服务；
- 构建时复制到 `dist`。

### 10.2 预创建

应用 setup 阶段创建隐藏窗口：

```text
visible(false)
focusable(false)
ignore_cursor_events(true)
always_on_top(true)
```

快捷键触发时只更新状态并 show，不再创建和聚焦。

### 10.3 状态

- Recording：正在录音；
- Transcribing：正在识别；
- Refining：正在整理；
- Delivering：正在输入；
- Completed：输入完成；
- Failed：处理失败。

### 10.4 延迟隐藏竞争

完成状态显示约 900ms 后隐藏。

问题：如果用户很快开始下一次录音，旧会话的延迟任务可能把新状态窗隐藏。

解决：使用原子 revision。

```text
每次状态更新 revision + 1
完成时记录当前 revision
延迟结束后只有 revision 未变化才隐藏
```

## 11. 并发与重复事件问题

### 11.1 React StrictMode

开发环境可能重复注册 effect。已有前端处理标志用于跳过重复完成事件，但后端 session 仍是最终防线。

### 11.2 处理中再次按快捷键

最初实现中，新录音启动失败后统一调用 `cancel_current()`。

如果失败原因是“上一条仍在处理”，这会错误取消正在处理的旧 session。

修复：

- “上一条仍处理中”只忽略新请求；
- 不取消旧 session；
- 长按启动失败后，松键事件先检查是否真的在录音。

### 11.3 失败恢复

- 录音启动失败：创建的 session 立即取消；
- 停止失败：清理当前 session；
- 处理失败：标记 Failed；
- 前端异常：调用 `abort_capture_session`；
- 投递失败：如果复制兜底成功，则 session 仍按 Completed 结束，但标记 fallback。

## 12. 测试

自动化测试覆盖：

- 合法状态顺序；
- 非法跳转；
- 并发 session 拒绝；
- session 完成后不可再读取；
- source/focus/history 只保存在 Rust；
- 状态集合完整；
- 指示器构建产物存在。

手动/平台测试：

- macOS Tauri 启动；
- Windows Rust 编译；
- Windows 普通窗口粘贴；
- 目标窗口关闭后的复制降级；
- 提升权限窗口 UIPI 降级。

## 13. 已知限制

- Windows `SetForegroundWindow` 成功不代表目标控件一定接受粘贴；
- enigo 不能确认目标应用是否真的消费 Ctrl+V；
- macOS 暂未恢复开始时窗口；
- 剪贴板只恢复文本或图片；
- 没有系统级输入法光标上下文；
- 状态窗没有可交互错误详情，因为它故意忽略鼠标。

## 14. 后续延伸

### 权限与焦点

- macOS AX/NSWorkspace 焦点快照；
- Windows UI Automation 控件类型检测；
- 检测焦点是否位于文本控件；
- UAC 权限级别说明。

### 输出可靠性

- 粘贴确认或目标应用适配；
- 富文本/多格式剪贴板备份；
- 可配置粘贴等待时长；
- 取消和超时状态。

### 可观察性

- 每阶段耗时结构化记录；
- session 级错误代码；
- 不含用户内容的诊断导出。

## 15. 相关文件

- `src-tauri/src/capture_session.rs`
- `src-tauri/src/focus_capture.rs`
- `src-tauri/src/recording.rs`
- `src-tauri/src/pipeline.rs`
- `src-tauri/src/output.rs`
- `src-tauri/src/indicator.rs`
- `public/indicator.html`
- `src/main.tsx`
