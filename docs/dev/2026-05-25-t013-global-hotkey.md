# T013 实现全局快捷键模块

> **归档说明：** 本文记录特定开发阶段的背景与决策，其中的 MVP、demo、比赛或旧分支流程表述仅用于保留历史，不代表 XiLuoLin 当前的开源项目定位与协作方式。当前信息请以根目录 `README.md`、`CONTRIBUTING.md` 和 `docs/roadmap.md` 为准。

## 任务目标

实现全局快捷键注册和监听,支持长按录音和切换式录音两种模式,让用户可以在任何应用中通过快捷键触发语音输入。

## 实际改动

### 1. 添加依赖

在 `src-tauri/Cargo.toml` 中添加:
```toml
tauri-plugin-global-shortcut = "2.0.0"
```

### 2. 创建快捷键模块

创建 `src-tauri/src/hotkey.rs`,实现:

**HotkeyState 状态管理**:
- `is_registered`: 是否已注册快捷键
- `current_shortcut`: 当前快捷键字符串
- `recording_mode`: 录音模式(长按/切换)
- `is_recording_via_hotkey`: 跟踪通过快捷键触发的录音状态

**Tauri 命令**:
- `register_hotkey`: 注册全局快捷键,支持重新注册
- `unregister_hotkey`: 注销全局快捷键

**录音模式**:
- **长按模式**: 按下快捷键开始录音,松开快捷键停止录音
- **切换模式**: 第一次按下开始录音,第二次按下停止录音

### 3. 注册插件和命令

在 `src-tauri/src/lib.rs` 中:
- 添加 `mod hotkey`
- 注册 `tauri-plugin-global-shortcut` 插件
- 在 `setup` 中初始化 `HotkeyState` 并从配置读取默认快捷键
- 注册 `register_hotkey` 和 `unregister_hotkey` 命令

### 4. 与录音模块集成

快捷键事件处理器直接调用 `recording::start_recording` 和 `recording::stop_recording`,并通过 `app.emit` 发送事件通知前端:
- `recording-completed`: 录音完成,携带 `RecordingResult`
- `recording-error`: 录音失败,携带错误信息

## 为什么这么做

### 技术选型

**选择 `tauri-plugin-global-shortcut`**:
- Tauri 官方插件,与 Tauri 2.x 生态集成良好
- 提供跨平台全局快捷键支持(Windows/macOS/Linux)
- 支持按下和松开事件监听,满足长按模式需求

### 架构设计

**独立的状态跟踪**:
- `RecordingState` 的 `is_recording` 字段是私有的,无法直接访问
- 在 `HotkeyState` 中维护 `is_recording_via_hotkey` 字段
- 避免修改已有的 `recording.rs` 模块

**事件驱动通信**:
- 使用 `app.emit` 发送事件,而不是直接调用前端
- 前端可以监听事件并更新 UI 状态
- 解耦后端和前端,便于后续扩展

**启动时自动注册**:
- 在 `setup` 中从配置读取快捷键并自动注册
- 用户无需手动触发注册,开箱即用

## 涉及文件

- 新增: `src-tauri/src/hotkey.rs` (220 行)
- 修改: `src-tauri/src/lib.rs` (+28 行)
- 修改: `src-tauri/Cargo.toml` (+1 行)
- 修改: `docs/dev/2026-05-25-t013-global-hotkey.md`
- 修改: `docs/dev/task-tracker.md` (状态 Todo → Review)

## 测试与验证

### 编译验证

```bash
cargo check
```

**结果**: ✅ 通过

### 前端构建验证

```bash
pnpm build
```

**结果**: ✅ 通过

### 手动测试计划

由于快捷键功能需要在运行时测试,以下是手动测试计划:

1. **启动应用测试**:
   - 运行 `pnpm tauri dev`
   - 检查应用是否正常启动
   - 检查控制台是否有快捷键注册相关错误

2. **默认快捷键测试**:
   - 按下默认快捷键 `Ctrl+Shift+Space` (Windows/Linux) 或 `Cmd+Shift+Space` (macOS)
   - 检查是否触发录音

3. **长按模式测试**:
   - 在设置中选择长按模式
   - 按住快捷键,检查是否开始录音
   - 松开快捷键,检查是否停止录音

4. **切换模式测试**:
   - 在设置中选择切换模式
   - 第一次按下快捷键,检查是否开始录音
   - 第二次按下快捷键,检查是否停止录音

5. **快捷键冲突测试**:
   - 尝试注册一个已被系统或其他应用占用的快捷键
   - 检查是否返回错误提示

6. **快捷键重新注册测试**:
   - 修改快捷键配置
   - 检查旧快捷键是否失效
   - 检查新快捷键是否生效

## 执行复盘

### 遇到的问题

**问题 1: API 版本不匹配**

初始代码基于计划中的 API 设计,但实际 `tauri-plugin-global-shortcut` 2.3.1 的 API 与预期不同:
- 回调函数签名是 `|app, shortcut, event|` 而不是 `|app, event|`
- `unregister` 需要 `Shortcut` 对象而不是字符串

**解决方案**: 查看编译错误,调整代码以匹配实际 API。

**问题 2: `RecordingState` 字段私有**

`RecordingState` 的 `is_recording` 字段是私有的,无法在 `hotkey.rs` 中直接访问。

**解决方案**: 在 `HotkeyState` 中维护独立的 `is_recording_via_hotkey` 字段,在启动和停止录音时同步更新。

**问题 3: `tokio::sync::Mutex` 不返回 `Result`**

初始代码使用 `if let Ok(state) = mutex.lock().await`,但 `tokio::sync::Mutex::lock()` 直接返回 `MutexGuard`,不是 `Result`。

**解决方案**: 移除 `if let Ok(...)` 包装,直接使用 `let state = mutex.lock().await`。

### 经验总结

1. **先编译再调整**: 不要假设 API,先尝试编译,根据错误信息调整代码。
2. **查看实际依赖版本**: `Cargo.lock` 中的实际版本可能与 `Cargo.toml` 中指定的不同。
3. **状态同步策略**: 当无法访问私有字段时,维护独立的状态副本是可行的方案。

## 未完成事项

1. **前端集成**: 需要在设置页添加快捷键配置界面(T020)
2. **事件监听**: 需要在前端监听 `recording-completed` 和 `recording-error` 事件(T015)
3. **实际运行测试**: 需要在 `pnpm tauri dev` 环境中进行手动测试
4. **跨平台测试**: 当前只在 Windows 上验证编译,需要在 macOS 和 Linux 上测试

## 后续建议

1. **添加快捷键冲突检测**: 在注册失败时,提供更友好的错误提示,建议用户更换快捷键
2. **支持多快捷键**: 未来可以支持为不同功能注册不同快捷键(如录音、停止、取消)
3. **快捷键可视化**: 在 UI 中显示当前注册的快捷键,方便用户确认
4. **热重载支持**: 配置更新后自动重新注册快捷键,无需重启应用
5. **添加公开方法**: 考虑在 `RecordingState` 中添加 `pub fn is_recording(&self) -> bool` 方法,避免状态重复维护

