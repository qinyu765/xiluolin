# T031 建立跨应用 CaptureSession 和可靠文本投递

## 任务目标

为快捷键语音输入建立统一 CaptureSession 状态，锁定录音开始时的目标窗口，避免录音状态窗抢焦点，并在自动粘贴成功后恢复用户原剪贴板。

## 实际改动

- 新增 Rust CaptureSession 状态，使用 UUID `session_id` 关联录音、处理和文本投递；原生窗口句柄仅保存在 Rust 内存中。
- 建立 `Recording → Transcribing → Refining → Delivering → Completed / Failed` 状态机并拒绝非法跳转或并发会话。
- Windows 快捷键录音开始时保存外部目标窗口；如果目标是 XiLuoLin 自身，则不执行外部自动粘贴。
- 录音开始和完成事件增加 `session_id`，`process_recording_file` 按会话更新识别和整理状态。
- 使用 `deliver_text(session_id, text)` 替代无目标信息的 `output_text(text)`：恢复目标窗口、写入剪贴板并模拟粘贴。
- 自动粘贴前备份文本或图片剪贴板；成功后恢复，失败时保留生成文本供手动粘贴。
- 应用内按钮录音完成后只复制结果，不向外部窗口发送粘贴快捷键。
- 状态窗移动到 `public/indicator.html`，由 Vite 统一提供；窗口在启动时预创建、不可聚焦、忽略鼠标，并显示录音、识别、整理、输入、完成和失败状态。
- 延迟隐藏使用 revision 防止旧会话的定时任务误隐藏新会话状态窗。
- 处理异常时可显式终止 CaptureSession，避免失败会话阻塞下一次录音。

## 为什么这么做

此前自动输出只向处理完成时的活动窗口发送粘贴快捷键，状态窗首次显示还会主动获取焦点。处理期间发生焦点变化时，文本可能进入错误窗口。同时自动粘贴会覆盖用户原剪贴板，录音、转写、整理和投递阶段也缺少统一状态。

本任务将目标窗口和状态生命周期收口到 Rust CaptureSession，使前端只持有不透明 `session_id`，并通过复制兜底保证目标窗口恢复或系统键盘注入失败时不丢失结果。

## 涉及文件

- `src-tauri/src/capture_session.rs`
- `src-tauri/src/focus_capture.rs`
- `src-tauri/src/recording.rs`
- `src-tauri/src/pipeline.rs`
- `src-tauri/src/output.rs`
- `src-tauri/src/indicator.rs`
- `public/indicator.html`
- `src/main.tsx`
- `src/types/voice.ts`
- `README.md`
- `docs/requirements-analysis.md`
- `docs/solution-design.md`

## 测试与验证

执行：

```bash
pnpm check
```

结果：

- TypeScript 类型检查通过。
- Vite production build 通过，并将 `public/indicator.html` 复制到构建目录。
- Rust 格式检查和 macOS `cargo check` 通过。
- Rust 测试共 37 个通过，其中新增 4 个 CaptureSession 状态测试和 1 个状态窗状态集合测试。
- `git diff --check` 通过。
- `pnpm tauri dev` 在 macOS 启动成功，状态窗预创建、配置读取和全局快捷键注册未出现启动错误。

覆盖场景：

1. CaptureSession 按合法状态顺序完成。
2. 上一会话未结束时拒绝创建第二会话。
3. 非法状态跳转被拒绝。
4. 前端只能取得 `session_id`，目标窗口信息不对外序列化。
5. 处理失败或前端异常时可以取消会话。
6. 状态窗覆盖录音、识别、整理、输入、完成和失败状态。

待 Windows 验证：

- Windows CI 编译和自动化测试。
- 普通权限窗口目标恢复、自动粘贴和剪贴板恢复。
- 目标窗口关闭后的复制降级。
- 提升权限窗口受 UIPI 阻止时的复制降级。

## 执行复盘

### 遇到的问题

1. 同一工作区同时存在另一个开源治理任务，分支和大量文档在实施过程中发生并发变化。
2. macOS 本机添加 Windows Rust target 后，交叉编译被缺少 MSVC C 头文件阻断，`ring` 无法找到 `assert.h`。
3. 状态窗原本从 `src-tauri/indicator.html` 加载并在显示时调用 `set_focus()`，既存在打包路径不一致，也会破坏目标焦点。
4. 延迟隐藏完成状态若不区分会话，可能在用户快速开始下一次录音时误隐藏新状态。

### 解决流程

1. 将 T031 隔离到独立 worktree 和临时功能分支，只迁移语音输入相关文件，避免混入另一任务的文档改动。
2. 使用本地 macOS 完整检查保证通用代码质量，Windows 原生 API 留给 Windows CI 验证。
3. 将状态窗资源移动到 Vite `public` 目录，启动时预创建并设置为不可聚焦。
4. 使用原子 revision 标记状态更新，只有没有新状态时才执行延迟隐藏。
5. 使用 CaptureSession 保存目标窗口并以 `session_id` 串联 Rust 和前端。

### 经验总结

- 跨应用输入的关键不是“发送一次 Ctrl+V”，而是从录音开始就保存目标和会话上下文。
- 状态悬浮窗必须默认不可聚焦，否则它本身会破坏输入目标。
- 自动粘贴成功后恢复剪贴板，失败时保留生成文本，才能同时保证体验和数据安全。
- 同一仓库发生并发任务时应使用 worktree 隔离，不能继续在共享工作区混合修改。

## 未完成事项

- Windows 原生焦点恢复和 UIPI 降级需要在 Windows runner 和真实桌面环境验证；macOS 本机尝试 Windows target 交叉检查时因缺少 MSVC C 头文件而无法替代 Windows runner。
- macOS 当前不恢复录音开始时的原目标窗口，只保证状态窗不抢焦点并向当前活动窗口粘贴。
- 自动化测试无法可靠判断第三方应用是否真正接收了模拟粘贴，需要手动验证。

## 后续建议

执行 T032，增加麦克风、快捷键、模型配置和自动粘贴能力的就绪检查与设置页提示。
