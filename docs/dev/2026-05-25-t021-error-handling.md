# T021: 实现错误处理和兜底

> **归档说明：** 本文记录特定开发阶段的背景与决策，其中的 MVP、demo、比赛或旧分支流程表述仅用于保留历史，不代表 XiLuoLin 当前的开源项目定位与协作方式。当前信息请以根目录 `README.md`、`CONTRIBUTING.md` 和 `docs/roadmap.md` 为准。

**任务 ID**: T021  
**优先级**: P0  
**状态**: Review  
**分支**: dev  
**创建时间**: 2026-05-25

## 任务目标

覆盖未配置 API Key、麦克风权限缺失、录音失败、ASR 调用失败、快速文本模型调用失败、自动粘贴失败、数据库写入失败等场景，确保每个失败场景都有明确提示和兜底方案。

## 实际改动

### 1. 添加 thiserror 依赖

在 `src-tauri/Cargo.toml` 中添加：
```toml
thiserror = "2.0"
```

### 2. 改进录音模块错误处理

**文件**: `src-tauri/src/recording.rs`

- 定义 `RecordingError` 枚举，包含所有录音相关错误类型
- 所有错误消息中文化
- 添加麦克风权限检查逻辑
- 在获取设备配置失败时，检测是否为权限问题

错误类型：
- `AlreadyRecording`: 录音已在进行中
- `NoRecordingInProgress`: 当前没有正在进行的录音
- `MicrophonePermissionDenied`: 麦克风权限缺失
- `NoInputDeviceAvailable`: 未找到可用的音频输入设备
- `DeviceConfigFailed`: 获取音频设备配置失败
- `FileCreationFailed`: 创建录音文件失败
- `StreamBuildFailed`: 构建录音流失败
- `StreamStartFailed`: 启动录音流失败
- `UnsupportedSampleFormat`: 不支持的音频采样格式
- `StateLockFailed`: 录音状态锁定失败

### 3. 前端添加 Toast 组件

**依赖**: 安装 `sonner` 库
```bash
pnpm add sonner
```

**文件**: `src/main.tsx`

- 导入 `toast` 和 `Toaster` 组件
- 在 App 组件根元素添加 `<Toaster position="top-center" richColors />`
- 所有错误处理函数使用 `toast.error()` 显示错误
- 成功操作使用 `toast.success()` 显示成功提示
- 警告信息使用 `toast.warning()` 显示警告

### 4. 前端错误处理改进

#### API Key 配置检查

在 `handleProcessAudio` 和 `handleStartRecording` 中添加：
```typescript
if (!appConfig?.asr_api_key || !appConfig?.openai_api_key) {
  toast.error("请先在设置页配置 API Key");
  setVoiceStatus("未配置 API Key，请前往设置页配置。");
  return;
}
```

#### 录音错误分类提示

在 `handleStartRecording` 中根据错误类型显示不同提示：
- 麦克风权限错误：提示在系统设置中开启权限
- 设备未找到错误：提示检查麦克风连接
- 其他错误：显示具体错误信息

#### 复制和输出操作反馈

- 复制成功：`toast.success("已复制到剪贴板")`
- 输出成功：根据输出方式显示不同提示
  - 键盘注入：`toast.success("已自动输入到光标位置")`
  - 剪贴板粘贴：`toast.success("已通过剪贴板输入")`
  - 兜底：`toast.warning("自动粘贴失败，已复制到剪贴板，请手动粘贴 (Ctrl+V)")`

#### 语音处理结果反馈

- 处理成功：`toast.success("语音处理完成")`
- 文本整理失败但保留原文：`toast.warning("文本整理失败，已保留原始识别文本")`
- 处理失败：`toast.error("语音处理失败：{错误信息}")`

## 为什么这么做

### 1. 使用 thiserror

`thiserror` 是 Rust 生态中标准的错误处理库，提供：
- 派生宏简化错误类型定义
- 自动实现 `std::error::Error` trait
- 清晰的错误消息格式化

### 2. 中文化错误消息

用户是中文用户，中文错误消息更友好，降低理解成本。

### 3. 麦克风权限检查

麦克风权限是录音功能的前置条件。在 Windows/macOS/Linux 上，权限被拒绝时 `cpal` 会返回特定错误。通过检测错误消息中的关键词（`permission`、`access`），可以识别权限问题并给出明确提示。

### 4. Toast 组件统一错误展示

Toast 组件优势：
- 非阻塞式提示，不打断用户操作
- 自动消失，不需要手动关闭
- 支持不同类型（error、success、warning），视觉区分明显
- `sonner` 库轻量、性能好、样式现代

### 5. API Key 配置前置检查

在录音或上传音频前检查 API Key 配置，避免用户录音后才发现配置缺失，浪费时间。

### 6. 错误分类提示

不同错误需要不同的用户操作：
- 权限错误 → 去系统设置开启权限
- 设备未找到 → 检查硬件连接
- 配置缺失 → 去设置页配置

分类提示让用户知道下一步该做什么。

## 涉及文件

- `src-tauri/Cargo.toml`: 添加 thiserror 依赖
- `src-tauri/src/recording.rs`: 改进录音模块错误处理
- `package.json`: 添加 sonner 依赖
- `src/main.tsx`: 添加 Toast 组件和改进前端错误处理

## 测试与验证

### 编译验证

```bash
# Rust 编译检查
cd src-tauri && cargo check
# 输出: Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.40s

# TypeScript 类型检查
pnpm exec tsc --noEmit
# 无输出，检查通过

# 前端构建
pnpm build
# 输出: ✓ built in 2.71s
```

### 手动测试场景

以下场景需要在真实环境中手动测试：

1. **未配置 API Key 场景**
   - 清空设置页的 API Key
   - 尝试录音或上传音频
   - 预期：显示 Toast 错误提示"请先在设置页配置 API Key"

2. **麦克风权限缺失场景**
   - 在系统设置中禁用麦克风权限
   - 尝试录音
   - 预期：显示 Toast 错误提示"麦克风权限缺失，请在系统设置中开启麦克风权限"

3. **麦克风设备未连接场景**
   - 拔掉麦克风或禁用音频输入设备
   - 尝试录音
   - 预期：显示 Toast 错误提示"未找到麦克风设备，请检查麦克风连接"

4. **ASR 调用失败场景**
   - 配置错误的 ASR API Key
   - 上传音频文件
   - 预期：显示 Toast 错误提示，包含 ASR 错误信息

5. **文本整理失败场景**
   - 配置错误的 OpenAI API Key
   - 上传音频文件（ASR 成功）
   - 预期：显示 Toast 警告"文本整理失败，已保留原始识别文本"，结果展示原始 ASR 文本

6. **自动粘贴失败场景**
   - 配置输出方式为"自动粘贴"
   - 完成语音输入后点击"输出"按钮
   - 如果自动粘贴失败，预期：显示 Toast 警告"自动粘贴失败，已复制到剪贴板，请手动粘贴 (Ctrl+V)"

7. **复制成功场景**
   - 完成语音输入后点击"复制"按钮
   - 预期：显示 Toast 成功提示"已复制到剪贴板"

8. **输出成功场景**
   - 完成语音输入后点击"输出"按钮
   - 预期：根据输出方式显示对应的成功提示

## 执行复盘

### 顺利的地方

1. **Rust 错误处理改进**：`thiserror` 库使用简单，错误类型定义清晰
2. **前端 Toast 集成**：`sonner` 库开箱即用，API 简洁
3. **编译验证**：所有编译检查一次通过，没有类型错误

### 遇到的问题

1. **Edit 工具字符串匹配失败**：在更新 `handleCopyFinalText` 等函数时，第一次 Edit 失败。通过 Read 工具重新读取文件，找到准确的字符串后成功更新。

### 改进建议

1. **权限检查可以更早**：可以在应用启动时检查麦克风权限，提前提示用户
2. **错误日志记录**：可以将错误信息记录到本地日志文件，方便调试
3. **错误重试机制**：对于网络相关错误（ASR、OpenAI 调用），可以添加自动重试逻辑

## 未完成事项

以下场景的真实测试需要在本地环境中手动执行：

1. 麦克风权限缺失场景
2. 麦克风设备未连接场景
3. ASR 调用失败场景（错误 API Key）
4. 文本整理失败场景（错误 API Key）
5. 自动粘贴失败场景
6. 数据库写入失败场景（需要模拟磁盘满或权限不足）

## 后续建议

1. **添加错误日志系统**：使用 `tracing` 或 `log` 库记录错误到文件
2. **添加错误统计**：记录各类错误发生频率，用于产品改进
3. **添加网络重试**：对 ASR 和 OpenAI 调用添加指数退避重试
4. **添加离线检测**：检测网络连接状态，提前提示用户
5. **添加权限预检查**：应用启动时检查麦克风权限，引导用户授权
