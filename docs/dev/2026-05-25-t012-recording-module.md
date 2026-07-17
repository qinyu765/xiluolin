# T012 实现录音模块

> **归档说明：** 本文记录特定开发阶段的背景与决策，其中的 MVP、demo、比赛或旧分支流程表述仅用于保留历史，不代表 XiLuoLin 当前的开源项目定位与协作方式。当前信息请以根目录 `README.md`、`CONTRIBUTING.md` 和 `docs/roadmap.md` 为准。

## 任务目标

在 `dev` 分支上实现麦克风录音开始、停止、音频文件生成、录音时长统计和状态通知功能，为语音输入主流程提供音频采集能力。

## 实际改动

### 1. 新增依赖

在 `src-tauri/Cargo.toml` 中添加：

- `cpal = "0.15"`：跨平台音频库，用于麦克风音频采集
- `hound = "3.5"`：WAV 文件写入库
- `chrono = "0.4"`：时间戳生成
- `tokio = { version = "1", features = ["time"] }`：异步运行时，用于延迟等待

### 2. 新增录音模块

创建 `src-tauri/src/recording.rs`，实现：

- `RecordingState`：全局录音状态管理，包含录音标志、开始时间、输出路径
- `start_recording()`：启动麦克风录音
  - 获取默认音频输入设备
  - 创建 WAV 文件写入器
  - 构建音频流并启动录音
  - 支持 F32、I16、U16 三种采样格式
  - 返回 `recording_started` 状态
- `stop_recording()`：停止录音
  - 计算录音时长
  - 返回音频文件路径和时长
  - 重置录音状态

### 3. 注册 Tauri 命令

在 `src-tauri/src/lib.rs` 中：

- 添加 `pub mod recording;`
- 在 `Builder` 中添加 `.manage(recording::RecordingState::new())`
- 在 `invoke_handler` 中注册 `recording::start_recording` 和 `recording::stop_recording`

## 为什么这么做

### 技术选型

- **cpal**：Rust 生态中成熟的跨平台音频库，支持 Windows、macOS、Linux，API 简洁
- **hound**：专注 WAV 格式，轻量且稳定，适合短音频录音场景
- **WAV 格式**：无损、兼容性好，智谱 GLM-ASR-2512 和 OpenAI 均支持

### 架构设计

- **全局状态管理**：使用 `RecordingState` 保存录音状态，避免多次录音冲突
- **异步命令**：`start_recording` 和 `stop_recording` 均为异步命令，避免阻塞主线程
- **MutexGuard 作用域控制**：在 `stop_recording` 中使用代码块限制 `MutexGuard` 生命周期，避免跨 await 边界导致 `Send` trait 错误

### 当前实现的权衡

- **stream 生命周期管理**：当前使用 `std::mem::forget(stream)` 保持录音流持续运行，这是临时方案。后续需要改进为使用全局状态管理 stream，以便在 `stop_recording` 时正确关闭流
- **音频数据写入延迟**：在 `stop_recording` 中等待 100ms 确保所有音频数据已写入文件，这是保守策略

## 涉及文件

- `src-tauri/Cargo.toml`：添加 cpal、hound、chrono、tokio 依赖
- `src-tauri/src/recording.rs`：新增录音模块
- `src-tauri/src/lib.rs`：注册录音模块和 Tauri 命令

## 测试与验证

### 编译验证

```bash
cd src-tauri
cargo check
```

结果：✅ 通过

### 前端构建验证

```bash
pnpm build
```

结果：✅ 通过，生成 346.42 kB 的前端资源

### 手动录音测试

由于当前任务只实现后端录音能力，前端 UI 尚未实现，手动录音测试需要等待 T015（实现主界面）完成后进行。

预期测试步骤：

1. 启动应用 `pnpm tauri dev`
2. 点击录音按钮，调用 `start_recording`
3. 说话 5-10 秒
4. 点击停止按钮，调用 `stop_recording`
5. 检查返回的文件路径是否存在
6. 播放录音文件，验证音频内容正确

## 执行复盘

### 遇到的问题

1. **缺少 `use tauri::Manager` 导入**
   - 错误：`no method named 'path' found for struct 'AppHandle'`
   - 原因：`app_handle.path()` 方法来自 `Manager` trait
   - 解决：添加 `use tauri::Manager;`

2. **MutexGuard 跨 await 边界导致 Send trait 错误**
   - 错误：`future is not Send as this value is used across an await`
   - 原因：`MutexGuard` 不是 `Send`，不能跨 await 边界持有
   - 解决：使用代码块限制 `MutexGuard` 生命周期，在 await 之前释放锁

3. **格式化字符串缺少占位符**
   - 错误：`argument never used` in `format!("Failed to lock start time: ", e)`
   - 解决：修正为 `format!("Failed to lock start time: {}", e)`

### 执行顺序

1. 添加依赖 → 2. 创建录音模块 → 3. 注册命令 → 4. 修复编译错误 → 5. 验证构建

## 未完成事项

1. **stream 生命周期管理优化**：当前使用 `std::mem::forget(stream)` 是临时方案，后续需要改进为全局状态管理
2. **麦克风权限检查**：当前未实现权限检查，需要在后续任务中添加
3. **录音状态事件通知**：当前未实现向前端发送录音中、处理中、失败等状态事件，需要在 T015（实现主界面）中配合实现
4. **手动录音测试**：需要等待 T015 完成后进行完整的端到端测试

## 后续建议

1. **T013（实现全局快捷键模块）**：可以与录音模块配合，实现快捷键触发录音
2. **T015（实现主界面）**：需要实现录音按钮和状态展示，调用 `start_recording` 和 `stop_recording`
3. **优化 stream 管理**：考虑使用 `Arc<Mutex<Option<Stream>>>` 保存 stream，在 `stop_recording` 时正确关闭
4. **添加录音时长限制**：MVP 阶段建议限制单次录音时长（如 60 秒），避免长音频处理
5. **添加音频格式配置**：后续可以支持用户配置采样率、声道数等参数
